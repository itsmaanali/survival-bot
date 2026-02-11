use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use std::sync::Arc;
use tracing::{info, warn};

use super::super::AppState;

/// GET /ws — WebSocket upgrade for real-time cycle updates
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    info!("New WebSocket client connected");

    let mut rx = state.broadcast_tx.subscribe();

    loop {
        tokio::select! {
            // Receive broadcast cycle updates and forward to client
            result = rx.recv() => {
                match result {
                    Ok(update) => {
                        let json = match serde_json::to_string(&update) {
                            Ok(j) => j,
                            Err(e) => {
                                warn!(error = %e, "Failed to serialize cycle update");
                                continue;
                            }
                        };

                        if socket.send(Message::Text(json)).await.is_err() {
                            info!("WebSocket client disconnected");
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!(lagged = n, "WebSocket client lagged behind");
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
            // Handle incoming messages from client (ping/pong, close)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => {
                        info!("WebSocket client disconnected");
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
