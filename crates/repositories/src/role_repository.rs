use sqlx::MySqlPool;
use template_studio_shared::models::role::Role;
use anyhow::Result;

pub struct RoleRepository {
    pool: MySqlPool,
}

impl RoleRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Role>> {
        let role = sqlx::query_as::<_, Role>(
            "SELECT id, name, display_name, description, sort, status, created_at, updated_at FROM roles WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(role)
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<Role>> {
        let role = sqlx::query_as::<_, Role>(
            "SELECT id, name, display_name, description, sort, status, created_at, updated_at FROM roles WHERE name = ?"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(role)
    }

    pub async fn list_roles(&self) -> Result<Vec<Role>> {
        let roles = sqlx::query_as::<_, Role>(
            "SELECT id, name, display_name, description, sort, status, created_at, updated_at FROM roles ORDER BY sort ASC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(roles)
    }

    pub async fn create_role(&self, name: &str, display_name: &str, description: &str, sort: i32) -> Result<i64> {
        let result = sqlx::query(
            "INSERT INTO roles (name, display_name, description, sort) VALUES (?, ?, ?, ?)"
        )
        .bind(name)
        .bind(display_name)
        .bind(description)
        .bind(sort)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_id() as i64)
    }

    pub async fn update_role(&self, id: i64, display_name: Option<&str>, description: Option<&str>, sort: Option<i32>, status: Option<i8>) -> Result<bool> {
        let mut sets = Vec::new();
        if display_name.is_some() { sets.push("display_name = ?"); }
        if description.is_some() { sets.push("description = ?"); }
        if sort.is_some() { sets.push("sort = ?"); }
        if status.is_some() { sets.push("status = ?"); }
        if sets.is_empty() { return Ok(false); }

        let sql = format!("UPDATE roles SET {} WHERE id = ?", sets.join(", "));
        let mut query = sqlx::query(&sql);
        if let Some(v) = display_name { query = query.bind(v); }
        if let Some(v) = description { query = query.bind(v); }
        if let Some(v) = sort { query = query.bind(v); }
        if let Some(v) = status { query = query.bind(v); }
        query = query.bind(id);

        let result = query.execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_role(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM roles WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_role_permissions(&self, role_id: i64) -> Result<Vec<i64>> {
        let ids: Vec<(i64,)> = sqlx::query_as(
            "SELECT permission_id FROM role_permissions WHERE role_id = ?"
        )
        .bind(role_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(ids.into_iter().map(|(id,)| id).collect())
    }

    pub async fn assign_permissions(&self, role_id: i64, permission_ids: &[i64]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM role_permissions WHERE role_id = ?")
            .bind(role_id)
            .execute(&mut *tx)
            .await?;
        for pid in permission_ids {
            sqlx::query("INSERT INTO role_permissions (role_id, permission_id) VALUES (?, ?)")
                .bind(role_id)
                .bind(pid)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}
