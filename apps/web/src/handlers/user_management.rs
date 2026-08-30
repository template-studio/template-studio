use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use template_studio_shared::models::auth::AuthUser;
use template_studio_shared::models::user::{
    AssignRolesRequest, CreateUserRequest, UpdateUserRequest,
};
use template_studio_shared::utils::response::ApiResponse;
use validator::Validate;

pub type AppState = super::super::AppState;

/// 用户列表
pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.user_service.list_users().await {
        Ok(users) => Ok(Json(
            serde_json::to_value(ApiResponse::success_with_message(
                json!({
                    "list": users,
                    "total": users.len()
                }),
                "OK",
            ))
            .unwrap_or_default(),
        )),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 创建用户
pub async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.user_service.create_user(&request).await {
        Ok(id) => Ok(Json(
            serde_json::to_value(ApiResponse::success_with_message(
                json!({ "id": id }),
                "创建用户成功",
            ))
            .unwrap_or_default(),
        )),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 更新用户
pub async fn update_user(
    State(state): State<AppState>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.user_service.update_user(&request).await {
        Ok(_) => Ok(Json(
            serde_json::to_value(ApiResponse::<()>::success_msg("更新用户成功"))
                .unwrap_or_default(),
        )),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 删除用户
pub async fn delete_user(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.user_service.delete_user(id).await {
        Ok(_) => {
            state
                .audit_service
                .record(&template_studio_services::audit_service::AuditEntry {
                    user_id: auth_user.user_id,
                    username: auth_user.username.clone(),
                    action: "user.delete".to_string(),
                    resource_type: "user".to_string(),
                    resource_id: Some(id.to_string()),
                    detail: None,
                    ip: None,
                    user_agent: None,
                })
                .await;
            Ok(Json(
                serde_json::to_value(ApiResponse::<()>::success_msg("删除用户成功"))
                    .unwrap_or_default(),
            ))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 分配角色
pub async fn assign_roles(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    Json(request): Json<AssignRolesRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state
        .user_service
        .assign_roles(user_id, &request.role_ids)
        .await
    {
        Ok(_) => Ok(Json(
            serde_json::to_value(ApiResponse::<()>::success_msg("分配角色成功"))
                .unwrap_or_default(),
        )),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

fn error_response(
    status: StatusCode,
    message: &str,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Err((
        status,
        Json(json!({
            "code": status.as_u16() as i32,
            "message": message
        })),
    ))
}
