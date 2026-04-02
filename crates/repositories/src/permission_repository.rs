use sqlx::MySqlPool;
use template_studio_shared::models::permission::Permission;
use anyhow::Result;

pub struct PermissionRepository {
    pool: MySqlPool,
}

impl PermissionRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn list_permissions(&self) -> Result<Vec<Permission>> {
        let permissions = sqlx::query_as::<_, Permission>(
            "SELECT id, name, display_name, type, parent_id, sort, status, created_at, updated_at FROM permissions WHERE status = 1 ORDER BY sort ASC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(permissions)
    }

    pub async fn list_all_permissions(&self) -> Result<Vec<Permission>> {
        let permissions = sqlx::query_as::<_, Permission>(
            "SELECT id, name, display_name, type, parent_id, sort, status, created_at, updated_at FROM permissions ORDER BY sort ASC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(permissions)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Permission>> {
        let perm = sqlx::query_as::<_, Permission>(
            "SELECT id, name, display_name, type, parent_id, sort, status, created_at, updated_at FROM permissions WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(perm)
    }
}
