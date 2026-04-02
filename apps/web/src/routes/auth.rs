use axum::{
    routing::post,
    Router,
};
use crate::handlers::auth;
use super::super::AppState;

/// 认证路由（仅登录，公开不需要 auth middleware）
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(auth::login))
}
