use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// 分类数据模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub sort: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建分类请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateCategoryRequest {
    #[validate(length(min = 1, message = "分类名称不能为空"))]
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub sort: Option<i32>,
}

/// 更新分类请求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCategoryRequest {
    #[validate(range(min = 1, message = "分类ID不能为空"))]
    pub id: i64,
    #[validate(length(min = 1, message = "分类名称不能为空"))]
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub sort: Option<i32>,
}

/// 分类列表查询
#[derive(Debug, Deserialize, Validate)]
pub struct CategoryListQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub name: Option<String>,
}