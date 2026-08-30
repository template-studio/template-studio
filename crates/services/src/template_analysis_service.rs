//! 模板变量分析服务

use std::path::PathBuf;

/// 模板变量检测正则（{{ var }} / {{ obj.field }} / {{ var | filter }}）
static TEMPLATE_VAR_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"\{\{\s*([a-zA-Z_][\w\.]*)(?:\s*\|\s*[^\}]*)?\s*\}\}")
        .expect("模板变量正则编译失败")
});

use template_studio_shared::{models::template_analysis::*, utils::error::AppError};
use tokio::fs;

/// 模板变量分析服务
pub struct TemplateAnalysisService {
    base_path: PathBuf,
}

impl TemplateAnalysisService {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// 分析模板中的变量使用情况
    pub async fn analyze_variables(
        &self,
        template_id: i64,
    ) -> Result<VariableAnalysisResponse, AppError> {
        let template_root_path = self.base_path.join(template_id.to_string());
        let template_path = template_root_path.join("src");

        tracing::info!(
            "分析模板变量，模板ID: {}, 路径: {:?}",
            template_id,
            template_path
        );

        // 如果 src/ 目录不存在，尝试直接使用模板根目录
        let analysis_path = if template_path.exists() {
            template_path
        } else {
            tracing::info!("src/ 目录不存在，使用根目录: {:?}", template_root_path);
            template_root_path.clone()
        };

        if !analysis_path.exists() {
            tracing::warn!("分析路径不存在: {:?}", analysis_path);
            return Ok(VariableAnalysisResponse {
                detected_variables: vec![],
                missing_variables: vec![],
                unused_variables: vec![],
                conflict_variables: vec![],
                total_variable_count: 0,
                analyzed_file_count: 0,
            });
        }

        // 收集所有模板文件（递归扫描子目录）
        let mut files = Vec::new();
        self.collect_template_files(&analysis_path, &mut files)
            .await;

        tracing::info!("找到 {} 个文件需要分析", files.len());

        // 检测变量
        let mut detected_variables: std::collections::HashMap<String, DetectedVariable> =
            std::collections::HashMap::new();

        for file_path in &files {
            let content = match fs::read_to_string(file_path).await {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("读取文件失败 {:?}: {:?}", file_path, e);
                    continue;
                }
            };

            let file_name = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            tracing::info!("分析文件: {} (大小: {} 字节)", file_name, content.len());

            // 使用正则表达式检测模板变量（LazyLock 静态编译，不在循环内重复构建）
            let re = &*TEMPLATE_VAR_RE;

            let mut count = 0;
            for captures in re.captures_iter(&content) {
                if let Some(var_name) = captures.get(1) {
                    let var_name_str = var_name.as_str().to_string();
                    let context = captures
                        .get(0)
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default();

                    tracing::debug!("检测到变量: {} 上下文: {}", var_name_str, context);

                    detected_variables
                        .entry(var_name_str.clone())
                        .and_modify(|var| {
                            if !var.files.contains(&file_name) {
                                var.files.push(file_name.clone());
                            }
                            if !var.contexts.contains(&context) {
                                var.contexts.push(context.clone());
                            }
                        })
                        .or_insert_with(|| DetectedVariable {
                            name: var_name_str.clone(),
                            var_type: "string".to_string(),
                            files: vec![file_name.clone()],
                            contexts: vec![context],
                            suggestions: "建议使用 string 类型，或根据实际用途选择其他类型"
                                .to_string(),
                        });

                    count += 1;
                }
            }

            tracing::info!("文件 {} 检测到 {} 个变量", file_name, count);
        }

        let detected_vars: Vec<DetectedVariable> = detected_variables.into_values().collect();
        let total_count = detected_vars.len();
        let analyzed_count = files.len();

        // 读取已定义的变量
        let defined_variables = self
            .load_defined_variables(template_id)
            .await
            .unwrap_or_default();

        // 找出缺失的变量（在模板中使用但未定义的）
        let missing_variables: Vec<MissingVariable> = detected_vars
            .iter()
            .filter(|detected| !defined_variables.contains(&detected.name))
            .map(|detected| MissingVariable {
                name: detected.name.clone(),
                var_type: detected.var_type.clone(),
                files: detected.files.clone(),
                contexts: detected.contexts.clone(),
                suggestions: format!("建议添加此变量到变量定义中"),
            })
            .collect();

        tracing::info!(
            "分析完成: 检测到 {} 个变量，已定义 {} 个变量，缺失 {} 个变量",
            total_count,
            defined_variables.len(),
            missing_variables.len()
        );

        Ok(VariableAnalysisResponse {
            detected_variables: detected_vars,
            missing_variables,
            unused_variables: vec![],
            conflict_variables: vec![],
            total_variable_count: total_count,
            analyzed_file_count: analyzed_count,
        })
    }

    /// 加载已定义的变量列表
    async fn load_defined_variables(&self, template_id: i64) -> Result<Vec<String>, AppError> {
        let variables_file_path = self
            .base_path
            .join(template_id.to_string())
            .join(".meta")
            .join("variables")
            .join("variables.json");

        if !variables_file_path.exists() {
            tracing::info!("变量定义文件不存在: {:?}", variables_file_path);
            return Ok(vec![]);
        }

        match fs::read_to_string(&variables_file_path).await {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(value) => {
                    if let Some(obj) = value.as_object() {
                        let variable_names: Vec<String> = obj.keys().cloned().collect();
                        tracing::info!(
                            "加载到 {} 个已定义变量: {:?}",
                            variable_names.len(),
                            variable_names
                        );
                        Ok(variable_names)
                    } else {
                        tracing::warn!("变量定义文件格式错误，不是对象类型");
                        Ok(vec![])
                    }
                }
                Err(e) => {
                    tracing::error!("解析变量定义文件失败: {:?}", e);
                    Ok(vec![])
                }
            },
            Err(e) => {
                tracing::error!("读取变量定义文件失败: {:?}", e);
                Ok(vec![])
            }
        }
    }

    /// 递归收集模板文件（使用非递归方式避免异步递归问题）
    async fn collect_template_files(
        &self,
        dir_path: &std::path::Path,
        files: &mut Vec<std::path::PathBuf>,
    ) {
        let mut dirs_to_visit = vec![dir_path.to_path_buf()];

        while let Some(current_dir) = dirs_to_visit.pop() {
            if let Ok(mut entries) = fs::read_dir(&current_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.is_file() {
                        // 只处理文本文件（包括 Tera 模板文件）
                        if let Some(ext) = path.extension() {
                            let ext_str = ext.to_str().unwrap_or("");
                            if [
                                "go", "rs", "py", "js", "ts", "java", "html", "htm", "tml", "tera",
                                "txt", "md", "css", "json", "xml",
                            ]
                            .contains(&ext_str)
                            {
                                tracing::info!("发现文件: {:?}", path);
                                files.push(path);
                            }
                        }
                    } else if path.is_dir() {
                        // 将子目录加入待访问队列
                        dirs_to_visit.push(path);
                    }
                }
            }
        }
    }
}
