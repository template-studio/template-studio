use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// 编程语言数据模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Language {
    pub id: u32,  // 改为 u32 以匹配 INT UNSIGNED
    pub name: String,
    pub display_name: String,
    pub code: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_popular: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建语言请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateLanguageRequest {
    #[validate(length(min = 1, message = "语言名称不能为空"))]
    pub name: String,
    #[validate(length(min = 1, message = "显示名称不能为空"))]
    pub display_name: String,
    #[validate(length(min = 1, message = "语言代码不能为空"))]
    pub code: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_popular: Option<i32>,
}

/// 更新语言请求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateLanguageRequest {
    #[validate(range(min = 1, message = "语言ID不能为空"))]
    pub id: u32,  // 改为 u32
    #[validate(length(min = 1, message = "语言名称不能为空"))]
    pub name: String,
    #[validate(length(min = 1, message = "显示名称不能为空"))]
    pub display_name: String,
    #[validate(length(min = 1, message = "语言代码不能为空"))]
    pub code: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    #[serde(rename = "isPopular")]
    pub is_popular: Option<i32>,
}

/// 语言列表查询
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct LanguageListQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub name: Option<String>,
    pub is_popular: Option<i32>,
}

impl Default for LanguageListQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(20),
            name: None,
            is_popular: None,
        }
    }
}

/// API响应用的语言模型（前端专用格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageResponse {
    pub id: u32,
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub code: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    #[serde(rename = "isPopular")]
    pub is_popular: i32,
}

impl From<Language> for LanguageResponse {
    fn from(language: Language) -> Self {
        Self {
            id: language.id,
            name: language.name.clone(),
            display_name: language.display_name,
            code: language.code,
            icon: language.icon,
            color: language.color,
            is_popular: language.is_popular,
        }
    }
}