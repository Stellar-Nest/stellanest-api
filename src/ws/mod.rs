use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, RwLock};

/// WebSocket message format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub channel: String,
    pub data: serde_json::Value,
}

/// Hub manages WebSocket connections and channels.
pub struct Hub {
    /// Broadcast sender per channel.
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<WsMessage>>>>,
}

impl Hub {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Spawn the hub background task (currently a no-op; channels are created on demand).
    pub fn start(self) -> Arc<Self> {
        Arc::new(self)
    }

    /// Get or create a broadcast channel.
    pub async fn get_or_create_channel(&self, channel: &str) -> broadcast::Sender<WsMessage> {
        let mut channels = self.channels.write().await;
        channels
            .entry(channel.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(256);
                tx
            })
            .clone()
    }

    /// Broadcast a message to a channel.
    pub async fn broadcast(&self, channel: &str, msg: WsMessage) {
        if let Some(tx) = self.channels.read().await.get(channel) {
            let _ = tx.send(msg);
        }
    }
}

/// Axum WebSocket handler.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<crate::AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: crate::AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut subscriptions: Vec<tokio::sync::mpsc::Receiver<WsMessage>> = vec![];

    // Spawn task to forward messages to client
    let (tx, mut rx) = tokio::sync::mpsc::channel::<WsMessage>(64);

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = sender.send(Message::Text(json)).await;
            }
        }
    });

    // Handle incoming messages (subscriptions)
    let tx_clone = tx.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                    match ws_msg.msg_type.as_str() {
                        "subscribe" => {
                            // TODO: Subscribe to channel via Hub
                            tracing::info!("subscribe to channel: {}", ws_msg.channel);
                        }
                        "unsubscribe" => {
                            tracing::info!("unsubscribe from channel: {}", ws_msg.channel);
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}
