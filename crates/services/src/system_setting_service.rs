use std::sync::Arc;
use template_studio_shared::models::system_setting::*;
use template_studio_repositories::SystemSettingRepository;
use anyhow::Result;

/// 系统设置业务逻辑层
pub struct SystemSettingService {
    repository: Arc<SystemSettingRepository>,
}

impl SystemSettingService {
    pub fn new(repository: Arc<SystemSettingRepository>) -> Self {
        Self { repository }
    }

    /// 获取设置列表
    pub async fn get_settings(&self, group: Option<&str>, key: Option<&str>) -> Result<Vec<SystemSetting>> {
        if let (Some(g), Some(k)) = (group, key) {
            let setting = self.repository.get_by_key(g, k).await?;
            Ok(setting.into_iter().collect())
        } else if let Some(g) = group {
            self.repository.get_by_group(g).await
        } else {
            self.repository.get_all().await
        }
    }

    /// 更新单个设置
    pub async fn update_setting(&self, request: &UpdateSettingRequest) -> Result<()> {
        self.repository.upsert(
            &request.group,
            &request.key,
            request.value.as_deref().unwrap_or(""),
            None,
        ).await?;
        Ok(())
    }

    /// 批量更新设置
    pub async fn batch_update_settings(&self, request: &BatchUpdateSettingsRequest) -> Result<()> {
        let items: Vec<(String, String, Option<String>)> = request
            .items
            .iter()
            .map(|item| {
                (item.key.clone(), item.value.clone().unwrap_or_default(), None)
            })
            .collect();

        self.repository.batch_upsert(&request.group, &items).await?;
        Ok(())
    }
}
