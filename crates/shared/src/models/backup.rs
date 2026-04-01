//! 备份与恢复数据模型
//!
//! 模板备份使用专有的 .tsbk 格式（基于 ZIP），包含 SHA256 校验防止篡改

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// 备份格式版本
pub const BACKUP_FORMAT_VERSION: &str = "1.0";

/// 备份文件扩展名
pub const BACKUP_FILE_EXTENSION: &str = ".tsbk";

// =============== 备份清单结构 ===============

/// 备份清单（manifest.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    /// 备份格式版本
    pub version: String,
    /// 格式标识符
    pub format: String,
    /// 备份创建时间（ISO 8601）
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    /// 原始模板 ID
    #[serde(rename = "templateId")]
    pub template_id: i64,
    /// 模板名称
    #[serde(rename = "templateName")]
    pub template_name: String,
    /// 整体校验和（SHA256）
    pub checksum: String,
    /// 各文件校验和
    #[serde(rename = "filesChecksum")]
    pub files_checksum: HashMap<String, String>,
}

// =============== 备份文件内容结构 ===============

/// 备份中的模板信息（template.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupTemplateInfo {
    /// 模板 ID
    pub id: i64,
    /// 模板名称
    pub name: String,
    /// 模板描述
    pub description: String,
    /// 分类 ID
    #[serde(rename = "categoryId")]
    pub category_id: i64,
    /// 模板类型
    #[serde(rename = "templateType")]
    pub template_type: String,
    /// 类型配置
    #[serde(rename = "typeConfig")]
    pub type_config: Option<String>,
    /// 简介
    pub introduction: Option<String>,
    /// 是否推荐
    #[serde(rename = "isFeatured")]
    pub is_featured: i32,
    /// Logo
    pub logo: Option<String>,
    /// 图标
    pub icon: Option<String>,
    /// Git 仓库路径
    #[serde(rename = "gitRepoPath")]
    pub git_repo_path: Option<String>,
    /// 当前版本
    #[serde(rename = "currentVersion")]
    pub current_version: Option<String>,
    /// 语言列表
    pub languages: Vec<BackupLanguageInfo>,
}

/// 备份中的语言信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupLanguageInfo {
    /// 语言 ID
    #[serde(rename = "languageId")]
    pub language_id: u32,
    /// 是否主要语言
    #[serde(rename = "isPrimary")]
    pub is_primary: i32,
}

/// 备份中的文件条件（conditions.json 数组元素）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFileCondition {
    /// 文件路径
    #[serde(rename = "filePath")]
    pub file_path: String,
    /// 条件表达式
    pub condition: Option<String>,
    /// 条件类型
    #[serde(rename = "conditionType")]
    pub condition_type: Option<String>,
}

// =============== API 请求结构 ===============

/// 创建备份请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateBackupRequest {
    /// 模板 ID
    #[validate(range(min = 1, message = "模板ID不能为空"))]
    #[serde(rename = "templateId")]
    pub template_id: i64,
    /// 是否包含测试数据
    #[serde(rename = "includeTestData")]
    #[serde(default = "default_true")]
    pub include_test_data: bool,
    /// 是否包含文件条件
    #[serde(rename = "includeConditions")]
    #[serde(default = "default_true")]
    pub include_conditions: bool,
}

fn default_true() -> bool {
    true
}

/// 预览备份响应
#[derive(Debug, Serialize)]
pub struct BackupPreviewResponse {
    /// 清单信息
    pub manifest: BackupManifest,
    /// 模板名称
    #[serde(rename = "templateName")]
    pub template_name: String,
    /// 文件数量
    #[serde(rename = "fileCount")]
    pub file_count: usize,
    /// 是否包含变量定义
    #[serde(rename = "hasVariables")]
    pub has_variables: bool,
    /// 是否包含测试数据
    #[serde(rename = "hasTestData")]
    pub has_test_data: bool,
    /// 是否包含文件条件
    #[serde(rename = "hasConditions")]
    pub has_conditions: bool,
    /// 文件路径列表
    #[serde(rename = "filePaths")]
    pub file_paths: Vec<String>,
    /// 校验是否通过
    #[serde(rename = "checksumValid")]
    pub checksum_valid: bool,
}

/// 恢复备份请求
#[derive(Debug, Deserialize, Validate)]
pub struct RestoreBackupRequest {
    /// 目标模板 ID
    #[validate(range(min = 1, message = "模板ID不能为空"))]
    #[serde(rename = "templateId")]
    pub template_id: i64,
}

/// 恢复备份响应
#[derive(Debug, Serialize)]
pub struct RestoreBackupResponse {
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果失败）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 恢复统计
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<RestoreStats>,
}

/// 恢复统计信息
#[derive(Debug, Serialize)]
pub struct RestoreStats {
    /// 恢复的文件数
    #[serde(rename = "filesRestored")]
    pub files_restored: usize,
    /// 恢复的变量数
    #[serde(rename = "variablesRestored")]
    pub variables_restored: usize,
    /// 恢复的条件数
    #[serde(rename = "conditionsRestored")]
    pub conditions_restored: usize,
    /// 是否恢复了测试数据
    #[serde(rename = "testDataRestored")]
    pub test_data_restored: bool,
}

// =============== 备份数据结构（内部使用） ===============

/// 完整的备份数据
#[derive(Debug)]
pub struct BackupData {
    /// 清单
    pub manifest: BackupManifest,
    /// 模板信息
    pub template_info: BackupTemplateInfo,
    /// 变量定义（JSON 字符串）
    pub variables: Option<String>,
    /// 文件条件列表
    pub conditions: Vec<BackupFileCondition>,
    /// 测试数据（JSON 字符串）
    pub test_data: Option<String>,
    /// 文件内容（路径 -> 内容）
    pub files: HashMap<String, String>,
}
