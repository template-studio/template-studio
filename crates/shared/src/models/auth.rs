use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub exp: i64,
    pub iat: i64,
}

/// 认证类型
#[derive(Debug, Clone)]
pub enum AuthType {
    /// JWT 会话认证，拥有全部权限
    Jwt,
    /// PAT 令牌认证，权限受限于 scopes
    Pat,
}

/// 中间件提取的认证用户
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: i64,
    pub username: String,
    pub auth_type: AuthType,
    /// PAT 令牌的权限范围，JWT 认证时为 None（视为全权限）
    pub scopes: Option<Arc<Vec<String>>>,
}

impl AuthUser {
    /// 检查是否拥有指定 scope 权限
    /// JWT 认证始终返回 true（全权限）
    pub fn has_scope(&self, scope: &str) -> bool {
        match &self.scopes {
            None => true,
            Some(s) => s.iter().any(|sc| sc == scope),
        }
    }
}

/// JWT 配置
#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expire_hours: i64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "template-studio-jwt-secret-change-in-production".to_string(),
            expire_hours: 72,
        }
    }
}
