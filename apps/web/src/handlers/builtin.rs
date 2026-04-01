use axum::{
    extract::State,
    response::Json,
};
use serde_json::Value;

use template_studio_template_core::get_builtin_functions_response;

pub type AppState = super::super::AppState;

/// 获取内置函数列表
pub async fn get_builtin_functions(
    State(_state): State<AppState>,
) -> Result<Json<Value>, (axum::http::StatusCode, Json<Value>)> {
    let response = get_builtin_functions_response();

    Ok(Json(serde_json::json!({
        "code": 0,
        "data": response,
        "message": "获取内置函数成功"
    })))
}
