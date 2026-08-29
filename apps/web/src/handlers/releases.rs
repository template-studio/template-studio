//! 模板版本发布处理器

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use template_studio_shared::models::release::*;
use validator::Validate;

pub type AppState = super::super::AppState;

/// 创建发布版本
/// POST /api/v1/templates/:id/releases
pub async fn create_release(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<CreateReleaseRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证请求参数
    if let Err(errors) = payload.validate() {
        return error_response(
            StatusCode::BAD_REQUEST,
            &format!("参数验证失败: {}", errors),
        );
    }

    // TODO: 从认证上下文获取创建者信息
    let creator_id = 1i64;
    let creator_name = "System".to_string();

    match state
        .release_service
        .create_release(id, payload, creator_id, creator_name)
        .await
    {
        Ok(response) => Ok(Json(json!({
            "code": 0,
            "message": "发布成功",
            "data": response
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 获取模板的所有版本列表
/// GET /api/v1/templates/:id/releases
pub async fn list_releases(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.release_service.list_versions(id).await {
        Ok(versions) => Ok(Json(json!({
            "code": 0,
            "message": "OK",
            "data": VersionsListResponse {
                template_id: id,
                versions,
            }
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 回滚到指定版本
/// POST /api/v1/templates/:id/releases/:version/rollback
pub async fn rollback_version(
    State(state): State<AppState>,
    Path((id, version)): Path<(i64, String)>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.release_service.rollback_version(id, &version).await {
        Ok(response) => Ok(Json(json!({
            "code": 0,
            "message": "回滚成功",
            "data": response
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 标记版本为已弃用
/// POST /api/v1/templates/:id/releases/:version/deprecate
pub async fn deprecate_version(
    State(state): State<AppState>,
    Path((id, version)): Path<(i64, String)>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.release_service.deprecate_version(id, &version).await {
        Ok(()) => Ok(Json(json!({
            "code": 0,
            "message": "版本已标记为弃用"
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 重置到最新版本
/// POST /api/v1/templates/:id/releases/reset-to-latest
pub async fn reset_to_latest(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.release_service.reset_to_latest(id).await {
        Ok(response) => Ok(Json(json!({
            "code": 0,
            "message": format!("已重置到版本 {}", response.version),
            "data": response
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
