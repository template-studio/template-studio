use async_trait::async_trait;
use serde_json::json;

use super::{AiTool, ToolError};
use super::render::scan_template_files;
use crate::types::ToolResult;

/// 渲染对比工具
pub struct RenderDiffTool;

#[async_trait]
impl AiTool for RenderDiffTool {
    fn name(&self) -> &str {
        "render_diff"
    }

    fn description(&self) -> &str {
        "对比两组变量的渲染结果差异"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "template_path": {
                    "type": "string",
                    "description": "模板目录路径"
                },
                "variables_a": {
                    "type": "object",
                    "description": "第一组变量"
                },
                "variables_b": {
                    "type": "object",
                    "description": "第二组变量"
                }
            },
            "required": ["template_path", "variables_a", "variables_b"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError> {
        let template_path = args["template_path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("缺少 template_path 参数".to_string()))?;

        let variables_a = args["variables_a"]
            .as_object()
            .ok_or_else(|| ToolError::InvalidArgument("variables_a 必须是对象".to_string()))?;

        let variables_b = args["variables_b"]
            .as_object()
            .ok_or_else(|| ToolError::InvalidArgument("variables_b 必须是对象".to_string()))?;

        let template_dir = std::path::Path::new(template_path);
        if !template_dir.exists() {
            return Err(ToolError::ExecutionFailed(format!("模板目录不存在: {}", template_path)));
        }

        // 扫描模板文件
        let mut files: Vec<(String, String)> = Vec::new();
        scan_template_files(template_dir, template_dir, &mut files)?;

        // 渲染两组变量
        let vars_a_json = serde_json::to_string(variables_a)?;
        let vars_b_json = serde_json::to_string(variables_b)?;

        let template_vars_a = template_studio_template_core::Variables::from_json(&vars_a_json)
            .map_err(|e| ToolError::ExecutionFailed(format!("解析变量 A 失败: {}", e)))?;
        let template_vars_b = template_studio_template_core::Variables::from_json(&vars_b_json)
            .map_err(|e| ToolError::ExecutionFailed(format!("解析变量 B 失败: {}", e)))?;

        let mut diffs = Vec::new();
        let mut total_files = 0;
        let mut diff_count = 0;

        for (rel_path, content) in &files {
            total_files += 1;

            let render_a = template_studio_template_core::render_string(content, &template_vars_a, None);
            let render_b = template_studio_template_core::render_string(content, &template_vars_b, None);

            match (render_a, render_b) {
                (Ok(result_a), Ok(result_b)) => {
                    if result_a.success && result_b.success {
                        if result_a.content != result_b.content {
                            diff_count += 1;
                            let changes = compute_diff(&result_a.content, &result_b.content);
                            diffs.push(json!({
                                "path": rel_path,
                                "has_diff": true,
                                "changes": changes,
                                "size_a": result_a.content.len(),
                                "size_b": result_b.content.len(),
                            }));
                        } else {
                            diffs.push(json!({
                                "path": rel_path,
                                "has_diff": false,
                            }));
                        }
                    } else {
                        let error_a = if !result_a.success { result_a.error.map(|e| e.message) } else { None };
                        let error_b = if !result_b.success { result_b.error.map(|e| e.message) } else { None };
                        diffs.push(json!({
                            "path": rel_path,
                            "has_diff": true,
                            "render_error_a": error_a,
                            "render_error_b": error_b,
                        }));
                        diff_count += 1;
                    }
                }
                (Err(e), _) | (_, Err(e)) => {
                    diffs.push(json!({
                        "path": rel_path,
                        "has_diff": true,
                        "error": format!("{}", e),
                    }));
                    diff_count += 1;
                }
            }
        }

        // 统计变量差异
        let mut var_diffs = Vec::new();
        let all_keys: std::collections::HashSet<String> = variables_a.keys()
            .chain(variables_b.keys())
            .cloned()
            .collect();

        for key in &all_keys {
            let val_a = variables_a.get(key);
            let val_b = variables_b.get(key);
            if val_a != val_b {
                var_diffs.push(json!({
                    "variable": key,
                    "value_a": val_a,
                    "value_b": val_b,
                }));
            }
        }

        let result = json!({
            "template": template_path,
            "total_files": total_files,
            "diff_count": diff_count,
            "same_count": total_files - diff_count,
            "variable_diffs": var_diffs,
            "file_diffs": diffs,
        });

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&result)?,
            error: None,
        })
    }
}

/// 计算简单的行级差异
fn compute_diff(content_a: &str, content_b: &str) -> Vec<serde_json::Value> {
    let lines_a: Vec<&str> = content_a.lines().collect();
    let lines_b: Vec<&str> = content_b.lines().collect();

    let mut changes = Vec::new();
    let max_len = lines_a.len().max(lines_b.len());

    for i in 0..max_len {
        let line_a = lines_a.get(i);
        let line_b = lines_b.get(i);

        match (line_a, line_b) {
            (Some(a), Some(b)) => {
                if a != b {
                    changes.push(json!({
                        "line": i + 1,
                        "type": "modified",
                        "old": a,
                        "new": b,
                    }));
                }
            }
            (Some(a), None) => {
                changes.push(json!({
                    "line": i + 1,
                    "type": "removed",
                    "old": a,
                }));
            }
            (None, Some(b)) => {
                changes.push(json!({
                    "line": i + 1,
                    "type": "added",
                    "new": b,
                }));
            }
            (None, None) => unreachable!(),
        }
    }

    changes
}

/// 直接渲染对比（便捷函数）
pub async fn render_diff(
    template_path: &str,
    variables_a: &serde_json::Value,
    variables_b: &serde_json::Value,
) -> Result<serde_json::Value, ToolError> {
    let tool = RenderDiffTool;
    let args = json!({
        "template_path": template_path,
        "variables_a": variables_a,
        "variables_b": variables_b
    });
    let result = tool.execute(args).await?;
    serde_json::from_str(&result.output)
        .map_err(|e| ToolError::ExecutionFailed(format!("解析结果失败: {}", e)))
}
