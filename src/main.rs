
use axum::{routing::get, Extension, Router};
use std::collections::HashMap;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tera::Tera;
use tokio::sync::broadcast;
use tower::limit::ConcurrencyLimitLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod handlers;
mod models;
mod responses;
mod websocket;

use crate::handlers::{chat, index, login, not_found, post_login, post_registration, registration};
use models::{LoginResponse, RegisterLoginRequest, RegisterResponse, UserInfo};
use websocket::join_chat;

// Shared state between all connections.
pub struct AppState {
    user_info: Mutex<HashMap<String, UserInfo>>,
    // Channel used to send messages to all connected clients.
    tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_chat=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("base.html", include_str!("../resources/base.html")),
        ("chat.html", include_str!("../resources/chat.html")),
        ("index.html", include_str!("../resources/index.html")),
        ("login.html", include_str!("../resources/login.html")),
        (
            "registration.html",
            include_str!("../resources/registration.html"),
        ),
    ])
    .unwrap();

    let user_info = Mutex::new(HashMap::new());
    let (tx, _rx) = broadcast::channel(50); // 50 Chatters maximum

    let app_state = Arc::new(AppState { user_info, tx });

    let routes = Router::new()
        .route("/", get(index))
        .route("/index", get(index))
        .route("/join", get(join_chat))
        .route("/chat", get(chat))
        .route("/register", get(registration).post(post_registration))
        .route("/login", get(login).post(post_login))
        .fallback(not_found)
        .layer(Extension(Arc::new(tera)))
        .layer(ConcurrencyLimitLayer::new(100))
        .with_state(app_state);

    // run it with hyper on localhost:3000
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);
    let s = axum::Server::bind(&addr)
        .serve(routes.into_make_service())
        .await;

    if let Err(e) = s {
        tracing::error!("server error: {}", e);
    }
}
