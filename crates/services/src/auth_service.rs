use std::sync::Arc;
use anyhow::{anyhow, Result};
use bcrypt::{verify, hash, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use template_studio_shared::models::auth::{Claims, JwtConfig};
use template_studio_shared::models::user::{LoginRequest, LoginResponse, UserInfoResponse, RegisterRequest};
use template_studio_repositories::UserRepository;

pub struct AuthService {
    user_repo: Arc<UserRepository>,
    jwt_config: JwtConfig,
}

impl AuthService {
    pub fn new(user_repo: Arc<UserRepository>, jwt_config: JwtConfig) -> Self {
        Self { user_repo, jwt_config }
    }

    pub async fn login(&self, req: &LoginRequest) -> Result<LoginResponse> {
        let user = self.user_repo
            .find_by_username(&req.username)
            .await?
            .ok_or_else(|| anyhow!("用户名或密码错误"))?;

        if user.status != 1 {
            return Err(anyhow!("该账号已被禁用"));
        }

        let valid = verify(&req.password, &user.password_hash)
            .map_err(|_| anyhow!("密码验证失败"))?;
        if !valid {
            return Err(anyhow!("用户名或密码错误"));
        }

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

        let roles = self.user_repo.get_user_roles(user.id).await?
            .into_iter().map(|r| r.name).collect();

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
        if self.user_repo.find_by_username(&req.username).await?.is_some() {
            return Err(anyhow!("用户名已存在"));
        }

        let password_hash = hash(&req.password, DEFAULT_COST)?;
        let email = req.email.clone().unwrap_or_default();

        let user_id = self.user_repo.create_user(
            &req.username,
            &password_hash,
            &email,
        ).await?;

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
        Ok(LoginResponse { token, roles: vec![] })
    }

    pub async fn get_user_info(&self, user_id: i64) -> Result<UserInfoResponse> {
        let user = self.user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| anyhow!("用户不存在"))?;

        let permissions = self.user_repo.get_user_permissions(user_id).await?;
        let roles = self.user_repo.get_user_roles(user_id).await?
            .into_iter().map(|r| r.name).collect();

        Ok(UserInfoResponse {
            username: user.username,
            email: user.email,
            avatar: user.avatar,
            roles,
            permissions,
        })
    }
}
