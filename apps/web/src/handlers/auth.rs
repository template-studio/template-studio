use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    Extension,
};
use serde_json::{json, Value};
use template_studio_shared::models::user::{LoginRequest, ChangePasswordRequest};
use template_studio_shared::models::auth::AuthUser;
use validator::Validate;

pub type AppState = super::super::AppState;

/// 用户登录（公开接口，不需要认证）
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.auth_service.login(&request).await {
        Ok(resp) => Ok(Json(json!({
            "code": 200,
            "message": "登录成功",
            "result": resp
        }))),
        Err(e) => error_response(StatusCode::UNAUTHORIZED, &e.to_string()),
    }
}

/// 获取当前用户信息（需认证）
pub async fn get_info(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.auth_service.get_user_info(auth_user.user_id).await {
        Ok(info) => Ok(Json(json!({
            "code": 200,
            "result": info
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 修改密码
pub async fn change_password(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.user_service.change_password(auth_user.user_id, &request).await {
        Ok(_) => Ok(Json(json!({
            "code": 200,
            "message": "密码修改成功"
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

fn error_response(status: StatusCode, message: &str) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Err((status, Json(json!({
        "code": status.as_u16(),
        "message": message
    }))))
}
