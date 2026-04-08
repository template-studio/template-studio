use async_trait::async_trait;
use serde_json::json;

use super::{AiTool, ToolError};
use crate::types::ToolResult;

/// 变量分析工具 - 从模板文件提取变量
pub struct AnalyzeVariablesTool;

#[async_trait]
impl AiTool for AnalyzeVariablesTool {
    fn name(&self) -> &str {
        "analyze_variables"
    }

    fn description(&self) -> &str {
        "分析模板文件，提取所有变量并推断类型和 schema"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "template_path": {
                    "type": "string",
                    "description": "模板文件或目录路径"
                }
            },
            "required": ["template_path"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError> {
        let template_path = args["template_path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("缺少 template_path 参数".to_string()))?;

        // 读取模板文件
        let content = tokio::fs::read_to_string(template_path)
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("读取模板文件失败: {}", e)))?;

        // 正则提取变量
        let variables = extract_variables_regex(&content);

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&variables)?,
            error: None,
        })
    }
}

/// 使用正则表达式提取模板变量
fn extract_variables_regex(content: &str) -> Vec<serde_json::Value> {
    use regex::Regex;

    let re = Regex::new(r"\{\{\s*(\w+)(?:\s*\|[^}]*)?\s*\}\}").unwrap();
    let mut variables = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for cap in re.captures_iter(content) {
        let name = cap[1].to_string();
        if seen.insert(name.clone()) {
            variables.push(json!({
                "name": name,
                "type": "string",
                "title": name,
                "description": format!("模板变量 {}", name),
                "required": true,
                "source": "regex"
            }));
        }
    }

    variables
}

/// 变量验证工具
pub struct ValidateVariablesTool;

#[async_trait]
impl AiTool for ValidateVariablesTool {
    fn name(&self) -> &str {
        "validate_variables"
    }

    fn description(&self) -> &str {
        "验证变量完整性，检查必填变量是否齐全"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "template_path": {
                    "type": "string",
                    "description": "模板文件路径"
                },
                "variables": {
                    "type": "object",
                    "description": "变量值 JSON"
                }
            },
            "required": ["template_path", "variables"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError> {
        let template_path = args["template_path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("缺少 template_path 参数".to_string()))?;

        let variables = args["variables"]
            .as_object()
            .ok_or_else(|| ToolError::InvalidArgument("variables 必须是对象".to_string()))?;

        let content = tokio::fs::read_to_string(template_path)
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("读取模板文件失败: {}", e)))?;

        let required_vars = extract_variables_regex(&content);
        let mut missing = Vec::new();

        for var in &required_vars {
            let name = var["name"].as_str().unwrap_or_default();
            if !variables.contains_key(name) {
                missing.push(name.to_string());
            }
        }

        let result = json!({
            "valid": missing.is_empty(),
            "missing": missing,
            "total_required": required_vars.len(),
            "provided": variables.len()
        });

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&result)?,
            error: None,
        })
    }
}
