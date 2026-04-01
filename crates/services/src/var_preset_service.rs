use template_studio_repositories::VarPresetRepository;
use template_studio_shared::{
    models::var_preset::*,
    utils::{validation::validate_request, error::AppError},
};
use std::sync::Arc;

/// 变量预设业务服务
pub struct VarPresetService {
    repository: Arc<VarPresetRepository>,
}

impl VarPresetService {
    pub fn new(repository: Arc<VarPresetRepository>) -> Self {
        Self { repository }
    }

    /// 创建变量预设
    pub async fn create_var_preset(&self, request: CreateVarPresetRequest) -> Result<u64, AppError> {
        // 验证请求数据
        validate_request(&request)?;

        // 检查名称是否重复
        if let Some(_) = self.repository.get_by_name(&request.name).await? {
            return Err(AppError::Duplicate(format!("变量预设名称 '{}' 已存在", request.name)));
        }

        // 创建变量预设
        let var_preset_id = self.repository.create(&request).await?;
        tracing::info!("创建变量预设成功: id={}, name={}", var_preset_id, request.name);

        Ok(var_preset_id)
    }

    /// 获取变量预设详情
    pub async fn get_var_preset(&self, id: u64) -> Result<Option<VarPreset>, AppError> {
        let var_preset = self.repository.get_by_id(id).await?;
        Ok(var_preset)
    }

    /// 更新变量预设
    pub async fn update_var_preset(&self, request: UpdateVarPresetRequest) -> Result<(), AppError> {
        // 验证请求数据
        validate_request(&request)?;

        // 检查变量预设是否存在
        let _existing = self.repository.get_by_id(request.id).await?
            .ok_or_else(|| AppError::NotFound(format!("变量预设 ID {} 不存在", request.id)))?;

        // 更新变量预设
        let updated = self.repository.update(&request).await?;
        if updated {
            tracing::info!("更新变量预设成功: id={}", request.id);
        }
        Ok(())
    }

    /// 删除变量预设
    pub async fn delete_var_preset(&self, id: u64) -> Result<(), AppError> {
        // 检查变量预设是否存在
        let _var_preset = self.repository.get_by_id(id).await?
            .ok_or_else(|| AppError::NotFound(format!("变量预设 ID {} 不存在", id)))?;

        // 删除变量预设
        let deleted = self.repository.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("变量预设 {} 不存在", id)));
        }

        tracing::info!("删除变量预设成功: id={}", id);
        Ok(())
    }

    /// 切换变量预设启用/禁用状态
    pub async fn toggle_var_preset(&self, request: ToggleVarPresetRequest) -> Result<(), AppError> {
        // 验证请求数据
        validate_request(&request)?;

        // 检查变量预设是否存在
        let _var_preset = self.repository.get_by_id(request.id).await?
            .ok_or_else(|| AppError::NotFound(format!("变量预设 ID {} 不存在", request.id)))?;

        // 切换状态
        let toggled = self.repository.toggle(&request).await?;
        if !toggled {
            return Err(AppError::NotFound(format!("变量预设 {} 不存在", request.id)));
        }

        let status = request.is_enabled.map(|v| if v == 1 { "启用" } else { "禁用" }).unwrap_or("切换");
        tracing::info!("{}变量预设成功: id={}", status, request.id);
        Ok(())
    }

    /// 获取变量预设列表
    pub async fn list_var_presets(&self, query: VarPresetListQuery) -> Result<Vec<VarPreset>, AppError> {
        let var_presets = self.repository.list(&query).await?
            .items; // 只返回数据，不考虑分页

        Ok(var_presets)
    }

    /// 获取所有变量预设
    pub async fn get_all_var_presets(&self) -> Result<Vec<VarPreset>, AppError> {
        let var_presets = self.repository.get_all().await?;
        Ok(var_presets)
    }

    /// 根据分类获取变量预设
    pub async fn get_var_presets_by_category(&self, category: &str) -> Result<Vec<VarPreset>, AppError> {
        let query = VarPresetListQuery {
            page: None,
            page_size: None,
            name: None,
            category: Some(category.to_string()),
            is_enabled: Some(1), // 只获取启用的
        };

        let var_presets = self.repository.list(&query).await?
            .items;

        Ok(var_presets)
    }

    /// 获取启用的变量预设
    pub async fn get_enabled_var_presets(&self) -> Result<Vec<VarPreset>, AppError> {
        let query = VarPresetListQuery {
            page: None,
            page_size: None,
            name: None,
            category: None,
            is_enabled: Some(1),
        };

        let var_presets = self.repository.list(&query).await?
            .items;

        Ok(var_presets)
    }

    /// 获取可用的预设变量列表（用于编辑器，支持分页和关键词搜索）
    pub async fn get_available_var_presets(&self, query: AvailableVarPresetQuery) -> Result<AvailableVarPresetResponse, AppError> {
        let page_num = query.page_num.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).min(100);

        // 获取所有启用的变量预设
        let list_query = VarPresetListQuery {
            page: None,
            page_size: None,
            name: None,
            category: None,
            is_enabled: Some(1), // 只获取启用的
        };

        let mut all_var_presets = self.repository.list(&list_query).await?
            .items;

        // 应用关键词搜索
        if let Some(keyword) = &query.keyword {
            if !keyword.is_empty() {
                let keyword_lower = keyword.to_lowercase();
                all_var_presets.retain(|var_preset| {
                    var_preset.name.to_lowercase().contains(&keyword_lower)
                        || var_preset.display_name.to_lowercase().contains(&keyword_lower)
                        || var_preset.description.as_ref().map_or(false, |desc| {
                            desc.to_lowercase().contains(&keyword_lower)
                        })
                });
            }
        }

        let total = all_var_presets.len();
        let offset = ((page_num - 1) * page_size) as usize;
        let end_index = (offset + page_size as usize).min(total);

        // 分页
        let page_var_presets = if offset < total {
            all_var_presets[offset..end_index].to_vec()
        } else {
            vec![]
        };

        // 转换为简化格式
        let list: Vec<AvailableVarPresetItem> = page_var_presets
            .into_iter()
            .map(AvailableVarPresetItem::from)
            .collect();

        Ok(AvailableVarPresetResponse {
            list,
            total,
            page_num,
            page_size,
        })
    }

    /// 获取变量预设总数
    pub async fn get_preset_count(&self) -> Result<i64, AppError> {
        let count = self.repository.count_all().await?;
        Ok(count)
    }
}