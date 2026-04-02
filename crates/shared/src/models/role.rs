use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// 角色数据模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub sort: i32,
    pub status: i8,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建角色请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateRoleRequest {
    #[validate(length(min = 1, max = 50, message = "角色标识长度1-50"))]
    pub name: String,
    #[validate(length(min = 1, max = 100, message = "角色名称长度1-100"))]
    pub display_name: String,
    #[validate(length(max = 255))]
    pub description: Option<String>,
    pub sort: Option<i32>,
    pub permission_ids: Option<Vec<i64>>,
}

/// 更新角色请求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateRoleRequest {
    pub id: i64,
    #[validate(length(min = 1, max = 100))]
    pub display_name: Option<String>,
    #[validate(length(max = 255))]
    pub description: Option<String>,
    pub sort: Option<i32>,
    pub status: Option<i8>,
}

/// 分配权限请求
#[derive(Debug, Deserialize)]
pub struct AssignPermissionsRequest {
    pub permission_ids: Vec<i64>,
}

/// 角色+权限组合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleWithPermissions {
    #[serde(flatten)]
    pub role: Role,
    pub permissions: Vec<i64>,
}
