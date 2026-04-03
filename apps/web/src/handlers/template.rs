use axum::{
    extract::{Path, Query, State, Multipart, Extension},
    http::StatusCode,
    response::Json,
};
use template_studio_shared::models::auth::AuthUser;
use serde::Deserialize;
use serde_json::{json, Value};
use template_studio_shared::models::template::*;
use validator::Validate;
use base64::{Engine as _, engine::general_purpose};
use bytes::Bytes;
use std::io::{Read, Write, Seek};
use zip::read::ZipArchive;
use zip::{ZipWriter, write::FileOptions};
use std::collections::HashSet;
use axum::response::Response;
use axum::body::Body;
use std::fs::File;

pub type AppState = super::super::AppState;

/// Git操作函数指针类型
type GitInitFn = fn(&std::path::PathBuf, &str, Option<&str>, Option<&str>) -> Result<(), anyhow::Error>;

// 全局Git初始化函数指针
static mut GIT_INIT_FN: Option<GitInitFn> = None;

/// 设置Git初始化函数
pub fn set_git_init_fn(f: GitInitFn) {
    unsafe {
        GIT_INIT_FN = Some(f);
    }
}

/// 执行Git初始化
async fn execute_git_init(
    repo_path: &std::path::PathBuf,
    template_name: &str,
) -> Result<(), anyhow::Error> {
    let f = unsafe {
        GIT_INIT_FN.ok_or_else(|| anyhow::anyhow!("Git初始化函数未设置"))?
    };

    // 在blocking task中执行同步Git操作
    let repo_path = repo_path.clone();
    let template_name = template_name.to_string();
    tokio::task::spawn_blocking(move || {
        f(&repo_path, &template_name, Some("Template Studio"), Some("template@studio.local"))
    }).await?
}

/// 切换推荐状态请求
#[derive(Debug, Deserialize, Validate)]
pub struct ToggleFeaturedRequest {
    #[validate(range(min = 1, message = "模板ID不能为空"))]
    pub id: i64,
    #[validate(range(min = 0, max = 1, message = "推荐状态必须为0或1"))]
    #[serde(rename = "isFeatured")]
    pub is_featured: i32,
}

/// 获取模板列表
pub async fn list_templates(
    State(state): State<AppState>,
    Query(query): Query<TemplateListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证查询参数
    if let Err(e) = query.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.template_service.list_templates_original_format(query).await {
        Ok(template_list_response) => Ok(Json(json!({
            "code": 0,
            "message": "OK",
            "data": template_list_response
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 公开模板列表（只返回 visibility=public, status=active）
pub async fn list_public_templates_studio(
    State(state): State<AppState>,
    Query(mut query): Query<TemplateListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    query.public_only = Some(true);
    list_templates(State(state), Query(query)).await
}

/// 获取模板详情
pub async fn get_template(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.template_service.get_template(id).await {
        Ok(Some(template)) => Ok(Json(json!({
            "code": 0,
            "data": template,
            "message": "获取模板成功"
        }))),
        Ok(None) => error_response(StatusCode::NOT_FOUND, "模板不存在"),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 获取模板详情（通过查询参数）
pub async fn get_template_detail(
    State(state): State<AppState>,
    Query(params): Query<TemplateDetailQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.template_service.get_template(params.id).await {
        Ok(Some(template)) => Ok(Json(json!({
            "code": 0,
            "data": template,
            "message": "获取模板成功"
        }))),
        Ok(None) => error_response(StatusCode::NOT_FOUND, "模板不存在"),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 模板详情查询参数
#[derive(Debug, Deserialize)]
pub struct TemplateDetailQuery {
    pub id: i64,
}

/// 获取文件内容查询参数
#[derive(Debug, Deserialize, Validate)]
pub struct FileContentQuery {
    #[validate(range(min = 1, message = "模板ID不能为空"))]
    #[serde(rename = "templateId")]
    pub template_id: i64,
    #[validate(length(min = 1, message = "文件路径不能为空"))]
    #[serde(rename = "filePath")]
    pub file_path: String,
}

/// 获取模板文件内容
pub async fn get_template_file_content(
    State(state): State<AppState>,
    Query(params): Query<FileContentQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证参数
    if let Err(e) = params.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    // 获取模板存储路径
    let template_path = state.storage_manager.get_template_path(params.template_id);
    let file_path = template_path.join(&params.file_path);

    tracing::info!("读取文件: template_id={}, file_path={:?}", params.template_id, file_path);

    // 检查文件是否存在
    if !file_path.exists() {
        return error_response(StatusCode::NOT_FOUND, "文件不存在");
    }

    // 读取文件内容
    match tokio::fs::read_to_string(&file_path).await {
        Ok(content) => {
            tracing::info!("文件读取成功: size={} bytes", content.len());
            Ok(Json(json!({
                "code": 0,
                "data": {
                    "content": content,
                    "filePath": params.file_path
                },
                "message": "获取文件内容成功"
            })))
        }
        Err(e) => {
            tracing::error!("文件读取失败: {:?}", e);
            // 如果是二进制文件，尝试以二进制方式读取
            if let Ok(bytes) = tokio::fs::read(&file_path).await {
                // 转换为base64返回
                let base64_content = general_purpose::STANDARD.encode(&bytes);
                Ok(Json(json!({
                    "code": 0,
                    "data": {
                        "content": base64_content,
                        "filePath": params.file_path,
                        "isBinary": true
                    },
                    "message": "获取文件内容成功"
                })))
            } else {
                error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
            }
        }
    }
}

/// 添加文件请求
#[derive(Debug, Deserialize, Validate)]
pub struct AddFileRequest {
    #[validate(range(min = 1, message = "模板ID不能为空"))]
    #[serde(rename = "templateId")]
    pub template_id: i64,
    #[validate(length(min = 1, message = "文件名不能为空"))]
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "parentPath")]
    pub parent_path: String,
    #[serde(rename = "isDirectory")]
    pub is_directory: i32,
}

/// 添加模板文件
pub async fn add_template_file(
    State(state): State<AppState>,
    Json(request): Json<AddFileRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证请求
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    // 获取模板存储路径
    let template_path = state.storage_manager.get_template_path(request.template_id);
    let parent_path = if request.parent_path.is_empty() {
        template_path.clone()
    } else {
        template_path.join(&request.parent_path)
    };

    // 创建文件或目录的完整路径
    let file_path = parent_path.join(&request.file_name);

    tracing::info!("创建文件: template_id={}, file_name={}, parent_path={}, is_directory={}",
        request.template_id, request.file_name, request.parent_path, request.is_directory);

    // 检查父目录是否存在
    if !parent_path.exists() {
        return error_response(StatusCode::NOT_FOUND, "父目录不存在");
    }

    // 检查文件是否已存在
    if file_path.exists() {
        return error_response(StatusCode::CONFLICT, "文件已存在");
    }

    // 创建目录或文件
    if request.is_directory == 1 {
        // 创建目录
        if let Err(e) = tokio::fs::create_dir_all(&file_path).await {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("创建目录失败: {}", e));
        }
        tracing::info!("目录创建成功: {:?}", file_path);
    } else {
        // 创建空文件
        if let Err(e) = tokio::fs::write(&file_path, b"").await {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("创建文件失败: {}", e));
        }
        tracing::info!("文件创建成功: {:?}", file_path);
    }

    // 计算相对路径（相对于模板根目录）
    let relative_path = match file_path.strip_prefix(&template_path) {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(_) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "无法计算相对路径"),
    };

    // 获取文件大小
    let file_size = if request.is_directory == 1 {
        0
    } else {
        match tokio::fs::metadata(&file_path).await {
            Ok(metadata) => metadata.len() as i64,
            Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("获取文件信息失败: {}", e)),
        }
    };

    // 计算MD5（仅文件）
    let md5_hash = if request.is_directory == 0 {
        match tokio::fs::read(&file_path).await {
            Ok(content) => {
                if content.is_empty() {
                    String::new()
                } else {
                    format!("{:x}", md5::compute(content))
                }
            }
            Err(_) => String::new(),
        }
    } else {
        String::new()
    };

    // 返回新创建的文件节点信息（与文件树格式一致）
    Ok(Json(json!({
        "code": 0,
        "data": {
            "id": chrono::Utc::now().timestamp_millis(), // 临时ID
            "fileName": request.file_name,
            "filePath": relative_path,
            "isDirectory": request.is_directory,
            "parentId": 0, // 前端会根据 parentPath 重新组织树结构
            "fileSize": file_size,
            "md5": md5_hash
        },
        "message": "文件创建成功"
    })))
}

/// 删除文件请求参数
#[derive(Debug, Deserialize, Validate)]
pub struct DeleteFileRequest {
    #[validate(range(min = 1, message = "模板ID不能为空"))]
    #[serde(rename = "templateId")]
    pub template_id: i64,
    #[validate(length(min = 1, message = "文件路径不能为空"))]
    #[serde(rename = "filePath")]
    pub file_path: String,
}

/// 删除模板文件
pub async fn delete_template_file(
    State(state): State<AppState>,
    Query(params): Query<DeleteFileRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证参数
    if let Err(e) = params.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    // 获取模板存储路径
    let template_path = state.storage_manager.get_template_path(params.template_id);
    let file_path = template_path.join(&params.file_path);

    tracing::info!("删除文件: template_id={}, file_path={:?}", params.template_id, file_path);

    // 检查文件是否存在
    if !file_path.exists() {
        return error_response(StatusCode::NOT_FOUND, "文件不存在");
    }

    // 删除文件或目录
    if let Err(e) = tokio::fs::remove_file(&file_path).await {
        // 如果是目录，尝试删除目录
        if let Err(dir_err) = tokio::fs::remove_dir_all(&file_path).await {
            tracing::error!("删除失败: {:?}, 目录删除也失败: {:?}", e, dir_err);
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("删除失败: {}", e));
        }
    }

    tracing::info!("文件删除成功: {:?}", file_path);

    Ok(Json(json!({
        "code": 0,
        "message": "文件删除成功"
    })))
}

/// 编辑文件请求
#[derive(Debug, Deserialize, Validate)]
pub struct EditFileRequest {
    #[validate(range(min = 1, message = "模板ID不能为空"))]
    #[serde(rename = "templateId")]
    pub template_id: i64,
    #[validate(length(min = 1, message = "文件路径不能为空"))]
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(rename = "content")]
    pub content: String,
}

/// 编辑（保存）模板文件
pub async fn edit_template_file(
    State(state): State<AppState>,
    Json(request): Json<EditFileRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证请求
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    // 获取模板存储路径
    let template_path = state.storage_manager.get_template_path(request.template_id);
    let file_path = template_path.join(&request.file_path);

    tracing::info!("保存文件: template_id={}, file_path={:?}", request.template_id, file_path);

    // 检查文件是否存在
    if !file_path.exists() {
        return error_response(StatusCode::NOT_FOUND, "文件不存在");
    }

    // 写入文件内容
    if let Err(e) = tokio::fs::write(&file_path, &request.content).await {
        tracing::error!("文件保存失败: {:?}", e);
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("文件保存失败: {}", e));
    }

    tracing::info!("文件保存成功: {:?}, size={} bytes", file_path, request.content.len());

    Ok(Json(json!({
        "code": 0,
        "message": "文件保存成功"
    })))
}

/// 移动文件请求
#[derive(Debug, Deserialize, Validate)]
pub struct MoveFileRequest {
    #[validate(range(min = 1, message = "模板ID不能为空"))]
    #[serde(rename = "templateId")]
    pub template_id: i64,
    #[validate(length(min = 1, message = "文件路径不能为空"))]
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[validate(length(min = 1, message = "新路径不能为空"))]
    #[serde(rename = "newPath")]
    pub new_path: String,
}

/// 移动（重命名）模板文件
pub async fn move_template_file(
    State(state): State<AppState>,
    Json(request): Json<MoveFileRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证请求
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    // 获取模板存储路径
    let template_path = state.storage_manager.get_template_path(request.template_id);
    let source_path = template_path.join(&request.file_path);
    let target_path = template_path.join(&request.new_path);

    tracing::info!("移动文件: template_id={}, from={:?}, to={:?}", request.template_id, source_path, target_path);

    // 检查源文件是否存在
    if !source_path.exists() {
        return error_response(StatusCode::NOT_FOUND, "源文件不存在");
    }

    // 检查目标路径是否已存在
    if target_path.exists() {
        return error_response(StatusCode::CONFLICT, "目标路径已存在");
    }

    // 检查目标父目录是否存在
    if let Some(target_parent) = target_path.parent() {
        if !target_parent.exists() {
            return error_response(StatusCode::NOT_FOUND, "目标父目录不存在");
        }
    }

    // 执行移动操作
    if let Err(e) = tokio::fs::rename(&source_path, &target_path).await {
        tracing::error!("文件移动失败: {:?}", e);

        // 如果是跨设备移动，尝试复制后删除
        if e.kind() == std::io::ErrorKind::InvalidInput {
            tracing::info!("跨设备移动，尝试复制后删除");
            if let Err(copy_err) = tokio::fs::copy(&source_path, &target_path).await {
                tracing::error!("文件复制失败: {:?}", copy_err);
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("文件移动失败: {}", copy_err));
            }
            // 删除源文件
            if let Err(del_err) = tokio::fs::remove_file(&source_path).await {
                tracing::error!("删除源文件失败: {:?}", del_err);
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("删除源文件失败: {}", del_err));
            }
        } else {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("文件移动失败: {}", e));
        }
    }

    tracing::info!("文件移动成功: {:?} -> {:?}", source_path, target_path);

    Ok(Json(json!({
        "code": 0,
        "message": "文件移动成功"
    })))
}

/// 获取模板类型列表
pub async fn get_template_types(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.template_service.get_template_types().await {
        Ok(template_types) => {
            let response = TemplateTypesResponse {
                template_types,
            };
            Ok(Json(json!({
                "code": 0,
                "data": response,
                "message": "OK"
            })))
        },
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 切换模板推荐状态
pub async fn toggle_featured(
    State(state): State<AppState>,
    Json(request): Json<ToggleFeaturedRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证请求数据
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.template_service.toggle_featured(request.id, request.is_featured).await {
        Ok(()) => Ok(Json(json!({
            "code": 0,
            "message": "切换推荐状态成功"
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 删除模板请求参数
#[derive(Debug, Deserialize)]
pub struct DeleteTemplateRequest {
    pub id: i64,
}

/// 删除模板
pub async fn delete_template(
    State(state): State<AppState>,
    Query(params): Query<DeleteTemplateRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.template_service.delete_template(params.id).await {
        Ok(()) => Ok(Json(json!({
            "code": 0,
            "message": "删除模板成功"
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 创建模板
pub async fn create_template(
    State(state): State<AppState>,
    Json(request): Json<CreateTemplateRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证请求数据
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    // 保存请求名称，用于Git初始化
    let template_name = request.name.clone();

    match state.template_service.create_template(request).await {
        Ok(template_id) => {
            // 模板创建成功后，初始化Git仓库
            let git_repo_path = state.storage_manager.get_template_path(template_id);

            match execute_git_init(&git_repo_path, &template_name).await {
                Ok(_) => {
                    tracing::info!("Git仓库初始化成功: template_id={}, path={:?}", template_id, git_repo_path);
                }
                Err(e) => {
                    tracing::error!("Git仓库初始化失败: template_id={}, error={}", template_id, e);
                    // 注意：这里不返回错误，因为模板已经创建成功
                    // Git初始化失败不应该影响模板创建
                }
            }

            Ok(Json(json!({
                "code": 0,
                "data": template_id,
                "message": "模板创建成功"
            })))
        }
        Err(e) => {
            // 根据错误类型返回不同的状态码
            let status = match e.to_string().contains("不存在") {
                true => StatusCode::NOT_FOUND,
                false => StatusCode::INTERNAL_SERVER_ERROR,
            };
            error_response(status, &e.to_string())
        }
    }
}

/// 错误响应
fn error_response(status: StatusCode, message: &str) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Err((status, Json(json!({
        "code": status.as_u16() as i32,
        "message": message
    }))))
}

/// 检查文件是否为文本文件
/// 通过读取文件前几个字节进行判断
fn is_text_file(content: &[u8]) -> bool {
    // 只检查前 8KB
    let sample_size = content.len().min(8192);
    let sample = &content[..sample_size];

    // 检查是否包含 NULL 字节（二进制文件的特征）
    if sample.contains(&0) {
        return false;
    }

    // 计算控制字符的比例（排除常见的文本控制字符）
    let control_count = sample.iter()
        .filter(|&&b| {
            // 排除常见的文本控制字符：\t, \n, \r, \f (换页符)
            b < 0x20 && b != 0x09 && b != 0x0A && b != 0x0D && b != 0x0C
        })
        .count();

    let control_ratio = control_count as f64 / sample_size as f64;

    // 如果控制字符比例超过 30%，认为是二进制文件
    if control_ratio > 0.3 {
        return false;
    }

    // 检查是否包含 UTF-8 无效序列
    if std::str::from_utf8(sample).is_err() {
        // 不是有效的 UTF-8，可能是二进制文件
        // 但有些文本文件可能使用其他编码（如 GBK），所以不直接返回 false
        // 可以进一步检查，但为了简化，我们假设非 UTF-8 也可能是文本
    }

    // 检查常见的高值字节比例（二进制文件通常有很多 >127 的字节）
    let high_byte_count = sample.iter().filter(|&&b| b > 127).count();
    let high_byte_ratio = high_byte_count as f64 / sample_size as f64;

    // 如果超过 80% 的字节都 > 127，可能是二进制文件
    if high_byte_ratio > 0.8 && sample_size > 100 {
        return false;
    }

    true
}

/// 上传代码文件
pub async fn upload_code(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut template_id: Option<i64> = None;
    let mut parent_path: Option<String> = None;
    let mut file_name: Option<String> = None;
    let mut file_content: Option<Bytes> = None;

    // 解析 multipart 数据
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("读取multipart字段失败: {:?}", e);
        (StatusCode::BAD_REQUEST, Json(json!({
            "code": StatusCode::BAD_REQUEST.as_u16() as i32,
            "message": format!("读取请求数据失败: {}", e)
        })))
    })? {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "templateId" => {
                let value = field.text().await.map_err(|e| {
                    tracing::error!("读取templateId失败: {:?}", e);
                    (StatusCode::BAD_REQUEST, Json(json!({
                        "code": StatusCode::BAD_REQUEST.as_u16() as i32,
                        "message": format!("读取templateId失败: {}", e)
                    })))
                })?;
                template_id = value.parse().ok();
            }
            "parentPath" => {
                let value = field.text().await.map_err(|e| {
                    tracing::error!("读取parentPath失败: {:?}", e);
                    (StatusCode::BAD_REQUEST, Json(json!({
                        "code": StatusCode::BAD_REQUEST.as_u16() as i32,
                        "message": format!("读取parentPath失败: {}", e)
                    })))
                })?;
                // 如果 parentPath 不为空，则使用；否则为根目录
                if !value.is_empty() {
                    parent_path = Some(value);
                }
            }
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                file_content = Some(Bytes::from(field.bytes().await.map_err(|e| {
                    tracing::error!("读取文件内容失败: {:?}", e);
                    (StatusCode::BAD_REQUEST, Json(json!({
                        "code": StatusCode::BAD_REQUEST.as_u16() as i32,
                        "message": format!("读取文件内容失败: {}", e)
                    })))
                })?));
            }
            _ => {
                // 忽略未知字段
            }
        }
    }

    // 验证必需参数
    let template_id = template_id.ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(json!({
            "code": StatusCode::BAD_REQUEST.as_u16() as i32,
            "message": "缺少templateId参数"
        })))
    })?;

    let file_name = file_name.ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(json!({
            "code": StatusCode::BAD_REQUEST.as_u16() as i32,
            "message": "缺少文件"
        })))
    })?;

    let file_content = file_content.ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(json!({
            "code": StatusCode::BAD_REQUEST.as_u16() as i32,
            "message": "缺少文件内容"
        })))
    })?;

    // 检查是否为文本文件
    let is_text = is_text_file(&file_content);

    if !is_text {
        tracing::warn!("上传的文件可能是二进制文件: {}", file_name);
        return Err((StatusCode::BAD_REQUEST, Json(json!({
            "code": StatusCode::BAD_REQUEST.as_u16() as i32,
            "message": "只支持上传文本文件"
        }))));
    }

    // 获取模板存储路径
    let template_path = state.storage_manager.get_template_path(template_id);

    // 构造文件保存路径
    let file_path = if let Some(pp) = parent_path {
        // 如果有父目录路径，使用它
        format!("{}/{}", pp, file_name)
    } else {
        // 否则为根目录
        file_name.clone()
    };

    let full_path = template_path.join(&file_path);

    // 确保父目录存在
    if let Some(parent) = full_path.parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                tracing::error!("创建目录失败: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                    "code": StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i32,
                    "message": format!("创建目录失败: {}", e)
                })))
            })?;
        }
    }

    // 检查文件是否已存在
    if full_path.exists() {
        return Err((StatusCode::CONFLICT, Json(json!({
            "code": StatusCode::CONFLICT.as_u16() as i32,
            "message": "文件已存在"
        }))));
    }

    // 保存文件
    tokio::fs::write(&full_path, &file_content).await.map_err(|e| {
        tracing::error!("写入文件失败: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "code": StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i32,
            "message": format!("写入文件失败: {}", e)
        })))
    })?;

    tracing::info!("文件上传成功: template_id={}, path={:?}, is_text={}", template_id, full_path, is_text);

    Ok(Json(json!({
        "code": 0,
        "data": {
            "fileName": file_name,
            "filePath": file_path,
            "isTextFile": is_text
        },
        "message": "文件上传成功"
    })))
}

/// 上传ZIP包
pub async fn upload_zip(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut template_id: Option<i64> = None;
    let mut zip_content: Option<Vec<u8>> = None;

    // 解析 multipart 数据
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("读取multipart字段失败: {:?}", e);
        (StatusCode::BAD_REQUEST, Json(json!({
            "code": StatusCode::BAD_REQUEST.as_u16() as i32,
            "message": format!("读取请求数据失败: {}", e)
        })))
    })? {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "templateId" => {
                let value = field.text().await.map_err(|e| {
                    tracing::error!("读取templateId失败: {:?}", e);
                    (StatusCode::BAD_REQUEST, Json(json!({
                        "code": StatusCode::BAD_REQUEST.as_u16() as i32,
                        "message": format!("读取templateId失败: {}", e)
                    })))
                })?;
                template_id = value.parse().ok();
            }
            "zipFile" => {
                zip_content = Some(field.bytes().await.map_err(|e| {
                    tracing::error!("读取ZIP文件失败: {:?}", e);
                    (StatusCode::BAD_REQUEST, Json(json!({
                        "code": StatusCode::BAD_REQUEST.as_u16() as i32,
                        "message": format!("读取ZIP文件失败: {}", e)
                    })))
                })?.to_vec());
            }
            _ => {}
        }
    }

    // 验证必需参数
    let template_id = template_id.ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(json!({
            "code": StatusCode::BAD_REQUEST.as_u16() as i32,
            "message": "缺少templateId参数"
        })))
    })?;

    let zip_content = zip_content.ok_or_else(|| {
        (StatusCode::BAD_REQUEST, Json(json!({
            "code": StatusCode::BAD_REQUEST.as_u16() as i32,
            "message": "缺少ZIP文件"
        })))
    })?;

    // 获取模板存储路径
    let template_path = state.storage_manager.get_template_path(template_id);

    // 确保模板目录存在
    if !template_path.exists() {
        tokio::fs::create_dir_all(&template_path).await.map_err(|e| {
            tracing::error!("创建模板目录失败: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "code": StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i32,
                "message": format!("创建模板目录失败: {}", e)
            })))
        })?;
    }

    // 在 blocking task 中解压 ZIP
    let template_path_clone = template_path.clone();
    let result = tokio::task::spawn_blocking(move || {
        process_zip_upload(template_path_clone, &zip_content)
    }).await.map_err(|e| {
        tracing::error!("解压ZIP失败: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "code": StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i32,
            "message": format!("解压ZIP失败: {}", e)
        })))
    })?;

    let (success_count, failed_files) = result.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "code": StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i32,
            "message": e
        })))
    })?;

    tracing::info!("ZIP上传成功: template_id={}, success_count={}, failed_files={:?}",
        template_id, success_count, failed_files);

    Ok(Json(json!({
        "code": 0,
        "data": {
            "successCount": success_count,
            "failedFiles": failed_files
        },
        "message": "ZIP包上传完成"
    })))
}

/// 检查模板是否已有自定义文件
/// 排除：.meta 文件夹、.git 文件夹、.gitignore、README.md
fn check_template_has_custom_files(template_path: &std::path::Path) -> Result<(), String> {
    // 如果模板目录不存在，说明是新模板，允许上传
    if !template_path.exists() {
        return Ok(());
    }

    // 遍历模板目录
    let entries = std::fs::read_dir(template_path)
        .map_err(|e| format!("读取模板目录失败: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let file_name = entry.file_name();

        // 转换为字符串
        let file_name_str = file_name.to_string_lossy();

        // 排除 .meta 文件夹、.git 文件夹、.gitignore、README.md
        if file_name_str == ".meta" || file_name_str == ".git" || file_name_str == ".gitignore" || file_name_str == "README.md" {
            continue;
        }

        // 如果存在其他文件或文件夹，说明已有自定义内容
        return Err(format!(
            "当前模板已经有自定义文件（{}），不允许上传ZIP。如需使用ZIP模板，请先清空当前模板文件。",
            file_name_str
        ));
    }

    Ok(())
}

/// 处理ZIP上传（在 blocking task 中执行）
fn process_zip_upload(
    template_path: std::path::PathBuf,
    zip_content: &[u8],
) -> Result<(usize, Vec<String>), String> {
    // 首先检查模板目录是否已有自定义文件
    check_template_has_custom_files(&template_path)?;

    let mut failed_files = Vec::new();
    let mut success_count = 0;

    // 需要排除的文件夹
    let excluded_dirs = HashSet::from([".git", "node_modules", ".meta"]);

    let cursor = std::io::Cursor::new(zip_content);
    let mut archive = ZipArchive::new(cursor).map_err(|e| format!("打开ZIP文件失败: {}", e))?;

    // 首先扫描是否需要特殊处理的文件
    let mut has_gitignore = false;
    let mut has_readme = false;
    let mut gitignore_content = String::new();
    let mut readme_content = Vec::new();

    // 收集所有文件信息
    let mut files_to_extract: Vec<(String, Vec<u8>)> = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("读取ZIP文件失败: {}", e))?;
        let file_path = file.name().to_string();

        // 检查是否在排除的文件夹中
        let path_parts: Vec<&str> = file_path.split('/').collect();
        let is_excluded = path_parts.iter()
            .take(path_parts.len().saturating_sub(1))
            .any(|part| excluded_dirs.contains(part));

        if is_excluded {
            tracing::info!("跳过排除的文件夹: {}", file_path);
            continue;
        }

        // 检查特殊文件
        if file_path == ".gitignore" {
            has_gitignore = true;
            let mut content = String::new();
            file.read_to_string(&mut content).map_err(|e| format!("读取.gitignore失败: {}", e))?;
            gitignore_content = content;
            continue;
        }

        if file_path == "README.md" || file_path.ends_with("/README.md") {
            has_readme = true;
            let mut content = Vec::new();
            file.read_to_end(&mut content).map_err(|e| format!("读取README.md失败: {}", e))?;
            readme_content = content;
            continue;
        }

        // 普通文件，读取内容
        if !file.name().ends_with('/') {
            let mut content = Vec::new();
            file.read_to_end(&mut content).map_err(|e| format!("读取文件{}失败: {}", file_path, e))?;
            files_to_extract.push((file_path, content));
        }
    }

    // 处理 .gitignore 合并
    if has_gitignore {
        let existing_gitignore_path = template_path.join(".gitignore");
        if existing_gitignore_path.exists() {
            // 读取现有的 .gitignore
            let existing_content = std::fs::read_to_string(&existing_gitignore_path)
                .unwrap_or_default();

            // 合并内容（去重）
            let mut combined_rules = HashSet::new();
            for line in existing_content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    combined_rules.insert(line.to_string());
                }
            }
            for line in gitignore_content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    combined_rules.insert(line.to_string());
                }
            }

            // 写入合并后的内容
            let merged_content = combined_rules.into_iter().collect::<Vec<_>>().join("\n");
            std::fs::write(&existing_gitignore_path, format!("{}\n", merged_content))
                .map_err(|e| format!("写入合并的.gitignore失败: {}", e))?;

            tracing::info!(".gitignore已合并");
        } else {
            // 直接写入新的 .gitignore
            std::fs::write(template_path.join(".gitignore"), &gitignore_content)
                .map_err(|e| format!("写入.gitignore失败: {}", e))?;
        }
    }

    // 处理 README.md 重命名
    if has_readme {
        let readme_upload_path = template_path.join("README.upload.md");
        std::fs::write(&readme_upload_path, readme_content)
            .map_err(|e| format!("写入README.upload.md失败: {}", e))?;
        tracing::info!("README.md已重命名为README.upload.md");
        success_count += 1;
    }

    // 提取普通文件
    for (file_path, content) in files_to_extract {
        let full_path = template_path.join(&file_path);

        // 确保父目录存在
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录{}失败: {}", parent.display(), e))?;
        }

        // 检查文件是否已存在
        if full_path.exists() {
            tracing::warn!("文件已存在，跳过: {}", file_path);
            failed_files.push(format!("{} (已存在)", file_path));
            continue;
        }

        // 写入文件
        std::fs::write(&full_path, content)
            .map_err(|e| {
                tracing::error!("写入文件{}失败: {:?}", file_path, e);
                format!("写入文件{}失败: {}", file_path, e)
            })?;

        success_count += 1;
    }

    Ok((success_count, failed_files))
}

/// 导出模板为ZIP
pub async fn export_template(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Response<Body>, (StatusCode, Json<Value>)> {
    // 获取模板存储路径
    let template_path = state.storage_manager.get_template_path(id);

    // 检查模板目录是否存在
    if !template_path.exists() {
        return Err((StatusCode::NOT_FOUND, Json(json!({
            "code": 404,
            "message": "模板目录不存在"
        }))));
    }

    // 在 blocking task 中打包 ZIP
    let template_path_clone = template_path.clone();
    let zip_bytes = tokio::task::spawn_blocking(move || {
        create_template_zip(template_path_clone)
    }).await.map_err(|e| {
        tracing::error!("创建ZIP失败: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "code": 500,
            "message": format!("创建ZIP失败: {}", e)
        })))
    })?
    .map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "code": 500,
            "message": e
        })))
    })?;

    // 返回 ZIP 文件
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/zip")
        .header("Content-Disposition", format!("attachment; filename=\"template_{}.zip\"", id))
        .body(Body::from(zip_bytes))
        .map_err(|e| {
            tracing::error!("构建响应失败: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "code": 500,
                "message": format!("构建响应失败: {}", e)
            })))
        })?;

    Ok(response)
}

/// 下载特定版本的模板
/// GET /api/v1/templates/:id/releases/:version/download
pub async fn download_template_version(
    State(state): State<AppState>,
    Path((id, version)): Path<(i64, String)>,
) -> Result<Response<Body>, (StatusCode, Json<Value>)> {
    // 构建版本存储路径
    let version_path = state.storage_manager.get_release_path(id, &version);

    // 检查版本目录是否存在
    if !version_path.exists() {
        return Err((StatusCode::NOT_FOUND, Json(json!({
            "code": 404,
            "message": format!("版本目录不存在: {}", version)
        }))));
    }

    // 在 blocking task 中打包 ZIP
    let version_path_clone = version_path.clone();
    let zip_bytes = tokio::task::spawn_blocking(move || {
        create_template_zip(version_path_clone)
    }).await.map_err(|e| {
        tracing::error!("创建ZIP失败: {:?}", e);
        format!("创建ZIP失败: {}", e)
    })
    .map_err(|e| {
        tracing::error!("打包任务失败: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "code": 500,
            "message": e
        })))
    })?
    .map_err(|e| {
        tracing::error!("获取ZIP失败: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "code": 500,
            "message": e
        })))
    })?;

    // 返回 ZIP 文件
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/zip")
        .header("Content-Disposition", format!("attachment; filename=\"template_{}_{}.zip\"", id, version))
        .body(Body::from(zip_bytes))
        .map_err(|e| {
            tracing::error!("构建响应失败: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "code": 500,
                "message": format!("构建响应失败: {}", e)
            })))
        })?;

    Ok(response)
}

/// 创建模板ZIP文件（在 blocking task 中执行）
fn create_template_zip(template_path: std::path::PathBuf) -> Result<Vec<u8>, String> {
    use std::fs::File;
    use std::io::Cursor;

    let mut zip_buffer = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut zip_buffer));

        // 需要排除的目录（只排除.git，保留.meta用于变量解析）
        let excluded_dirs = HashSet::from([".git"]);

        // 遍历模板目录
        let entries = std::fs::read_dir(&template_path)
            .map_err(|e| format!("读取模板目录失败: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
            let path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // 跳过排除的目录
            if path.is_dir() {
                if excluded_dirs.contains(file_name_str.as_ref()) {
                    continue;
                }
                // 递归添加目录
                add_dir_to_zip(&mut zip, &path, &template_path, &excluded_dirs)?;
            } else {
                // 添加文件
                if let Some(relative_path) = path.strip_prefix(&template_path).ok() {
                    let relative_path_str = relative_path.to_string_lossy().replace('\\', "/");

                    // 检查是否在排除的目录中
                    let should_exclude = relative_path_str.split('/')
                        .take(relative_path_str.split('/').count().saturating_sub(1))
                        .any(|part| excluded_dirs.contains(part));

                    if should_exclude {
                        continue;
                    }

                    // 添加文件到 ZIP
                    let mut file = File::open(&path)
                        .map_err(|e| format!("打开文件{}失败: {}", path.display(), e))?;

                    zip.start_file(relative_path_str.as_str(), FileOptions::default())
                        .map_err(|e| format!("创建ZIP条目{}失败: {}", relative_path_str, e))?;

                    let mut buffer = Vec::new();
                    std::io::copy(&mut file, &mut buffer)
                        .map_err(|e| format!("读取文件{}失败: {}", path.display(), e))?;

                    zip.write_all(&buffer)
                        .map_err(|e| format!("写入ZIP失败: {}", e))?;
                }
            }
        }

        zip.finish()
            .map_err(|e| format!("完成ZIP写入失败: {}", e))?;
    }

    Ok(zip_buffer)
}

/// 递归添加目录到ZIP
fn add_dir_to_zip<W: Write + Seek>(
    zip: &mut ZipWriter<W>,
    dir_path: &std::path::Path,
    base_path: &std::path::Path,
    excluded_dirs: &HashSet<&str>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir_path)
        .map_err(|e| format!("读取目录{}失败: {}", dir_path.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // 跳过排除的目录
            if excluded_dirs.contains(file_name_str.as_ref()) {
                continue;
            }

            // 递归处理子目录
            add_dir_to_zip(zip, &path, base_path, excluded_dirs)?;
        } else {
            if let Some(relative_path) = path.strip_prefix(base_path).ok() {
                let relative_path_str = relative_path.to_string_lossy().replace('\\', "/");

                // 检查是否在排除的目录中
                let should_exclude = relative_path_str.split('/')
                    .take(relative_path_str.split('/').count().saturating_sub(1))
                    .any(|part| excluded_dirs.contains(part));

                if should_exclude {
                    continue;
                }

                // 添加文件到 ZIP
                let mut file = File::open(&path)
                    .map_err(|e| format!("打开文件{}失败: {}", path.display(), e))?;

                zip.start_file(relative_path_str.as_str(), FileOptions::default())
                    .map_err(|e| format!("创建ZIP条目{}失败: {}", relative_path_str, e))?;

                let mut buffer = Vec::new();
                std::io::copy(&mut file, &mut buffer)
                    .map_err(|e| format!("读取文件{}失败: {}", path.display(), e))?;

                zip.write_all(&buffer)
                    .map_err(|e| format!("写入ZIP失败: {}", e))?;
            }
        }
    }

    Ok(())
}

/// 更新模板
pub async fn update_template(
    State(state): State<AppState>,
    Json(request): Json<UpdateTemplateRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证请求数据
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.template_service.update_template(request).await {
        Ok(()) => {
            tracing::info!("更新模板成功");
            Ok(Json(json!({
                "code": 0,
                "message": "更新模板成功"
            })))
        }
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

/// Fork 模板
pub async fn fork_template(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(mut request): Json<ForkTemplateRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 设置 owner_id 为当前用户
    request.owner_id = Some(auth_user.user_id);

    // 保存需要的信息
    let source_id = request.source_id;
    let storage_manager = state.storage_manager.clone();
    let template_service = state.template_service.clone();

    match state.template_service.fork_template(request).await {
        Ok(new_template_id) => {
            tracing::info!("Fork 模板成功: source_id={}, new_id={}", source_id, new_template_id);

            // 同步执行 Git 克隆，确保文件就绪后再返回
            let clone_result = tokio::task::spawn_blocking(move || {
                use template_studio_infrastructure::git::service::GitService;
                use template_studio_infrastructure::config::settings::GitConfig;

                let start_time = std::time::Instant::now();
                tracing::info!("开始 Git 克隆操作: source_id={}, new_id={}", source_id, new_template_id);

                let rt = tokio::runtime::Runtime::new().unwrap();
                let template_name = rt.block_on(async {
                    template_service.get_template(new_template_id).await
                        .ok()
                        .and_then(|t| t.map(|tmpl| tmpl.name))
                        .unwrap_or_else(|| {
                            tracing::warn!("无法获取模板名称，使用默认名称: template_id={}", new_template_id);
                            "Forked Template".to_string()
                        })
                });

                let source_path = storage_manager.get_template_path(source_id);
                let target_path = storage_manager.get_template_path(new_template_id);

                tracing::info!("Git 克隆路径: source={:?}, target={:?}", source_path, target_path);

                let git_config = GitConfig {
                    auto_init: true,
                    default_branch: "main".to_string(),
                };
                let git_service = GitService::new(git_config);

                let result = rt.block_on(async {
                    git_service.clone_and_clean(
                        &source_path,
                        &target_path,
                        &template_name,
                        Some("Template Studio"),
                        Some("template@studio.local")
                    ).await
                });

                let elapsed = start_time.elapsed();
                tracing::info!("Git 克隆完成: template_id={}, 耗时={:?}", new_template_id, elapsed);

                result
            }).await;

            match clone_result {
                Ok(Ok(_)) => tracing::info!("Git 仓库克隆成功: new_id={}", new_template_id),
                Ok(Err(e)) => tracing::error!("Git 仓库克隆失败: new_id={}, error={}", new_template_id, e),
                Err(e) => tracing::error!("Git 克隆任务异常: new_id={}, error={}", new_template_id, e),
            }

            Ok(Json(json!({
                "code": 0,
                "data": new_template_id,
                "message": "Fork 模板成功"
            })))
        }
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

// ===== 用户模板投稿 Handler =====

/// 创建用户模板
pub async fn create_user_template(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateTemplateRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.template_service.create_user_template(auth_user.user_id, request).await {
        Ok(id) => Ok(Json(json!({ "code": 200, "message": "模板创建成功", "result": { "id": id } }))),
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

/// 获取我的模板列表
pub async fn list_my_templates(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<UserTemplateListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.template_service.list_user_templates(auth_user.user_id, query).await {
        Ok(resp) => Ok(Json(json!({ "code": 200, "result": resp }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 更新用户模板
pub async fn update_user_template(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
    Json(mut request): Json<UpdateTemplateRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    request.id = id;
    match state.template_service.update_user_template(auth_user.user_id, request).await {
        Ok(_) => Ok(Json(json!({ "code": 200, "message": "模板更新成功" }))),
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

/// 删除用户模板
pub async fn delete_user_template(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.template_service.delete_user_template(auth_user.user_id, id).await {
        Ok(_) => Ok(Json(json!({ "code": 200, "message": "模板删除成功" }))),
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

/// 提交审核
pub async fn submit_for_review(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.template_service.submit_for_review(auth_user.user_id, id).await {
        Ok(_) => Ok(Json(json!({ "code": 200, "message": "已提交审核" }))),
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

/// 获取公开模板列表（无需认证）
pub async fn list_public_templates(
    State(state): State<AppState>,
    Query(query): Query<UserTemplateListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.template_service.list_public_templates(query).await {
        Ok(resp) => Ok(Json(json!({ "code": 200, "result": resp }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

// ===== 管理员审核 Handler =====

#[derive(Debug, Deserialize)]
pub struct PendingListQuery {
    pub page: Option<u32>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<u32>,
}

/// 获取待审核模板列表
pub async fn list_pending_templates(
    State(state): State<AppState>,
    Query(query): Query<PendingListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    match state.template_service.list_pending_templates(page, page_size).await {
        Ok(resp) => Ok(Json(json!({ "code": 200, "result": resp }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 审核模板
pub async fn review_template_admin(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<ReviewTemplateRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.template_service.review_template(auth_user.user_id, request).await {
        Ok(_) => Ok(Json(json!({ "code": 200, "message": "审核完成" }))),
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}
