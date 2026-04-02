use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use template_studio_shared::models::role::{CreateRoleRequest, UpdateRoleRequest, AssignPermissionsRequest};
use validator::Validate;

pub type AppState = super::super::AppState;

/// 角色列表
pub async fn list_roles(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.role_service.list_roles().await {
        Ok(roles) => Ok(Json(json!({
            "code": 0,
            "message": "OK",
            "data": {
                "list": roles,
                "total": roles.len()
            }
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 创建角色
pub async fn create_role(
    State(state): State<AppState>,
    Json(request): Json<CreateRoleRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.role_service.create_role(&request).await {
        Ok(id) => Ok(Json(json!({
            "code": 0,
            "data": { "id": id },
            "message": "创建角色成功"
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 更新角色
pub async fn update_role(
    State(state): State<AppState>,
    Json(request): Json<UpdateRoleRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.role_service.update_role(&request).await {
        Ok(_) => Ok(Json(json!({
            "code": 0,
            "message": "更新角色成功"
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 删除角色
pub async fn delete_role(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.role_service.delete_role(id).await {
        Ok(_) => Ok(Json(json!({
            "code": 0,
            "message": "删除角色成功"
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 分配权限
pub async fn assign_permissions(
    State(state): State<AppState>,
    Path(role_id): Path<i64>,
    Json(request): Json<AssignPermissionsRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.role_service.assign_permissions(role_id, &request.permission_ids).await {
        Ok(_) => Ok(Json(json!({
            "code": 0,
            "message": "分配权限成功"
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 获取角色权限
pub async fn get_role_permissions(
    State(state): State<AppState>,
    Path(role_id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.role_service.get_role_permissions(role_id).await {
        Ok(ids) => Ok(Json(json!({
            "code": 0,
            "data": { "permission_ids": ids }
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

fn error_response(status: StatusCode, message: &str) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Err((status, Json(json!({
        "code": status.as_u16() as i32,
        "message": message
    }))))
}
