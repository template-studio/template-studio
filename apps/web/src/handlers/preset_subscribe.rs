//! 预设变量订阅处理器

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use template_studio_shared::models::preset_subscribe::*;
use validator::Validate;

pub type AppState = super::super::AppState;

/// 获取订阅列表
pub async fn get_subscribe_list(
    State(state): State<AppState>,
    Path(template_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let template_id_i64 = match template_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "无效的模板ID"),
    };

    let subscribe_path = state.storage_manager.get_template_meta_subscribe_path(template_id_i64);

    match state
        .preset_subscribe_service
        .get_subscribe_list(&template_id, &subscribe_path)
        .await
    {
        Ok(response) => Ok(Json(json!({
            "code": 0,
            "data": response,
            "message": "获取订阅列表成功"
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 添加订阅
pub async fn subscribe(
    State(state): State<AppState>,
    Json(request): Json<SubscribeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证请求数据
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    let template_id = match request.template_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "无效的模板ID"),
    };

    let subscribe_path = state.storage_manager.get_template_meta_subscribe_path(template_id);

    match state
        .preset_subscribe_service
        .subscribe(request, &subscribe_path)
        .await
    {
        Ok(()) => Ok(Json(json!({
            "code": 0,
            "message": "订阅成功"
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 取消订阅
pub async fn unsubscribe(
    State(state): State<AppState>,
    Path((template_id, preset_id)): Path<(String, u64)>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let template_id_i64 = match template_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "无效的模板ID"),
    };

    let subscribe_path = state.storage_manager.get_template_meta_subscribe_path(template_id_i64);
    let request = UnsubscribeRequest {
        template_id,
        preset_id,
    };

    match state
        .preset_subscribe_service
        .unsubscribe(request, &subscribe_path)
        .await
    {
        Ok(()) => Ok(Json(json!({
            "code": 0,
            "message": "取消订阅成功"
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 获取预设变量列表（兼容外部API格式）
pub async fn get_preset_variables(
    State(state): State<AppState>,
    Path(template_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let template_id_i64 = match template_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "无效的模板ID"),
    };

    let subscribe_path = state.storage_manager.get_template_meta_subscribe_path(template_id_i64);

    match state
        .preset_subscribe_service
        .get_preset_variables(&template_id, &subscribe_path)
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
