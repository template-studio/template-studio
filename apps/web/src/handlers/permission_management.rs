use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};

pub type AppState = super::super::AppState;

/// 权限列表
pub async fn list_permissions(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.permission_service.list_permissions().await {
        Ok(permissions) => Ok(Json(json!({
            "code": 0,
            "message": "OK",
            "data": {
                "list": permissions,
                "total": permissions.len()
            }
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 权限树
pub async fn get_permission_tree(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.permission_service.get_permission_tree().await {
        Ok(tree) => Ok(Json(json!({
            "code": 0,
            "data": tree
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
