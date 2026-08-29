//! # 文件树构建模块
//!
//! 负责构建包含所有依赖的完整文件树
//!
//! ## 核心特性
//!
//! - **自动解析依赖** - 自动添加 include/extends/import 文件
//! - **循环检测** - 检测并报告循环依赖
//! - **条件过滤** - 根据条件筛选文件
//! - **WASM兼容** - 无文件系统依赖

use crate::conditions::ConditionsYaml;
use crate::dependency_analyzer::TeraDependencyAnalyzer;
use crate::tree::{IncludeDependency, TemplateFile};
use crate::types::Variables;
use std::collections::{HashMap, HashSet};

/// 文件树构建器
///
/// 负责从初始文件列表构建包含所有依赖的完整树
pub struct TreeBuilder {
    /// 依赖分析器
    analyzer: TeraDependencyAnalyzer,

    /// 是否自动解析依赖
    auto_resolve_dependencies: bool,

    /// 条件配置（可选）
    conditions: Option<ConditionsYaml>,
}

impl TreeBuilder {
    /// 创建新的树构建器
    pub fn new() -> Self {
        Self {
            analyzer: TeraDependencyAnalyzer::new(),
            auto_resolve_dependencies: true,
            conditions: None,
        }
    }

    /// 设置是否自动解析依赖
    pub fn with_auto_resolve(mut self, enabled: bool) -> Self {
        self.auto_resolve_dependencies = enabled;
        self
    }

    /// 设置条件配置
    pub fn with_conditions(mut self, conditions: ConditionsYaml) -> Self {
        self.conditions = Some(conditions);
        self
    }

    /// 构建完整的依赖树
    ///
    /// # 参数
    ///
    /// * `files` - 初始文件列表
    ///
    /// # 返回
    ///
    /// 包含所有依赖的完整文件树
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use template_studio_template_core::tree_builder::TreeBuilder;
    /// use template_studio_template_core::tree::TemplateFile;
    ///
    /// let builder = TreeBuilder::new();
    /// let files = vec![/* 初始文件 */];
    /// let complete_tree = builder.build_complete_tree(files).unwrap();
    /// ```
    pub fn build_complete_tree(
        &self,
        files: Vec<TemplateFile>,
    ) -> Result<Vec<TemplateFile>, String> {
        if !self.auto_resolve_dependencies {
            // 不自动解析，直接返回原文件
            return Ok(files);
        }

        // 1. 建立文件索引
        let file_map: HashMap<String, TemplateFile> = files
            .into_iter()
            .map(|f| (f.file_path.clone(), f))
            .collect();

        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        let mut result = Vec::new();

        // 2. 递归添加所有依赖
        for file_path in file_map.keys().cloned().collect::<Vec<_>>() {
            self.collect_dependencies(
                &file_path,
                &file_map,
                &mut result,
                &mut visiting,
                &mut visited,
            )?;
        }

        // 3. 更新依赖信息到文件节点
        self.update_dependency_info(&mut result, &file_map)?;

        Ok(result)
    }

    /// 递归收集文件依赖
    ///
    /// 深度优先遍历依赖图，添加所有依赖文件
    fn collect_dependencies(
        &self,
        file_path: &str,
        file_map: &HashMap<String, TemplateFile>,
        result: &mut Vec<TemplateFile>,
        visiting: &mut HashSet<String>,
        visited: &mut HashSet<String>,
    ) -> Result<(), String> {
        // 防止循环依赖
        if visiting.contains(file_path) {
            return Err(format!(
                "检测到循环依赖: {} -> ... -> {}",
                file_path, file_path
            ));
        }

        if visited.contains(file_path) {
            return Ok(());
        }

        // 检查文件是否存在
        let file = match file_map.get(file_path) {
            Some(f) => f,
            None => {
                // 文件不存在，跳过（可能是 Optional 依赖）
                #[cfg(feature = "logging")]
                tracing::warn!("依赖文件不存在，跳过: {}", file_path);
                return Ok(());
            }
        };

        visiting.insert(file_path.to_string());

        // 分析文件依赖
        let deps = self.analyzer.analyze(&file.file_content)?;

        // 1. 先添加 extends 父模板（父模板优先）
        if let Some(ref parent_path) = deps.extends {
            self.collect_dependencies(parent_path, file_map, result, visiting, visited)?;
        }

        // 2. 添加 import 宏文件
        for import_dep in &deps.imports {
            self.collect_dependencies(&import_dep.path, file_map, result, visiting, visited)?;
        }

        // 3. 添加 include 文件
        for include_dep in &deps.includes {
            let paths = match include_dep {
                IncludeDependency::Single(p) => vec![p.clone()],
                IncludeDependency::Multiple(ps) => ps.clone(),
                IncludeDependency::Optional(p) => vec![p.clone()],
            };

            for path in paths {
                // 对于 Optional，如果文件不存在就跳过
                if matches!(include_dep, IncludeDependency::Optional(_))
                    && !file_map.contains_key(&path)
                {
                    continue;
                }

                if file_map.contains_key(&path) {
                    self.collect_dependencies(&path, file_map, result, visiting, visited)?;
                }
            }
        }

        // 4. 添加当前文件
        result.push(file.clone());

        visiting.remove(file_path);
        visited.insert(file_path.to_string());

        Ok(())
    }

    /// 更新文件的依赖信息
    ///
    /// 将解析出的依赖信息添加到文件节点中
    fn update_dependency_info(
        &self,
        result: &mut [TemplateFile],
        _file_map: &HashMap<String, TemplateFile>,
    ) -> Result<(), String> {
        for file in result.iter_mut() {
            // 重新分析依赖信息并保存到文件节点
            let deps = self.analyzer.analyze(&file.file_content)?;

            file.extends = deps.extends;

            file.includes = if deps.includes.is_empty() {
                None
            } else {
                Some(deps.includes)
            };

            file.imports = if deps.imports.is_empty() {
                None
            } else {
                Some(deps.imports.into_iter().map(|d| d.path).collect())
            };
        }

        Ok(())
    }

    /// 根据条件过滤文件
    ///
    /// # 参数
    ///
    /// * `files` - 文件列表
    /// * `variables` - 渲染变量
    ///
    /// # 返回
    ///
    /// 满足条件的文件列表
    ///
    /// # 级联过滤规则
    ///
    /// - 如果目录被条件过滤，其所有子文件也会被过滤
    /// - 无条件的目录和文件始终包含
    pub fn filter_by_conditions(
        &self,
        files: Vec<TemplateFile>,
        variables: &Variables,
    ) -> Vec<TemplateFile> {
        filter_files_by_conditions(files, variables)
    }
}

/// 按文件条件过滤文件树（服务端渲染、WASM、桌面端、CLI 共用的统一入口）
///
/// 语义：
/// - 无条件 → 默认生成；条件评估失败 → 默认生成（fail-open）
/// - 目录条件不满足 → 级联剔除其全部子节点
pub fn filter_files_by_conditions(
    files: Vec<TemplateFile>,
    variables: &Variables,
) -> Vec<TemplateFile> {
    use std::collections::{HashMap, HashSet};

    // 1. 构建 parent_id -> children_ids 的映射
    let mut parent_to_children: HashMap<i64, Vec<i64>> = HashMap::new();
    for file in &files {
        if file.parent_id != 0 {
            parent_to_children
                .entry(file.parent_id)
                .or_default()
                .push(file.id);
        }
    }

    // 2. 收集所有需要过滤掉的节点 ID（包括子文件）
    let mut filtered_ids: HashSet<i64> = HashSet::new();

    for file in &files {
        // 无条件的文件/目录始终包含
        let condition = match &file.condition {
            Some(cond) => cond,
            None => {
                #[cfg(feature = "logging")]
                tracing::debug!("文件/目录无条件，包含: {}", file.file_path);
                continue;
            }
        };

        // 评估条件
        let should_generate = match condition.evaluate(variables.as_value()) {
            Ok(result) => result,
            Err(e) => {
                #[cfg(feature = "logging")]
                tracing::warn!(
                    "文件/目录 {} 条件评估失败: {}, 将默认生成",
                    file.file_path,
                    e
                );
                true // 评估失败时默认生成
            }
        };

        // 如果条件不满足，标记该节点及其所有子节点为过滤
        if !should_generate {
            #[cfg(feature = "logging")]
            tracing::info!(
                "文件/目录 {} 被条件过滤 (条件: {:?}, 变量: {:?})",
                file.file_path,
                condition,
                variables.as_value()
            );

            // 递归收集所有子节点 ID
            let mut to_visit = vec![file.id];
            while let Some(current_id) = to_visit.pop() {
                filtered_ids.insert(current_id);
                if let Some(children) = parent_to_children.get(&current_id) {
                    to_visit.extend(children);
                }
            }
        } else {
            #[cfg(feature = "logging")]
            tracing::debug!(
                "文件/目录 {} 条件评估通过，包含: {} (条件: {:?})",
                file.file_path,
                should_generate,
                condition
            );
        }
    }

    // 3. 过滤掉所有被标记的节点
    files
        .into_iter()
        .filter(|file| {
            let included = !filtered_ids.contains(&file.id);
            if !included {
                #[cfg(feature = "logging")]
                tracing::debug!("文件/目录 {} 被级联过滤", file.file_path);
            }
            included
        })
        .collect()
}

impl Default for TreeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_simple_tree() {
        let builder = TreeBuilder::new();

        let files = vec![
            TemplateFile {
                id: 1,
                file_path: "main.html".to_string(),
                file_name: "main.html".to_string(),
                file_content: "{% include \"header.html\" %}".to_string(),
                is_directory: 0,
                parent_id: 0,
                filesize: 50,
                extends: None,
                includes: None,
                imports: None,
                condition: None,
                is_dependency: false,
                required_by: None,
            },
            TemplateFile {
                id: 2,
                file_path: "header.html".to_string(),
                file_name: "header.html".to_string(),
                file_content: "<h1>Header</h1>".to_string(),
                is_directory: 0,
                parent_id: 0,
                filesize: 20,
                extends: None,
                includes: None,
                imports: None,
                condition: None,
                is_dependency: false,
                required_by: None,
            },
        ];

        let result = builder.build_complete_tree(files).unwrap();

        // 应该包含 main.html 和 header.html
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_detect_circular_dependency() {
        let builder = TreeBuilder::new();

        let files = vec![
            TemplateFile {
                id: 1,
                file_path: "a.html".to_string(),
                file_name: "a.html".to_string(),
                file_content: "{% include \"b.html\" %}".to_string(),
                is_directory: 0,
                parent_id: 0,
                filesize: 30,
                extends: None,
                includes: None,
                imports: None,
                condition: None,
                is_dependency: false,
                required_by: None,
            },
            TemplateFile {
                id: 2,
                file_path: "b.html".to_string(),
                file_name: "b.html".to_string(),
                file_content: "{% include \"a.html\" %}".to_string(),
                is_directory: 0,
                parent_id: 0,
                filesize: 30,
                extends: None,
                includes: None,
                imports: None,
                condition: None,
                is_dependency: false,
                required_by: None,
            },
        ];

        let result = builder.build_complete_tree(files);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("循环依赖"));
    }

    #[test]
    fn test_extends_chain() {
        let builder = TreeBuilder::new();

        let files = vec![
            TemplateFile {
                id: 1,
                file_path: "child.html".to_string(),
                file_name: "child.html".to_string(),
                file_content:
                    "{% extends \"parent.html\" %}\n{% block content %}Child{% endblock %}"
                        .to_string(),
                is_directory: 0,
                parent_id: 0,
                filesize: 80,
                extends: None,
                includes: None,
                imports: None,
                condition: None,
                is_dependency: false,
                required_by: None,
            },
            TemplateFile {
                id: 2,
                file_path: "parent.html".to_string(),
                file_name: "parent.html".to_string(),
                file_content:
                    "{% extends \"base.html\" %}\n{% block content %}Parent{% endblock %}"
                        .to_string(),
                is_directory: 0,
                parent_id: 0,
                filesize: 80,
                extends: None,
                includes: None,
                imports: None,
                condition: None,
                is_dependency: false,
                required_by: None,
            },
            TemplateFile {
                id: 3,
                file_path: "base.html".to_string(),
                file_name: "base.html".to_string(),
                file_content: "<html>{% block content %}{% endblock %}</html>".to_string(),
                is_directory: 0,
                parent_id: 0,
                filesize: 50,
                extends: None,
                includes: None,
                imports: None,
                condition: None,
                is_dependency: false,
                required_by: None,
            },
        ];

        let result = builder.build_complete_tree(files).unwrap();

        // 应该包含所有三个文件，且父模板在前
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].file_path, "base.html");
        assert_eq!(result[1].file_path, "parent.html");
        assert_eq!(result[2].file_path, "child.html");
    }

    #[test]
    fn test_auto_disabled() {
        let builder = TreeBuilder::new().with_auto_resolve(false);

        let files = vec![TemplateFile {
            id: 1,
            file_path: "main.html".to_string(),
            file_name: "main.html".to_string(),
            file_content: "{% include \"header.html\" %}".to_string(),
            is_directory: 0,
            parent_id: 0,
            filesize: 50,
            extends: None,
            includes: None,
            imports: None,
            condition: None,
            is_dependency: false,
            required_by: None,
        }];

        let result = builder.build_complete_tree(files).unwrap();

        // 不自动解析时，只返回原始文件
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_path, "main.html");
    }
}
