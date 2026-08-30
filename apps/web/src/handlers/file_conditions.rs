//! 文件条件管理处理器

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use template_studio_shared::models::auth::AuthUser;

pub type AppState = super::super::AppState;

use template_studio_template_core::Condition;

/// 获取文件条件的请求参数
#[derive(Debug, Deserialize)]
pub struct GetFileConditionRequest {
    #[serde(rename = "templateId")]
    pub template_id: i64,
    #[serde(rename = "filePath")]
    pub file_path: String,
}

/// 获取文件条件
/// GET /api/v1/editor/file-conditions?templateId=:templateId&filePath=:filePath
pub async fn get_file_condition(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<GetFileConditionRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(resp) =
        crate::handlers::access::ensure_template_access(&state, &auth_user, params.template_id)
            .await
    {
        return Err(resp);
    }

    match state
        .file_conditions_service
        .get_file_condition(params.template_id, &params.file_path)
        .await
    {
        Ok(Some(condition)) => {
            let condition_json = serde_json::to_value(&condition).unwrap_or_default();
            Ok(Json(json!({
                "code": 0,
                "message": "OK",
                "data": {
                    "template_id": params.template_id,
                    "file_path": params.file_path,
                    "condition": condition_json
                }
            })))
        }
        Ok(None) => Ok(Json(json!({
            "code": 0,
            "message": "OK",
            "data": {
                "template_id": params.template_id,
                "file_path": params.file_path,
                "condition": null
            }
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 设置文件条件的请求体
#[derive(Debug, Deserialize)]
pub struct SetFileConditionRequest {
    #[serde(rename = "templateId")]
    pub template_id: i64,
    #[serde(rename = "filePath")]
    pub file_path: String,
    pub condition: Condition,
}

/// 设置文件条件
/// POST /api/v1/editor/file-conditions
pub async fn set_file_condition(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<SetFileConditionRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(resp) =
        crate::handlers::access::ensure_template_access(&state, &auth_user, req.template_id).await
    {
        return Err(resp);
    }

    match state
        .file_conditions_service
        .set_file_condition(req.template_id, &req.file_path, req.condition)
        .await
    {
        Ok(()) => Ok(Json(json!({
            "code": 0,
            "message": "条件设置成功",
            "data": {
                "template_id": req.template_id,
                "file_path": req.file_path
            }
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 删除文件条件的请求参数
#[derive(Debug, Deserialize)]
pub struct DeleteFileConditionRequest {
    #[serde(rename = "templateId")]
    pub template_id: i64,
    #[serde(rename = "filePath")]
    pub file_path: String,
}

/// 删除文件条件
/// DELETE /api/v1/editor/file-conditions
pub async fn delete_file_condition(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<DeleteFileConditionRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(resp) =
        crate::handlers::access::ensure_template_access(&state, &auth_user, req.template_id).await
    {
        return Err(resp);
    }

    match state
        .file_conditions_service
        .delete_file_condition(req.template_id, &req.file_path)
        .await
    {
        Ok(()) => Ok(Json(json!({
            "code": 0,
            "message": "条件删除成功",
            "data": {
                "template_id": req.template_id,
                "file_path": req.file_path
            }
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 导出条件为 YAML
/// GET /api/v1/editor/templates/:templateId/conditions/export
pub async fn export_conditions_yaml(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(template_id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(resp) =
        crate::handlers::access::ensure_template_access(&state, &auth_user, template_id).await
    {
        return Err(resp);
    }

    match state
        .file_conditions_service
        .export_conditions_yaml(template_id)
        .await
    {
        Ok(yaml_content) => Ok(Json(json!({
            "code": 0,
            "message": "导出成功",
            "data": {
                "template_id": template_id,
                "yaml": yaml_content
            }
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 导入条件 YAML 的请求体
#[derive(Debug, Deserialize)]
pub struct ImportConditionsRequest {
    pub yaml: String,
}

/// 从 YAML 导入条件
/// POST /api/v1/editor/templates/:templateId/conditions/import
pub async fn import_conditions_yaml(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(template_id): Path<i64>,
    Json(req): Json<ImportConditionsRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(resp) =
        crate::handlers::access::ensure_template_access(&state, &auth_user, template_id).await
    {
        return Err(resp);
    }

    match state
        .file_conditions_service
        .import_conditions_yaml(template_id, &req.yaml)
        .await
    {
        Ok(count) => Ok(Json(json!({
            "code": 0,
            "message": format!("成功导入 {} 个条件", count),
            "data": {
                "template_id": template_id,
                "count": count
            }
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 评估文件条件的请求体
#[derive(Debug, Deserialize)]
pub struct EvaluateConditionRequest {
    #[serde(rename = "templateId")]
    pub template_id: i64,
    #[serde(rename = "filePath")]
    pub file_path: String,
    pub variables: Value,
}

/// 评估文件条件
/// POST /api/v1/editor/file-conditions/evaluate
pub async fn evaluate_file_condition(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<EvaluateConditionRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(resp) =
        crate::handlers::access::ensure_template_access(&state, &auth_user, req.template_id).await
    {
        return Err(resp);
    }

    match state
        .file_conditions_service
        .should_generate_file(req.template_id, &req.file_path, &req.variables)
        .await
    {
        Ok(result) => Ok(Json(json!({
            "code": 0,
            "message": if result { "文件将生成" } else { "文件将不生成" },
            "data": {
                "template_id": req.template_id,
                "file_path": req.file_path,
                "should_generate": result
            }
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 错误响应辅助函数
fn error_response(
    status: StatusCode,
    message: &str,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Err((
        status,
        Json(json!({
            "code": status.as_u16(),
            "message": message
        })),
    ))
}
