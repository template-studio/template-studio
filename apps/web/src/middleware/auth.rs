use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
    extract::State,
};
use serde_json::json;
use template_studio_shared::models::auth::AuthUser;
use crate::AppState;

/// JWT 认证中间件 - 从 token header 提取并验证 JWT
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let token = request
        .headers()
        .get("token")
        .and_then(|v| v.to_str().ok());

    let token = match token {
        Some(t) if !t.is_empty() => t,
        _ => {
            return unauthorized_response("未提供认证令牌");
        }
    };

    match state.auth_service.verify_token(token) {
        Ok(claims) => {
            let auth_user = AuthUser {
                user_id: claims.sub,
                username: claims.username,
            };
            request.extensions_mut().insert(auth_user);
            next.run(request).await
        }
        Err(_) => unauthorized_response("认证令牌无效或已过期"),
    }
}

fn unauthorized_response(message: &str) -> Response {
    let body = json!({
        "code": 401,
        "message": message,
        "result": null
    })
    .to_string();

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("content-type", "application/json")
        .body(body.into())
        .unwrap()
}
