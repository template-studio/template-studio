use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// 变量预设数据模型（数据库存储格式）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VarPreset {
    pub id: u64,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    pub schema_json: Option<String>,
    pub default_data_json: Option<String>,
    pub icon: Option<String>,
    pub sort: i32,
    pub is_enabled: i32,
    pub version: Option<String>,
    pub created_by: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// API响应用的语言模型（前端专用格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarPresetResponse {
    pub id: u64,
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    pub schema_json: Option<String>,
    pub default_data_json: Option<String>,
    pub icon: Option<String>,
    pub sort: i32,
    #[serde(rename = "isEnabled")]
    pub is_enabled: i32,
    pub version: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,  // 改为字符串格式以匹配原系统
    #[serde(rename = "updatedAt")]
    pub updated_at: String,  // 改为字符串格式以匹配原系统
}

impl From<VarPreset> for VarPresetResponse {
    fn from(var_preset: VarPreset) -> Self {
        Self {
            id: var_preset.id,
            name: var_preset.name,
            display_name: var_preset.display_name,
            description: var_preset.description,
            category: var_preset.category,
            schema_json: var_preset.schema_json,
            default_data_json: var_preset.default_data_json,
            icon: var_preset.icon,
            sort: var_preset.sort,
            is_enabled: var_preset.is_enabled,
            version: var_preset.version,
            created_at: var_preset.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: var_preset.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

/// API详情响应用的结构（详情接口专用格式，嵌套在varPreset对象中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarPresetDetailResponse {
    #[serde(rename = "varPreset")]
    pub var_preset: VarPresetDetailData,
}

/// 变量预设详情数据（详情接口专用格式，使用camelCase字段名）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarPresetDetailData {
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
    #[serde(rename = "isEnabled")]
    pub is_enabled: i32,
    pub version: String,
    #[serde(rename = "createdBy")]
    pub created_by: Option<u64>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

impl From<VarPreset> for VarPresetDetailData {
    fn from(var_preset: VarPreset) -> Self {
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
            is_enabled: var_preset.is_enabled,
            version: var_preset.version.unwrap_or_else(|| "1.0".to_string()),
            created_by: var_preset.created_by,
            created_at: var_preset.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: var_preset.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

/// 创建变量预设请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateVarPresetRequest {
    #[validate(length(min = 1, message = "变量预设名称不能为空"))]
    pub name: String,
    #[validate(length(min = 1, message = "显示名称不能为空"))]
    pub display_name: String,
    pub description: Option<String>,
    #[validate(length(min = 1, message = "分类不能为空"))]
    pub category: String,
    pub schema_json: Option<String>,
    pub default_data_json: Option<String>,
    pub icon: Option<String>,
    pub sort: Option<i32>,
    pub is_enabled: Option<i32>,
    pub version: Option<String>,
    pub created_by: Option<u64>,
}

/// 更新变量预设请求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateVarPresetRequest {
    #[validate(range(min = 1, message = "变量预设ID不能为空"))]
    pub id: u64,
    #[validate(length(min = 1, message = "变量预设名称不能为空"))]
    pub name: String,
    #[validate(length(min = 1, message = "显示名称不能为空"))]
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub description: Option<String>,
    #[validate(length(min = 1, message = "分类不能为空"))]
    #[serde(rename = "category")]
    pub category: String,
    #[serde(rename = "schemaJson")]
    #[validate(length(min = 1, message = "schemaJson不能为空"))]
    pub schema_json: String,  // 原API中这是必需字段
    #[serde(rename = "defaultDataJson")]
    pub default_data_json: Option<String>,
    pub icon: Option<String>,
    pub sort: Option<i32>,
    #[serde(rename = "isEnabled")]
    pub is_enabled: Option<i32>,
    pub version: Option<String>,
}

/// 变量预设启用/禁用请求
#[derive(Debug, Deserialize, Validate)]
pub struct ToggleVarPresetRequest {
    #[validate(range(min = 1, message = "变量预设ID不能为空"))]
    pub id: u64,
    #[serde(rename = "isEnabled")]
    #[validate(range(min = 0, max = 1, message = "isEnabled必须为0或1"))]
    pub is_enabled: Option<i32>,  // 可选，如果不提供则自动切换
}

/// 变量预设详情查询
#[derive(Debug, Deserialize, Validate)]
pub struct VarPresetDetailQuery {
    #[validate(range(min = 1, message = "变量预设ID不能为空"))]
    pub id: u64,
}

/// 变量预设列表查询
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct VarPresetListQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub name: Option<String>,
    pub category: Option<String>,
    pub is_enabled: Option<i32>,
}

impl Default for VarPresetListQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(20),
            name: None,
            category: None,
            is_enabled: None,
        }
    }
}

/// 可用预设变量列表查询（用于编辑器）
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct AvailableVarPresetQuery {
    pub page_num: Option<u32>,
    pub page_size: Option<u32>,
    pub keyword: Option<String>,
}

impl Default for AvailableVarPresetQuery {
    fn default() -> Self {
        Self {
            page_num: Some(1),
            page_size: Some(20),
            keyword: None,
        }
    }
}

/// 可用预设变量列表项（简化版，用于编辑器选择）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableVarPresetItem {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
}

impl From<VarPreset> for AvailableVarPresetItem {
    fn from(var_preset: VarPreset) -> Self {
        Self {
            id: var_preset.id,
            name: var_preset.name,
            description: var_preset.description,
        }
    }
}

/// 可用预设变量列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableVarPresetResponse {
    pub list: Vec<AvailableVarPresetItem>,
    pub total: usize,
    pub page_num: u32,
    pub page_size: u32,
}