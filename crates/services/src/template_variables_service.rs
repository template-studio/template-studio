//! 模板变量服务
//! 处理模板变量和测试数据的存储

use template_studio_shared::utils::error::AppError;
use template_studio_infrastructure::config::storage::StorageManager;
use std::sync::Arc;

/// 模板变量服务
pub struct TemplateVariablesService {
    storage_manager: Arc<StorageManager>,
}

impl TemplateVariablesService {
    pub fn new(storage_manager: Arc<StorageManager>) -> Self {
        Self { storage_manager }
    }

    /// 获取模板变量
    pub async fn get_variables(&self, template_id: i64) -> Result<String, AppError> {
        let path = self.storage_manager.get_variables_file_path(template_id);

        // 如果文件不存在，返回空对象
        if !path.exists() {
            return Ok("{}".to_string());
        }

        self.storage_manager
            .read_json_file(&path)
            .await
            .map_err(|e| AppError::Internal(format!("读取变量文件失败: {}", e)))
    }

    /// 保存模板变量
    pub async fn save_variables(&self, template_id: i64, content: &str) -> Result<(), AppError> {
        // 验证 JSON 格式
        if let Err(e) = serde_json::from_str::<serde_json::Value>(content) {
            return Err(AppError::Validation(format!("无效的 JSON 格式: {}", e)));
        }

        let path = self.storage_manager.get_variables_file_path(template_id);
        self.storage_manager
            .write_json_file(&path, content)
            .await
            .map_err(|e| AppError::Internal(format!("保存变量文件失败: {}", e)))
    }

    /// 获取测试数据
    pub async fn get_test_data(&self, template_id: i64) -> Result<String, AppError> {
        let path = self.storage_manager.get_test_file_path(template_id);

        // 如果文件不存在，返回空对象
        if !path.exists() {
            return Ok("{}".to_string());
        }

        self.storage_manager
            .read_json_file(&path)
            .await
            .map_err(|e| AppError::Internal(format!("读取测试数据文件失败: {}", e)))
    }

    /// 保存测试数据
    pub async fn save_test_data(&self, template_id: i64, content: &str) -> Result<(), AppError> {
        // 验证 JSON 格式
        if let Err(e) = serde_json::from_str::<serde_json::Value>(content) {
            return Err(AppError::Validation(format!("无效的 JSON 格式: {}", e)));
        }

        let path = self.storage_manager.get_test_file_path(template_id);
        self.storage_manager
            .write_json_file(&path, content)
            .await
            .map_err(|e| AppError::Internal(format!("保存测试数据文件失败: {}", e)))
    }
}
