use async_trait::async_trait;
use serde_json::json;

use super::{AiTool, ToolError};
use crate::client::AiClient;
use crate::context::ProjectContext;
use crate::prompts;
use crate::types::*;

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
pub fn extract_variables_regex(content: &str) -> Vec<serde_json::Value> {
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

/// 变量填充工具 - 基于项目上下文自动填充变量
pub struct FillVariablesTool {
    client: std::sync::Arc<dyn AiClient>,
}

impl FillVariablesTool {
    pub fn new(client: std::sync::Arc<dyn AiClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl AiTool for FillVariablesTool {
    fn name(&self) -> &str {
        "fill_variables"
    }

    fn description(&self) -> &str {
        "根据项目上下文自动填充模板变量"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "template_path": {
                    "type": "string",
                    "description": "模板文件或目录路径"
                },
                "project_context": {
                    "type": "object",
                    "description": "项目上下文（表结构、类型映射等）"
                },
                "variables": {
                    "type": "object",
                    "description": "变量定义（可选，不提供则自动分析）"
                }
            },
            "required": ["template_path", "project_context"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError> {
        let template_path = args["template_path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("缺少 template_path 参数".to_string()))?;

        let context_json = &args["project_context"];

        // 读取模板文件
        let content = tokio::fs::read_to_string(template_path)
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("读取模板文件失败: {}", e)))?;

        // 提取变量
        let variables = extract_variables_regex(&content);

        // 构建 prompt
        let prompt = prompts::variable::build_fill_prompt(
            &serde_json::to_string_pretty(&variables)?,
            &serde_json::to_string_pretty(context_json)?,
        );

        // 调用 AI
        let request = ChatRequest {
            messages: vec![
                ChatMessage {
                    role: MessageRole::System,
                    content: prompts::SYSTEM_PROMPT.to_string(),
                },
                ChatMessage {
                    role: MessageRole::User,
                    content: prompt,
                },
            ],
            model: String::new(), // 由 client 配置决定
            temperature: Some(0.3),
            max_tokens: Some(2000),
        };

        let response = self.client.chat(request).await
            .map_err(|e| ToolError::ExecutionFailed(format!("AI 调用失败: {}", e)))?;

        // 解析 AI 响应
        let result: serde_json::Value = serde_json::from_str(&response.content)
            .unwrap_or_else(|_| json!({ "raw_response": response.content }));

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&result)?,
            error: None,
        })
    }
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

/// 直接填充变量（便捷函数）
pub async fn fill_variables(
    client: &dyn AiClient,
    template_path: &str,
    context: &ProjectContext,
) -> Result<VariableFillResult, ToolError> {
    let content = tokio::fs::read_to_string(template_path)
        .await
        .map_err(|e| ToolError::ExecutionFailed(format!("读取模板文件失败: {}", e)))?;

    let variables = extract_variables_regex(&content);

    let prompt = prompts::variable::build_fill_prompt(
        &serde_json::to_string_pretty(&variables)?,
        &context.to_summary(),
    );

    let request = ChatRequest {
        messages: vec![
            ChatMessage {
                role: MessageRole::System,
                content: prompts::SYSTEM_PROMPT.to_string(),
            },
            ChatMessage {
                role: MessageRole::User,
                content: prompt,
            },
        ],
        model: String::new(),
        temperature: Some(0.3),
        max_tokens: Some(2000),
    };

    let response = client.chat(request).await
        .map_err(|e| ToolError::ExecutionFailed(format!("AI 调用失败: {}", e)))?;

    let result: serde_json::Value = serde_json::from_str(&response.content)
        .unwrap_or_else(|_| json!({ "raw_response": response.content }));

    Ok(VariableFillResult {
        template: template_path.to_string(),
        project: context.project_name.clone(),
        filled: result.get("filled").cloned().unwrap_or_default(),
        confidence: result.get("confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32,
        ai_reasoning: result.get("reasoning")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

/// 直接分析变量（便捷函数）
pub async fn analyze_variables(
    template_path: &str,
) -> Result<VariableAnalysisResult, ToolError> {
    let content = tokio::fs::read_to_string(template_path)
        .await
        .map_err(|e| ToolError::ExecutionFailed(format!("读取模板文件失败: {}", e)))?;

    let variables = extract_variables_regex(&content);
    let total = variables.len();
    let auto_inferred = variables.iter()
        .filter(|v| v["source"].as_str() == Some("inferred"))
        .count();

    Ok(VariableAnalysisResult {
        template: template_path.to_string(),
        variables: variables.into_iter()
            .map(|v| serde_json::from_value(v).unwrap_or_default())
            .collect(),
        total,
        auto_inferred,
    })
}
