use axum::{
    routing::{get, post, put},
    Router,
};
use crate::handlers::auth;
use super::super::AppState;

/// 认证路由（公开，不需要 auth middleware）
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(auth::login))
        .route("/info", get(auth::get_info))
        .route("/password", put(auth::change_password))
}
