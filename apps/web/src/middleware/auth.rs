use std::sync::Arc;
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
    extract::State,
};
use serde_json::json;
use template_studio_shared::models::auth::{AuthUser, AuthType};
use crate::AppState;

const PAT_PREFIX: &str = "ts_pat_";

/// 认证中间件 - 区分 JWT 和 PAT 认证
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

    if token.starts_with(PAT_PREFIX) {
        // PAT 令牌认证
        match state.pat_service.validate(token).await {
            Ok(validation) => {
                let auth_user = AuthUser {
                    user_id: validation.user_id,
                    username: String::new(), // PAT 无法获取 username，按需查询
                    auth_type: AuthType::Pat,
                    scopes: Some(Arc::new(validation.scopes)),
                };
                request.extensions_mut().insert(auth_user);
                next.run(request).await
            }
            Err(_) => unauthorized_response("令牌无效或已过期"),
        }
    } else {
        // JWT 会话认证
        match state.auth_service.verify_token(token) {
            Ok(claims) => {
                let auth_user = AuthUser {
                    user_id: claims.sub,
                    username: claims.username,
                    auth_type: AuthType::Jwt,
                    scopes: None,
                };
                request.extensions_mut().insert(auth_user);
                next.run(request).await
            }
            Err(_) => unauthorized_response("认证令牌无效或已过期"),
        }
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
