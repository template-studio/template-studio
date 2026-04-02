use std::sync::Arc;
use anyhow::Result;
use template_studio_shared::models::permission::{Permission, PermissionTree};
use template_studio_repositories::PermissionRepository;

pub struct PermissionService {
    perm_repo: Arc<PermissionRepository>,
}

impl PermissionService {
    pub fn new(perm_repo: Arc<PermissionRepository>) -> Self {
        Self { perm_repo }
    }

    pub async fn list_permissions(&self) -> Result<Vec<Permission>> {
        self.perm_repo.list_permissions().await
    }

    pub async fn list_all_permissions(&self) -> Result<Vec<Permission>> {
        self.perm_repo.list_all_permissions().await
    }

    pub async fn get_permission_tree(&self) -> Result<Vec<PermissionTree>> {
        let permissions = self.perm_repo.list_all_permissions().await?;
        Ok(build_tree(&permissions, None))
    }
}

fn build_tree(permissions: &[Permission], parent_id: Option<i64>) -> Vec<PermissionTree> {
    permissions
        .iter()
        .filter(|p| p.parent_id == parent_id)
        .map(|p| PermissionTree {
            id: p.id,
            name: p.name.clone(),
            display_name: p.display_name.clone(),
            r#type: p.r#type.clone(),
            sort: p.sort,
            children: build_tree(permissions, Some(p.id)),
        })
        .collect()
}
