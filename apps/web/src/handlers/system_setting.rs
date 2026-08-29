use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use template_studio_shared::models::system_setting::*;
use validator::Validate;

pub type AppState = super::super::AppState;

/// 获取设置列表
pub async fn get_settings(
    State(state): State<AppState>,
    Query(query): Query<GetSettingsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state
        .system_setting_service
        .get_settings(query.group.as_deref(), query.key.as_deref())
        .await
    {
        Ok(settings) => Ok(Json(json!({
            "code": 0,
            "message": "OK",
            "data": settings
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 公开获取设置（按 group）
pub async fn get_public_settings(
    State(state): State<AppState>,
    Path(group): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state
        .system_setting_service
        .get_settings(Some(&group), None)
        .await
    {
        Ok(settings) => Ok(Json(json!({
            "code": 0,
            "message": "OK",
            "data": settings
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 更新单个设置
pub async fn update_setting(
    State(state): State<AppState>,
    Json(request): Json<UpdateSettingRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.system_setting_service.update_setting(&request).await {
        Ok(()) => Ok(Json(json!({
            "code": 0,
            "message": "更新设置成功"
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 批量更新设置
pub async fn batch_update_settings(
    State(state): State<AppState>,
    Json(request): Json<BatchUpdateSettingsRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state
        .system_setting_service
        .batch_update_settings(&request)
        .await
    {
        Ok(()) => Ok(Json(json!({
            "code": 0,
            "message": "批量更新设置成功"
        }))),
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
