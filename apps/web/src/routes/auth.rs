use super::super::AppState;
use crate::handlers::{auth, email};
use axum::{
    routing::{get, post},
    Router,
};

/// 认证路由（公开，不需要 auth middleware）
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(auth::login))
        .route("/register", post(auth::register))
        .route("/forgot-password", post(email::forgot_password))
        .route("/reset-password", post(email::reset_password))
        .route("/users/:username", get(auth::public_profile))
}
