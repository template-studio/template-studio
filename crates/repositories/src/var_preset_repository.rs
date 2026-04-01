use sqlx::MySqlPool;
use template_studio_shared::models::var_preset::*;
use template_studio_shared::utils::response::PagedResponse;
use anyhow::Result;

/// 变量预设数据访问层
pub struct VarPresetRepository {
    pool: MySqlPool,
}

impl VarPresetRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// 创建变量预设
    pub async fn create(&self, request: &CreateVarPresetRequest) -> Result<u64> {
        let sort = request.sort.unwrap_or(0);
        let is_enabled = request.is_enabled.unwrap_or(1);
        let version = request.version.as_deref().unwrap_or("1.0");

        let result = sqlx::query(
            "INSERT INTO var_preset (name, display_name, description, category, schema_json, default_data_json, icon, sort, is_enabled, version, created_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&request.name)
        .bind(&request.display_name)
        .bind(&request.description)
        .bind(&request.category)
        .bind(&request.schema_json)
        .bind(&request.default_data_json)
        .bind(&request.icon)
        .bind(sort)
        .bind(is_enabled)
        .bind(version)
        .bind(request.created_by)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_id())
    }

    /// 根据ID获取变量预设
    pub async fn get_by_id(&self, id: u64) -> Result<Option<VarPreset>> {
        let var_preset = sqlx::query_as::<_, VarPreset>(
            "SELECT id, name, display_name, description, category, schema_json, default_data_json, icon, sort, is_enabled, version, created_by, created_at, updated_at FROM var_preset WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(var_preset)
    }

    /// 根据名称获取变量预设
    pub async fn get_by_name(&self, name: &str) -> Result<Option<VarPreset>> {
        let var_preset = sqlx::query_as::<_, VarPreset>(
            "SELECT id, name, display_name, description, category, schema_json, default_data_json, icon, sort, is_enabled, version, created_by, created_at, updated_at FROM var_preset WHERE name = ?"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(var_preset)
    }

    /// 更新变量预设
    pub async fn update(&self, request: &UpdateVarPresetRequest) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE var_preset SET name = ?, display_name = ?, description = ?, category = ?, schema_json = ?, default_data_json = ?, icon = ?, sort = ?, is_enabled = ?, version = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
        )
        .bind(&request.name)
        .bind(&request.display_name)
        .bind(&request.description)
        .bind(&request.category)
        .bind(&request.schema_json)
        .bind(&request.default_data_json)
        .bind(&request.icon)
        .bind(request.sort)
        .bind(request.is_enabled)
        .bind(&request.version)
        .bind(request.id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 删除变量预设
    pub async fn delete(&self, id: u64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM var_preset WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 切换变量预设启用/禁用状态
    pub async fn toggle(&self, request: &ToggleVarPresetRequest) -> Result<bool> {
        let result = if let Some(is_enabled) = request.is_enabled {
            // 如果明确指定了状态，则设置为指定状态
            sqlx::query("UPDATE var_preset SET is_enabled = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
                .bind(is_enabled)
                .bind(request.id)
                .execute(&self.pool)
                .await?
        } else {
            // 如果没有指定状态，则自动切换
            sqlx::query("UPDATE var_preset SET is_enabled = CASE WHEN is_enabled = 1 THEN 0 ELSE 1 END, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
                .bind(request.id)
                .execute(&self.pool)
                .await?
        };

        Ok(result.rows_affected() > 0)
    }

    /// 分页获取变量预设列表
    pub async fn list(&self, query: &VarPresetListQuery) -> Result<PagedResponse<VarPreset>> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).min(100);
        let offset = (page - 1) * page_size;

        // 简化实现：先获取所有数据，然后返回分页结果
        let mut all_var_presets = self.get_all().await?;

        // 应用过滤条件
        if let Some(name) = &query.name {
            all_var_presets.retain(|var_preset| var_preset.name.to_lowercase().contains(&name.to_lowercase()));
        }

        if let Some(category) = &query.category {
            all_var_presets.retain(|var_preset| var_preset.category == *category);
        }

        if let Some(is_enabled) = query.is_enabled {
            all_var_presets.retain(|var_preset| var_preset.is_enabled == is_enabled);
        }

        let total = all_var_presets.len() as u32;

        // 分页
        let start_index = offset as usize;
        let end_index = (start_index + page_size as usize).min(all_var_presets.len());
        let var_presets = if start_index < all_var_presets.len() {
            all_var_presets[start_index..end_index].to_vec()
        } else {
            vec![]
        };

        Ok(PagedResponse::new(var_presets, total, page, page_size))
    }

    /// 获取所有变量预设
    pub async fn get_all(&self) -> Result<Vec<VarPreset>> {
        let var_presets = sqlx::query_as::<_, VarPreset>(
            "SELECT id, name, display_name, description, category, schema_json, default_data_json, icon, sort, is_enabled, version, created_by, created_at, updated_at FROM var_preset ORDER BY sort ASC, id DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(var_presets)
    }

    /// 统计变量预设总数
    pub async fn count_all(&self) -> Result<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM var_preset")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
        Ok(count)
    }
}