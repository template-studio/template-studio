use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// 模板数据模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Template {
    pub id: i64, // 恢复为i64，使用SQL类型转换
    pub name: String,
    pub description: String,
    pub category_id: i64, // 恢复为i64，使用SQL类型转换
    pub is_featured: i32,
    pub logo: Option<String>,
    pub introduction: Option<String>,
    pub icon: Option<String>,
    pub template_type: String,
    pub type_config: Option<String>,
    pub git_repo_path: Option<String>, // 保持可选
    pub current_version: Option<String>, // 保持可选
    pub owner_id: Option<i64>,
    pub visibility: Option<String>,
    pub status: Option<String>,
    pub reviewed_at: Option<String>,
    pub reviewed_by: Option<i64>,
    pub download_count: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建模板请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateTemplateRequest {
    #[validate(length(min = 1, message = "模板名称不能为空"))]
    pub name: String,
    #[validate(length(min = 1, message = "模板描述不能为空"))]
    pub description: String,
    pub introduction: Option<String>,
    #[validate(range(min = 1, message = "分类ID不能为空"))]
    #[serde(rename = "categoryId")]
    pub category_id: i64,
    #[validate(length(min = 1, message = "模板类型不能为空"))]
    #[serde(rename = "templateType")]
    pub template_type: String,
    #[serde(rename = "typeConfig")]
    pub type_config: Option<String>,
    pub languages: Vec<TemplateLanguageRequest>,
    pub visibility: Option<String>,
    #[serde(rename = "ownerId")]
    pub owner_id: Option<i64>,
}

/// 更新模板请求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTemplateRequest {
    #[validate(range(min = 1, message = "模板ID不能为空"))]
    pub id: i64,
    #[validate(length(min = 1, message = "模板名称不能为空"))]
    pub name: String,
    #[validate(length(min = 1, message = "模板描述不能为空"))]
    pub description: String,
    pub introduction: Option<String>,
    #[validate(range(min = 1, message = "分类ID不能为空"))]
    #[serde(rename = "categoryId")]
    pub category_id: i64,
    #[serde(rename = "templateType")]
    pub template_type: Option<String>,
    #[serde(rename = "typeConfig")]
    pub type_config: Option<String>,
    pub languages: Vec<TemplateLanguageRequest>,
    pub visibility: Option<String>,
}

/// 模板语言关联请求
#[derive(Debug, Deserialize, Validate)]
pub struct TemplateLanguageRequest {
    #[validate(range(min = 1, message = "语言ID不能为空"))]
    #[serde(rename = "languageId")]
    pub language_id: u32,  // 改为 u32
    #[serde(rename = "isPrimary")]
    pub is_primary: i32,
}

/// 模板列表查询
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct TemplateListQuery {
    pub page: Option<u32>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<u32>,
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "categoryId")]
    pub category_id: Option<i64>,
    #[serde(rename = "languageId")]
    pub language_id: Option<u32>,  // 改为 u32
    #[serde(rename = "isFeatured")]
    pub is_featured: Option<i32>,
    #[serde(rename = "templateType")]
    pub template_type: Option<String>,
}

impl Default for TemplateListQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(20),
            name: None,
            description: None,
            category_id: None,
            language_id: None,
            is_featured: None,
            template_type: None,
        }
    }
}

/// 模板详情响应
#[derive(Debug, Serialize)]
pub struct TemplateDetailResponse {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub introduction: Option<String>,
    #[serde(rename = "categoryId")]
    pub category_id: i64,
    #[serde(rename = "isFeatured")]
    pub is_featured: i32,
    pub logo: Option<String>,
    pub icon: Option<String>,
    #[serde(rename = "templateType")]
    pub template_type: String,
    #[serde(rename = "typeConfig")]
    pub type_config: Option<String>,
    #[serde(rename = "gitRepoPath")]
    pub git_repo_path: String,
    #[serde(rename = "currentVersion")]
    pub current_version: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    pub languages: Vec<TemplateLanguageInfo>,
}

/// 模板语言信息
#[derive(Debug, Serialize)]
pub struct TemplateLanguageInfo {
    pub id: i64,
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub code: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    #[serde(rename = "isPopular")]
    pub is_popular: i32,
    #[serde(rename = "isPrimary")]
    pub is_primary: i32,
}

/// Fork模板请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ForkTemplateRequest {
    #[validate(range(min = 1, message = "源模板ID不能为空"))]
    #[serde(rename = "sourceId")]
    pub source_id: i64,
    #[validate(length(min = 1, message = "新模板名称不能为空"))]
    pub name: String,
    #[validate(length(min = 1, message = "新模板描述不能为空"))]
    pub description: String,
    pub introduction: Option<String>,
    #[serde(rename = "categoryId")]
    pub category_id: Option<i64>,
}

/// 模板导出请求
#[derive(Debug, Deserialize, Validate)]
pub struct ExportTemplateRequest {
    #[validate(range(min = 1, message = "模板ID不能为空"))]
    pub id: i64,
    #[validate(custom(function = "validate_export_format"))]
    pub format: String,
}

/// 验证导出格式
fn validate_export_format(format: &str) -> Result<(), validator::ValidationError> {
    if format == "files" || format == "json" {
        Ok(())
    } else {
        Err(validator::ValidationError::new("导出格式必须为files或json"))
    }
}

/// 模板类型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateTypeInfo {
    pub value: String,
    pub label: String,
    pub description: String,
}

/// 模板类型响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateTypesResponse {
    #[serde(rename = "templateTypes")]
    pub template_types: Vec<TemplateTypeInfo>,
}

/// 模板列表响应（匹配原系统格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateListResponse {
    #[serde(rename = "currentPage")]
    pub current_page: u32,
    pub total: u32,
    #[serde(rename = "templatesList")]
    pub templates_list: Vec<TemplateItem>,
}

/// 模板项目（匹配原系统格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateItem {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub introduction: Option<String>,
    #[serde(rename = "categoryId")]
    pub category_id: i64,
    #[serde(rename = "isFeatured")]
    pub is_featured: i32,
    #[serde(rename = "templateType")]
    pub template_type: String,
    #[serde(rename = "typeConfig")]
    pub type_config: Option<String>,
    pub visibility: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "ownerId")]
    pub owner_id: Option<i64>,
    #[serde(rename = "ownerName")]
    pub owner_name: Option<String>,
    #[serde(rename = "downloadCount")]
    pub download_count: Option<i32>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub languages: Vec<TemplateLanguageItem>,
}

/// 模板语言关联（匹配原系统格式）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TemplateLanguageItem {
    pub id: i64,
    #[serde(rename = "templateId")]
    pub template_id: i64,
    #[serde(rename = "languageId")]
    pub language_id: i64,
    #[serde(rename = "isPrimary")]
    pub is_primary: i32,
}

// ===== 用户模板投稿 =====

/// 用户模板列表查询
#[derive(Debug, Clone, Deserialize)]
pub struct UserTemplateListQuery {
    pub page: Option<u32>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<u32>,
    pub keyword: Option<String>,
    pub visibility: Option<String>,
    #[serde(rename = "categoryId")]
    pub category_id: Option<i64>,
    #[serde(rename = "ownerId")]
    pub owner_id: Option<i64>,
}

/// 审核请求（管理员用）
#[derive(Debug, Deserialize)]
pub struct ReviewTemplateRequest {
    #[serde(rename = "templateId")]
    pub template_id: i64,
    pub action: String,   // approve / reject
    pub reason: Option<String>,
}

/// 审核记录
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TemplateReview {
    pub id: i64,
    pub template_id: i64,
    pub reviewer_id: i64,
    pub action: String,
    pub reason: Option<String>,
    pub created_at: Option<String>,
}

/// 模板详情响应（扩展版，含投稿信息）
#[derive(Debug, Serialize)]
pub struct TemplateDetailExtResponse {
    #[serde(rename = "templateId")]
    pub template_id: i64,
    pub name: String,
    pub description: String,
    pub introduction: Option<String>,
    #[serde(rename = "categoryId")]
    pub category_id: i64,
    #[serde(rename = "templateType")]
    pub template_type: String,
    pub visibility: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "ownerId")]
    pub owner_id: Option<i64>,
    #[serde(rename = "ownerName")]
    pub owner_name: Option<String>,
    #[serde(rename = "downloadCount")]
    pub download_count: Option<i32>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub languages: Vec<TemplateLanguageItem>,
}