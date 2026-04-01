//! 预设变量订阅模型
//! 存储模板对预设变量的订阅关系

use chrono::Utc;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// 预设变量订阅数据（存储在磁盘文件中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetSubscribe {
    pub id: u64,
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    #[serde(rename = "schemaJson")]
    pub schema_json: String,
    #[serde(rename = "defaultDataJson")]
    pub default_data_json: Option<String>,
    pub icon: Option<String>,
    pub sort: i32,
    pub version: Option<String>,
    #[serde(rename = "subscribedAt")]
    pub subscribed_at: String,
}

/// 从 VarPreset 转换为 PresetSubscribe
impl From<crate::models::var_preset::VarPreset> for PresetSubscribe {
    fn from(var_preset: crate::models::var_preset::VarPreset) -> Self {
        Self {
            id: var_preset.id,
            name: var_preset.name,
            display_name: var_preset.display_name,
            description: var_preset.description,
            category: var_preset.category,
            schema_json: var_preset.schema_json.unwrap_or_else(|| "{}".to_string()),
            default_data_json: var_preset.default_data_json,
            icon: var_preset.icon,
            sort: var_preset.sort,
            version: var_preset.version,
            subscribed_at: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

/// 订阅列表请求
#[derive(Debug, Deserialize, Validate)]
pub struct SubscribeListRequest {
    pub template_id: String,
}

/// 订阅请求
#[derive(Debug, Deserialize, Validate)]
pub struct SubscribeRequest {
    #[validate(length(min = 1, message = "模板ID不能为空"))]
    pub template_id: String,
    pub preset_ids: Vec<u64>,
}

/// 取消订阅请求
#[derive(Debug, Deserialize, Validate)]
pub struct UnsubscribeRequest {
    #[validate(length(min = 1, message = "模板ID不能为空"))]
    pub template_id: String,
    #[validate(range(min = 1, message = "预设ID不能为空"))]
    pub preset_id: u64,
}

/// 订阅列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeListResponse {
    pub template_id: String,
    pub subscribes: Vec<PresetSubscribe>,
    pub total: usize,
}

/// 预设变量响应（用于前端，兼容外部API格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetVariableResponse {
    pub id: u64,
    #[serde(rename = "templateId")]
    pub template_id: String,
    #[serde(rename = "presetId")]
    pub preset_id: u64,
    #[serde(rename = "presetName")]
    pub preset_name: String,
    pub description: Option<String>,
    pub schema: String,
}

/// 预设变量列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetVariablesResponse {
    pub list: Vec<PresetVariableResponse>,
}
