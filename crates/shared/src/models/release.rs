//! 模板版本管理相关数据模型

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use validator::Validate;

/// 模板版本信息
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TemplateVersion {
    pub id: i64,
    #[serde(rename = "templateId")]
    pub template_id: i64,
    pub version: String,
    #[serde(rename = "commitHash")]
    pub commit_hash: Option<String>,
    #[serde(rename = "commitMessage")]
    pub commit_message: Option<String>,
    pub changelog: Option<String>,
    #[serde(rename = "isLatest")]
    pub is_latest: bool,
    #[serde(rename = "isDeprecated")]
    pub is_deprecated: bool,
    #[serde(rename = "creatorId")]
    pub creator_id: Option<i64>,
    #[serde(rename = "creatorName")]
    pub creator_name: Option<String>,
    #[serde(rename = "fileCount")]
    pub file_count: i32,
    #[serde(rename = "totalSize")]
    pub total_size: i64,
    #[serde(rename = "storagePath")]
    pub storage_path: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

/// 创建发布版本请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateReleaseRequest {
    /// 版本号（可选，不填则自动生成）
    #[serde(rename = "version")]
    pub version: Option<String>,

    /// 发布日志
    #[serde(rename = "changelog")]
    pub changelog: Option<String>,

    /// Git 提交信息
    #[serde(rename = "message")]
    pub message: Option<String>,
}

/// 创建发布版本响应
#[derive(Debug, Clone, Serialize)]
pub struct CreateReleaseResponse {
    pub id: i64,
    pub version: String,
    pub commit_hash: Option<String>,
    pub storage_path: String,
    pub is_latest: bool,
    pub created_at: DateTime<Utc>,
    pub file_count: i32,
    pub total_size: i64,
}

/// 版本列表响应
#[derive(Debug, Clone, Serialize)]
pub struct VersionsListResponse {
    #[serde(rename = "templateId")]
    pub template_id: i64,
    pub versions: Vec<TemplateVersion>,
}

/// 回滚版本响应
#[derive(Debug, Clone, Serialize)]
pub struct RollbackResponse {
    pub previous_version: String,
    pub current_version: String,
}

/// 重置到最新版本响应
#[derive(Debug, Clone, Serialize)]
pub struct ResetToLatestResponse {
    /// 重置到的版本号
    pub version: String,
    /// 清理的未跟踪文件数量
    #[serde(rename = "deletedFiles")]
    pub deleted_files: i32,
}

/// 版本详情响应
#[derive(Debug, Clone, Serialize)]
pub struct VersionDetailResponse {
    pub id: i64,
    pub version: String,
    pub template_id: i64,
    pub commit_hash: Option<String>,
    pub commit_message: Option<String>,
    pub changelog: Option<String>,
    pub is_latest: bool,
    pub is_deprecated: bool,
    pub created_at: DateTime<Utc>,
    pub file_count: i32,
    pub total_size: i64,
    pub storage_path: String,
    pub creator: Option<CreatorInfo>,
    pub metadata_snapshot: Option<serde_json::Value>,
}

/// 创建者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatorInfo {
    pub id: i64,
    pub name: String,
}

/// 预览模板文件请求
#[derive(Debug, Deserialize, Validate)]
pub struct PreviewTemplateRequest {
    #[validate(range(min = 1, message = "模板ID不能为空"))]
    #[serde(rename = "templateId")]
    pub template_id: i64,

    #[validate(length(min = 1, message = "文件路径不能为空"))]
    #[serde(rename = "filePath")]
    pub file_path: String,

    pub variables: serde_json::Value,
}

/// 生成模板文件请求
#[derive(Debug, Deserialize, Validate)]
pub struct GenerateTemplateRequest {
    #[validate(range(min = 1, message = "模板ID不能为空"))]
    #[serde(rename = "templateId")]
    pub template_id: i64,

    #[validate(length(min = 1, message = "文件路径不能为空"))]
    #[serde(rename = "filePath")]
    pub file_path: String,

    pub variables: serde_json::Value,
}

/// 预览/生成响应数据
#[derive(Debug, Clone, Serialize)]
pub struct TemplateRenderData {
    #[serde(rename = "fileContent")]
    pub file_content: String,

    #[serde(rename = "fileName")]
    pub file_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,  // 仅 generate 接口返回
}
