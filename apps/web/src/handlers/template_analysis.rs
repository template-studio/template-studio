//! 模板变量分析处理器

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};

pub type AppState = super::super::AppState;

/// 分析模板变量
pub async fn analyze_variables(
    State(state): State<AppState>,
    Path(template_id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state
        .template_analysis_service
        .analyze_variables(template_id)
        .await
    {
        Ok(response) => Ok(Json(json!({
            "code": 0,
            "data": response,
            "message": "OK"
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 错误响应
fn error_response(status: StatusCode, message: &str) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Err((
        status,
        Json(json!({
            "code": status.as_u16() as i32,
            "message": message
        })),
    ))
}
