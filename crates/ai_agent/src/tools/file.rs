use async_trait::async_trait;
use serde_json::json;

use super::{AiTool, ToolError};
use crate::types::ToolResult;

/// 文件编辑工具（行级操作）
pub struct EditFileTool;

#[async_trait]
impl AiTool for EditFileTool {
    fn name(&self) -> &str {
        "edit_file"
    }

    fn description(&self) -> &str {
        "编辑文件（行级操作：插入、替换、删除、追加）"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "文件路径"
                },
                "operation": {
                    "type": "string",
                    "enum": ["insert", "replace", "delete", "append"],
                    "description": "操作类型"
                },
                "line": {
                    "type": "integer",
                    "description": "行号（insert/replace/delete 时必填，1-based）"
                },
                "end_line": {
                    "type": "integer",
                    "description": "结束行号（replace/delete 范围操作时使用，inclusive）"
                },
                "content": {
                    "type": "string",
                    "description": "插入/替换的内容"
                }
            },
            "required": ["path", "operation"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError> {
        let path = args["path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("缺少 path 参数".to_string()))?;

        let operation = args["operation"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("缺少 operation 参数".to_string()))?;

        // 备份原文件
        let backup_path = format!("{}.bak", path);
        if std::path::Path::new(path).exists() {
            tokio::fs::copy(path, &backup_path).await?;
        }

        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("读取文件失败: {}", e)))?;

        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        let line = args["line"].as_u64().map(|n| n as usize);
        let end_line = args["end_line"].as_u64().map(|n| n as usize);
        let insert_content = args["content"].as_str().unwrap_or("");

        match operation {
            "insert" => {
                let line_num = line.ok_or_else(|| ToolError::InvalidArgument("insert 操作需要 line 参数".to_string()))?;
                if line_num == 0 || line_num > lines.len() + 1 {
                    return Err(ToolError::InvalidArgument(format!("行号超出范围: {} (总共 {} 行)", line_num, lines.len())));
                }
                let new_lines: Vec<String> = insert_content.lines().map(|s| s.to_string()).collect();
                for (i, new_line) in new_lines.into_iter().enumerate() {
                    lines.insert(line_num - 1 + i, new_line);
                }
            }
            "replace" => {
                let start = line.ok_or_else(|| ToolError::InvalidArgument("replace 操作需要 line 参数".to_string()))?;
                let end = end_line.unwrap_or(start);
                if start == 0 || start > lines.len() {
                    return Err(ToolError::InvalidArgument(format!("起始行号超出范围: {}", start)));
                }
                if end > lines.len() {
                    return Err(ToolError::InvalidArgument(format!("结束行号超出范围: {}", end)));
                }
                let new_lines: Vec<String> = insert_content.lines().map(|s| s.to_string()).collect();
                let drain_range = (start - 1)..end;
                lines.splice(drain_range, new_lines);
            }
            "delete" => {
                let start = line.ok_or_else(|| ToolError::InvalidArgument("delete 操作需要 line 参数".to_string()))?;
                let end = end_line.unwrap_or(start);
                if start == 0 || start > lines.len() {
                    return Err(ToolError::InvalidArgument(format!("起始行号超出范围: {}", start)));
                }
                if end > lines.len() {
                    return Err(ToolError::InvalidArgument(format!("结束行号超出范围: {}", end)));
                }
                let drain_range = (start - 1)..end;
                let deleted_count = drain_range.len();
                lines.drain(drain_range);
                let result = json!({
                    "operation": "delete",
                    "deleted_lines": deleted_count,
                    "remaining_lines": lines.len(),
                    "backup": backup_path
                });
                let new_content = lines.join("\n");
                tokio::fs::write(path, &new_content).await?;
                return Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&result)?,
                    error: None,
                });
            }
            "append" => {
                lines.push(insert_content.to_string());
            }
            _ => {
                return Err(ToolError::InvalidArgument(format!("未知操作: {}", operation)));
            }
        }

        let new_content = lines.join("\n");
        tokio::fs::write(path, &new_content).await?;

        let result = json!({
            "operation": operation,
            "total_lines": lines.len(),
            "backup": backup_path
        });

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&result)?,
            error: None,
        })
    }
}

/// 直接编辑文件（便捷函数）
pub async fn edit_file(
    path: &str,
    operation: &str,
    line: Option<usize>,
    end_line: Option<usize>,
    content: Option<&str>,
) -> Result<serde_json::Value, ToolError> {
    let tool = EditFileTool;
    let mut args = json!({
        "path": path,
        "operation": operation
    });
    if let Some(l) = line {
        args["line"] = json!(l);
    }
    if let Some(el) = end_line {
        args["end_line"] = json!(el);
    }
    if let Some(c) = content {
        args["content"] = json!(c);
    }
    let result = tool.execute(args).await?;
    serde_json::from_str(&result.output)
        .map_err(|e| ToolError::ExecutionFailed(format!("解析结果失败: {}", e)))
}
