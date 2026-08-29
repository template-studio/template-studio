use super::Database;
use sqlx::Row;

impl Database {
    // ===== 项目表规范管理 =====

    /// 获取项目的表规范配置
    pub async fn get_table_preferences(
        &self,
        project_id: i64,
    ) -> Result<Option<serde_json::Value>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM table_preferences WHERE project_id = ?1")
            .bind(project_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| {
            serde_json::json!({
                "id": r.get::<i64, _>("id"),
                "projectId": r.get::<i64, _>("project_id"),
                "pkEnabled": r.get::<i32, _>("pk_enabled") == 1,
                "pkFieldName": r.get::<String, _>("pk_field_name"),
                "pkFieldType": r.get::<String, _>("pk_field_type"),
                "pkAutoIncrement": r.get::<i32, _>("pk_auto_increment") == 1,
                "pkComment": r.get::<Option<String>, _>("pk_comment"),
                "auditEnabled": r.get::<i32, _>("audit_enabled") == 1,
                "auditFields": r.get::<Option<String>, _>("audit_fields"),
                "softDeleteEnabled": r.get::<i32, _>("soft_delete_enabled") == 1,
                "softDeleteField": r.get::<String, _>("soft_delete_field"),
                "softDeleteFieldType": r.get::<String, _>("soft_delete_field_type"),
                "softDeleteNullable": r.get::<i32, _>("soft_delete_nullable") == 1,
                "softDeleteDefault": r.get::<Option<String>, _>("soft_delete_default"),
                "softDeleteComment": r.get::<Option<String>, _>("soft_delete_comment"),
                "booleanPrefix": r.get::<Option<String>, _>("boolean_prefix"),
                "datetimeSuffix": r.get::<Option<String>, _>("datetime_suffix"),
                "engineType": r.get::<Option<String>, _>("engine_type"),
                "charset": r.get::<Option<String>, _>("charset"),
                "collation": r.get::<Option<String>, _>("collation"),
                "createdAt": r.get::<String, _>("created_at"),
                "updatedAt": r.get::<String, _>("updated_at"),
            })
        }))
    }

    /// 保存或更新项目表规范配置
    pub async fn save_table_preferences(
        &self,
        project_id: i64,
        prefs: serde_json::Value,
    ) -> Result<i64, sqlx::Error> {
        // 检查是否已存在
        let existing = sqlx::query("SELECT id FROM table_preferences WHERE project_id = ?1")
            .bind(project_id)
            .fetch_optional(&self.pool)
            .await?;

        let pk_enabled = prefs["pkEnabled"].as_bool().unwrap_or(true);
        let pk_field_name = prefs["pkFieldName"].as_str().unwrap_or("id");
        let pk_field_type = prefs["pkFieldType"].as_str().unwrap_or("BIGINT");
        let pk_auto_increment = prefs["pkAutoIncrement"].as_bool().unwrap_or(true);
        let pk_comment = prefs["pkComment"].as_str();
        let audit_enabled = prefs["auditEnabled"].as_bool().unwrap_or(true);
        let audit_fields = prefs["auditFields"].as_str();
        let soft_delete_enabled = prefs["softDeleteEnabled"].as_bool().unwrap_or(false);
        let soft_delete_field = prefs["softDeleteField"].as_str().unwrap_or("deleted_at");
        let soft_delete_field_type = prefs["softDeleteFieldType"].as_str().unwrap_or("TIMESTAMP");
        let soft_delete_nullable = prefs["softDeleteNullable"].as_bool().unwrap_or(true);
        let soft_delete_default = prefs["softDeleteDefault"].as_str();
        let soft_delete_comment = prefs["softDeleteComment"].as_str();
        let boolean_prefix = prefs["booleanPrefix"].as_str();
        let datetime_suffix = prefs["datetimeSuffix"].as_str();
        let engine_type = prefs["engineType"].as_str();
        let charset = prefs["charset"].as_str();
        let collation = prefs["collation"].as_str();

        if let Some(row) = existing {
            // 更新
            let id = row.get::<i64, _>("id");

            sqlx::query(
                "UPDATE table_preferences SET
                    pk_enabled = ?1,
                    pk_field_name = ?2,
                    pk_field_type = ?3,
                    pk_auto_increment = ?4,
                    pk_comment = ?5,
                    audit_enabled = ?6,
                    audit_fields = ?7,
                    soft_delete_enabled = ?8,
                    soft_delete_field = ?9,
                    soft_delete_field_type = ?10,
                    soft_delete_nullable = ?11,
                    soft_delete_default = ?12,
                    soft_delete_comment = ?13,
                    boolean_prefix = ?14,
                    datetime_suffix = ?15,
                    engine_type = ?16,
                    charset = ?17,
                    collation = ?18,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = ?19",
            )
            .bind(pk_enabled as i32)
            .bind(pk_field_name)
            .bind(pk_field_type)
            .bind(pk_auto_increment as i32)
            .bind(pk_comment)
            .bind(audit_enabled as i32)
            .bind(audit_fields)
            .bind(soft_delete_enabled as i32)
            .bind(soft_delete_field)
            .bind(soft_delete_field_type)
            .bind(soft_delete_nullable as i32)
            .bind(soft_delete_default)
            .bind(soft_delete_comment)
            .bind(boolean_prefix)
            .bind(datetime_suffix)
            .bind(engine_type)
            .bind(charset)
            .bind(collation)
            .bind(id)
            .execute(&self.pool)
            .await?;

            Ok(id)
        } else {
            // 插入
            let result = sqlx::query(
                "INSERT INTO table_preferences (
                    project_id,
                    pk_enabled,
                    pk_field_name,
                    pk_field_type,
                    pk_auto_increment,
                    pk_comment,
                    audit_enabled,
                    audit_fields,
                    soft_delete_enabled,
                    soft_delete_field,
                    soft_delete_field_type,
                    soft_delete_nullable,
                    soft_delete_default,
                    soft_delete_comment,
                    boolean_prefix,
                    datetime_suffix,
                    engine_type,
                    charset,
                    collation
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)"
            )
            .bind(project_id)
            .bind(pk_enabled as i32)
            .bind(pk_field_name)
            .bind(pk_field_type)
            .bind(pk_auto_increment as i32)
            .bind(pk_comment)
            .bind(audit_enabled as i32)
            .bind(audit_fields)
            .bind(soft_delete_enabled as i32)
            .bind(soft_delete_field)
            .bind(soft_delete_field_type)
            .bind(soft_delete_nullable as i32)
            .bind(soft_delete_default)
            .bind(soft_delete_comment)
            .bind(boolean_prefix)
            .bind(datetime_suffix)
            .bind(engine_type)
            .bind(charset)
            .bind(collation)
            .execute(&self.pool)
            .await?;

            Ok(result.last_insert_rowid())
        }
    }
}
