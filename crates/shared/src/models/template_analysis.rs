//! 模板变量分析模型

use serde::{Deserialize, Serialize};
use validator::Validate;

/// 分析变量请求
#[derive(Debug, Deserialize, Validate)]
pub struct AnalyzeVariablesRequest {
    #[validate(range(min = 1, message = "模板ID不能为空"))]
    pub template_id: i64,
}

/// 检测到的变量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedVariable {
    pub name: String,
    #[serde(rename = "type")]
    pub var_type: String,
    pub files: Vec<String>,
    pub contexts: Vec<String>,
    pub suggestions: String,
}

/// 缺失的变量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingVariable {
    pub name: String,
    #[serde(rename = "type")]
    pub var_type: String,
    pub files: Vec<String>,
    pub contexts: Vec<String>,
    pub suggestions: String,
}

/// 未使用的变量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedVariable {
    pub name: String,
    #[serde(rename = "type")]
    pub var_type: String,
    pub files: Vec<String>,
}

/// 冲突的变量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictVariable {
    pub name: String,
    #[serde(rename = "type")]
    pub var_type: String,
    pub conflicts: Vec<String>,
}

/// 变量分析响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableAnalysisResponse {
    #[serde(rename = "detectedVariables")]
    pub detected_variables: Vec<DetectedVariable>,
    #[serde(rename = "missingVariables")]
    pub missing_variables: Vec<MissingVariable>,
    #[serde(rename = "unusedVariables")]
    pub unused_variables: Vec<UnusedVariable>,
    #[serde(rename = "conflictVariables")]
    pub conflict_variables: Vec<ConflictVariable>,
    #[serde(rename = "totalVariableCount")]
    pub total_variable_count: usize,
    #[serde(rename = "analyzedFileCount")]
    pub analyzed_file_count: usize,
}
