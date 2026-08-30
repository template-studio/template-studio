//! 模板渲染处理器

use axum::{extract::State, http::StatusCode, response::Json};
use serde::{de::Error, Deserialize, Deserializer};
use serde_json::{json, Value};
use template_studio_shared::utils::response::ApiResponse;

pub type AppState = super::super::AppState;

/// 辅助类型：可以反序列化字符串或整数
fn deserialize_string_or_int<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        _ => Err(Error::custom("expected string or number")),
    }
}

/// 渲染请求
#[derive(Debug, Deserialize)]
pub struct RenderRequest {
    #[serde(rename = "templateId", deserialize_with = "deserialize_string_or_int")]
    pub template_id: String,
    #[serde(rename = "filePath")]
    pub file_path: String,
    pub variables: Option<Value>,
}

/// 渲染文件树请求
#[derive(Debug, Deserialize)]
pub struct RenderFileTreeRequest {
    #[serde(rename = "templateId", deserialize_with = "deserialize_string_or_int")]
    pub template_id: String,
    pub variables: Option<Value>,
}

/// 渲染单个模板文件
pub async fn render_file(
    State(state): State<AppState>,
    Json(payload): Json<RenderRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 解析 templateId
    let template_id = match payload.template_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "无效的模板ID"),
    };

    // 使用传入的 variables，如果没有则使用空对象
    let variables = payload.variables.unwrap_or_else(|| json!({}));

    match state
        .template_render_service
        .render_file(template_id, &payload.file_path, &variables)
        .await
    {
        Ok(result) => Ok(Json(
            serde_json::to_value(ApiResponse::success_with_message(result, "OK"))
                .unwrap_or_default(),
        )),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 渲染整个文件树
pub async fn render_file_tree(
    State(state): State<AppState>,
    Json(payload): Json<RenderFileTreeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 解析 templateId
    let template_id = match payload.template_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "无效的模板ID"),
    };

    // 使用传入的 variables，如果没有则使用空对象
    let variables = payload.variables.unwrap_or_else(|| json!({}));

    // 1. 获取文件树
    let file_tree_response = match state
        .file_tree_service
        .get_template_file_tree(template_id)
        .await
    {
        Ok(response) => response,
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("获取文件树失败: {}", e),
            )
        }
    };

    // 2. 渲染文件树
    match state
        .template_render_service
        .render_file_tree(template_id, file_tree_response.tree, &variables)
        .await
    {
        Ok(result) => Ok(Json(
            serde_json::to_value(ApiResponse::success_with_message(result, "OK"))
                .unwrap_or_default(),
        )),
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
