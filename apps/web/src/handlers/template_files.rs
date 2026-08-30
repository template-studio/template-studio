//! 模板文件处理器（preview 和 generate 功能）

use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::{Json, Response},
};
use serde_json::{json, Value};
use template_studio_shared::models::release::*;
use template_studio_shared::utils::error::AppError;
use template_studio_shared::utils::response::ApiResponse;

pub type AppState = super::super::AppState;

/// 预览模板文件（编辑器预览，从工作目录读取）
/// POST /api/v1/template-files/preview
pub async fn preview_template_file(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 解析参数（兼容字符串和数字）
    let template_id = match payload.get("templateId") {
        Some(v) => match v {
            serde_json::Value::Number(n) => n.as_i64().unwrap_or(0),
            serde_json::Value::String(s) => s.parse::<i64>().unwrap_or(0),
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"code": 400, "message": "templateId 类型错误"})),
                ))
            }
        },
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"code": 400, "message": "templateId 缺失"})),
            ))
        }
    };

    let file_path = match payload.get("filePath") {
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"code": 400, "message": "filePath 缺失或类型错误"})),
            ))
        }
    };

    let variables = payload
        .get("variables")
        .cloned()
        .unwrap_or_else(|| json!({}));

    // 从工作目录（开发模式）读取
    let template_path = state.storage_manager.get_template_path(template_id);

    match state
        .template_render_service
        .render_file_from_path(&template_path, &file_path, &variables)
        .await
    {
        Ok(result) => {
            let response = TemplateRenderData {
                file_content: result.file_content,
                file_name: result.file_name,
                version: None, // preview 不返回版本号
            };

            Ok(Json(
                serde_json::to_value(ApiResponse::success_with_message(response, "OK"))
                    .unwrap_or_default(),
            ))
        }
        Err(AppError::NotFound(msg)) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"code": 404, "message": msg})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code": 500, "message": e.to_string()})),
        )),
    }
}

/// 生成模板文件（生产模式，从发布版本读取）
/// POST /api/v1/template-files/generate
pub async fn generate_template_file(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 解析参数（兼容字符串和数字）
    let template_id = match payload.get("templateId") {
        Some(v) => match v {
            serde_json::Value::Number(n) => n.as_i64().unwrap_or(0),
            serde_json::Value::String(s) => s.parse::<i64>().unwrap_or(0),
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"code": 400, "message": "templateId 类型错误"})),
                ))
            }
        },
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"code": 400, "message": "templateId 缺失"})),
            ))
        }
    };

    let file_path = match payload.get("filePath") {
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"code": 400, "message": "filePath 缺失或类型错误"})),
            ))
        }
    };

    let variables = payload
        .get("variables")
        .cloned()
        .unwrap_or_else(|| json!({}));

    // 1. 查询当前最新版本
    let version = match state.release_service.get_latest_version(template_id).await {
        Ok(v) => v.clone(),
        Err(e) => return error_response(StatusCode::NOT_FOUND, &e.to_string()),
    };

    // 2. 从发布版本目录读取（version 经存储层路径校验）
    let release_path = state
        .storage_manager
        .get_release_path(template_id, &version)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"code": 400, "message": e.to_string()})),
            )
        })?;

    match state
        .template_render_service
        .render_file_from_path(&release_path, &file_path, &variables)
        .await
    {
        Ok(result) => {
            let response = TemplateRenderData {
                file_content: result.file_content,
                file_name: result.file_name,
                version: Some(version), // generate 返回使用的版本号
            };

            Ok(Json(
                serde_json::to_value(ApiResponse::success_with_message(response, "OK"))
                    .unwrap_or_default(),
            ))
        }
        Err(AppError::NotFound(msg)) => error_response(StatusCode::NOT_FOUND, &msg),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 预览整个文件树（编辑器预览模式，从工作目录读取）
/// POST /api/v1/template-files/preview-tree
pub async fn preview_file_tree(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 解析参数（兼容字符串和数字）
    let template_id = match payload.get("templateId") {
        Some(v) => match v {
            serde_json::Value::Number(n) => n.as_i64().unwrap_or(0),
            serde_json::Value::String(s) => s.parse::<i64>().unwrap_or(0),
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"code": 400, "message": "templateId 类型错误"})),
                ))
            }
        },
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"code": 400, "message": "templateId 缺失"})),
            ))
        }
    };
    let variables = payload
        .get("variables")
        .cloned()
        .unwrap_or_else(|| json!({}));

    // 1. 获取文件树
    let file_tree_response = match state
        .file_tree_service
        .get_template_file_tree(template_id)
        .await
    {
        Ok(response) => response,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"code": 500, "message": format!("获取文件树失败: {}", e)})),
            ))
        }
    };

    // 2. 从工作目录读取并渲染文件树
    let _template_path = state.storage_manager.get_template_path(template_id);

    match state
        .template_render_service
        .render_file_tree(template_id, file_tree_response.tree, &variables)
        .await
    {
        Ok(result) => Ok(Json(
            serde_json::to_value(ApiResponse::success_with_message(result, "OK"))
                .unwrap_or_default(),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code": 500, "message": e.to_string()})),
        )),
    }
}

/// 生成整个文件树（生产模式，从发布版本读取）
/// POST /api/v1/template-files/generate-tree
pub async fn generate_file_tree(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 解析参数（兼容字符串和数字）
    let template_id = match payload.get("templateId") {
        Some(v) => match v {
            serde_json::Value::Number(n) => n.as_i64().unwrap_or(0),
            serde_json::Value::String(s) => s.parse::<i64>().unwrap_or(0),
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"code": 400, "message": "templateId 类型错误"})),
                ))
            }
        },
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"code": 400, "message": "templateId 缺失"})),
            ))
        }
    };
    let variables = payload
        .get("variables")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let version_option = payload.get("version").and_then(|v| v.as_str());

    // 1. 确定要使用的版本
    let version = if let Some(v) = version_option {
        v.to_string()
    } else {
        match state.release_service.get_latest_version(template_id).await {
            Ok(v) => v,
            Err(e) => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"code": 404, "message": e.to_string()})),
                ))
            }
        }
    };

    // 2. 确定发布版本目录（用于验证版本存在；version 经存储层路径校验）
    let _release_path = state
        .storage_manager
        .get_release_path(template_id, &version)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"code": 400, "message": e.to_string()})),
            )
        })?;

    // 3. 获取文件树（从发布版本）
    // 注意：由于发布版本目录结构相同，可以直接使用同一个文件树
    let file_tree_response = match state
        .file_tree_service
        .get_template_file_tree(template_id)
        .await
    {
        Ok(response) => response,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"code": 500, "message": format!("获取文件树失败: {}", e)})),
            ))
        }
    };

    // 4. 渲染文件树
    match state
        .template_render_service
        .render_file_tree(template_id, file_tree_response.tree, &variables)
        .await
    {
        Ok(result) => {
            // 构建响应，添加版本信息
            let mut response_data = serde_json::json!(result);
            if let Some(obj) = response_data.as_object_mut() {
                obj.insert("version".to_string(), serde_json::json!(version));
            }
            Ok(Json(
                serde_json::to_value(ApiResponse::success_with_message(response_data, "OK"))
                    .unwrap_or_default(),
            ))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"code": 500, "message": e.to_string()})),
        )),
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

/// 获取模板变量定义（生产模式，从发布版本读取）
/// GET /api/v1/template-files/variables?templateId={id}&version={version}
pub async fn get_template_variables(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 解析 templateId
    let template_id = match params.get("templateId") {
        Some(id_str) => match id_str.parse::<i64>() {
            Ok(id) => id,
            Err(_) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"code": 400, "message": "templateId 格式错误"})),
                ))
            }
        },
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"code": 400, "message": "templateId 缺失"})),
            ))
        }
    };

    let version_option = params.get("version");

    // 1. 确定要使用的版本
    let version = if let Some(v) = version_option {
        v.to_string()
    } else {
        // 没有版本号，查询数据库找到最新版本
        match state.release_service.get_latest_version(template_id).await {
            Ok(v) => v,
            Err(e) => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(
                        json!({"code": 404, "message": format!("该模板暂无发布版本，请先在编辑器中发布版本: {}", e)}),
                    ),
                ))
            }
        }
    };

    // 2. 从发布版本目录读取变量定义（version 经存储层路径校验）
    let release_path = state
        .storage_manager
        .get_release_path(template_id, &version)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"code": 400, "message": e.to_string()})),
            )
        })?;
    let variables_json_path =
        std::path::Path::new(&release_path).join(".meta/variables/variables.json");

    // 直接读取文件内容，文件本身就是变量定义JSON
    let field_schema_json = tokio::fs::read_to_string(&variables_json_path)
        .await
        .unwrap_or_else(|_| "{}".to_string());

    Ok(Json(
        serde_json::to_value(ApiResponse::success_with_message(
            json!({
                "fieldSchemaJson": field_schema_json,
                "version": version
            }),
            "OK",
        ))
        .unwrap_or_default(),
    ))
}

/// 生成并下载ZIP文件（生产模式，从发布版本读取）
/// POST /api/v1/template-files/generate-zip
pub async fn generate_zip(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Response, (StatusCode, Json<Value>)> {
    // 解析参数（兼容字符串和数字）
    let template_id = match payload.get("templateId") {
        Some(v) => match v {
            serde_json::Value::Number(n) => n.as_i64().unwrap_or(0),
            serde_json::Value::String(s) => s.parse::<i64>().unwrap_or(0),
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"code": 400, "message": "templateId 类型错误"})),
                ))
            }
        },
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"code": 400, "message": "templateId 缺失"})),
            ))
        }
    };

    let variables = payload
        .get("variables")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let version_option = payload.get("version").and_then(|v| v.as_str());
    let file_name = payload
        .get("fileName")
        .and_then(|v| v.as_str())
        .unwrap_or("project");

    // 1. 确定要使用的版本
    let version = if let Some(v) = version_option {
        v.to_string()
    } else {
        match state.release_service.get_latest_version(template_id).await {
            Ok(v) => v,
            Err(e) => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"code": 404, "message": e.to_string()})),
                ))
            }
        }
    };

    // 2. 确定发布版本目录（用于验证版本存在；version 经存储层路径校验）
    let _release_path = state
        .storage_manager
        .get_release_path(template_id, &version)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"code": 400, "message": e.to_string()})),
            )
        })?;

    // 3. 获取文件树
    let file_tree_response = match state
        .file_tree_service
        .get_template_file_tree(template_id)
        .await
    {
        Ok(response) => response,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"code": 500, "message": format!("获取文件树失败: {}", e)})),
            ))
        }
    };

    // 4. 渲染文件树
    let render_result = match state
        .template_render_service
        .render_file_tree(template_id, file_tree_response.tree, &variables)
        .await
    {
        Ok(result) => result,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"code": 500, "message": format!("渲染文件树失败: {}", e)})),
            ))
        }
    };

    // 5. 生成ZIP文件
    let zip_bytes = match create_zip_from_rendered_tree(&render_result).await {
        Ok(bytes) => bytes,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"code": 500, "message": format!("生成ZIP失败: {}", e)})),
            ))
        }
    };

    // 6. 返回ZIP文件
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/zip")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}.zip\"", file_name),
        )
        .body(Body::from(zip_bytes))
        .unwrap())
}

/// 从渲染后的文件树创建ZIP
async fn create_zip_from_rendered_tree(
    render_result: &template_studio_services::template_render_service::RenderFileTreeResponse,
) -> Result<Vec<u8>, String> {
    use std::io::{Cursor, Write};
    use zip::{write::FileOptions, ZipWriter};

    let buffer = Cursor::new(Vec::new());
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let mut zip = ZipWriter::new(buffer);

    // 辅助函数：递归添加文件到ZIP
    fn add_files_to_zip(
        zip: &mut ZipWriter<Cursor<Vec<u8>>>,
        nodes: &[template_studio_services::template_render_service::RenderedFileInfo],
        base_path: &str,
        options: FileOptions,
    ) -> Result<(), String> {
        for node in nodes {
            let full_path = if base_path.is_empty() {
                node.file_name.clone()
            } else {
                format!("{}/{}", base_path, node.file_name)
            };

            if node.is_directory == 1 {
                // 添加目录
                zip.add_directory(full_path.clone() + "/", options)
                    .map_err(|e| format!("添加目录失败: {}", e))?;

                // 递归处理子文件
                if let Some(children) = &node.children {
                    add_files_to_zip(zip, children, &full_path, options)?;
                }
            } else {
                // 跳过渲染失败的文件
                if node.render_error.is_some() {
                    continue;
                }

                // 添加文件
                if let Some(content) = &node.file_content {
                    zip.start_file(&full_path, options)
                        .map_err(|e| format!("开始文件失败: {}", e))?;
                    zip.write_all(content.as_bytes())
                        .map_err(|e| format!("写入文件失败: {}", e))?;
                }
            }
        }
        Ok(())
    }

    add_files_to_zip(&mut zip, &render_result.tree, "", options)
        .map_err(|e| format!("构建ZIP失败: {}", e))?;

    let buffer = zip.finish().map_err(|e| format!("完成ZIP失败: {}", e))?;

    Ok(buffer.into_inner())
}

/// 清除模板渲染缓存
/// POST /api/v1/template-files/clear-cache
pub async fn clear_cache(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 解析参数（兼容字符串和数字）
    let template_id = match payload.get("templateId") {
        Some(v) => match v {
            serde_json::Value::Number(n) => n.as_i64().unwrap_or(0),
            serde_json::Value::String(s) => s.parse::<i64>().unwrap_or(0),
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"code": 400, "message": "templateId 类型错误"})),
                ))
            }
        },
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"code": 400, "message": "templateId 缺失"})),
            ))
        }
    };

    // 清除缓存：L2 依赖树缓存 + 引擎已编译模板环境缓存
    state.template_render_service.clear_cache(template_id).await;
    template_studio_template_core::clear_template_cache();

    Ok(Json(
        serde_json::to_value(ApiResponse::success_with_message(
            json!({
                "templateId": template_id
            }),
            "缓存已清除",
        ))
        .unwrap_or_default(),
    ))
}
