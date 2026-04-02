//! WASM 引擎管理 Handler
//!
//! 提供前端获取 WASM 引擎信息的 API，支持：
//! - 获取引擎信息（版本、过滤器、函数列表）
//! - 下载 WASM 文件

use axum::{
    body::Body,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

// ============================================================================
// 响应类型
// ============================================================================

/// 引擎信息响应
#[derive(Debug, Serialize, Deserialize)]
pub struct EngineInfoResponse {
    /// 引擎版本号
    pub version: String,
    /// 构建时间
    pub build_time: String,
    /// WASM 文件大小（字节）
    pub size: u64,
    /// 支持的过滤器列表
    pub filters: Vec<FilterInfo>,
    /// 支持的内置函数列表
    pub functions: Vec<String>,
    /// SHA256 校验和
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

/// 过滤器信息
#[derive(Debug, Serialize, Deserialize)]
pub struct FilterInfo {
    /// 过滤器名称
    pub name: String,
    /// 过滤器描述
    pub description: String,
    /// 示例
    pub example: String,
}

// ============================================================================
// Handler 函数
// ============================================================================

/// 获取 WASM 引擎信息
///
/// GET /api/v1/engine/info
///
/// 返回当前 WASM 引擎的版本、支持的过滤器和函数等信息
pub async fn get_engine_info() -> impl IntoResponse {
    // 获取核心引擎的过滤器信息
    let filter_infos = template_studio_template_core::get_available_filters();
    let filters: Vec<FilterInfo> = filter_infos
        .iter()
        .map(|f| FilterInfo {
            name: f.name.clone(),
            description: f.description.clone(),
            example: f.example.clone(),
        })
        .collect();

    // 获取内置函数
    let function_categories = template_studio_template_core::get_builtin_function_categories();
    let functions: Vec<String> = function_categories
        .iter()
        .flat_map(|cat| cat.functions.iter().map(|f| f.name.clone()))
        .collect();

    // 尝试获取 WASM 文件大小
    let wasm_path = get_wasm_path();
    let (size, checksum) = match tokio::fs::metadata(&wasm_path).await {
        Ok(metadata) => {
            let size = metadata.len();
            // TODO: 计算校验和（可选）
            let checksum = None;
            (size, checksum)
        }
        Err(e) => {
            tracing::warn!("WASM file not found at {:?}: {}", wasm_path, e);
            (0, None)
        }
    };

    let info = EngineInfoResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_time: option_env!("BUILD_TIME").unwrap_or("unknown").to_string(),
        size,
        filters,
        functions,
        checksum,
    };

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "code": 0,
            "message": "success",
            "data": info
        })),
    )
}

/// 下载 WASM 引擎文件
///
/// GET /api/v1/engine/download
///
/// 返回 WASM 二进制文件，支持浏览器缓存
pub async fn download_engine() -> impl IntoResponse {
    let wasm_path = get_wasm_path();

    match tokio::fs::read(&wasm_path).await {
        Ok(data) => {
            tracing::info!(
                "Serving WASM engine: {} bytes",
                data.len()
            );

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/wasm")
                .header(
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"template-engine.wasm\"",
                )
                .header(header::CACHE_CONTROL, "public, max-age=86400") // 1 天缓存
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .body(Body::from(data))
                .unwrap()
        }
        Err(e) => {
            tracing::error!("Failed to read WASM file: {}", e);

            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "code": 404,
                        "message": format!("WASM engine not found: {}", e),
                        "data": null
                    })
                    .to_string(),
                ))
                .unwrap()
        }
    }
}

/// 检查引擎更新
///
/// GET /api/v1/engine/check-update?version=xxx
///
/// 检查是否有新版本可用
pub async fn check_engine_update(
    axum::extract::Query(params): axum::extract::Query<CheckUpdateParams>,
) -> impl IntoResponse {
    let current_version = env!("CARGO_PKG_VERSION");

    let has_update = params.version.as_deref() != Some(current_version);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "code": 0,
            "message": "success",
            "data": {
                "current_version": current_version,
                "client_version": params.version,
                "has_update": has_update
            }
        })),
    )
}

/// 检查更新参数
#[derive(Debug, Deserialize)]
pub struct CheckUpdateParams {
    /// 客户端当前版本
    pub version: Option<String>,
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 获取 WASM 文件路径
///
/// 优先级：
/// 1. 环境变量 WASM_PATH
/// 2. 相对于当前工作目录的 pkg 目录
/// 3. 相对于 crate 的 pkg 目录
fn get_wasm_path() -> std::path::PathBuf {
    // 1. 检查环境变量
    if let Ok(path) = std::env::var("WASM_PATH") {
        let path = std::path::PathBuf::from(path);
        if path.exists() {
            return path;
        }
    }

    // 2. 检查工作目录下的 pkg
    let work_dir_path = std::path::PathBuf::from("crates/template_core_wasm/pkg/template_studio_template_core_wasm_bg.wasm");
    if work_dir_path.exists() {
        return work_dir_path;
    }

    // 3. 返回默认路径（即使不存在，让后续处理报错）
    work_dir_path
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_info_response() {
        let info = EngineInfoResponse {
            version: "0.1.0".to_string(),
            build_time: "2024-01-01T00:00:00Z".to_string(),
            size: 1024,
            filters: vec![FilterInfo {
                name: "upper".to_string(),
                description: "转换为大写".to_string(),
                example: "{{ name | upper }}".to_string(),
            }],
            functions: vec!["now".to_string()],
            checksum: None,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("upper"));
    }
}
