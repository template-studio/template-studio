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

impl JwtConfig {
    /// 从环境变量 `TEMPLATE_STUDIO_JWT_SECRET` 读取签名密钥。
    ///
    /// - 显式配置：使用配置值
    /// - 未配置 + debug 构建：沿用开发默认值（仅限本地开发便利）
    /// - 未配置 + release 构建：生成随机临时密钥兜底（进程重启后已签发令牌全部失效，
    ///   多实例部署会互不认账——生产必须显式配置）
    pub fn from_env() -> Self {
        let secret = match std::env::var("TEMPLATE_STUDIO_JWT_SECRET") {
            Ok(s) if !s.trim().is_empty() => s,
            _ if cfg!(debug_assertions) => {
                "template-studio-jwt-secret-change-in-production".to_string()
            }
            _ => {
                let generated = format!("{}{}", uuid::Uuid::new_v4(), uuid::Uuid::new_v4());
                eprintln!("[安全提示] 未配置环境变量 TEMPLATE_STUDIO_JWT_SECRET，已生成临时密钥（重启后所有登录令牌失效，生产环境必须显式配置）");
                generated
            }
        };
        Self {
            secret,
            expire_hours: 72,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_env_uses_configured_secret() {
        // 串行操作环境变量，避免测试间竞争
        std::env::set_var("TEMPLATE_STUDIO_JWT_SECRET", "my-test-secret");
        let config = JwtConfig::from_env();
        assert_eq!(config.secret, "my-test-secret");
        assert_eq!(config.expire_hours, 72);
        std::env::remove_var("TEMPLATE_STUDIO_JWT_SECRET");

        // 未配置时不得包含生产兜底以外的固定值
        let fallback = JwtConfig::from_env();
        if cfg!(debug_assertions) {
            assert_eq!(
                fallback.secret,
                "template-studio-jwt-secret-change-in-production"
            );
        } else {
            assert_eq!(fallback.secret.len(), 72); // 两个 UUID v4 简单格式拼接
        }
    }
}
