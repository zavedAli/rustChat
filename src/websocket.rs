use warp::Filter;
use warp::ws::{Message, WebSocket};
use futures::{StreamExt, SinkExt};
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChatMessage {
    username: String,
    message: String,
}

pub async fn handle_connection(ws: WebSocket, sender: broadcast::Sender<ChatMessage>) {
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    let mut receiver = sender.subscribe();

    // Spawn a task to handle messages from the client
    tokio::spawn(async move {
        while let Some(result) = user_ws_rx.next().await {
            match result {
                Ok(msg) => {
                    if let Ok(text) = msg.to_str() {
                        if let Ok(chat_message) = serde_json::from_str::<ChatMessage>(text) {
                            if sender.send(chat_message).is_err() {
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("websocket error: {}", e);
                    break;
                }
            }
        }
    });

    // Spawn a task to handle messages to the client
    tokio::spawn(async move {
        while let Ok(chat_message) = receiver.recv().await {
            let msg = serde_json::to_string(&chat_message).unwrap();
            if user_ws_tx.send(Message::text(msg)).await.is_err() {
                break;
            }
        }
    });
}

pub fn websocket_filter(sender: broadcast::Sender<ChatMessage>) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path("ws")
        .and(warp::ws())
        .and(with_sender(sender))
        .map(|ws: warp::ws::Ws, sender| {
            ws.on_upgrade(move |socket| handle_connection(socket, sender))
        })
}

fn with_sender(sender: broadcast::Sender<ChatMessage>) -> impl Filter<Extract = (broadcast::Sender<ChatMessage>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || sender.clone())
}