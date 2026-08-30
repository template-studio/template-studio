use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use template_studio_shared::models::role::Role;
use template_studio_shared::models::user::PermissionItem;
use template_studio_shared::models::user::{User, UserListItem};

pub struct UserRepository {
    pool: MySqlPool,
}

impl UserRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, email, avatar, bio, status, last_login_at, created_at, updated_at, failed_login_count, locked_until FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, email, avatar, bio, status, last_login_at, created_at, updated_at, failed_login_count, locked_until FROM users WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    pub async fn list_users(&self) -> Result<Vec<UserListItem>> {
        let users = sqlx::query_as::<_, UserListItem>(
            "SELECT id, username, email, avatar, bio, status, last_login_at, created_at, updated_at FROM users ORDER BY id ASC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }

    pub async fn create_user(
        &self,
        username: &str,
        password_hash: &str,
        email: &str,
    ) -> Result<i64> {
        let result =
            sqlx::query("INSERT INTO users (username, password_hash, email) VALUES (?, ?, ?)")
                .bind(username)
                .bind(password_hash)
                .bind(email)
                .execute(&self.pool)
                .await?;
        Ok(result.last_insert_id() as i64)
    }

    pub async fn update_user(
        &self,
        id: i64,
        email: Option<&str>,
        avatar: Option<&str>,
        status: Option<i8>,
    ) -> Result<bool> {
        let mut sets = Vec::new();
        if email.is_some() {
            sets.push("email = ?");
        }
        if avatar.is_some() {
            sets.push("avatar = ?");
        }
        if status.is_some() {
            sets.push("status = ?");
        }
        if sets.is_empty() {
            return Ok(false);
        }

        let sql = format!("UPDATE users SET {} WHERE id = ?", sets.join(", "));
        let mut query = sqlx::query(&sql);
        if let Some(v) = email {
            query = query.bind(v);
        }
        if let Some(v) = avatar {
            query = query.bind(v);
        }
        if let Some(v) = status {
            query = query.bind(v);
        }
        query = query.bind(id);

        let result = query.execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_password(&self, id: i64, password_hash: &str) -> Result<bool> {
        let result = sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
            .bind(password_hash)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// 记录一次登录失败（返回更新后的连续失败次数）
    pub async fn record_login_failure(&self, id: i64) -> Result<i32> {
        sqlx::query("UPDATE users SET failed_login_count = failed_login_count + 1 WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        let count: i32 =
            sqlx::query_scalar("SELECT failed_login_count FROM users WHERE id = ?")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
        Ok(count)
    }

    /// 登录成功后清零失败计数与锁定
    pub async fn clear_login_failures(&self, id: i64) -> Result<()> {
        sqlx::query("UPDATE users SET failed_login_count = 0, locked_until = NULL WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 锁定账号至指定时间
    pub async fn lock_user_until(&self, id: i64, until: DateTime<Utc>) -> Result<()> {
        sqlx::query("UPDATE users SET locked_until = ? WHERE id = ?")
            .bind(until)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_last_login(&self, id: i64) -> Result<()> {
        sqlx::query("UPDATE users SET last_login_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_user(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_user_roles(&self, user_id: i64) -> Result<Vec<Role>> {
        let roles = sqlx::query_as::<_, Role>(
            "SELECT r.id, r.name, r.display_name, r.description, r.sort, r.status, r.created_at, r.updated_at \
             FROM roles r INNER JOIN user_roles ur ON r.id = ur.role_id WHERE ur.user_id = ? ORDER BY r.sort ASC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(roles)
    }

    pub async fn get_user_permissions(&self, user_id: i64) -> Result<Vec<PermissionItem>> {
        let permissions = sqlx::query_as::<_, (String, String)>(
            "SELECT DISTINCT p.name, p.display_name \
             FROM permissions p \
             INNER JOIN role_permissions rp ON p.id = rp.permission_id \
             INNER JOIN user_roles ur ON rp.role_id = ur.role_id \
             WHERE ur.user_id = ? AND p.status = 1 \
             ORDER BY p.name ASC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(permissions
            .into_iter()
            .map(|(value, label)| PermissionItem { value, label })
            .collect())
    }

    /// 幂等确保用户拥有指定角色（按角色名），已存在则跳过
    pub async fn ensure_role_by_name(&self, user_id: i64, role_name: &str) -> Result<bool> {
        let result = sqlx::query(
            "INSERT IGNORE INTO user_roles (user_id, role_id) \
             SELECT ?, r.id FROM roles r WHERE r.name = ?",
        )
        .bind(user_id)
        .bind(role_name)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn assign_roles(&self, user_id: i64, role_ids: &[i64]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM user_roles WHERE user_id = ?")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        for role_id in role_ids {
            sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)")
                .bind(user_id)
                .bind(role_id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// 更新个人资料（bio、avatar）
    pub async fn update_profile(
        &self,
        id: i64,
        bio: Option<&str>,
        avatar: Option<&str>,
    ) -> Result<bool> {
        let mut sets = Vec::new();
        if bio.is_some() {
            sets.push("bio = ?");
        }
        if avatar.is_some() {
            sets.push("avatar = ?");
        }
        if sets.is_empty() {
            return Ok(false);
        }

        let sql = format!("UPDATE users SET {} WHERE id = ?", sets.join(", "));
        let mut query = sqlx::query(&sql);
        if let Some(v) = bio {
            query = query.bind(v);
        }
        if let Some(v) = avatar {
            query = query.bind(v);
        }
        query = query.bind(id);

        let result = query.execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    /// 根据用户名查找公开信息
    pub async fn find_public_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, email, avatar, bio, status, last_login_at, created_at, updated_at FROM users WHERE username = ? AND status = 1"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }
}
