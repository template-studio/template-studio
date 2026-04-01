use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// 系统设置数据模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SystemSetting {
    pub id: i64,
    pub group: String,
    pub key: String,
    pub value: Option<String>,
    pub description: Option<String>,
    pub sort: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 更新设置请求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSettingRequest {
    #[validate(length(min = 1, message = "group 不能为空"))]
    pub group: String,
    #[validate(length(min = 1, message = "key 不能为空"))]
    pub key: String,
    pub value: Option<String>,
}

/// 批量更新设置请求
#[derive(Debug, Deserialize, Validate)]
pub struct BatchUpdateSettingsRequest {
    #[validate(length(min = 1, message = "group 不能为空"))]
    pub group: String,
    pub items: Vec<UpdateSettingItem>,
}

/// 批量更新中的单项
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSettingItem {
    #[validate(length(min = 1, message = "key 不能为空"))]
    pub key: String,
    pub value: Option<String>,
}

/// 获取设置查询参数
#[derive(Debug, Deserialize)]
pub struct GetSettingsQuery {
    pub group: Option<String>,
    pub key: Option<String>,
}
