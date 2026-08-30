use anyhow::{anyhow, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::sync::Arc;
use template_studio_repositories::UserRepository;
use template_studio_shared::models::auth::{Claims, JwtConfig};
use template_studio_shared::models::user::{
    LoginRequest, LoginResponse, RegisterRequest, UserInfoResponse,
};
use tracing::{info, warn};

/// 连续失败 N 次触发账号锁定
const MAX_FAILED_LOGINS: i32 = 5;
/// 锁定时长（分钟）
const LOCKOUT_MINUTES: i64 = 15;

pub struct AuthService {
    user_repo: Arc<UserRepository>,
    jwt_config: JwtConfig,
}

impl AuthService {
    pub fn new(user_repo: Arc<UserRepository>, jwt_config: JwtConfig) -> Self {
        Self {
            user_repo,
            jwt_config,
        }
    }

    pub async fn login(&self, req: &LoginRequest) -> Result<LoginResponse> {
        info!("登录请求: username={}", req.username);

        let user = self
            .user_repo
            .find_by_username(&req.username)
            .await?
            .ok_or_else(|| {
                warn!("登录失败: 用户 '{}' 不存在", req.username);
                anyhow!("用户名或密码错误")
            })?;

        if user.status != 1 {
            warn!("登录失败: 用户 '{}' 已被禁用", req.username);
            return Err(anyhow!("该账号已被禁用"));
        }

        // 账号级锁定：防分布式撞库（IP 级限速之外的第二道闸）
        if let Some(until) = user.locked_until {
            if Utc::now() < until {
                let mins = (until - Utc::now()).num_minutes() + 1;
                warn!(
                    "登录被拒: 用户 '{}' 账号锁定中（剩 {} 分钟）",
                    req.username, mins
                );
                return Err(anyhow!(
                    "失败次数过多，账号已锁定，请约 {} 分钟后再试",
                    mins
                ));
            }
        }

        let valid =
            verify(&req.password, &user.password_hash).map_err(|_| anyhow!("密码验证失败"))?;
        if !valid {
            warn!("登录失败: 用户 '{}' 密码错误", req.username);
            let count = self.user_repo.record_login_failure(user.id).await?;
            if count >= MAX_FAILED_LOGINS {
                let until = Utc::now() + Duration::minutes(LOCKOUT_MINUTES);
                self.user_repo.lock_user_until(user.id, until).await?;
                warn!(
                    "账号锁定: 用户 '{}' 连续失败 {} 次，锁至 {}",
                    req.username, count, until
                );
            }
            return Err(anyhow!("用户名或密码错误"));
        }

        // 登录成功：清零失败计数
        self.user_repo.clear_login_failures(user.id).await?;

        info!("登录成功: user_id={}, username={}", user.id, user.username);

        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: user.id,
            username: user.username.clone(),
            exp: now + self.jwt_config.expire_hours * 3600,
            iat: now,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_config.secret.as_bytes()),
        )?;

        self.user_repo.update_last_login(user.id).await?;

        let roles = self.get_roles_safe(user.id, &user.username).await?;

        Ok(LoginResponse { token, roles })
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_config.secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(data.claims)
    }

    /// 用户注册：创建账号并自动登录
    pub async fn register(&self, req: &RegisterRequest) -> Result<LoginResponse> {
        if self
            .user_repo
            .find_by_username(&req.username)
            .await?
            .is_some()
        {
            return Err(anyhow!("用户名已存在"));
        }

        let password_hash = hash(&req.password, DEFAULT_COST)?;
        let email = req.email.clone().unwrap_or_default();

        let user_id = self
            .user_repo
            .create_user(&req.username, &password_hash, &email)
            .await?;

        self.user_repo.update_last_login(user_id).await?;

        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: user_id,
            username: req.username.clone(),
            exp: now + self.jwt_config.expire_hours * 3600,
            iat: now,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_config.secret.as_bytes()),
        )?;

        // 新注册用户默认没有角色
        Ok(LoginResponse {
            token,
            roles: vec![],
        })
    }

    pub async fn get_user_info(&self, user_id: i64) -> Result<UserInfoResponse> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| anyhow!("用户不存在"))?;

        let permissions = self.user_repo.get_user_permissions(user_id).await?;
        let roles = self.get_roles_safe(user_id, &user.username).await?;

        Ok(UserInfoResponse {
            username: user.username,
            email: user.email,
            avatar: user.avatar,
            bio: user.bio,
            roles,
            permissions,
        })
    }

    /// 获取用户角色，admin 用户兜底确保拥有 super_admin 角色
    async fn get_roles_safe(&self, user_id: i64, username: &str) -> Result<Vec<String>> {
        if username == "admin" {
            let _ = self
                .user_repo
                .ensure_role_by_name(user_id, "super_admin")
                .await;
        }
        let roles = self
            .user_repo
            .get_user_roles(user_id)
            .await?
            .into_iter()
            .map(|r| r.name)
            .collect();
        Ok(roles)
    }

    /// 获取用户角色名列表（供 admin 角色校验中间件使用，每次查库保证角色变更即时生效）
    pub async fn get_user_role_names(&self, user_id: i64) -> Result<Vec<String>> {
        let roles = self
            .user_repo
            .get_user_roles(user_id)
            .await?
            .into_iter()
            .map(|r| r.name)
            .collect();
        Ok(roles)
    }
}
