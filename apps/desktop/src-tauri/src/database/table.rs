use sqlx::Row;
use super::{Database, DbTable};

impl Database {
    /// ===== 表操作 =====
    /// 为项目创建表
    pub async fn create_table(
        &self,
        project_id: i64,
        name: &str,
        comment: Option<&str>,
        engine: Option<&str>,
        table_type: &str,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO db_tables (project_id, name, comment, engine, table_type)
             VALUES (?1, ?2, ?3, ?4, ?5)"
        )
        .bind(project_id)
        .bind(name)
        .bind(comment)
        .bind(engine)
        .bind(table_type)
        .execute(&self.pool)
        .await?;

        // 更新项目的表计数
        sqlx::query(
            "UPDATE projects SET table_count = (
                SELECT COUNT(*) FROM db_tables WHERE project_id = ?1
            ), updated_at = datetime('now') WHERE id = ?1"
        )
        .bind(project_id)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 获取项目的所有表
    pub async fn get_project_tables(&self, project_id: i64) -> Result<Vec<DbTable>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, project_id, name, comment, engine, table_type, row_count, column_count, created_at, updated_at
             FROM db_tables
             WHERE project_id = ?1
             ORDER BY name"
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        let tables = rows.into_iter().map(|row| {
            DbTable {
                id: row.get("id"),
                project_id: row.get("project_id"),
                name: row.get("name"),
                comment: row.get("comment"),
                engine: row.get("engine"),
                table_type: row.get("table_type"),
                row_count: row.get("row_count"),
                column_count: row.get("column_count"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }).collect();

        Ok(tables)
    }

    /// 删除表
    pub async fn delete_table(&self, table_id: i64) -> Result<(), sqlx::Error> {
        // 删除表（由于有外键约束，列会自动级联删除）
        sqlx::query("DELETE FROM db_tables WHERE id = ?1")
            .bind(table_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 更新表信息
    pub async fn update_table(
        &self,
        table_id: i64,
        name: &str,
        comment: Option<&str>,
        engine: Option<&str>,
        table_type: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE db_tables
             SET name = ?1, comment = ?2, engine = ?3, table_type = ?4, updated_at = datetime('now')
             WHERE id = ?5"
        )
        .bind(name)
        .bind(comment)
        .bind(engine)
        .bind(table_type)
        .bind(table_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
