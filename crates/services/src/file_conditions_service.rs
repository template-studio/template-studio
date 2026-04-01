//! 文件条件管理服务（基于文件系统）
//!
//! 提供文件生成条件的增删改查功能，条件保存在 .meta/variables/conditions.yml

use template_studio_shared::utils::error::AppError;
use template_studio_template_core::{
    Condition, ConditionsYaml, FileCondition,
};
use template_studio_infrastructure::config::storage::StorageManager;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::fs;

/// 文件条件管理服务
pub struct FileConditionsService {
    storage_manager: Arc<StorageManager>,
}

impl FileConditionsService {
    pub fn new(storage_manager: Arc<StorageManager>) -> Self {
        Self { storage_manager }
    }

    /// 获取条件文件路径
    async fn get_conditions_file_path(&self, template_id: i64) -> Result<PathBuf, AppError> {
        let conditions_path = self.storage_manager
            .get_template_path(template_id)
            .join(".meta/variables/conditions.yml");
        Ok(conditions_path)
    }

    /// 从 YAML 文件加载所有条件
    async fn load_conditions(&self, template_id: i64) -> Result<ConditionsYaml, AppError> {
        let conditions_path = self.get_conditions_file_path(template_id).await?;

        if !conditions_path.exists() {
            return Ok(ConditionsYaml::new());
        }

        let content = fs::read_to_string(&conditions_path)
            .await
            .map_err(|e| AppError::TemplateRender(format!("读取条件文件失败: {}", e)))?;

        ConditionsYaml::from_yaml(&content)
            .map_err(|e| AppError::TemplateRender(format!("解析条件文件失败: {}", e)))
    }

    /// 保存条件到 YAML 文件
    async fn save_conditions(&self, template_id: i64, yaml: &ConditionsYaml) -> Result<(), AppError> {
        let conditions_path = self.get_conditions_file_path(template_id).await?;

        if let Some(parent) = conditions_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::TemplateRender(format!("创建目录失败: {}", e)))?;
        }

        let content = yaml.to_yaml()
            .map_err(|e| AppError::TemplateRender(format!("序列化条件失败: {}", e)))?;

        fs::write(&conditions_path, content)
            .await
            .map_err(|e| AppError::TemplateRender(format!("写入条件文件失败: {}", e)))?;

        Ok(())
    }

    /// 获取文件的条件
    pub async fn get_file_condition(
        &self,
        template_id: i64,
        file_path: &str,
    ) -> Result<Option<Condition>, AppError> {
        let yaml = self.load_conditions(template_id).await?;
        Ok(yaml.get_condition_by_path(file_path))
    }

    /// 设置文件的条件
    pub async fn set_file_condition(
        &self,
        template_id: i64,
        file_path: &str,
        condition: Condition,
    ) -> Result<(), AppError> {
        condition.validate()
            .map_err(|e| AppError::Validation(format!("无效的条件: {}", e)))?;

        let mut yaml = self.load_conditions(template_id).await?;

        // 生成唯一ID（使用当前时间戳的毫秒数）
        let id = chrono::Utc::now().timestamp_millis();

        let file_condition = FileCondition {
            id,
            path: file_path.to_string(),
            condition: Some(condition),
        };

        yaml.add_condition(file_condition);
        self.save_conditions(template_id, &yaml).await?;

        Ok(())
    }

    /// 删除文件的条件
    pub async fn delete_file_condition(
        &self,
        template_id: i64,
        file_path: &str,
    ) -> Result<(), AppError> {
        let mut yaml = self.load_conditions(template_id).await?;
        yaml.remove_condition_by_path(file_path);
        self.save_conditions(template_id, &yaml).await?;
        Ok(())
    }

    /// 导出模板的条件为 YAML
    pub async fn export_conditions_yaml(
        &self,
        template_id: i64,
    ) -> Result<String, AppError> {
        let yaml = self.load_conditions(template_id).await?;
        yaml.to_yaml()
            .map_err(|e| AppError::TemplateRender(format!("生成 YAML 失败: {}", e)))
    }

    /// 从 YAML 导入条件
    pub async fn import_conditions_yaml(
        &self,
        template_id: i64,
        yaml_content: &str,
    ) -> Result<usize, AppError> {
        let yaml = ConditionsYaml::from_yaml(yaml_content)
            .map_err(|e| AppError::Validation(format!("解析 YAML 失败: {}", e)))?;

        yaml.validate()
            .map_err(|e| AppError::Validation(format!("条件验证失败: {}", e)))?;

        self.save_conditions(template_id, &yaml).await?;
        Ok(yaml.conditions.len())
    }

    /// 评估文件是否应该生成
    pub async fn should_generate_file(
        &self,
        template_id: i64,
        file_path: &str,
        variables: &serde_json::Value,
    ) -> Result<bool, AppError> {
        if let Some(condition) = self.get_file_condition(template_id, file_path).await? {
            let result = condition.evaluate(variables)
                .map_err(|e| AppError::TemplateRender(format!("评估条件失败: {}", e)))?;
            Ok(result)
        } else {
            Ok(true)
        }
    }

    /// 文件移动时更新条件中的路径
    pub async fn on_file_moved(
        &self,
        template_id: i64,
        old_path: &str,
        new_path: &str,
    ) -> Result<(), AppError> {
        let mut yaml = self.load_conditions(template_id).await?;
        yaml.update_file_path(old_path, new_path);
        self.save_conditions(template_id, &yaml).await?;
        Ok(())
    }

    /// 文件重命名时更新条件中的路径
    pub async fn on_file_renamed(
        &self,
        template_id: i64,
        old_path: &str,
        new_path: &str,
    ) -> Result<(), AppError> {
        self.on_file_moved(template_id, old_path, new_path).await
    }

    /// 文件删除时删除条件
    pub async fn on_file_deleted(
        &self,
        template_id: i64,
        file_path: &str,
    ) -> Result<(), AppError> {
        self.delete_file_condition(template_id, file_path).await
    }

    /// 获取模板的所有条件摘要（用于文件树显示标记）
    /// 返回一个 HashMap，key 是文件路径，value 是条件摘要
    pub async fn get_conditions_summary(
        &self,
        template_id: i64,
    ) -> Result<std::collections::HashMap<String, String>, AppError> {
        let yaml = self.load_conditions(template_id).await?;

        let mut summary = std::collections::HashMap::new();
        for file_condition in &yaml.conditions {
            if let Some(ref condition) = file_condition.condition {
                let description = Self::describe_condition(condition);
                summary.insert(file_condition.path.clone(), description);
            }
        }

        Ok(summary)
    }

    /// 将条件描述为人类可读的字符串
    fn describe_condition(condition: &Condition) -> String {
        use template_studio_template_core::ConditionType;

        match &condition.condition_type {
            ConditionType::If => {
                if let (Some(operator), Some(value)) = (&condition.operator, &condition.value) {
                    let op_str = Self::operator_to_string(&format!("{:?}", operator).to_lowercase());
                    let value_str = Self::value_to_string(value);
                    format!("如果 {} {} {}", condition.variable, op_str, value_str)
                } else {
                    "条件".to_string()
                }
            }
            ConditionType::And => {
                if let Some(conditions) = &condition.conditions {
                    format!("且（{} 个条件）", conditions.len())
                } else {
                    "且".to_string()
                }
            }
            ConditionType::Or => {
                if let Some(conditions) = &condition.conditions {
                    format!("或（{} 个条件）", conditions.len())
                } else {
                    "或".to_string()
                }
            }
            ConditionType::Not => {
                if let Some(cond) = &condition.conditions {
                    if let Some(first) = cond.first() {
                        format!("非（{}", Self::describe_condition(first))
                    } else {
                        "非".to_string()
                    }
                } else {
                    "非".to_string()
                }
            }
            ConditionType::Switch => {
                if let Some(cases) = &condition.cases {
                    format!("多分支 {}（{} 个分支）", condition.variable, cases.len())
                } else {
                    format!("多分支 {}", condition.variable)
                }
            }
        }
    }

    /// 将操作符转换为可读字符串
    fn operator_to_string(op: &str) -> String {
        // Operator 是 enum，序列化后是小写字符串
        match op {
            "eq" => "==".to_string(),
            "ne" => "!=".to_string(),
            "gt" => ">".to_string(),
            "lt" => "<".to_string(),
            "gte" => ">=".to_string(),
            "lte" => "<=".to_string(),
            "in" => "∈".to_string(),
            "not_in" => "∉".to_string(),
            "contains" => "包含".to_string(),
            _ => op.to_string(),
        }
    }

    /// 将值转换为可读字符串
    fn value_to_string(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            _ => value.to_string(),
        }
    }
}
