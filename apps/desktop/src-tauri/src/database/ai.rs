use super::Database;
use sqlx::Row;

impl Database {
    /// ===== AI 服务相关操作 =====
    /// 获取所有 AI 提供商
    pub async fn get_all_ai_providers(&self) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, provider_name, display_name, provider_type, api_key, api_endpoint,
                    is_enabled, is_default, temperature, max_tokens, timeout_seconds,
                    created_at, updated_at
             FROM ai_providers
             ORDER BY id ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        let providers = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.get::<i64, _>("id"),
                    "providerName": row.get::<String, _>("provider_name"),
                    "displayName": row.get::<String, _>("display_name"),
                    "providerType": row.get::<String, _>("provider_type"),
                    "apiKey": row
                        .get::<Option<String>, _>("api_key")
                        .map(|k| crate::database::credential::decrypt(&k).unwrap_or_default()),
                    "apiEndpoint": row.get::<Option<String>, _>("api_endpoint"),
                    "isEnabled": row.get::<i32, _>("is_enabled") == 1,
                    "isDefault": row.get::<i32, _>("is_default") == 1,
                    "temperature": row.get::<f64, _>("temperature"),
                    "maxTokens": row.get::<i32, _>("max_tokens"),
                    "timeoutSeconds": row.get::<i32, _>("timeout_seconds"),
                    "createdAt": row.get::<String, _>("created_at"),
                    "updatedAt": row.get::<String, _>("updated_at"),
                })
            })
            .collect();

        Ok(providers)
    }

    /// 根据 provider_name 获取 AI 提供商
    pub async fn get_ai_provider(
        &self,
        provider_name: &str,
    ) -> Result<Option<serde_json::Value>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, provider_name, display_name, provider_type, api_key, api_endpoint,
                    is_enabled, is_default, temperature, max_tokens, timeout_seconds,
                    created_at, updated_at
             FROM ai_providers
             WHERE provider_name = ?1",
        )
        .bind(provider_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            serde_json::json!({
                "id": r.get::<i64, _>("id"),
                "providerName": r.get::<String, _>("provider_name"),
                "displayName": r.get::<String, _>("display_name"),
                "providerType": r.get::<String, _>("provider_type"),
                "apiKey": r
                    .get::<Option<String>, _>("api_key")
                    .map(|k| crate::database::credential::decrypt(&k).unwrap_or_default()),
                "apiEndpoint": r.get::<Option<String>, _>("api_endpoint"),
                "isEnabled": r.get::<i32, _>("is_enabled") == 1,
                "isDefault": r.get::<i32, _>("is_default") == 1,
                "temperature": r.get::<f64, _>("temperature"),
                "maxTokens": r.get::<i32, _>("max_tokens"),
                "timeoutSeconds": r.get::<i32, _>("timeout_seconds"),
                "createdAt": r.get::<String, _>("created_at"),
                "updatedAt": r.get::<String, _>("updated_at"),
            })
        }))
    }

    /// 保存或更新 AI 提供商配置
    pub async fn save_ai_provider(
        &self,
        provider_name: &str,
        display_name: &str,
        provider_type: &str,
        api_key: Option<&str>,
        api_endpoint: Option<&str>,
        is_enabled: bool,
        temperature: f64,
        max_tokens: i32,
    ) -> Result<i64, sqlx::Error> {
        let id = sqlx::query(
            "INSERT INTO ai_providers (
                provider_name, display_name, provider_type, api_key, api_endpoint,
                is_enabled, temperature, max_tokens
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(provider_name) DO UPDATE SET
                display_name = excluded.display_name,
                provider_type = excluded.provider_type,
                api_key = excluded.api_key,
                api_endpoint = excluded.api_endpoint,
                is_enabled = excluded.is_enabled,
                temperature = excluded.temperature,
                max_tokens = excluded.max_tokens,
                updated_at = datetime('now')
            RETURNING id",
        )
        .bind(provider_name)
        .bind(display_name)
        .bind(provider_type)
        .bind(
            api_key
                .map(crate::database::credential::encrypt)
                .transpose()
                .map_err(|e| {
                    sqlx::Error::Io(std::io::Error::other(format!("凭据加密失败: {}", e)))
                })?,
        )
        .bind(api_endpoint)
        .bind(if is_enabled { 1 } else { 0 })
        .bind(temperature)
        .bind(max_tokens)
        .fetch_one(&self.pool)
        .await?
        .get::<i64, _>("id");

        Ok(id)
    }

    /// 切换 AI 提供商启用状态
    pub async fn toggle_ai_provider(
        &self,
        provider_name: &str,
        is_enabled: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE ai_providers
             SET is_enabled = ?1, updated_at = datetime('now')
             WHERE provider_name = ?2",
        )
        .bind(if is_enabled { 1 } else { 0 })
        .bind(provider_name)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除 AI 提供商
    pub async fn delete_ai_provider(&self, provider_name: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM ai_providers WHERE provider_name = ?1")
            .bind(provider_name)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 获取提供商的所有模型（分组）
    pub async fn get_ai_provider_models_grouped(
        &self,
        provider_name: &str,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT group_id, COUNT(*) as count
             FROM ai_models
             WHERE provider_name = ?1
             GROUP BY group_id
             ORDER BY group_id ASC",
        )
        .bind(provider_name)
        .fetch_all(&self.pool)
        .await?;

        let mut groups = Vec::new();

        for row in rows {
            let group_id: String = row.get("group_id");
            let count: i64 = row.get("count");

            // 获取该分组下的所有模型
            let model_rows = sqlx::query(
                "SELECT id, model_id, model_name, description, max_tokens, supports_functions, supports_vision
                 FROM ai_models
                 WHERE provider_name = ?1 AND group_id = ?2
                 ORDER BY id ASC"
            )
            .bind(provider_name)
            .bind(&group_id)
            .fetch_all(&self.pool)
            .await?;

            let models: Vec<serde_json::Value> = model_rows
                .into_iter()
                .map(|m| {
                    serde_json::json!({
                        "id": m.get::<i64, _>("id"),
                        "modelId": m.get::<String, _>("model_id"),
                        "modelName": m.get::<String, _>("model_name"),
                        "description": m.get::<Option<String>, _>("description"),
                        "maxTokens": m.get::<i32, _>("max_tokens"),
                        "supportsFunctions": m.get::<i32, _>("supports_functions") == 1,
                        "supportsVision": m.get::<i32, _>("supports_vision") == 1,
                    })
                })
                .collect();

            groups.push(serde_json::json!({
                "groupId": group_id,
                "groupName": get_group_display_name(&group_id),
                "count": count,
                "models": models,
            }));
        }

        Ok(groups)
    }

    /// 添加 AI 模型
    pub async fn add_ai_model(
        &self,
        model_id: &str,
        model_name: &str,
        provider_name: &str,
        group_id: &str,
        description: Option<&str>,
        max_tokens: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO ai_models (model_id, model_name, provider_name, group_id, description, max_tokens)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        )
        .bind(model_id)
        .bind(model_name)
        .bind(provider_name)
        .bind(group_id)
        .bind(description)
        .bind(max_tokens)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 批量添加 AI 模型（忽略已存在的 model_id）
    pub async fn batch_add_ai_models(
        &self,
        models: &[(&str, &str, &str, &str, Option<&str>, i32)],
    ) -> Result<i64, sqlx::Error> {
        let mut count = 0i64;
        for (model_id, model_name, provider_name, group_id, description, max_tokens) in models {
            let result = sqlx::query(
                "INSERT INTO ai_models (model_id, model_name, provider_name, group_id, description, max_tokens)
                 SELECT ?1, ?2, ?3, ?4, ?5, ?6
                 WHERE NOT EXISTS (
                     SELECT 1 FROM ai_models WHERE model_id = ?1 AND provider_name = ?3
                 )"
            )
            .bind(model_id)
            .bind(model_name)
            .bind(provider_name)
            .bind(group_id)
            .bind(description)
            .bind(max_tokens)
            .execute(&self.pool)
            .await?;

            count += result.rows_affected() as i64;
        }
        Ok(count)
    }

    /// 删除 AI 模型
    pub async fn delete_ai_model(&self, model_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM ai_models WHERE id = ?1")
            .bind(model_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 更新 AI 模型
    pub async fn update_ai_model(
        &self,
        model_id: i64,
        new_model_id: &str,
        model_name: &str,
        group_id: &str,
        description: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE ai_models
             SET model_id = ?1, model_name = ?2, group_id = ?3, description = ?4
             WHERE id = ?5",
        )
        .bind(new_model_id)
        .bind(model_name)
        .bind(group_id)
        .bind(description)
        .bind(model_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

/// 获取分组的显示名称
fn get_group_display_name(group_id: &str) -> &'static str {
    match group_id {
        "chat" => "对话模型",
        "code" => "代码模型",
        "image" => "图像模型",
        "embedding" => "嵌入模型",
        _ => "其他模型",
    }
}
