use std::{collections::HashMap, sync::{atomic::{AtomicUsize, Ordering}, Arc, Mutex}};
use tracing::{info, warn};
use axum::{extract::{ws::WebSocket, Path, Query, State, WebSocketUpgrade}, http::StatusCode, response::Response, routing::{any, delete, get, post}, Json, Router};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use futures_util::{sink::SinkExt, stream::{StreamExt, SplitSink, SplitStream}};

use crate::{models::AppState, utils::parse_args};

mod utils;
mod models;

async fn client_ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(|socket| client_socket_handler(socket, state))
}

async fn client_socket_handler(mut socket: WebSocket, state: Arc<AppState>) {
    let connection_count = state.active_connections.fetch_add(1, Ordering::Relaxed);
    info!("New client connection. Total: {}", connection_count + 1);

    let (sender, receiver) = socket.split();
    let mut message_receiver = state.client_sender.subscribe();

    let state_clone = state.clone();
    let reader_handle = tokio::spawn(async move {
        reader_client_task(receiver, state_clone).await;
    });

    let writer_handle = tokio::spawn(async move {
        writer_client_task(sender, message_receiver).await;
    });

    tokio::select! {
        _ = reader_handle => {
            info!("Client reader task completed first");
        }
        _ = writer_handle => {
            info!("Client writer task completed first");
        }
    }

    let final_count = state.active_connections.fetch_sub(1, Ordering::Relaxed);
    info!("Client connection closed. Total: {}", final_count - 1);
}

async fn reader_client_task(mut receiver: SplitStream<WebSocket>, state: Arc<AppState>) {
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(msg) => {
                info!("Received client message: {:?}", msg);
            }
            Err(e) => {
                warn!("Error receiving client message: {:?}", e);
                break;
            }
        }
    }
}

async fn writer_client_task(mut sender: SplitSink<WebSocket, axum::extract::ws::Message>, mut message_receiver: broadcast::Receiver<models::ChatMessage>) {
    while let Ok(chat_message) = message_receiver.recv().await {
        let msg_text = serde_json::to_string(&chat_message).unwrap_or_else(|_| "{}".to_string());
        if sender.send(axum::extract::ws::Message::Text(msg_text.into())).await.is_err() {
            warn!("Error sending client message");
            break;
        }
    }
}

async fn admin_ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(|socket| admin_socket_handler(socket, state))
}

async fn admin_socket_handler(mut socket: WebSocket, state: Arc<AppState>) {
    let connection_count = state.active_connections.fetch_add(1, Ordering::Relaxed);
    info!("New admin connection. Total: {}", connection_count + 1);

    let (sender, receiver) = socket.split();
    let mut message_receiver = state.admin_panel_sender.subscribe();

    let state_clone = state.clone();
    let reader_handle = tokio::spawn(async move {
        reader_admin_task(receiver, state_clone).await;
    });

    let writer_handle = tokio::spawn(async move {
        writer_admin_task(sender, message_receiver).await;
    });

    tokio::select! {
        _ = reader_handle => {
            info!("Admin reader task completed first");
        }
        _ = writer_handle => {
            info!("Admin writer task completed first");
        }
    }

    let final_count = state.active_connections.fetch_sub(1, Ordering::Relaxed);
    info!("Admin connection closed. Total: {}", final_count - 1);
}

async fn reader_admin_task(
    mut receiver: SplitStream<WebSocket>,
    state: Arc<AppState>
) {
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(msg) => {
                info!("Received admin message: {:?}", msg);
            }
            Err(e) => {
                warn!("Error receiving admin message: {:?}", e);
                break;
            }
        }
    }
}

async fn writer_admin_task(mut sender: SplitSink<WebSocket, axum::extract::ws::Message>, mut message_receiver: broadcast::Receiver<models::ChatMessage>) {
    while let Ok(chat_message) = message_receiver.recv().await {
        let msg_text = serde_json::to_string(&chat_message).unwrap_or_else(|_| "{}".to_string());
        if sender.send(axum::extract::ws::Message::Text(msg_text.into())).await.is_err() {
            warn!("Error sending admin message");
            break;
        }
    }
}

async fn publish_message(
    State(state): State<Arc<AppState>>,
    Path(params): Path<std::collections::HashMap<String, String>>,
    ) -> (StatusCode, Json<serde_json::Value>) {
    if let Some(id) = params.get("id") {
        let id_num = id.parse::<u128>().unwrap_or(0);

        info!("Publishing message with id: {}", id_num);
        
        utils::publish_message(
            &state.db_conn.lock().unwrap(),
            id_num,
            &state.client_sender
        ).expect("Failed to publish message");

        (StatusCode::OK, 
            Json(serde_json::json!({
                "status": "success",
                "message": format!("Message {} published", id)
            }))
        )
    } else {
        (StatusCode::BAD_REQUEST, 
            Json(serde_json::json!({
                "status": "error",
                "message": "Missing id parameter"
            }))
        )
    }
}

async fn listen_channel(
    State(state): State<Arc<AppState>>,
    Path(params): Path<std::collections::HashMap<String, String>>,
) -> (StatusCode, Json<serde_json::Value>) {
    if let (Some(platform), Some(id)) = (params.get("platform"), params.get("id")) {
        info!("Listening to channel {} on platform {}", id, platform);

        let result = match platform.as_str() {
            "twitch" => {
                utils::listen_to_twitch(
                    id.clone(),
                    state.admin_panel_sender.clone(),
                    state.db_conn.clone(),
                    state.listened_channels.clone(),
                ).await
            },
            "youtube" => {
                utils::listen_to_youtube(
                    id.clone(),
                    state.admin_panel_sender.clone(),
                    state.db_conn.clone(),
                    state.listened_channels.clone(),
                ).await
            },
            _ => {
                Err(anyhow::anyhow!("Unknown platform: {}", platform))
            }
        };

        match result {
            Ok(_) => (StatusCode::OK, 
                Json(serde_json::json!({
                    "status": "success",
                    "message": format!("Started listening to {} on {}", id, platform)
                }))
            ),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, 
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Failed to listen to {} on {}: {:?}", id, platform, e)
                }))
            ),
        }
    } else {
        (StatusCode::BAD_REQUEST, 
            Json(serde_json::json!({
                "status": "error",
                "message": "Missing platform or id parameter"
            }))
        )
    }
}

async fn unlisten_channel(
    State(state): State<Arc<AppState>>,
    Path(params): Path<std::collections::HashMap<String, String>>,
) -> (StatusCode, Json<serde_json::Value>) {
    if let (Some(platform), Some(id)) = (params.get("platform"), params.get("id")) {
        info!("Unlistening from channel {} on platform {}", id, platform);

        match utils::stop_listening_to_channel(
            &platform.clone(),
            &id.clone(),
            state.listened_channels.clone()
        ) {
            Ok(_) => (StatusCode::OK, 
                Json(serde_json::json!({
                    "status": "success",
                    "message": format!("Stopped listening to {} on {}", id, platform)
                }))
            ),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, 
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Failed to stop listening to {} on {}: {:?}", id, platform, e)
                }))
            ),
            
        }

    } else {
        (StatusCode::BAD_REQUEST, 
            Json(serde_json::json!({
                "status": "error",
                "message": "Missing platform or id parameter"
            }))
        )
    }
}

async fn add_channel(
    State(state): State<Arc<AppState>>,
    Path(params): Path<std::collections::HashMap<String, String>>,
) -> (StatusCode, Json<serde_json::Value>) {
    if let (Some(platform), Some(id)) = (params.get("platform"), params.get("id")) {
        info!("Adding channel {} on platform {}", id, platform);

        match utils::add_channel(&state.db_conn.lock().unwrap(), &id, &platform) {
            Ok(_) => (StatusCode::OK, 
                Json(serde_json::json!({
                    "status": "success",
                    "message": format!("Channel {} on {} added", id, platform)
                }))
            ),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, 
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Failed to add channel {} on {}: {:?}", id, platform, e)
                }))
            ),
        }
    } else {
        (StatusCode::BAD_REQUEST, 
            Json(serde_json::json!({
                "status": "error",
                "message": "Missing platform or id parameter"
            }))
        )
    }
}

async fn get_channels(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<serde_json::Value>) {
    match utils::get_channels(&state.db_conn.lock().unwrap()) {
        Ok(channels) => (StatusCode::OK, 
            Json(serde_json::json!({
                "status": "success",
                "channels": channels
            }))
        ),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, 
            Json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to get channels: {:?}", e)
            }))
        ),
    }
}

async fn delete_channel(
    State(state): State<Arc<AppState>>,
    Path(params): Path<std::collections::HashMap<String, String>>,
) -> (StatusCode, Json<serde_json::Value>) {
    if let (Some(platform), Some(id)) = (params.get("platform"), params.get("id")) {
        info!("Deleting channel {} on platform {}", id, platform);        
        match utils::delete_channel(&state.db_conn.lock().unwrap(), &platform, &id) {
            Ok(_) => (StatusCode::OK, 
                Json(serde_json::json!({
                    "status": "success",
                    "message": format!("Channel {} on {} deleted", id, platform)
                }))
            ),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, 
                Json(serde_json::json!({
                    "status": "error",
                    "message": format!("Failed to delete channel {} on {}: {:?}", id, platform, e)
                }))
            ),
        }
    } else {
        (StatusCode::BAD_REQUEST, 
            Json(serde_json::json!({
                "status": "error",
                "message": "Missing platform or id parameter"
            }))
        )
    }
} 

async fn get_messages(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let limit = params.get("limit").and_then(|v| v.parse::<usize>().ok()).unwrap_or(50);
    let before = params.get("before").and_then(|v| v.parse::<u64>().ok());

    match utils::get_messages(limit, before, &state.db_conn.lock().unwrap()) {
        Ok(messages) => {
            let json_messages: Vec<serde_json::Value> = messages.iter().map(|msg| {
                serde_json::json!({
                    "id": msg.id.to_string(),
                    "platform": msg.platform,
                    "channel": msg.channel,
                    "username": msg.username,
                    "content": msg.content,
                    "additional_info": msg.additional_info,
                    "timestamp": msg.timestamp,
                    "published": msg.published
                })
            }).collect();

            info!("Retrieved {} messages", json_messages.len());

            (StatusCode::OK, 
                Json(serde_json::json!({
                    "status": "success",
                    "messages": json_messages
                }))
            )
        },

        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, 
            Json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to get messages: {:?}", e)
            }))
        ),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    let conn = utils::initialize_db();

    let (admin_panel_sender, _) = broadcast::channel(1000);
    let (client_sender, _) = broadcast::channel(1000);

    let state = Arc::new(AppState {
        db_conn: Arc::new(Mutex::new(conn)),
        admin_panel_sender: admin_panel_sender.clone(),
        client_sender: client_sender.clone(),
        active_connections: AtomicUsize::new(0),
        listened_channels: Arc::new(Mutex::new(HashMap::new())),
    });

    let all_channels = utils::get_channels(&state.db_conn.lock().unwrap()).expect("Failed to get channels");
    for channel in all_channels {
        if channel.listen {
            match channel.platform.as_str() {
                "twitch" => {
                    let result = utils::listen_to_twitch(
                        channel.name.clone(),
                        admin_panel_sender.clone(),
                        state.db_conn.clone(),
                        state.listened_channels.clone(),
                    ).await;

                    if let Err(e) = result {
                        warn!("Error starting Twitch listener for {}: {:?}", channel.name, e);
                    } else {
                        info!("Started Twitch listener for {}", channel.name);
                    }
                },

                "youtube" => {
                    let result = utils::listen_to_youtube(
                        channel.name.clone(),
                        admin_panel_sender.clone(),
                        state.db_conn.clone(),
                        state.listened_channels.clone(),
                    ).await;

                    if let Err(e) = result {
                        warn!("Error starting YouTube listener for {}: {:?}", channel.name, e);
                    } else {
                        info!("Started YouTube listener for {}", channel.name);
                    }
                },

                _ => {
                    eprintln!("Unknown platform: {}", channel.platform);
                }
            }
        }
    }

    let args = parse_args();

    let app = Router::new()
        .fallback_service(ServeDir::new("static"))
        .route("/api/ws", any(client_ws_handler))
        .route("/api/admin/ws", any(admin_ws_handler))
        
        .route("/api/messages", get(get_messages))
        .route("/api/publish/{id}", post(publish_message))
        
        .route("/api/channels", get(get_channels))
        .route("/api/channels/{platform}/{id}", post(add_channel))
        .route("/api/channels/{platform}/{id}", delete(delete_channel))
        
        .route("/api/listen/{platform}/{id}", post(listen_channel))
        .route("/api/unlisten/{platform}/{id}", post(unlisten_channel))

        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_headers(Any)
                .allow_origin(Any),
        )
        .with_state(state);
    
    tracing::info!("Server running on {}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind((args.host, args.port)).await
        .expect("Failed to bind TCP listener");

    axum::serve(listener, app).await.unwrap();
}