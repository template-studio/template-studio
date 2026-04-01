use sqlx::MySqlPool;
use template_studio_shared::models::system_setting::SystemSetting;
use anyhow::Result;

/// 系统设置数据访问层
pub struct SystemSettingRepository {
    pool: MySqlPool,
}

impl SystemSettingRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// 按 group 获取设置列表
    pub async fn get_by_group(&self, group: &str) -> Result<Vec<SystemSetting>> {
        let settings = sqlx::query_as::<_, SystemSetting>(
            "SELECT id, `group`, `key`, value, description, sort, created_at, updated_at FROM system_settings WHERE `group` = ? ORDER BY sort ASC"
        )
        .bind(group)
        .fetch_all(&self.pool)
        .await?;

        Ok(settings)
    }

    /// 按 group + key 获取单个设置
    pub async fn get_by_key(&self, group: &str, key: &str) -> Result<Option<SystemSetting>> {
        let setting = sqlx::query_as::<_, SystemSetting>(
            "SELECT id, `group`, `key`, value, description, sort, created_at, updated_at FROM system_settings WHERE `group` = ? AND `key` = ?"
        )
        .bind(group)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(setting)
    }

    /// 获取所有设置
    pub async fn get_all(&self) -> Result<Vec<SystemSetting>> {
        let settings = sqlx::query_as::<_, SystemSetting>(
            "SELECT id, `group`, `key`, value, description, sort, created_at, updated_at FROM system_settings ORDER BY `group` ASC, sort ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(settings)
    }

    /// 插入或更新设置（upsert）
    pub async fn upsert(&self, group: &str, key: &str, value: &str, description: Option<&str>) -> Result<bool> {
        let result = sqlx::query(
            "INSERT INTO system_settings (`group`, `key`, value, description) VALUES (?, ?, ?, ?) ON DUPLICATE KEY UPDATE value = VALUES(value), description = VALUES(description), updated_at = CURRENT_TIMESTAMP"
        )
        .bind(group)
        .bind(key)
        .bind(value)
        .bind(description)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 批量 upsert
    pub async fn batch_upsert(&self, group: &str, items: &[(String, String, Option<String>)]) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        for (key, value, description) in items {
            sqlx::query(
                "INSERT INTO system_settings (`group`, `key`, value, description) VALUES (?, ?, ?, ?) ON DUPLICATE KEY UPDATE value = VALUES(value), description = VALUES(description), updated_at = CURRENT_TIMESTAMP"
            )
            .bind(group)
            .bind(key)
            .bind(value)
            .bind(description.as_deref())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// 删除设置
    pub async fn delete(&self, group: &str, key: &str) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM system_settings WHERE `group` = ? AND `key` = ?"
        )
        .bind(group)
        .bind(key)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
