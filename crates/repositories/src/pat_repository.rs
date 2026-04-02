use sqlx::MySqlPool;
use template_studio_shared::models::pat::{PersonalAccessToken, PatListItem};
use anyhow::Result;

pub struct PatRepository {
    pool: MySqlPool,
}

impl PatRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, user_id: i64, name: &str, token_hash: &str, token_prefix: &str, expires_at: Option<chrono::NaiveDateTime>) -> Result<i64> {
        let result = sqlx::query(
            "INSERT INTO personal_access_tokens (user_id, name, token_hash, token_prefix, expires_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(name)
        .bind(token_hash)
        .bind(token_prefix)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_id() as i64)
    }

    pub async fn list_by_user(&self, user_id: i64) -> Result<Vec<PatListItem>> {
        let items = sqlx::query_as::<_, PatListItem>(
            "SELECT id, name, token_prefix, last_used_at, expires_at, created_at \
             FROM personal_access_tokens WHERE user_id = ? ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(items)
    }

    pub async fn find_by_hash(&self, token_hash: &str) -> Result<Option<PersonalAccessToken>> {
        let token = sqlx::query_as::<_, PersonalAccessToken>(
            "SELECT id, user_id, name, token_hash, token_prefix, last_used_at, expires_at, created_at \
             FROM personal_access_tokens WHERE token_hash = ?"
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?;
        Ok(token)
    }

    pub async fn delete(&self, id: i64, user_id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM personal_access_tokens WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_last_used(&self, id: i64) -> Result<()> {
        sqlx::query("UPDATE personal_access_tokens SET last_used_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn count_by_user(&self, user_id: i64) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM personal_access_tokens WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0)
    }

    pub async fn list_by_prefix_like(&self, prefix: &str) -> Result<Vec<PersonalAccessToken>> {
        let items = sqlx::query_as::<_, PersonalAccessToken>(
            "SELECT id, user_id, name, token_hash, token_prefix, last_used_at, expires_at, created_at \
             FROM personal_access_tokens WHERE token_prefix LIKE ?"
        )
        .bind(format!("{}%", prefix))
        .fetch_all(&self.pool)
        .await?;
        Ok(items)
    }
}
