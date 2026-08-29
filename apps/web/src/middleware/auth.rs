use crate::AppState;
use axum::{
    extract::Request, extract::State, http::StatusCode, middleware::Next, response::Response,
};
use serde_json::json;
use std::sync::Arc;
use template_studio_shared::models::auth::{AuthType, AuthUser};

const PAT_PREFIX: &str = "ts_pat_";

/// 认证中间件 - 区分 JWT 和 PAT 认证
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let token = match extract_token(&request) {
        Some(t) => t,
        None => return unauthorized_response("未提供认证令牌"),
    };

    match authenticate(&state, token).await {
        Ok(auth_user) => {
            request.extensions_mut().insert(auth_user);
            next.run(request).await
        }
        Err(msg) => unauthorized_response(msg),
    }
}

/// 管理端中间件 - 认证 + super_admin 角色校验
///
/// 角色每次查库获取，角色变更（授权/撤销）即时生效；
/// PAT 令牌面向模板 CI 场景，不允许访问管理端
pub async fn admin_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let token = match extract_token(&request) {
        Some(t) => t,
        None => return unauthorized_response("未提供认证令牌"),
    };

    let auth_user = match authenticate(&state, token).await {
        Ok(u) => u,
        Err(msg) => return unauthorized_response(msg),
    };

    if matches!(auth_user.auth_type, AuthType::Pat) {
        return forbidden_response("PAT 令牌不允许访问管理接口");
    }

    let roles = state
        .auth_service
        .get_user_role_names(auth_user.user_id)
        .await
        .unwrap_or_default();

    if !roles.iter().any(|r| r == "super_admin") {
        return forbidden_response("需要管理员权限");
    }

    request.extensions_mut().insert(auth_user);
    next.run(request).await
}

fn extract_token(request: &Request) -> Option<&str> {
    request
        .headers()
        .get("token")
        .and_then(|v| v.to_str().ok())
        .filter(|t| !t.is_empty())
}

async fn authenticate(state: &AppState, token: &str) -> Result<AuthUser, &'static str> {
    if token.starts_with(PAT_PREFIX) {
        // PAT 令牌认证
        match state.pat_service.validate(token).await {
            Ok(validation) => Ok(AuthUser {
                user_id: validation.user_id,
                username: String::new(), // PAT 无法获取 username，按需查询
                auth_type: AuthType::Pat,
                scopes: Some(Arc::new(validation.scopes)),
            }),
            Err(_) => Err("令牌无效或已过期"),
        }
    } else {
        // JWT 会话认证
        match state.auth_service.verify_token(token) {
            Ok(claims) => Ok(AuthUser {
                user_id: claims.sub,
                username: claims.username,
                auth_type: AuthType::Jwt,
                scopes: None,
            }),
            Err(_) => Err("认证令牌无效或已过期"),
        }
    }
}

fn unauthorized_response(message: &str) -> Response {
    error_response(StatusCode::UNAUTHORIZED, 401, message)
}

fn forbidden_response(message: &str) -> Response {
    error_response(StatusCode::FORBIDDEN, 403, message)
}

fn error_response(status: StatusCode, code: i32, message: &str) -> Response {
    let body = json!({
        "code": code,
        "message": message,
        "result": null
    })
    .to_string();

    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(body.into())
        .unwrap()
}
