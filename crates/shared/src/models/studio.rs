use serde::{Deserialize, Serialize};

/// Studio首页请求参数
#[derive(Debug, Deserialize)]
pub struct StudioIndexRequest {
    /// 分类限制数量，默认6
    #[serde(rename = "categoryLimit")]
    pub category_limit: Option<u32>,
    /// 推荐模板限制数量，默认8
    #[serde(rename = "featuredLimit")]
    pub featured_limit: Option<u32>,
}

impl Default for StudioIndexRequest {
    fn default() -> Self {
        Self {
            category_limit: Some(6),
            featured_limit: Some(8),
        }
    }
}

/// Studio首页响应
#[derive(Debug, Serialize)]
pub struct StudioIndexResponse {
    /// 统计数据
    pub statistics: IndexStatistics,
    /// 分类及其模板
    pub categories: Vec<CategoryWithTemplates>,
    /// 推荐模板
    #[serde(rename = "featuredTemplates")]
    pub featured_templates: Vec<FeaturedTemplate>,
}

/// 统计数据
#[derive(Debug, Serialize)]
pub struct IndexStatistics {
    /// 总模板数
    #[serde(rename = "totalTemplates")]
    pub total_templates: i64,
    /// 总分类数
    #[serde(rename = "totalCategories")]
    pub total_categories: i64,
    /// 总语言数
    #[serde(rename = "totalLanguages")]
    pub total_languages: i64,
    /// 推荐模板数
    #[serde(rename = "featuredTemplates")]
    pub featured_templates: i64,
}

/// 分类及其模板
#[derive(Debug, Serialize)]
pub struct CategoryWithTemplates {
    /// 分类ID
    pub id: i64,
    /// 分类名称
    pub name: String,
    /// 分类描述
    pub description: Option<String>,
    /// 分类图标
    pub icon: Option<String>,
    /// 模板列表
    pub templates: Vec<CategoryTemplate>,
}

/// 分类下的模板信息
#[derive(Debug, Serialize)]
pub struct CategoryTemplate {
    /// 模板ID
    pub id: i64,
    /// 模板名称
    pub name: String,
    /// 模板描述
    pub description: String,
    /// 模板类型
    #[serde(rename = "templateType")]
    pub template_type: String,
    /// 创建时间
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// 推荐模板
#[derive(Debug, Serialize)]
pub struct FeaturedTemplate {
    /// 模板ID
    pub id: i64,
    /// 模板名称
    pub name: String,
    /// 模板描述
    pub description: String,
    /// 模板介绍
    pub introduction: Option<String>,
    /// 模板类型
    #[serde(rename = "templateType")]
    pub template_type: String,
    /// 分类ID
    #[serde(rename = "categoryId")]
    pub category_id: i64,
    /// 分类名称
    #[serde(rename = "categoryName")]
    pub category_name: String,
    /// Logo
    pub logo: Option<String>,
    /// 图标
    pub icon: Option<String>,
    /// 创建时间
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// 语言列表
    pub languages: Vec<StudioTemplateLanguage>,
}

/// 模板语言关联（简化版本，用于studio index）
#[derive(Debug, Serialize)]
pub struct StudioTemplateLanguage {
    /// 语言ID
    #[serde(rename = "languageId")]
    pub language_id: i64,
    /// 语言名称
    pub name: String,
    /// 是否主要语言
    #[serde(rename = "isPrimary")]
    pub is_primary: i32,
}