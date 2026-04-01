//! 模板变量处理器

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};

pub type AppState = super::super::AppState;

/// 获取模板变量
pub async fn get_variables(
    State(state): State<AppState>,
    Path(template_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let template_id_i64 = match template_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "无效的模板ID"),
    };

    match state
        .template_variables_service
        .get_variables(template_id_i64)
        .await
    {
        Ok(content) => {
            // 包装成原始API格式以适配前端
            Ok(Json(json!({
                "code": 0,
                "message": "OK",
                "data": {
                    "templateExpose": {
                        "id": 0,
                        "templateId": template_id_i64,
                        "fieldSchemaJson": content,
                        "version": "1.0",
                        "description": "",
                        "createdAt": "",
                        "updatedAt": ""
                    }
                }
            })))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 保存模板变量
pub async fn save_variables(
    State(state): State<AppState>,
    Path(template_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let template_id_i64 = match template_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "无效的模板ID"),
    };

    // 从 payload 中提取 content 字段
    let content = match payload.get("content") {
        Some(c) => match c.as_str() {
            Some(s) => s,
            None => return error_response(StatusCode::BAD_REQUEST, "content 必须是字符串"),
        },
        None => return error_response(StatusCode::BAD_REQUEST, "缺少 content 字段"),
    };

    match state
        .template_variables_service
        .save_variables(template_id_i64, content)
        .await
    {
        Ok(_) => Ok(Json(json!({
            "code": 0,
            "message": "保存成功"
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 获取测试数据
pub async fn get_test_data(
    State(state): State<AppState>,
    Path(template_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let template_id_i64 = match template_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "无效的模板ID"),
    };

    match state
        .template_variables_service
        .get_test_data(template_id_i64)
        .await
    {
        Ok(content) => Ok(Json(json!({
            "code": 0,
            "data": content,
            "message": "OK"
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 保存测试数据
pub async fn save_test_data(
    State(state): State<AppState>,
    Path(template_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let template_id_i64 = match template_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "无效的模板ID"),
    };

    // 从 payload 中提取 content 字段
    let content = match payload.get("content") {
        Some(c) => match c.as_str() {
            Some(s) => s,
            None => return error_response(StatusCode::BAD_REQUEST, "content 必须是字符串"),
        },
        None => return error_response(StatusCode::BAD_REQUEST, "缺少 content 字段"),
    };

    match state
        .template_variables_service
        .save_test_data(template_id_i64, content)
        .await
    {
        Ok(_) => Ok(Json(json!({
            "code": 0,
            "message": "保存成功"
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
