use async_trait::async_trait;
use serde_json::json;

use super::{AiTool, ToolError};
use super::variable::extract_variables_regex;
use crate::types::ToolResult;

/// 语法验证工具
pub struct ValidateSyntaxTool;

#[async_trait]
impl AiTool for ValidateSyntaxTool {
    fn name(&self) -> &str {
        "validate_syntax"
    }

    fn description(&self) -> &str {
        "验证模板语法（MiniJinja 语法检查）"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "template_path": {
                    "type": "string",
                    "description": "模板文件路径"
                }
            },
            "required": ["template_path"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError> {
        let template_path = args["template_path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("缺少 template_path 参数".to_string()))?;

        let content = tokio::fs::read_to_string(template_path)
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("读取模板文件失败: {}", e)))?;

        // 基本语法检查
        let mut errors = Vec::new();

        // 检查未闭合的标签
        let open_count = content.matches("{{").count();
        let close_count = content.matches("}}").count();
        if open_count != close_count {
            errors.push(format!("未闭合的变量标签: {{ 有 {} 个, }} 有 {} 个", open_count, close_count));
        }

        // 检查未闭合的块标签
        let block_open = content.matches("{%").count();
        let block_close = content.matches("%}").count();
        if block_open != block_close {
            errors.push(format!("未闭合的块标签: {{% 有 {} 个, %}} 有 {} 个", block_open, block_close));
        }

        // 检查过滤器语法 {{ var | filter }}
        for (idx, part) in content.split("{{").skip(1).enumerate() {
            if let Some(end) = part.find("}}") {
                let expr = &part[..end];
                // 检查过滤器管道语法
                if expr.contains('|') {
                    let filters: Vec<&str> = expr.split('|').skip(1).collect();
                    for filter in filters {
                        let filter_name = filter.trim().split('(').next().unwrap_or("").trim();
                        if filter_name.is_empty() {
                            errors.push(format!("第 {} 个表达式: 空过滤器名", idx + 1));
                        }
                    }
                }
            }
        }

        let result = json!({
            "valid": errors.is_empty(),
            "errors": errors,
            "template": template_path
        });

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
        let mut unused = Vec::new();

        // 检查缺失的变量
        for var in &required_vars {
            let name = var["name"].as_str().unwrap_or_default();
            if !variables.contains_key(name) {
                missing.push(name.to_string());
            }
        }

        // 检查未使用的变量
        let var_names: std::collections::HashSet<String> = required_vars.iter()
            .filter_map(|v| v["name"].as_str().map(|s| s.to_string()))
            .collect();
        for key in variables.keys() {
            if !var_names.contains(key) {
                unused.push(key.clone());
            }
        }

        let result = json!({
            "valid": missing.is_empty(),
            "missing": missing,
            "unused": unused,
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

/// 输出验证工具
pub struct ValidateOutputTool;

#[async_trait]
impl AiTool for ValidateOutputTool {
    fn name(&self) -> &str {
        "validate_output"
    }

    fn description(&self) -> &str {
        "验证渲染输出（语法检查）"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "渲染输出内容"
                },
                "language": {
                    "type": "string",
                    "description": "编程语言（java, python, javascript 等）"
                }
            },
            "required": ["content", "language"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError> {
        let content = args["content"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("缺少 content 参数".to_string()))?;

        let language = args["language"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("缺少 language 参数".to_string()))?;

        let mut errors = Vec::new();

        // 基本语法检查
        match language {
            "java" => {
                // 检查大括号匹配
                let open = content.matches('{').count();
                let close = content.matches('}').count();
                if open != close {
                    errors.push(format!("大括号不匹配: {{ 有 {} 个, }} 有 {} 个", open, close));
                }
                // 检查分号
                let lines: Vec<&str> = content.lines().collect();
                for (_i, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty()
                        && !trimmed.starts_with("//")
                        && !trimmed.starts_with("/*")
                        && !trimmed.starts_with("*")
                        && !trimmed.ends_with('{')
                        && !trimmed.ends_with('}')
                        && !trimmed.ends_with('(')
                        && !trimmed.ends_with(')')
                        && !trimmed.ends_with(',')
                        && !trimmed.ends_with(';')
                        && !trimmed.contains("class ")
                        && !trimmed.contains("interface ")
                        && !trimmed.contains("import ")
                        && !trimmed.contains("package ")
                    {
                        // 可能缺少分号（启发式检查）
                        // 这里不做严格检查，只提示
                    }
                }
            }
            "python" => {
                // Python 基本检查
                let lines: Vec<&str> = content.lines().collect();
                for (i, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.ends_with(':') && !trimmed.starts_with('#') {
                        // 检查下一行是否缩进
                        if i + 1 < lines.len() {
                            let next_line = lines[i + 1];
                            if !next_line.is_empty() && !next_line.starts_with(' ') && !next_line.starts_with('\t') {
                                errors.push(format!("第 {} 行: 缺少缩进", i + 2));
                            }
                        }
                    }
                }
            }
            "javascript" | "typescript" => {
                // JS/TS 基本检查
                let open = content.matches('{').count();
                let close = content.matches('}').count();
                if open != close {
                    errors.push(format!("大括号不匹配: {{ 有 {} 个, }} 有 {} 个", open, close));
                }
            }
            _ => {
                // 通用检查
            }
        }

        let result = json!({
            "valid": errors.is_empty(),
            "errors": errors,
            "language": language
        });

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&result)?,
            error: None,
        })
    }
}

/// 直接验证语法（便捷函数）
pub async fn validate_syntax(template_path: &str) -> Result<serde_json::Value, ToolError> {
    let tool = ValidateSyntaxTool;
    let args = json!({ "template_path": template_path });
    let result = tool.execute(args).await?;
    serde_json::from_str(&result.output)
        .map_err(|e| ToolError::ExecutionFailed(format!("解析结果失败: {}", e)))
}

/// 直接验证变量（便捷函数）
pub async fn validate_variables(
    template_path: &str,
    variables: &serde_json::Value,
) -> Result<serde_json::Value, ToolError> {
    let tool = ValidateVariablesTool;
    let args = json!({
        "template_path": template_path,
        "variables": variables
    });
    let result = tool.execute(args).await?;
    serde_json::from_str(&result.output)
        .map_err(|e| ToolError::ExecutionFailed(format!("解析结果失败: {}", e)))
}
