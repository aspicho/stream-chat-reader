use std::{collections::HashMap, sync::{atomic::AtomicUsize, Arc, Mutex}};

use blake3::Hash;
use tokio::sync::broadcast;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub id: u128,
    pub platform: String,
    pub channel: String,
    pub username: String,
    pub content: String,
    pub additional_info: Option<String>,
    pub timestamp: u64,
    pub published: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MessageConfirmation {
    pub id: u128,
    pub allowed: bool,
}

pub struct AppState {
    pub db_conn: Arc<Mutex<rusqlite::Connection>>,
    pub admin_panel_sender: broadcast::Sender<ChatMessage>,
    pub client_sender: broadcast::Sender<ChatMessage>,
    pub active_connections: AtomicUsize,
    pub listened_channels: Arc<Mutex<HashMap<String, HashMap<String, tokio::task::JoinHandle<()>>>>>, 
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Channel {
    pub id: u128,
    pub name: String,
    pub platform: String,
    pub listen: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum AdminCommand {
    ListenChannel { name: String, platform: String },
    UnlistenChannel { name: String, platform: String },
    ConfirmMessage { id: String },
    AddChannel { name: String, platform: String },
    RemoveChannel { name: String, platform: String },
    GetChannels,
    GetMessages { limit: usize, before: Option<u64> },
}