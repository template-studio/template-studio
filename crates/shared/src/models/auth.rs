use serde::{Deserialize, Serialize};

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub exp: i64,
    pub iat: i64,
}

/// 中间件提取的认证用户
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: i64,
    pub username: String,
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
