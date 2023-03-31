use crate::auth::{Claims, JWT_EXPIRES_IN_MINUTES, JWT_SECRET};
use crate::{RegisterLoginRequest, RegisterLoginResponse, UserInfo};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::{Extension, Json};
use jsonwebtoken::{EncodingKey, Header};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tera::Tera;
use tracing::info;

use crate::auth;
use crate::responses::Responses;
use crate::AppState;

pub async fn not_found() -> StatusCode {
    return StatusCode::NOT_FOUND;
}

pub async fn index(Extension(templates): Extension<Arc<Tera>>) -> impl IntoResponse {
    Html(
        templates
            .render("index.html", &tera::Context::new())
            .unwrap(),
    )
}

pub async fn registration(Extension(templates): Extension<Arc<Tera>>) -> impl IntoResponse {
    Html(
        templates
            .render("registration.html", &tera::Context::new())
            .unwrap(),
    )
}

pub async fn post_registration(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<RegisterLoginRequest>, // if the request doesn't deserialize to RegisterRequest, it will return an error
) -> Json<RegisterLoginResponse> {
    tracing::info!("Username: {}", payload.username);
    tracing::info!("Password: {}", payload.password);

    // Validate username and password lengths
    match payload.validate() {
        Ok(_) => {}
        Err(e) => {
            return Json(RegisterLoginResponse {
                success: false,
                message: e.to_string(),
                token: None,
            });
        }
    }

    // Check if username already exists
    if app_state
        .user_info
        .lock()
        .unwrap()
        .contains_key(&payload.username)
    {
        return Json(RegisterLoginResponse {
            success: false,
            message: "Username already exists".to_string(),
            token: None,
        });
    } else {
        // TODO: Hash password and store in database

        app_state.user_info.lock().unwrap().insert(
            payload.username.clone(),
            UserInfo::create_user(payload.username, payload.password),
        );

        return Json(RegisterLoginResponse {
            success: true,
            message: "User registered successfully".to_string(),
            token: None,
        });
    }
}

pub async fn post_login(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<RegisterLoginRequest>,
) -> Json<RegisterLoginResponse> {
    // Validate username and password lengths
    match payload.validate() {
        Ok(_) => {}
        Err(e) => {
            return Json(RegisterLoginResponse {
                success: false,
                message: e.to_string(),
                token: None,
            });
        }
    }

    // Check correct password
    match app_state.user_info.lock().unwrap().get(&payload.username) {
        Some(user) => {
            if user.verify_password(&payload.password) {
                // Create JWT
                let token = auth::create_jwt(payload.username);
                info!("Token: {}", token);

                return Json(RegisterLoginResponse {
                    success: true,
                    message: "Login successfull!".to_string(),
                    token: Some(token),
                });
            } else {
                // Wrong password
                return Json(RegisterLoginResponse {
                    success: false,
                    message: "User does not exist or wrong password".to_string(),
                    token: None,
                });
            }
        }
        None => {
            // User does not exist
            return Json(RegisterLoginResponse {
                success: false,
                message: "User does not exist or wrong password".to_string(),
                token: None,
            });
        }
    }
}
