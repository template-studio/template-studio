use super::Database;
use sqlx::Row;

impl Database {
    /// ===== 系统级类型映射操作 =====
    /// 获取系统级类型映射
    pub async fn get_system_type_mappings(&self) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT
                stm.id,
                stm.language_id,
                l.name as language_name,
                stm.db_type,
                stm.pattern,
                stm.target_type,
                stm.priority,
                stm.created_at,
                stm.updated_at
             FROM system_type_mappings stm
             LEFT JOIN languages l ON stm.language_id = l.id
             ORDER BY l.name, stm.db_type, stm.priority DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        let mappings = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.get::<i64, _>("id"),
                    "language_id": row.get::<i64, _>("language_id"),
                    "language_name": row.get::<String, _>("language_name"),
                    "db_type": row.get::<String, _>("db_type"),
                    "pattern": row.get::<String, _>("pattern"),
                    "target_type": row.get::<String, _>("target_type"),
                    "priority": row.get::<i32, _>("priority"),
                    "created_at": row.get::<String, _>("created_at"),
                    "updated_at": row.get::<String, _>("updated_at")
                })
            })
            .collect();

        Ok(mappings)
    }

    /// 根据语言和数据库类型获取系统级类型映射
    pub async fn get_system_type_mappings_by_lang_db(
        &self,
        language_id: i64,
        db_type: &str,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT
                stm.id,
                stm.language_id,
                l.name as language_name,
                stm.db_type,
                stm.pattern,
                stm.target_type,
                stm.priority,
                stm.created_at,
                stm.updated_at
             FROM system_type_mappings stm
             LEFT JOIN languages l ON stm.language_id = l.id
             WHERE stm.language_id = ?1 AND stm.db_type = ?2
             ORDER BY stm.priority DESC",
        )
        .bind(language_id)
        .bind(db_type)
        .fetch_all(&self.pool)
        .await?;

        let mappings = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.get::<i64, _>("id"),
                    "language_id": row.get::<i64, _>("language_id"),
                    "language_name": row.get::<String, _>("language_name"),
                    "db_type": row.get::<String, _>("db_type"),
                    "pattern": row.get::<String, _>("pattern"),
                    "target_type": row.get::<String, _>("target_type"),
                    "priority": row.get::<i32, _>("priority"),
                    "created_at": row.get::<String, _>("created_at"),
                    "updated_at": row.get::<String, _>("updated_at")
                })
            })
            .collect();

        Ok(mappings)
    }

    /// 创建系统级类型映射
    pub async fn create_system_type_mapping(
        &self,
        language_id: i64,
        db_type: &str,
        pattern: &str,
        target_type: &str,
        priority: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO system_type_mappings (language_id, db_type, pattern, target_type, priority)
             VALUES (?1, ?2, ?3, ?4, ?5)"
        )
        .bind(language_id)
        .bind(db_type)
        .bind(pattern)
        .bind(target_type)
        .bind(priority)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 更新系统级类型映射
    pub async fn update_system_type_mapping(
        &self,
        id: i64,
        target_type: &str,
        priority: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE system_type_mappings
             SET target_type = ?1, priority = ?2, updated_at = datetime('now')
             WHERE id = ?3",
        )
        .bind(target_type)
        .bind(priority)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除系统级类型映射
    pub async fn delete_system_type_mapping(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM system_type_mappings WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 批量保存系统级类型映射
    pub async fn batch_save_system_type_mappings(
        &self,
        mappings: Vec<serde_json::Value>,
    ) -> Result<(), sqlx::Error> {
        // 先删除所有现有映射
        sqlx::query("DELETE FROM system_type_mappings")
            .execute(&self.pool)
            .await?;

        // 重新插入新映射
        for mapping in mappings {
            let language_id: i64 = mapping["language_id"].as_i64().unwrap_or(0);
            let db_type: String = mapping["db_type"].as_str().unwrap_or("").to_string();
            let pattern: String = mapping["pattern"].as_str().unwrap_or("").to_string();
            let target_type: String = mapping["target_type"].as_str().unwrap_or("").to_string();
            let priority: i32 = mapping["priority"].as_i64().unwrap_or(10) as i32;

            if language_id > 0
                && !db_type.is_empty()
                && !pattern.is_empty()
                && !target_type.is_empty()
            {
                sqlx::query(
                    "INSERT INTO system_type_mappings (language_id, db_type, pattern, target_type, priority)
                     VALUES (?1, ?2, ?3, ?4, ?5)"
                )
                .bind(language_id)
                .bind(db_type)
                .bind(pattern)
                .bind(target_type)
                .bind(priority)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    /// ===== 项目级类型映射操作 =====
    /// 获取项目级类型映射
    pub async fn get_project_type_mappings(
        &self,
        project_id: i64,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT
                ptm.id,
                ptm.project_id,
                ptm.scope,
                ptm.db_type,
                ptm.pattern,
                ptm.target_type,
                ptm.priority,
                ptm.created_at,
                ptm.updated_at
             FROM project_type_mappings ptm
             WHERE ptm.project_id = ?1
             ORDER BY ptm.scope, ptm.db_type, ptm.priority DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        let mappings = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.get::<i64, _>("id"),
                    "project_id": row.get::<i64, _>("project_id"),
                    "scope": row.get::<String, _>("scope"),
                    "db_type": row.get::<String, _>("db_type"),
                    "pattern": row.get::<String, _>("pattern"),
                    "target_type": row.get::<String, _>("target_type"),
                    "priority": row.get::<i32, _>("priority"),
                    "created_at": row.get::<String, _>("created_at"),
                    "updated_at": row.get::<String, _>("updated_at")
                })
            })
            .collect();

        Ok(mappings)
    }

    /// 根据项目和范围获取项目级类型映射
    pub async fn get_project_type_mappings_by_scope(
        &self,
        project_id: i64,
        scope: &str,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT
                ptm.id,
                ptm.project_id,
                ptm.scope,
                ptm.db_type,
                ptm.pattern,
                ptm.target_type,
                ptm.priority,
                ptm.created_at,
                ptm.updated_at
             FROM project_type_mappings ptm
             WHERE ptm.project_id = ?1 AND ptm.scope = ?2
             ORDER BY ptm.db_type, ptm.priority DESC",
        )
        .bind(project_id)
        .bind(scope)
        .fetch_all(&self.pool)
        .await?;

        let mappings = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.get::<i64, _>("id"),
                    "project_id": row.get::<i64, _>("project_id"),
                    "scope": row.get::<String, _>("scope"),
                    "db_type": row.get::<String, _>("db_type"),
                    "pattern": row.get::<String, _>("pattern"),
                    "target_type": row.get::<String, _>("target_type"),
                    "priority": row.get::<i32, _>("priority"),
                    "created_at": row.get::<String, _>("created_at"),
                    "updated_at": row.get::<String, _>("updated_at")
                })
            })
            .collect();

        Ok(mappings)
    }

    /// 创建项目级类型映射
    pub async fn create_project_type_mapping(
        &self,
        project_id: i64,
        scope: &str,
        db_type: &str,
        pattern: &str,
        target_type: &str,
        priority: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO project_type_mappings (project_id, scope, db_type, pattern, target_type, priority)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        )
        .bind(project_id)
        .bind(scope)
        .bind(db_type)
        .bind(pattern)
        .bind(target_type)
        .bind(priority)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 更新项目级类型映射
    pub async fn update_project_type_mapping(
        &self,
        id: i64,
        target_type: &str,
        priority: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE project_type_mappings
             SET target_type = ?1, priority = ?2, updated_at = datetime('now')
             WHERE id = ?3",
        )
        .bind(target_type)
        .bind(priority)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除项目级类型映射
    pub async fn delete_project_type_mapping(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM project_type_mappings WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 批量保存项目级类型映射
    pub async fn batch_save_project_type_mappings(
        &self,
        project_id: i64,
        scope: &str,
        mappings: Vec<serde_json::Value>,
    ) -> Result<(), sqlx::Error> {
        // 先删除该项目该范围的所有现有映射
        sqlx::query("DELETE FROM project_type_mappings WHERE project_id = ?1 AND scope = ?2")
            .bind(project_id)
            .bind(scope)
            .execute(&self.pool)
            .await?;

        // 重新插入新映射
        for mapping in mappings {
            let db_type: String = mapping["db_type"].as_str().unwrap_or("").to_string();
            let pattern: String = mapping["pattern"].as_str().unwrap_or("").to_string();
            let target_type: String = mapping["target_type"].as_str().unwrap_or("").to_string();
            let priority: i32 = mapping["priority"].as_i64().unwrap_or(10) as i32;

            if !db_type.is_empty() && !pattern.is_empty() && !target_type.is_empty() {
                sqlx::query(
                    "INSERT INTO project_type_mappings (project_id, scope, db_type, pattern, target_type, priority)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
                )
                .bind(project_id)
                .bind(scope)
                .bind(db_type)
                .bind(pattern)
                .bind(target_type)
                .bind(priority)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    /// 复制系统级映射到项目级
    pub async fn copy_system_mappings_to_project(
        &self,
        project_id: i64,
        language_id: i64,
        scope: &str,
        db_type: &str,
    ) -> Result<(), sqlx::Error> {
        // 先删除该项目该范围的现有映射
        sqlx::query("DELETE FROM project_type_mappings WHERE project_id = ?1 AND scope = ?2 AND db_type = ?3")
            .bind(project_id)
            .bind(scope)
            .bind(db_type)
            .execute(&self.pool)
            .await?;

        // 复制系统级映射到项目级
        sqlx::query(
            "INSERT INTO project_type_mappings (project_id, scope, db_type, pattern, target_type, priority)
             SELECT ?1, ?2, stm.db_type, stm.pattern, stm.target_type, stm.priority
             FROM system_type_mappings stm
             WHERE stm.language_id = ?3 AND stm.db_type = ?4"
        )
        .bind(project_id)
        .bind(scope)
        .bind(language_id)
        .bind(db_type)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
