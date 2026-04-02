use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 权限数据模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Permission {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub r#type: String,
    pub parent_id: Option<i64>,
    pub sort: i32,
    pub status: i8,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 权限树节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionTree {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub r#type: String,
    pub sort: i32,
    pub children: Vec<PermissionTree>,
}
