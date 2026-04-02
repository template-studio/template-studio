use std::sync::Arc;
use anyhow::{anyhow, Result};
use bcrypt::hash;
use template_studio_shared::models::user::{UserListItem, CreateUserRequest, UpdateUserRequest, ChangePasswordRequest};
use template_studio_repositories::UserRepository;
use tracing::info;

const DEFAULT_COST: u32 = 12;

pub struct UserService {
    user_repo: Arc<UserRepository>,
}

impl UserService {
    pub fn new(user_repo: Arc<UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn list_users(&self) -> Result<Vec<UserListItem>> {
        self.user_repo.list_users().await
    }

    pub async fn create_user(&self, req: &CreateUserRequest) -> Result<i64> {
        if self.user_repo.find_by_username(&req.username).await?.is_some() {
            return Err(anyhow!("用户名已存在"));
        }

        let password_hash = hash(&req.password, DEFAULT_COST)?;
        let email = req.email.as_deref().unwrap_or("");

        let user_id = self.user_repo.create_user(&req.username, &password_hash, email).await?;
        info!("创建用户: {} (id={})", req.username, user_id);

        if let Some(role_ids) = &req.role_ids {
            if !role_ids.is_empty() {
                self.user_repo.assign_roles(user_id, role_ids).await?;
            }
        }

        Ok(user_id)
    }

    pub async fn update_user(&self, req: &UpdateUserRequest) -> Result<bool> {
        let existing = self.user_repo.find_by_id(req.id).await?
            .ok_or_else(|| anyhow!("用户不存在"))?;

        if let Some(new_password) = &req.password {
            let hash = hash(new_password, DEFAULT_COST)?;
            self.user_repo.update_password(req.id, &hash).await?;
        }

        let result = self.user_repo.update_user(
            req.id,
            req.email.as_deref().or(Some(existing.email.as_str())),
            req.avatar.as_deref().or(Some(existing.avatar.as_str())),
            req.status,
        ).await?;

        Ok(result)
    }

    pub async fn change_password(&self, user_id: i64, req: &ChangePasswordRequest) -> Result<bool> {
        let hash = hash(&req.new_password, DEFAULT_COST)?;
        self.user_repo.update_password(user_id, &hash).await
    }

    pub async fn delete_user(&self, id: i64) -> Result<bool> {
        self.user_repo.delete_user(id).await
    }

    pub async fn assign_roles(&self, user_id: i64, role_ids: &[i64]) -> Result<()> {
        self.user_repo.assign_roles(user_id, role_ids).await
    }
}
