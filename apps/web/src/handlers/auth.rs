use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde_json::{json, Value};
use template_studio_shared::models::user::{LoginRequest, ChangePasswordRequest, RegisterRequest};
use template_studio_shared::models::auth::AuthUser;
use template_studio_shared::models::pat::CreatePatRequest;
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

/// 用户注册（公开接口，不需要认证）
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.auth_service.register(&request).await {
        Ok(resp) => Ok(Json(json!({
            "code": 200,
            "message": "注册成功",
            "result": resp
        }))),
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e.to_string()),
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

/// 创建 PAT 令牌
pub async fn create_pat(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreatePatRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.pat_service.create(auth_user.user_id, &request).await {
        Ok(resp) => Ok(Json(json!({
            "code": 200,
            "message": "令牌创建成功",
            "result": resp
        }))),
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

/// 列出当前用户的 PAT 令牌
pub async fn list_pats(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.pat_service.list(auth_user.user_id).await {
        Ok(list) => Ok(Json(json!({
            "code": 200,
            "result": list
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 删除 PAT 令牌
pub async fn delete_pat(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.pat_service.delete(id, auth_user.user_id).await {
        Ok(true) => Ok(Json(json!({
            "code": 200,
            "message": "令牌已删除"
        }))),
        Ok(false) => error_response(StatusCode::NOT_FOUND, "令牌不存在"),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

fn error_response(status: StatusCode, message: &str) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Err((status, Json(json!({
        "code": status.as_u16(),
        "message": message
    }))))
}
