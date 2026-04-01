//! 模板备份与恢复处理器
//!
//! 提供模板备份创建、预览和恢复的 HTTP 接口

use axum::{
    extract::{Multipart, Query, State},
    http::{header, StatusCode},
    response::{Json, Response},
};
use serde::Deserialize;
use serde_json::{json, Value};
use template_studio_shared::utils::error::AppError;

pub type AppState = super::super::AppState;

/// 创建备份请求参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBackupQuery {
    /// 模板 ID
    pub template_id: i64,
    /// 是否包含测试数据
    #[serde(default = "default_true")]
    pub include_test_data: bool,
    /// 是否包含文件条件
    #[serde(default = "default_true")]
    pub include_conditions: bool,
}

fn default_true() -> bool {
    true
}

/// 创建模板备份
/// GET /api/v1/backup/create
pub async fn create_backup(
    State(state): State<AppState>,
    Query(query): Query<CreateBackupQuery>,
) -> Result<Response, (StatusCode, Json<Value>)> {
    tracing::info!(
        "Creating backup for template {}, include_test_data: {}, include_conditions: {}",
        query.template_id,
        query.include_test_data,
        query.include_conditions
    );

    match state
        .backup_service
        .create_backup(
            query.template_id,
            query.include_test_data,
            query.include_conditions,
        )
        .await
    {
        Ok(zip_data) => {
            // 生成文件名
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let filename = format!("template_{}_backup_{}.tsbk", query.template_id, timestamp);

            // 返回文件下载响应
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/octet-stream")
                .header(
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", filename),
                )
                .body(axum::body::Body::from(zip_data))
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "code": 500,
                            "message": format!("构建响应失败: {}", e)
                        })),
                    )
                })?)
        }
        Err(AppError::NotFound(msg)) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"code": 404, "message": msg})),
        )),
        Err(AppError::Validation(msg)) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"code": 400, "message": msg})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code": 500, "message": e.to_string()})),
        )),
    }
}

/// 预览备份文件
/// POST /api/v1/backup/preview (multipart)
pub async fn preview_backup(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut backup_file: Option<Vec<u8>> = None;

    // 解析 multipart 数据
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("读取 multipart 字段失败: {:?}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "code": 400,
                "message": format!("读取请求数据失败: {}", e)
            })),
        )
    })? {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "backupFile" {
            let data = field.bytes().await.map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "code": 400,
                        "message": format!("读取备份文件失败: {}", e)
                    })),
                )
            })?;
            backup_file = Some(data.to_vec());
        }
    }

    let backup_data = backup_file.ok_or((
        StatusCode::BAD_REQUEST,
        Json(json!({
            "code": 400,
            "message": "缺少备份文件"
        })),
    ))?;

    match state.backup_service.preview_backup(&backup_data).await {
        Ok(preview) => Ok(Json(json!({
            "code": 0,
            "message": "OK",
            "data": preview
        }))),
        Err(AppError::Validation(msg)) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"code": 400, "message": msg})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code": 500, "message": e.to_string()})),
        )),
    }
}

/// 恢复备份
/// POST /api/v1/backup/restore (multipart)
pub async fn restore_backup(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut template_id: Option<i64> = None;
    let mut backup_file: Option<Vec<u8>> = None;

    // 解析 multipart 数据
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("读取 multipart 字段失败: {:?}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "code": 400,
                "message": format!("读取请求数据失败: {}", e)
            })),
        )
    })? {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "templateId" => {
                let value = field.text().await.map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "code": 400,
                            "message": format!("读取 templateId 失败: {}", e)
                        })),
                    )
                })?;
                template_id = value.parse().ok();
            }
            "backupFile" => {
                let data = field.bytes().await.map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "code": 400,
                            "message": format!("读取备份文件失败: {}", e)
                        })),
                    )
                })?;
                backup_file = Some(data.to_vec());
            }
            _ => {}
        }
    }

    let template_id = template_id.ok_or((
        StatusCode::BAD_REQUEST,
        Json(json!({
            "code": 400,
            "message": "缺少 templateId"
        })),
    ))?;

    let backup_data = backup_file.ok_or((
        StatusCode::BAD_REQUEST,
        Json(json!({
            "code": 400,
            "message": "缺少备份文件"
        })),
    ))?;

    tracing::info!(
        "Restoring backup for template {}, file size: {} bytes",
        template_id,
        backup_data.len()
    );

    match state.backup_service.restore_backup(template_id, &backup_data).await {
        Ok(result) => Ok(Json(json!({
            "code": 0,
            "message": if result.success { "恢复成功" } else { "恢复失败" },
            "data": result
        }))),
        Err(AppError::Validation(msg)) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"code": 400, "message": msg})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code": 500, "message": e.to_string()})),
        )),
    }
}
