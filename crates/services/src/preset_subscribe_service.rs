//! 预设变量订阅服务

use template_studio_repositories::VarPresetRepository;
use template_studio_shared::{
    models::preset_subscribe::*,
    utils::{error::AppError, validation::validate_request},
};
use std::sync::Arc;

/// 预设变量订阅服务
pub struct PresetSubscribeService {
    var_preset_repository: Arc<VarPresetRepository>,
}

impl PresetSubscribeService {
    pub fn new(var_preset_repository: Arc<VarPresetRepository>) -> Self {
        Self {
            var_preset_repository,
        }
    }

    /// 获取订阅列表
    pub async fn get_subscribe_list(
        &self,
        template_id: &str,
        subscribe_path: &std::path::Path,
    ) -> Result<SubscribeListResponse, AppError> {
        let mut subscribes = Vec::new();

        // 读取订阅目录下的所有 JSON 文件
        if subscribe_path.exists() {
            let mut entries = tokio::fs::read_dir(subscribe_path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    // 读取文件内容
                    let content = tokio::fs::read_to_string(&path).await?;
                    if let Ok(subscribe) = serde_json::from_str::<PresetSubscribe>(&content) {
                        subscribes.push(subscribe);
                    }
                }
            }
        }

        let total = subscribes.len();

        Ok(SubscribeListResponse {
            template_id: template_id.to_string(),
            subscribes,
            total,
        })
    }

    /// 添加订阅
    pub async fn subscribe(
        &self,
        request: SubscribeRequest,
        subscribe_path: &std::path::Path,
    ) -> Result<(), AppError> {
        // 验证请求数据
        validate_request(&request)?;

        // 确保订阅目录存在
        tokio::fs::create_dir_all(subscribe_path).await?;

        // 为每个预设ID创建订阅文件
        for preset_id in request.preset_ids {
            // 从数据库获取预设变量详情
            if let Some(var_preset) = self.var_preset_repository.get_by_id(preset_id).await? {
                // 转换为订阅数据
                let subscribe = PresetSubscribe::from(var_preset);

                // 写入文件（格式化JSON）
                let file_path = subscribe_path.join(format!("{}.json", preset_id));
                let json_content = serde_json::to_string_pretty(&subscribe)?;

                tokio::fs::write(file_path, json_content).await?;
            }
        }

        Ok(())
    }

    /// 取消订阅
    pub async fn unsubscribe(
        &self,
        request: UnsubscribeRequest,
        subscribe_path: &std::path::Path,
    ) -> Result<(), AppError> {
        // 验证请求数据
        validate_request(&request)?;

        // 删除订阅文件
        let file_path = subscribe_path.join(format!("{}.json", request.preset_id));

        if file_path.exists() {
            tokio::fs::remove_file(file_path).await?;
        }

        Ok(())
    }

    /// 获取预设变量列表（兼容外部API格式）
    pub async fn get_preset_variables(
        &self,
        template_id: &str,
        subscribe_path: &std::path::Path,
    ) -> Result<PresetVariablesResponse, AppError> {
        let mut variables = Vec::new();
        let mut id_counter = 1u64;

        // 读取订阅目录下的所有 JSON 文件
        if subscribe_path.exists() {
            let mut entries = tokio::fs::read_dir(subscribe_path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    // 读取文件内容
                    let content = tokio::fs::read_to_string(&path).await?;
                    if let Ok(subscribe) = serde_json::from_str::<PresetSubscribe>(&content) {
                        // 转换为前端需要的格式
                        let variable = PresetVariableResponse {
                            id: id_counter,
                            template_id: template_id.to_string(),
                            preset_id: subscribe.id,
                            preset_name: subscribe.name.clone(),
                            description: subscribe.description,
                            schema: subscribe.schema_json,
                        };
                        variables.push(variable);
                        id_counter += 1;
                    }
                }
            }
        }

        Ok(PresetVariablesResponse { list: variables })
    }
}
