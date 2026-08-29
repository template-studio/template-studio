use super::{Database, Language};
use sqlx::Row;

impl Database {
    /// ===== 语言操作 =====
    /// 创建语言
    pub async fn create_language(
        &self,
        name: &str,
        icon: Option<&str>,
        color: Option<&str>,
        description: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO languages (name, icon, color, description) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(name)
        .bind(icon)
        .bind(color)
        .bind(description)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 获取所有语言
    pub async fn get_all_languages(&self) -> Result<Vec<Language>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, name, icon, color, description, is_builtin, is_active, created_at, updated_at
             FROM languages
             WHERE is_active = 1
             ORDER BY is_builtin DESC, name ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        let languages = rows
            .into_iter()
            .map(|row| Language {
                id: row.get("id"),
                name: row.get("name"),
                icon: row.get("icon"),
                color: row.get("color"),
                description: row.get("description"),
                is_builtin: row.get::<i32, _>("is_builtin") == 1,
                is_active: row.get::<i32, _>("is_active") == 1,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(languages)
    }

    /// 根据 ID 获取语言
    pub async fn get_language(&self, id: i64) -> Result<Option<Language>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, icon, color, description, is_builtin, is_active, created_at, updated_at
             FROM languages
             WHERE id = ?1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Language {
            id: r.get("id"),
            name: r.get("name"),
            icon: r.get("icon"),
            color: r.get("color"),
            description: r.get("description"),
            is_builtin: r.get::<i32, _>("is_builtin") == 1,
            is_active: r.get::<i32, _>("is_active") == 1,
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    /// 更新语言
    pub async fn update_language(
        &self,
        id: i64,
        name: &str,
        icon: Option<&str>,
        color: Option<&str>,
        description: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE languages
             SET name = ?1, icon = ?2, color = ?3, description = ?4, updated_at = datetime('now')
             WHERE id = ?5",
        )
        .bind(name)
        .bind(icon)
        .bind(color)
        .bind(description)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除语言
    pub async fn delete_language(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM languages WHERE id = ?1 AND is_builtin = 0")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// ===== 语言类型字段操作 =====
    /// 获取语言的所有类型字段
    pub async fn get_language_field_types(
        &self,
        language_id: i64,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, language_id, name, description, is_builtin, sort_order, created_at, updated_at
             FROM language_field_types
             WHERE language_id = ?1
             ORDER BY sort_order ASC, name ASC"
        )
        .bind(language_id)
        .fetch_all(&self.pool)
        .await?;

        let field_types: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.get::<i64, _>("id"),
                    "language_id": row.get::<i64, _>("language_id"),
                    "name": row.get::<String, _>("name"),
                    "description": row.get::<Option<String>, _>("description"),
                    "is_builtin": row.get::<i32, _>("is_builtin") == 1,
                    "sort_order": row.get::<i32, _>("sort_order"),
                    "created_at": row.get::<String, _>("created_at"),
                    "updated_at": row.get::<String, _>("updated_at"),
                })
            })
            .collect();

        Ok(field_types)
    }

    /// 创建语言类型字段
    pub async fn create_language_field_type(
        &self,
        language_id: i64,
        name: &str,
        description: Option<&str>,
        sort_order: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO language_field_types (language_id, name, description, is_builtin, sort_order)
             VALUES (?1, ?2, ?3, 0, ?4)"
        )
        .bind(language_id)
        .bind(name)
        .bind(description)
        .bind(sort_order)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 更新语言类型字段
    pub async fn update_language_field_type(
        &self,
        id: i64,
        name: &str,
        description: Option<&str>,
        sort_order: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE language_field_types
             SET name = ?1, description = ?2, sort_order = ?3, updated_at = datetime('now')
             WHERE id = ?4 AND is_builtin = 0",
        )
        .bind(name)
        .bind(description)
        .bind(sort_order)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除语言类型字段
    pub async fn delete_language_field_type(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM language_field_types WHERE id = ?1 AND is_builtin = 0")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 批量保存语言类型字段（用于初始化默认类型）
    pub async fn batch_save_language_field_types(
        &self,
        language_id: i64,
        field_types: Vec<serde_json::Value>,
    ) -> Result<(), sqlx::Error> {
        // 开始事务
        let mut tx = self.pool.begin().await?;

        // 删除该语言的所有非内置类型字段
        sqlx::query("DELETE FROM language_field_types WHERE language_id = ?1 AND is_builtin = 0")
            .bind(language_id)
            .execute(&mut *tx)
            .await?;

        // 插入新的类型字段
        for (index, field_type) in field_types.iter().enumerate() {
            let name = field_type
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let description = field_type.get("description").and_then(|v| v.as_str());
            let is_builtin = field_type
                .get("is_builtin")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            sqlx::query(
                "INSERT INTO language_field_types (language_id, name, description, is_builtin, sort_order)
                 VALUES (?1, ?2, ?3, ?4, ?5)"
            )
            .bind(language_id)
            .bind(name)
            .bind(description)
            .bind(if is_builtin { 1 } else { 0 })
            .bind(index as i32)
            .execute(&mut *tx)
            .await?;
        }

        // 提交事务
        tx.commit().await?;

        Ok(())
    }
}
