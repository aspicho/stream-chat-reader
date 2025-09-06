use std::{collections::HashMap, sync::{atomic::AtomicUsize, Arc, Mutex}};

use clap::Parser;
use tokio::sync::broadcast;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub id: String,
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

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The address to bind the server to
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    pub host: String,
    
    /// The port to bind the server to
    #[arg(short, long, default_value = "3000")]
    pub port: u16,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub listen: bool,
}