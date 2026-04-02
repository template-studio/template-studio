use std::sync::Arc;
use anyhow::{anyhow, Result};
use template_studio_shared::models::role::{Role, CreateRoleRequest, UpdateRoleRequest};
use template_studio_repositories::RoleRepository;
use tracing::info;

pub struct RoleService {
    role_repo: Arc<RoleRepository>,
}

impl RoleService {
    pub fn new(role_repo: Arc<RoleRepository>) -> Self {
        Self { role_repo }
    }

    pub async fn list_roles(&self) -> Result<Vec<Role>> {
        self.role_repo.list_roles().await
    }

    pub async fn create_role(&self, req: &CreateRoleRequest) -> Result<i64> {
        if self.role_repo.find_by_name(&req.name).await?.is_some() {
            return Err(anyhow!("角色标识已存在"));
        }

        let sort = req.sort.unwrap_or(0);
        let description = req.description.as_deref().unwrap_or("");

        let role_id = self.role_repo.create_role(&req.name, &req.display_name, description, sort).await?;
        info!("创建角色: {} (id={})", req.name, role_id);

        if let Some(permission_ids) = &req.permission_ids {
            if !permission_ids.is_empty() {
                self.role_repo.assign_permissions(role_id, permission_ids).await?;
            }
        }

        Ok(role_id)
    }

    pub async fn update_role(&self, req: &UpdateRoleRequest) -> Result<bool> {
        self.role_repo.find_by_id(req.id).await?
            .ok_or_else(|| anyhow!("角色不存在"))?;

        self.role_repo.update_role(
            req.id,
            req.display_name.as_deref(),
            req.description.as_deref(),
            req.sort,
            req.status,
        ).await
    }

    pub async fn delete_role(&self, id: i64) -> Result<bool> {
        self.role_repo.delete_role(id).await
    }

    pub async fn assign_permissions(&self, role_id: i64, permission_ids: &[i64]) -> Result<()> {
        self.role_repo.assign_permissions(role_id, permission_ids).await
    }

    pub async fn get_role_permissions(&self, role_id: i64) -> Result<Vec<i64>> {
        self.role_repo.get_role_permissions(role_id).await
    }
}
