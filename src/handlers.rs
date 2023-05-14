use crate::{LoginResponse, RegisterLoginRequest, RegisterResponse, UserInfo};
use axum::extract::State;


use axum::http::header::SET_COOKIE;
use axum::http::{Response, StatusCode};

use axum::response::{Html, IntoResponse};
use axum::{Extension, Json};

use std::sync::Arc;

use tera::Tera;
use tracing::info;



use crate::auth;

use crate::AppState;

pub async fn not_found() -> StatusCode {
    return StatusCode::NOT_FOUND;
}

pub async fn index(
    Extension(templates): Extension<Arc<Tera>>,
    _headers: axum::headers::HeaderMap,
) -> impl IntoResponse {
    // TODO: if authenticated

    let mut context = tera::Context::new();
    context.insert("user_authenticated", &false);

    Html(templates.render("index.html", &context).unwrap())
}

pub async fn chat(Extension(templates): Extension<Arc<Tera>>) -> impl IntoResponse {
    Html(
        templates
            .render("chat.html", &tera::Context::new())
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

pub async fn login(Extension(templates): Extension<Arc<Tera>>) -> impl IntoResponse {
    Html(
        templates
            .render("login.html", &tera::Context::new())
            .unwrap(),
    )
}

pub async fn post_registration(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<RegisterLoginRequest>, // if the request doesn't deserialize to RegisterRequest, it will return an error
) -> Json<RegisterResponse> {
    tracing::info!("Username: {}", payload.username);
    tracing::info!("Password: {}", payload.password);

    // Validate username and password lengths
    match payload.validate() {
        Ok(_) => {}
        Err(e) => {
            return Json(RegisterResponse {
                success: false,
                message: e.to_string(),
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
        return Json(RegisterResponse {
            success: false,
            message: "Username already exists".to_string(),
        });
    } else {
        // Hash password and store
        app_state.user_info.lock().unwrap().insert(
            payload.username.clone(),
            UserInfo::create_user(payload.username, payload.password),
        );

        return Json(RegisterResponse {
            success: true,
            message: "User registered successfully".to_string(),
        });
    }
}

pub async fn post_login(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<RegisterLoginRequest>,
) -> impl IntoResponse {
    match payload.validate() {
        Ok(_) => {}
        Err(e) => {
            return Response::builder()
                .status(400)
                .body(
                    Json(LoginResponse {
                        success: false,
                        message: e.to_string(),
                        token: None,
                    })
                    .into_response(),
                )
                .unwrap();
        }
    }

    // Check correct password
    match app_state.user_info.lock().unwrap().get(&payload.username) {
        Some(user) => {
            if user.verify_password(&payload.password) {
                // Create JWT
                let token = auth::create_jwt(payload.username);
                info!("Token: {}", token);

                return Response::builder()
                    .header("content-type", "application/json")
                    .header(SET_COOKIE, "Authorization=Bearer ".to_string() + &token)
                    .body(
                        Json(LoginResponse {
                            success: true,
                            message: "Login successful!".to_string(),
                            token: Some(token),
                        })
                        .into_response(),
                    )
                    .unwrap();
            } else {
                // Wrong password
                return Response::builder()
                    .status(401)
                    .body(
                        Json(LoginResponse {
                            success: false,
                            message: "User does not exist or wrong password".to_string(),
                            token: None,
                        })
                        .into_response(),
                    )
                    .unwrap();
            }
        }
        None => {
            // User does not exist
            return Response::builder()
                .status(401)
                .body(
                    Json(LoginResponse {
                        success: false,
                        message: "User does not exist or wrong password".to_string(),
                        token: None,
                    })
                    .into_response(),
                )
                .unwrap();
        }
    }
}
