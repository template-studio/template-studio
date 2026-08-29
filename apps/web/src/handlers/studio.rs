use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use template_studio_shared::models::studio::*;

pub type AppState = super::super::AppState;

/// Studio首页数据
pub async fn studio_index(
    State(state): State<AppState>,
    Query(query): Query<StudioIndexRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.template_service.get_studio_index(query).await {
        Ok(response) => Ok(Json(json!({
            "code": 0,
            "data": response,
            "message": "OK"
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 错误响应
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
