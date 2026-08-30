use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};
use template_studio_shared::utils::response::ApiResponse;

pub type AppState = super::super::AppState;

/// 权限列表
pub async fn list_permissions(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.permission_service.list_permissions().await {
        Ok(permissions) => Ok(Json(
            serde_json::to_value(ApiResponse::success_with_message(
                json!({
                    "list": permissions,
                    "total": permissions.len()
                }),
                "OK",
            ))
            .unwrap_or_default(),
        )),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 权限树
pub async fn get_permission_tree(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.permission_service.get_permission_tree().await {
        Ok(tree) => Ok(Json(
            serde_json::to_value(ApiResponse::success_with_message(tree, "操作成功"))
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
