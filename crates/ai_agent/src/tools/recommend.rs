use async_trait::async_trait;
use serde_json::json;

use super::{AiTool, ToolError};
use crate::types::ToolResult;

/// 模板推荐工具
pub struct RecommendTemplateTool;

#[async_trait]
impl AiTool for RecommendTemplateTool {
    fn name(&self) -> &str {
        "recommend_template"
    }

    fn description(&self) -> &str {
        "根据项目特征推荐合适的模板"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "项目路径（用于分析项目特征）"
                },
                "language": {
                    "type": "string",
                    "description": "编程语言偏好"
                },
                "category": {
                    "type": "string",
                    "description": "模板分类偏好"
                },
                "explain": {
                    "type": "boolean",
                    "description": "是否输出推荐理由",
                    "default": true
                }
            },
            "required": ["project_path"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError> {
        let project_path = args["project_path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArgument("缺少 project_path 参数".to_string()))?;

        let language_pref = args["language"].as_str();
        let category_pref = args["category"].as_str();
        let explain = args["explain"].as_bool().unwrap_or(true);

        let project_dir = std::path::Path::new(project_path);
        if !project_dir.exists() {
            return Err(ToolError::ExecutionFailed(format!("项目路径不存在: {}", project_path)));
        }

        // 分析项目特征
        let features = analyze_project_features(project_dir)?;

        // 根据特征匹配推荐
        let mut recommendations = Vec::new();

        // 基于语言推荐
        if let Some(detected_lang) = &features.primary_language {
            let lang_templates = get_templates_for_language(detected_lang);
            for tmpl in lang_templates {
                let mut score = tmpl.base_score;

                // 语言匹配加分
                if Some(detected_lang.as_str()) == language_pref {
                    score += 0.2;
                }

                // 分类匹配加分
                if Some(tmpl.category.as_str()) == category_pref {
                    score += 0.15;
                }

                // 框架匹配加分
                for framework in &features.frameworks {
                    if tmpl.tags.contains(&framework.to_lowercase()) {
                        score += 0.1;
                    }
                }

                recommendations.push(json!({
                    "name": tmpl.name,
                    "category": tmpl.category,
                    "language": detected_lang,
                    "score": score.min(1.0),
                    "tags": tmpl.tags,
                    "reason": if explain {
                        Some(build_recommendation_reason(detected_lang, &tmpl.name, &features))
                    } else {
                        None
                    }
                }));
            }
        }

        // 基于项目类型推荐
        if features.has_database {
            let db_templates = get_templates_for_database();
            for tmpl in db_templates {
                let exists = recommendations.iter().any(|r| r["name"] == tmpl.name);
                if !exists {
                    recommendations.push(json!({
                        "name": tmpl.name,
                        "category": tmpl.category,
                        "language": features.primary_language,
                        "score": tmpl.base_score,
                        "tags": tmpl.tags,
                        "reason": if explain { Some("项目包含数据库相关文件") } else { None }
                    }));
                }
            }
        }

        // 按分数排序
        recommendations.sort_by(|a, b| {
            b["score"].as_f64().unwrap_or(0.0)
                .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 取前 5 个
        recommendations.truncate(5);

        let result = json!({
            "project": project_path,
            "features": {
                "language": features.primary_language,
                "frameworks": features.frameworks,
                "has_database": features.has_database,
                "has_api": features.has_api,
                "has_tests": features.has_tests,
            },
            "recommendations": recommendations,
            "total": recommendations.len()
        });

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&result)?,
            error: None,
        })
    }
}

/// 项目特征
struct ProjectFeatures {
    primary_language: Option<String>,
    frameworks: Vec<String>,
    has_database: bool,
    has_api: bool,
    has_tests: bool,
}

/// 模板信息
struct TemplateInfo {
    name: String,
    category: String,
    base_score: f64,
    tags: Vec<String>,
}

/// 分析项目特征
fn analyze_project_features(project_dir: &std::path::Path) -> Result<ProjectFeatures, ToolError> {
    let mut languages = std::collections::HashMap::new();
    let mut frameworks = Vec::new();
    let mut has_database = false;
    let mut has_api = false;
    let mut has_tests = false;

    // 扫描文件
    scan_features(project_dir, project_dir, &mut languages, &mut frameworks, &mut has_database, &mut has_api, &mut has_tests)?;

    // 找出主要语言
    let primary_language = languages.iter()
        .max_by_key(|(_, count)| *count)
        .map(|(lang, _)| lang.clone());

    Ok(ProjectFeatures {
        primary_language,
        frameworks,
        has_database,
        has_api,
        has_tests,
    })
}

fn scan_features(
    base: &std::path::Path,
    dir: &std::path::Path,
    languages: &mut std::collections::HashMap<String, usize>,
    frameworks: &mut Vec<String>,
    has_database: &mut bool,
    has_api: &mut bool,
    has_tests: &mut bool,
) -> Result<(), ToolError> {
    if !dir.exists() {
        return Ok(());
    }

    let skip_dirs = ["target", "node_modules", ".git", ".idea", ".vscode", "dist", "build"];

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
            if skip_dirs.contains(&dir_name.as_ref()) || dir_name.starts_with('.') {
                continue;
            }

            // 检查是否是测试目录
            if dir_name == "test" || dir_name == "tests" || dir_name == "__tests__" {
                *has_tests = true;
            }

            scan_features(base, &path, languages, frameworks, has_database, has_api, has_tests)?;
        } else {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            let ext = path.extension().unwrap_or_default().to_string_lossy();

            // 统计语言
            match ext.as_ref() {
                "rs" => *languages.entry("rust".to_string()).or_insert(0) += 1,
                "py" => *languages.entry("python".to_string()).or_insert(0) += 1,
                "js" | "ts" | "jsx" | "tsx" => *languages.entry("javascript".to_string()).or_insert(0) += 1,
                "java" => *languages.entry("java".to_string()).or_insert(0) += 1,
                "go" => *languages.entry("go".to_string()).or_insert(0) += 1,
                "rb" => *languages.entry("ruby".to_string()).or_insert(0) += 1,
                "php" => *languages.entry("php".to_string()).or_insert(0) += 1,
                _ => {}
            }

            // 检测框架
            match file_name.as_ref() {
                "Cargo.toml" => {
                    frameworks.push("rust".to_string());
                }
                "package.json" => {
                    frameworks.push("node".to_string());
                }
                "pom.xml" | "build.gradle" => {
                    frameworks.push("java".to_string());
                    frameworks.push("maven".to_string());
                }
                "requirements.txt" | "setup.py" | "pyproject.toml" => {
                    frameworks.push("python".to_string());
                }
                "go.mod" => {
                    frameworks.push("go".to_string());
                }
                "Gemfile" => {
                    frameworks.push("ruby".to_string());
                }
                _ => {}
            }

            // 检测数据库相关
            if file_name.contains("migration") || file_name.contains("schema") || file_name.contains("model") {
                *has_database = true;
            }
            if ext.as_ref() == "sql" {
                *has_database = true;
            }

            // 检测 API 相关
            if file_name.contains("controller") || file_name.contains("handler") || file_name.contains("route") {
                *has_api = true;
            }
        }
    }

    Ok(())
}

fn get_templates_for_language(language: &str) -> Vec<TemplateInfo> {
    match language {
        "rust" => vec![
            TemplateInfo { name: "rust-web-api".to_string(), category: "web".to_string(), base_score: 0.8, tags: vec!["rust".to_string(), "axum".to_string(), "api".to_string()] },
            TemplateInfo { name: "rust-cli".to_string(), category: "cli".to_string(), base_score: 0.7, tags: vec!["rust".to_string(), "cli".to_string()] },
            TemplateInfo { name: "rust-lib".to_string(), category: "library".to_string(), base_score: 0.6, tags: vec!["rust".to_string(), "library".to_string()] },
        ],
        "python" => vec![
            TemplateInfo { name: "fastapi-project".to_string(), category: "web".to_string(), base_score: 0.8, tags: vec!["python".to_string(), "fastapi".to_string(), "api".to_string()] },
            TemplateInfo { name: "django-project".to_string(), category: "web".to_string(), base_score: 0.75, tags: vec!["python".to_string(), "django".to_string(), "web".to_string()] },
            TemplateInfo { name: "flask-project".to_string(), category: "web".to_string(), base_score: 0.7, tags: vec!["python".to_string(), "flask".to_string(), "web".to_string()] },
        ],
        "javascript" => vec![
            TemplateInfo { name: "nextjs-project".to_string(), category: "web".to_string(), base_score: 0.85, tags: vec!["javascript".to_string(), "nextjs".to_string(), "react".to_string()] },
            TemplateInfo { name: "express-api".to_string(), category: "web".to_string(), base_score: 0.8, tags: vec!["javascript".to_string(), "express".to_string(), "api".to_string()] },
            TemplateInfo { name: "vue-project".to_string(), category: "web".to_string(), base_score: 0.75, tags: vec!["javascript".to_string(), "vue".to_string(), "frontend".to_string()] },
        ],
        "java" => vec![
            TemplateInfo { name: "spring-boot".to_string(), category: "web".to_string(), base_score: 0.85, tags: vec!["java".to_string(), "spring".to_string(), "api".to_string()] },
            TemplateInfo { name: "mybatis-crud".to_string(), category: "crud".to_string(), base_score: 0.7, tags: vec!["java".to_string(), "mybatis".to_string(), "crud".to_string()] },
        ],
        "go" => vec![
            TemplateInfo { name: "go-gin-api".to_string(), category: "web".to_string(), base_score: 0.8, tags: vec!["go".to_string(), "gin".to_string(), "api".to_string()] },
            TemplateInfo { name: "go-cli".to_string(), category: "cli".to_string(), base_score: 0.7, tags: vec!["go".to_string(), "cli".to_string()] },
        ],
        _ => vec![],
    }
}

fn get_templates_for_database() -> Vec<TemplateInfo> {
    vec![
        TemplateInfo { name: "crud-generator".to_string(), category: "crud".to_string(), base_score: 0.6, tags: vec!["crud".to_string(), "database".to_string()] },
        TemplateInfo { name: "api-with-db".to_string(), category: "web".to_string(), base_score: 0.65, tags: vec!["api".to_string(), "database".to_string()] },
    ]
}

fn build_recommendation_reason(language: &str, _template_name: &str, features: &ProjectFeatures) -> String {
    let mut reasons = Vec::new();

    reasons.push(format!("检测到项目主要使用 {}", language));

    if !features.frameworks.is_empty() {
        reasons.push(format!("发现框架: {}", features.frameworks.join(", ")));
    }

    if features.has_database {
        reasons.push("项目包含数据库相关文件".to_string());
    }

    if features.has_api {
        reasons.push("项目包含 API 相关文件".to_string());
    }

    if features.has_tests {
        reasons.push("项目包含测试目录".to_string());
    }

    reasons.join("；")
}

/// 直接推荐模板（便捷函数）
pub async fn recommend_template(
    project_path: &str,
    language: Option<&str>,
    category: Option<&str>,
    explain: bool,
) -> Result<serde_json::Value, ToolError> {
    let tool = RecommendTemplateTool;
    let mut args = json!({
        "project_path": project_path,
        "explain": explain
    });
    if let Some(l) = language {
        args["language"] = json!(l);
    }
    if let Some(c) = category {
        args["category"] = json!(c);
    }
    let result = tool.execute(args).await?;
    serde_json::from_str(&result.output)
        .map_err(|e| ToolError::ExecutionFailed(format!("解析结果失败: {}", e)))
}
