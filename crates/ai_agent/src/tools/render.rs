use async_trait::async_trait;
use serde_json::json;

use super::{AiTool, ToolError};
use crate::types::ToolResult;

/// 渲染预览工具
pub struct RenderPreviewTool;

#[async_trait]
impl AiTool for RenderPreviewTool {
    fn name(&self) -> &str {
        "render_preview"
    }

    fn description(&self) -> &str {
        "渲染模板并预览结果（文件树 + 内容摘要）"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "template_path": {
                    "type": "string",
                    "description": "模板目录路径"
                },
                "variables": {
                    "type": "object",
                    "description": "变量值 JSON"
                },
                "full": {
                    "type": "boolean",
                    "description": "是否输出完整内容",
                    "default": false
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

        let full = args["full"].as_bool().unwrap_or(false);

        // 读取模板目录
        let template_dir = std::path::Path::new(template_path);
        if !template_dir.exists() {
            return Err(ToolError::ExecutionFailed(format!("模板目录不存在: {}", template_path)));
        }

        // 扫描模板文件
        let mut files = Vec::new();
        scan_template_files(template_dir, template_dir, &mut files)?;

        // 构建变量
        let vars_json = serde_json::to_string(variables)?;
        let template_vars = template_studio_template_core::Variables::from_json(&vars_json)
            .map_err(|e| ToolError::ExecutionFailed(format!("解析变量失败: {}", e)))?;

        // 渲染每个文件
        let mut rendered_files = Vec::new();
        for (rel_path, content) in &files {
            match template_studio_template_core::render_string(content, &template_vars, None) {
                Ok(result) => {
                    if result.success {
                        rendered_files.push(json!({
                            "path": rel_path,
                            "size": result.content.len(),
                            "content": if full { Some(result.content) } else { None },
                            "success": true
                        }));
                    } else {
                        rendered_files.push(json!({
                            "path": rel_path,
                            "success": false,
                            "error": result.error.map(|e| e.message).unwrap_or_default()
                        }));
                    }
                }
                Err(e) => {
                    rendered_files.push(json!({
                        "path": rel_path,
                        "success": false,
                        "error": format!("{}", e)
                    }));
                }
            }
        }

        let success_count = rendered_files.iter()
            .filter(|f| f["success"].as_bool().unwrap_or(false))
            .count();

        let result = json!({
            "template": template_path,
            "files": rendered_files,
            "total": files.len(),
            "success": success_count,
            "failed": files.len() - success_count
        });

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&result)?,
            error: None,
        })
    }
}

/// 渲染导出工具
pub struct RenderExportTool;

#[async_trait]
impl AiTool for RenderExportTool {
    fn name(&self) -> &str {
        "render_export"
    }

    fn description(&self) -> &str {
        "渲染模板并导出到指定目录"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "template_path": {
                    "type": "string",
                    "description": "模板目录路径"
                },
                "variables": {
                    "type": "object",
                    "description": "变量值 JSON"
                },
                "output_dir": {
                    "type": "string",
                    "description": "输出目录路径"
                }
            },
            "required": ["template_path", "variables", "output_dir"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError> {
        let template_path = args["template_path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("缺少 template_path 参数".to_string()))?;

        let variables = args["variables"]
            .as_object()
            .ok_or_else(|| ToolError::InvalidArgument("variables 必须是对象".to_string()))?;

        let output_dir = args["output_dir"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("缺少 output_dir 参数".to_string()))?;

        // 读取模板目录
        let template_dir = std::path::Path::new(template_path);
        if !template_dir.exists() {
            return Err(ToolError::ExecutionFailed(format!("模板目录不存在: {}", template_path)));
        }

        // 扫描模板文件
        let mut files = Vec::new();
        scan_template_files(template_dir, template_dir, &mut files)?;

        // 构建变量
        let vars_json = serde_json::to_string(variables)?;
        let template_vars = template_studio_template_core::Variables::from_json(&vars_json)
            .map_err(|e| ToolError::ExecutionFailed(format!("解析变量失败: {}", e)))?;

        // 创建输出目录
        std::fs::create_dir_all(output_dir)?;

        // 渲染并写入文件
        let mut exported_files = Vec::new();
        for (rel_path, content) in &files {
            let output_path = std::path::Path::new(output_dir).join(rel_path);

            // 创建父目录
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            match template_studio_template_core::render_string(content, &template_vars, None) {
                Ok(result) => {
                    if result.success {
                        std::fs::write(&output_path, &result.content)?;
                        exported_files.push(rel_path.clone());
                    } else {
                        return Err(ToolError::ExecutionFailed(
                            format!("渲染失败 {}: {}", rel_path, result.error.map(|e| e.message).unwrap_or_default())
                        ));
                    }
                }
                Err(e) => {
                    return Err(ToolError::ExecutionFailed(format!("渲染失败 {}: {}", rel_path, e)));
                }
            }
        }

        let result = json!({
            "output_dir": output_dir,
            "files_exported": exported_files.len(),
            "files": exported_files
        });

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&result)?,
            error: None,
        })
    }
}

/// 扫描模板文件
pub fn scan_template_files(
    base: &std::path::Path,
    dir: &std::path::Path,
    files: &mut Vec<(String, String)>,
) -> Result<(), ToolError> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // 跳过隐藏目录和 meta 目录
            let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
            if dir_name.starts_with('.') || dir_name == ".meta" {
                continue;
            }
            scan_template_files(base, &path, files)?;
        } else {
            // 跳过隐藏文件和 variables.json
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            if file_name.starts_with('.') || file_name == "variables.json" {
                continue;
            }

            let rel_path = path.strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            let content = std::fs::read_to_string(&path)
                .map_err(|e| ToolError::ExecutionFailed(format!("读取文件失败 {}: {}", path.display(), e)))?;

            files.push((rel_path, content));
        }
    }

    Ok(())
}

/// 直接渲染预览（便捷函数）
pub async fn render_preview(
    template_path: &str,
    variables: &serde_json::Value,
    full: bool,
) -> Result<serde_json::Value, ToolError> {
    let tool = RenderPreviewTool;
    let args = json!({
        "template_path": template_path,
        "variables": variables,
        "full": full
    });
    let result = tool.execute(args).await?;
    serde_json::from_str(&result.output)
        .map_err(|e| ToolError::ExecutionFailed(format!("解析结果失败: {}", e)))
}

/// 直接渲染导出（便捷函数）
pub async fn render_export(
    template_path: &str,
    variables: &serde_json::Value,
    output_dir: &str,
) -> Result<serde_json::Value, ToolError> {
    let tool = RenderExportTool;
    let args = json!({
        "template_path": template_path,
        "variables": variables,
        "output_dir": output_dir
    });
    let result = tool.execute(args).await?;
    serde_json::from_str(&result.output)
        .map_err(|e| ToolError::ExecutionFailed(format!("解析结果失败: {}", e)))
}
