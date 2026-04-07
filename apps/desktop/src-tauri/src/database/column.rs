use sqlx::Row;
use super::{Database, DbColumn};

impl Database {
    /// 创建列记录
    pub async fn create_column(
        &self,
        table_id: i64,
        name: &str,
        data_type: &str,
        length: Option<i64>,
        is_nullable: bool,
        is_primary_key: bool,
        is_unique: bool,
        default_value: Option<&str>,
        comment: Option<&str>,
        ordinal_position: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO db_columns (table_id, name, data_type, length, is_nullable, is_primary_key, is_unique, default_value, comment, ordinal_position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
        )
        .bind(table_id)
        .bind(name)
        .bind(data_type)
        .bind(length)
        .bind(is_nullable as i32)
        .bind(is_primary_key as i32)
        .bind(is_unique as i32)
        .bind(default_value)
        .bind(comment)
        .bind(ordinal_position)
        .execute(&self.pool)
        .await?;

        // 更新表的列计数
        sqlx::query(
            "UPDATE db_tables SET column_count = (
                SELECT COUNT(*) FROM db_columns WHERE table_id = ?1
            ), updated_at = datetime('now') WHERE id = ?1"
        )
        .bind(table_id)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 获取表的所有列
    pub async fn get_table_columns(&self, table_id: i64) -> Result<Vec<DbColumn>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, table_id, name, data_type, length, is_nullable, is_primary_key, is_unique, default_value, comment, ordinal_position, created_at
             FROM db_columns
             WHERE table_id = ?1
             ORDER BY ordinal_position"
        )
        .bind(table_id)
        .fetch_all(&self.pool)
        .await?;

        let columns = rows.into_iter().map(|row| {
            DbColumn {
                id: row.get("id"),
                table_id: row.get("table_id"),
                name: row.get("name"),
                data_type: row.get("data_type"),
                length: row.get("length"),
                is_nullable: row.get::<i32, _>("is_nullable") == 1,
                is_primary_key: row.get::<i32, _>("is_primary_key") == 1,
                is_unique: row.get::<i32, _>("is_unique") == 1,
                default_value: row.get("default_value"),
                comment: row.get("comment"),
                ordinal_position: row.get("ordinal_position"),
                created_at: row.get("created_at"),
            }
        }).collect();

        Ok(columns)
    }

    /// 更新列信息
    pub async fn update_column(
        &self,
        column_id: i64,
        name: &str,
        data_type: &str,
        length: Option<i64>,
        is_nullable: bool,
        is_primary_key: bool,
        is_unique: bool,
        default_value: Option<&str>,
        comment: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE db_columns
             SET name = ?1, data_type = ?2, length = ?3, is_nullable = ?4,
                 is_primary_key = ?5, is_unique = ?6, default_value = ?7, comment = ?8
             WHERE id = ?9"
        )
        .bind(name)
        .bind(data_type)
        .bind(length)
        .bind(is_nullable)
        .bind(is_primary_key)
        .bind(is_unique)
        .bind(default_value)
        .bind(comment)
        .bind(column_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除列
    pub async fn delete_column(&self, column_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM db_columns WHERE id = ?1")
            .bind(column_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 更新列位置
    pub async fn update_column_position(&self, column_id: i64, position: i32) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE db_columns SET ordinal_position = ?1 WHERE id = ?2")
            .bind(position)
            .bind(column_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
