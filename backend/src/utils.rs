use std::{collections::HashMap, sync::{Arc, Mutex}};

use brainrot::{twitch, youtube::{self, Action, ChatItem}, TwitchChat, TwitchChatEvent};
use futures_util::StreamExt;
use tracing::info;


pub fn initialize_db() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open("chat_messages.db").expect("Failed to open DB");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id BLOB,
            platform TEXT NOT NULL,
            channel TEXT NOT NULL,
            username TEXT NOT NULL,
            content TEXT NOT NULL,
            additional_info TEXT,
            timestamp INTEGER NOT NULL,
            published INTEGER NOT NULL DEFAULT 0
        )",
        [],
    ).expect("Failed to create table");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS channels (
            id blob PRIMARY KEY,
            name TEXT NOT NULL,
            platform TEXT NOT NULL,
            listen INTEGER NOT NULL DEFAULT 0
        )",
        [],
    ).expect("Failed to create channels table");

    conn
}

pub fn add_channel(conn: &rusqlite::Connection, name: &str, platform: &str) -> rusqlite::Result<uuid::Uuid> {
    let id = uuid::Uuid::now_v7();
    conn.execute(
        "INSERT INTO channels (id, name, platform) VALUES (?1, ?2, ?3)",
        rusqlite::params![id.as_bytes(), name, platform],
    )?;
    Ok(id)
}

pub fn get_channels(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<crate::models::Channel>> {
    let mut stmt = conn.prepare("SELECT id, name, platform, listen FROM channels")?;
    
    let channel_iter = stmt.query_map([], |row| {
        Ok(crate::models::Channel {
            id: row.get::<_, Vec<u8>>(0)?.as_slice().try_into().map(u128::from_le_bytes).unwrap_or(0),
            name: row.get(1)?,
            platform: row.get(2)?,
            listen: row.get::<_, i32>(3)? != 0,
        })
    })?;

    let mut channels = Vec::new();
    for channel in channel_iter {
        channels.push(channel?);
    }
    Ok(channels)
}

pub async fn listen_to_twitch(
    name: String,
    admin_panel_sender: tokio::sync::broadcast::Sender<crate::models::ChatMessage>,
    db_conn: Arc<Mutex<rusqlite::Connection>>,
    listened_channels: Arc<Mutex<HashMap<String, HashMap<String, tokio::task::JoinHandle<()>>>>>,
) -> anyhow::Result<()> {

    if listened_channels.lock().unwrap().contains_key("twitch") &&
        listened_channels.lock().unwrap().get("twitch").unwrap().contains_key(&name) {
        info!("Already listening to Twitch channel: {}", &name);
        return Ok(());
    }

    let name_for_handler = name.clone();
    let mut client = TwitchChat::new(name_for_handler.clone(), twitch::Anonymous).await?;
    
    let handler = tokio::spawn(async move {
        while let Some(event) = client.next().await {
            match event {
                Ok(TwitchChatEvent::Message { user, contents, .. }) => {
                    let content = contents.iter().map(|c| c.to_string()).collect::<String>();

                    info!("Twitch message from {}: {}", user.display_name, content);

                    let chat_message = crate::models::ChatMessage {
                        id: uuid::Uuid::now_v7().as_u128(),
                        platform: "twitch".to_string(),
                        channel: name_for_handler.clone(),
                        username: user.display_name.clone(),
                        content: content.clone(),
                        timestamp: chrono::Utc::now().timestamp_millis() as u64,
                        published: false,
                        additional_info: Some(serde_json::json!({
                            "username": user.username,
                            "id": user.id,
                            "display_color": user.display_color,
                            "sub_months": user.sub_months.map(|v| v.get()),
                            "role": format!("{:?}", user.role),
                            "returning_chatter": user.returning_chatter,
                        }).to_string()),
                    };

                    {
                        let conn = db_conn.lock().unwrap();
                        conn.execute(
                            "INSERT INTO messages (id, platform, channel, username, content, additional_info, timestamp, published) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0)",
                            rusqlite::params![
                                chat_message.id.to_le_bytes(),
                                chat_message.platform,
                                chat_message.channel,
                                chat_message.username,
                                chat_message.content,
                                chat_message.additional_info,
                                chat_message.timestamp as i64
                            ],
                        ).expect("Failed to insert message");
                    }

                    let _ = admin_panel_sender.send(chat_message);
                }
                Ok(_) => {}
                Err(e) => eprintln!("Error receiving Twitch message: {:?}", e),
            }
        }
    });

    listened_channels.lock().unwrap()
        .entry("twitch".to_string())
        .or_insert_with(HashMap::new)
        .insert(name.clone(), handler);

    Ok(())
}

pub fn delete_channel(conn: &rusqlite::Connection, platform: &str, name: &str) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM channels WHERE name = ?1 AND platform = ?2",
        rusqlite::params![name, platform],
    )?;
    Ok(())
}

pub fn get_messages(
    limit: usize,
    before: Option<u64>,
    conn: &rusqlite::Connection
) -> rusqlite::Result<Vec<crate::models::ChatMessage>> {
    let mut query = "SELECT id, platform, channel, username, content, additional_info, timestamp, published FROM messages".to_string();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(before_ts) = before {
        query.push_str(" WHERE timestamp < ?1");
        params.push(Box::new(before_ts as i64));
    }
    query.push_str(" ORDER BY timestamp DESC LIMIT ?");
    params.push(Box::new(limit as i64));

    let mut stmt = conn.prepare(&query)?;
    let message_iter = stmt.query_map(rusqlite::params_from_iter(params), |row| {
        Ok(crate::models::ChatMessage {
            id: row.get::<_, Vec<u8>>(0)?.as_slice().try_into().map(u128::from_le_bytes).unwrap_or(0),
            platform: row.get(1)?,
            channel: row.get(2)?,
            username: row.get(3)?,
            content: row.get(4)?,
            additional_info: row.get(5)?,
            timestamp: row.get::<_, i64>(6)? as u64,
            published: row.get::<_, i32>(7)? != 0,
        })
    })?;

    let mut messages = Vec::new();
    for message in message_iter {
        messages.push(message?);
    }

    Ok(messages)
}

pub async fn listen_to_youtube(
    name: String,
    admin_panel_sender: tokio::sync::broadcast::Sender<crate::models::ChatMessage>,
    db_conn: Arc<Mutex<rusqlite::Connection>>,
    listened_channels: Arc<Mutex<HashMap<String, HashMap<String, tokio::task::JoinHandle<()>>>>>,
) -> anyhow::Result<()> {

    if listened_channels.lock().unwrap().contains_key("youtube") &&
        listened_channels.lock().unwrap().get("youtube").unwrap().contains_key(&name) {
        info!("Already listening to YouTube channel: {}", name);
        return Ok(());
    }

    let name_for_handler = name.to_string();
    let context = youtube::ChatContext::new_from_channel(&name_for_handler.clone(), youtube::ChannelSearchOptions::LatestLiveOrUpcoming).await?;

    let handler = tokio::spawn(async move {
        match youtube::stream(&context).await {
            Ok(mut stream) => {
                
                while let Some(Ok(c)) = stream.next().await {
                    if let Action::AddChatItem {
                        item: ChatItem::TextMessage { message_renderer_base, message },
                        ..
                    } = c
                    {
                        let chat_message = crate::models::ChatMessage {
                            id: uuid::Uuid::now_v7().as_u128(),
                            platform: "youtube".to_string(),
                            channel: name_for_handler.clone(),
                            username: {
                                match &message_renderer_base.author_name {
                                    Some(name) => name.simple_text.clone(),
                                    None => "unknown".to_string()
                                }
                            },
                            content: {
                                match &message {
                                    Some(msg) => msg.runs.iter().map(|run| run.to_chat_string()).collect::<String>(),
                                    None => "".to_string()
                                }
                            },
                            timestamp: chrono::Utc::now().timestamp_millis() as u64,
                            published: false,
                            additional_info: None,
                        };

                        info!("YouTube message from {}: {}", chat_message.username, chat_message.content);

                        {
                            let conn = db_conn.lock().unwrap();
                            conn.execute(
                                "INSERT INTO messages (id, platform, channel, username, content, additional_info, timestamp, published) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0)",
                                rusqlite::params![
                                    chat_message.id.to_le_bytes(),
                                    chat_message.platform,
                                    chat_message.channel,
                                    chat_message.username,
                                    chat_message.content,
                                    chat_message.additional_info,
                                    chat_message.timestamp as i64
                                ],
                            ).expect("Failed to insert message");
                        }

                        let _ = admin_panel_sender.send(chat_message);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error creating YouTube stream: {:?}", e);
            }
        }
    });

    listened_channels.lock().unwrap()
        .entry("youtube".to_string())
        .or_insert_with(HashMap::new)
        .insert(name.to_string(), handler);

    Ok(())

}

pub fn new_sys_msg(content: &str) -> crate::models::ChatMessage {
    crate::models::ChatMessage {
        id: uuid::Uuid::now_v7().as_u128(),
        platform: "system".to_string(),
        channel: "system".to_string(),
        username: "system".to_string(),
        content: content.to_string(),
        additional_info: None,
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
        published: true,
    }
}

pub fn publish_message(
    conn: &rusqlite::Connection, 
    message_id: u128,
    client_sender: &tokio::sync::broadcast::Sender<crate::models::ChatMessage>
) -> rusqlite::Result<()> {
    let bytes = message_id.to_le_bytes();
    conn.execute(
        "UPDATE messages SET published = 1 WHERE id = ?1",
        rusqlite::params![bytes]
    )?;
    
    let mut stmt = conn.prepare("SELECT id, platform, channel, username, content, additional_info, timestamp, published FROM messages WHERE id = ?1")?;
    let mut rows = stmt.query(rusqlite::params![bytes])?;
    if let Some(row) = rows.next()? {
        let chat_message = crate::models::ChatMessage {
            id: row.get::<_, Vec<u8>>(0)?.as_slice().try_into().map(u128::from_le_bytes).unwrap_or(0),
            platform: row.get(1)?,
            channel: row.get(2)?,
            username: row.get(3)?,
            content: row.get(4)?,
            additional_info: row.get(5)?,
            timestamp: row.get::<_, i64>(6)? as u64,
            published: row.get::<_, i32>(7)? != 0,
        };
        let _ = client_sender.send(chat_message);
    }

    Ok(())
}

pub fn stop_listening_to_channel(
    platform: &str,
    name: &str,
    listened_channels: Arc<Mutex<HashMap<String, HashMap<String, tokio::task::JoinHandle<()>>>>>,
) -> anyhow::Result<()> {
    let mut channels = listened_channels.lock().unwrap();
    if let Some(platform_map) = channels.get_mut(platform) {
        if let Some(handle) = platform_map.remove(name) {
            handle.abort();
            info!("Stopped listening to {} channel: {}", platform, name);
        } else {
            info!("No active listener found for {} channel: {}", platform, name);
        }
    } else {
        info!("No active listeners for platform: {}", platform);
    }
    Ok(())
}