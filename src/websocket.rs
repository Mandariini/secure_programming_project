use crate::auth;
use axum::extract::State;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use futures::SinkExt;
use futures::StreamExt;
use sanitize_html::rules::predefined::DEFAULT;
use sanitize_html::sanitize_str;
use std::sync::Arc;
use std::time::{self, Duration};
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::AppState;

const MESSAGE_SLOW_MODE_TIME_IN_SECONDS: Duration = Duration::from_secs(2);
const MESSAGE_MAX_LENGTH: usize = 100;
const MAXIMUM_USERS: usize = 20;

fn sanitize_input(input: &str) -> String {
    let sanitized_input = sanitize_str(&DEFAULT, input).unwrap();

    // Trim the input to a maximum length
    let trimmed_input = sanitized_input
        .trim()
        .chars()
        .take(MESSAGE_MAX_LENGTH)
        .collect::<String>();

    trimmed_input
}

pub async fn join_chat(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    return ws.on_upgrade(|socket| chat_socket(socket, state));
}

async fn chat_socket(socket: WebSocket, state: Arc<AppState>) {
    let (sender, mut receiver) = socket.split();
    let sender: Arc<Mutex<futures::stream::SplitSink<WebSocket, Message>>> =
        Arc::new(tokio::sync::Mutex::new(sender));

    // Clones to be able to send messages from multiple tasks
    let sender_clone_1 = Arc::clone(&sender);
    let sender_clone_2 = Arc::clone(&sender);
    let sender_clone_3 = Arc::clone(&sender);

    if state.tx.receiver_count() >= MAXIMUM_USERS {
        sender
            .lock()
            .await
            .send(Message::Text(format!(
                "Maximum users reached, wait a bit and try again",
            )))
            .await
            .ok();

        return;
    }

    let mut username: String = "".to_string();
    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(received_message) = message {
            // Check if JWT token is valid.
            let received_token = received_message.trim_start_matches("Bearer ").to_string();
            let claims = auth::decode_jwt(&received_token);
            let mut sender = sender_clone_1.lock().await;

            match claims {
                Ok(claims) => {
                    username = claims.sub;
                    sender
                        .send(Message::Text(format!("Welcome {}!", username)))
                        .await
                        .ok();
                    break;
                }
                Err(e) => {
                    let _ = sender.send(Message::Text(format!("Error: {}", e))).await;
                    sender.close().await.ok();
                    return;
                }
            }
        }
    }

    let mut rx = state.tx.subscribe();

    let msg: String = format!("{} joined.", username); // Send the "joined" message to all subscribers.
    let _ = state.tx.send(msg);

    // Task that listens for messages from the broadcast channel and sends them to the websocket (client).
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            let mut sender = sender_clone_2.lock().await;
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let tx = state.tx.clone();
    let name = username.clone();

    // Task that listens for messages from the websocket (client) and sends them to the broadcast channel.
    let mut recv_task = tokio::spawn(async move {
        let mut prev_message_time = time::SystemTime::UNIX_EPOCH;

        // A message can be sent once a second, check here in backend but also frontend
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let mut sender = sender_clone_3.lock().await;
            if prev_message_time + MESSAGE_SLOW_MODE_TIME_IN_SECONDS > time::SystemTime::now() {
                // if previous message was less than a second ago, send an error message
                if sender
                    .send(Message::Text(
                        "##########Sending messages too fast, wait a bit##########".to_string(),
                    ))
                    .await
                    .is_err()
                {
                    break;
                }
                continue;
            }
            prev_message_time = time::SystemTime::now();

            // Trim the message to fit max size and remove unwanted characters
            let sanitized_text = sanitize_input(&text);

            // Add username before message.
            let _ = tx.send(format!("{}: {}", name, sanitized_text));
        }
    });

    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    // Send "user left" message
    let msg = format!("{} left.", username);
    let _ = state.tx.send(msg);
}
