use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// 用户数据模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub avatar: String,
    pub status: i8,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 用户列表项（不含密码）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserListItem {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub avatar: String,
    pub status: i8,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 登录请求
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "用户名不能为空"))]
    pub username: String,
    #[validate(length(min = 1, message = "密码不能为空"))]
    pub password: String,
}

/// 登录响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

/// 创建用户请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 50, message = "用户名长度1-50"))]
    pub username: String,
    #[validate(length(min = 6, max = 100, message = "密码长度6-100"))]
    pub password: String,
    #[validate(length(max = 100))]
    pub email: Option<String>,
    pub role_ids: Option<Vec<i64>>,
}

/// 更新用户请求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    pub id: i64,
    #[validate(length(max = 100))]
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub status: Option<i8>,
    pub password: Option<String>,
}

/// 修改密码请求
#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 6, message = "新密码至少6位"))]
    pub new_password: String,
}

/// 分配角色请求
#[derive(Debug, Deserialize)]
pub struct AssignRolesRequest {
    pub role_ids: Vec<i64>,
}

/// 用户信息响应（含权限）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfoResponse {
    pub username: String,
    pub email: String,
    pub avatar: String,
    pub permissions: Vec<PermissionItem>,
}

/// 权限项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionItem {
    pub value: String,
    pub label: String,
}
