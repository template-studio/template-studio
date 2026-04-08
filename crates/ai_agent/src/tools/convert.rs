use async_trait::async_trait;
use serde_json::json;
use std::collections::HashSet;
use std::path::Path;

use super::{AiTool, ToolError};
use crate::types::ToolResult;

/// 项目转模板工具
pub struct ConvertToTemplateTool;

#[async_trait]
impl AiTool for ConvertToTemplateTool {
    fn name(&self) -> &str {
        "convert_to_template"
    }

    fn description(&self) -> &str {
        "将项目转换为模板项目（识别重复模式，替换为变量）"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "项目路径"
                },
                "output_path": {
                    "type": "string",
                    "description": "输出模板路径"
                },
                "name": {
                    "type": "string",
                    "description": "模板名称"
                },
                "category": {
                    "type": "string",
                    "description": "模板分类"
                },
                "strategy": {
                    "type": "string",
                    "enum": ["conservative", "aggressive"],
                    "description": "变量识别策略：conservative（保守，只替换明确的模式）或 aggressive（激进，替换更多模式）",
                    "default": "conservative"
                }
            },
            "required": ["project_path", "output_path"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError> {
        let project_path = args["project_path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("缺少 project_path 参数".to_string()))?;

        let output_path = args["output_path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("缺少 output_path 参数".to_string()))?;

        let name = args["name"].as_str().unwrap_or("Untitled Template");
        let category = args["category"].as_str().unwrap_or("general");
        let strategy = args["strategy"].as_str().unwrap_or("conservative");

        let project_dir = Path::new(project_path);
        if !project_dir.exists() {
            return Err(ToolError::ExecutionFailed(format!("项目路径不存在: {}", project_path)));
        }

        // 扫描项目文件
        let mut files = Vec::new();
        scan_project_files(project_dir, project_dir, &mut files)?;

        // 识别变量模式
        let patterns = identify_patterns(&files, strategy);

        // 生成模板文件
        let output_dir = Path::new(output_path);
        std::fs::create_dir_all(output_dir)?;

        let mut template_files = Vec::new();

        for (rel_path, content) in &files {
            let mut template_content = content.clone();
            let mut used_vars = Vec::new();

            for pattern in &patterns {
                if template_content.contains(&pattern.original) {
                    template_content = template_content.replace(&pattern.original, &format!("{{{{ {} }}}}", pattern.var_name));
                    if !used_vars.contains(&pattern.var_name) {
                        used_vars.push(pattern.var_name.clone());
                    }
                }
            }

            // 写入模板文件
            let out_path = output_dir.join(rel_path);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&out_path, &template_content)?;

            template_files.push(json!({
                "path": rel_path,
                "variables": used_vars
            }));
        }

        // 生成 variables.json
        let variables_schema: Vec<serde_json::Value> = patterns.iter()
            .map(|p| {
                json!({
                    "name": p.var_name,
                    "type": p.var_type,
                    "title": p.title,
                    "description": p.description,
                    "required": true,
                    "default": p.original
                })
            })
            .collect();

        let meta_dir = output_dir.join(".meta").join("variables");
        std::fs::create_dir_all(&meta_dir)?;
        std::fs::write(
            meta_dir.join("variables.json"),
            serde_json::to_string_pretty(&variables_schema)?,
        )?;

        // 生成模板元信息
        let template_meta = json!({
            "name": name,
            "category": category,
            "version": "1.0.0",
            "files_count": template_files.len(),
            "variables_count": patterns.len()
        });
        std::fs::write(
            output_dir.join(".meta").join("template.json"),
            serde_json::to_string_pretty(&template_meta)?,
        )?;

        let result = json!({
            "output": output_path,
            "name": name,
            "category": category,
            "strategy": strategy,
            "files": template_files.len(),
            "variables": patterns.len(),
            "template_files": template_files,
            "variables_schema": variables_schema
        });

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&result)?,
            error: None,
        })
    }
}

/// 项目文件信息
struct PatternInfo {
    original: String,
    var_name: String,
    var_type: String,
    title: String,
    description: String,
}

/// 扫描项目文件（排除常见目录）
fn scan_project_files(
    base: &Path,
    dir: &Path,
    files: &mut Vec<(String, String)>,
) -> Result<(), ToolError> {
    if !dir.exists() {
        return Ok(());
    }

    let skip_dirs: HashSet<&str> = [
        "target", "node_modules", ".git", ".idea", ".vscode",
        "dist", "build", ".meta", "__pycache__", ".next",
    ].iter().cloned().collect();

    let skip_extensions: HashSet<&str> = [
        "exe", "dll", "so", "dylib", "bin", "o", "obj",
        "png", "jpg", "jpeg", "gif", "ico", "svg", "webp",
        "zip", "tar", "gz", "rar", "7z",
        "mp3", "mp4", "avi", "mov",
        "db", "sqlite", "sqlite3",
    ].iter().cloned().collect();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
            if skip_dirs.contains(dir_name.as_ref()) || dir_name.starts_with('.') {
                continue;
            }
            scan_project_files(base, &path, files)?;
        } else {
            let ext = path.extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();
            if skip_extensions.contains(ext.as_str()) {
                continue;
            }

            // 跳过过大的文件
            let metadata = std::fs::metadata(&path)?;
            if metadata.len() > 1024 * 100 {
                // 跳过 > 100KB 的文件
                continue;
            }

            let rel_path = path.strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            match std::fs::read_to_string(&path) {
                Ok(content) => files.push((rel_path, content)),
                Err(_) => continue, // 跳过非文本文件
            }
        }
    }

    Ok(())
}

/// 识别项目中的重复模式
fn identify_patterns(files: &[(String, String)], strategy: &str) -> Vec<PatternInfo> {
    let mut patterns = Vec::new();
    let mut seen = HashSet::new();

    // 收集所有文件名（不含扩展名）作为可能的模式
    let _file_stems: Vec<String> = files.iter()
        .filter_map(|(path, _)| {
            let p = Path::new(path);
            p.file_stem().map(|s| s.to_string_lossy().to_string())
        })
        .collect();

    // 分析文件名中的共同前缀/后缀
    for (rel_path, content) in files {
        let path = Path::new(rel_path);

        // 检查包名/命名空间模式
        let path_str = rel_path.replace('\\', "/");
        let segments: Vec<&str> = path_str.split('/').collect();

        // 识别包名模式（如 com.example.xxx）
        for segment in &segments {
            if segment.contains('.') && segment.matches('.').count() >= 2 {
                let parts: Vec<&str> = segment.split('.').collect();
                // 最后一部分可能是项目名
                if let Some(last) = parts.last() {
                    let var_name = "ProjectPackage";
                    if !seen.contains(var_name) && last.len() > 2 {
                        seen.insert(var_name.to_string());
                        patterns.push(PatternInfo {
                            original: segment.to_string(),
                            var_name: var_name.to_string(),
                            var_type: "string".to_string(),
                            title: "项目包名".to_string(),
                            description: "项目的包名/命名空间".to_string(),
                        });
                    }
                }
            }
        }

        // 识别类名/文件名模式
        if let Some(stem) = path.file_stem() {
            let stem_str = stem.to_string_lossy();
            // 如果多个文件使用相同的命名模式（如 XxxService, XxxController）
            let suffixes = ["Service", "Controller", "Repository", "Mapper", "Entity", "Model", "Dto", "Vo"];
            for suffix in &suffixes {
                if stem_str.ends_with(suffix) {
                    let prefix = &stem_str[..stem_str.len() - suffix.len()];
                    if prefix.len() > 0 && !seen.contains(&format!("ClassName{}", suffix)) {
                        let _var_name = format!("{{{{ {}Name }}}}", suffix);
                        seen.insert(format!("ClassName{}", suffix));
                        patterns.push(PatternInfo {
                            original: prefix.to_string(),
                            var_name: format!("{}Name", suffix),
                            var_type: "string".to_string(),
                            title: format!("{} 类名前缀", suffix),
                            description: format!("{} 的名称前缀", suffix),
                        });
                    }
                }
            }
        }

        // 识别内容中的重复字符串
        let words: Vec<&str> = content.split_whitespace().collect();
        for window in words.windows(3) {
            let phrase = window.join(" ");
            if phrase.len() > 5 && phrase.len() < 50 {
                // 统计出现次数
                let count = content.matches(&phrase).count();
                if count >= 3 && !seen.contains(&phrase) {
                    // 检查是否是变量名、关键字等
                    if !is_code_keyword(&phrase) {
                        seen.insert(phrase.clone());
                        if strategy == "aggressive" {
                            patterns.push(PatternInfo {
                                original: phrase.clone(),
                                var_name: format!("Text{}", patterns.len()),
                                var_type: "string".to_string(),
                                title: format!("重复文本 #{}", patterns.len()),
                                description: format!("在项目中出现 {} 次的文本", count),
                            });
                        }
                    }
                }
            }
        }
    }

    // 保守策略：只保留明确的模式
    if strategy == "conservative" {
        patterns.retain(|p| {
            p.var_name.contains("Package")
                || p.var_name.contains("Name")
                || p.var_name.contains("Project")
        });
    }

    patterns
}

/// 检查是否是代码关键字
fn is_code_keyword(s: &str) -> bool {
    let keywords = [
        "public", "private", "protected", "static", "final", "abstract",
        "class", "interface", "enum", "extends", "implements",
        "import", "package", "return", "if", "else", "for", "while",
        "try", "catch", "finally", "throw", "throws",
        "new", "this", "super", "null", "true", "false",
        "void", "int", "long", "double", "float", "boolean", "string",
        "pub", "fn", "let", "mut", "use", "mod", "struct", "impl",
        "async", "await", "match", "self", "Self",
        "def", "class", "import", "from", "return", "self",
        "function", "const", "var", "let", "export", "default",
    ];
    let lower = s.to_lowercase();
    keywords.iter().any(|k| k == &lower)
}

/// 直接转换项目为模板（便捷函数）
pub async fn convert_to_template(
    project_path: &str,
    output_path: &str,
    name: Option<&str>,
    category: Option<&str>,
    strategy: &str,
) -> Result<serde_json::Value, ToolError> {
    let tool = ConvertToTemplateTool;
    let mut args = json!({
        "project_path": project_path,
        "output_path": output_path,
        "strategy": strategy
    });
    if let Some(n) = name {
        args["name"] = json!(n);
    }
    if let Some(c) = category {
        args["category"] = json!(c);
    }
    let result = tool.execute(args).await?;
    serde_json::from_str(&result.output)
        .map_err(|e| ToolError::ExecutionFailed(format!("解析结果失败: {}", e)))
}
