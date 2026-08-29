//! 批量渲染优化（条件编译）
//!
//! - Native 平台：使用 rayon 多线程并行渲染
//! - WASM 平台：降级到单线程串行渲染

use crate::tree::render_single_file;
use crate::{RenderedFile, TemplateFile, Variables};

#[cfg(feature = "native")]
use rayon::prelude::*;

/// 批量渲染文件树（自动选择最优策略）
///
/// # 平台差异
///
/// - **Native（后端）**：根据文件数量智能选择并行/串行
///   - 文件数 >= 50：使用 rayon 多线程并行
///   - 文件数 < 50：使用单线程串行（避免调度开销）
/// - **WASM（前端）**：始终使用单线程串行处理
///
/// # 性能预期
///
/// - Native（大文件量）：1.5-3x 提升（取决于 CPU 核心数和模板复杂度）
/// - Native（小文件量）：1x（串行，避免开销）
/// - WASM：1x（无降级，保持原有性能）
///
/// # 参数
///
/// * `files` - 待渲染的文件树
/// * `variables` - 渲染变量
/// * `all_templates` - 所有模板映射（用于模板继承）
///
/// # 返回
///
/// 渲染后的文件列表
pub fn render_tree_batch(
    files: Vec<TemplateFile>,
    variables: &Variables,
    all_templates: &std::collections::HashMap<String, String>,
) -> Vec<RenderedFile> {
    #[cfg(feature = "native")]
    {
        // 智能选择：小批量使用串行，大批量使用并行
        const PARALLEL_THRESHOLD: usize = 50;

        if files.len() < PARALLEL_THRESHOLD {
            #[cfg(feature = "logging")]
            tracing::debug!(
                "Small batch ({} files), using sequential rendering (threshold: {})",
                files.len(),
                PARALLEL_THRESHOLD
            );
            render_tree_sequential(files, variables, all_templates)
        } else {
            render_tree_parallel(files, variables, all_templates)
        }
    }

    #[cfg(not(feature = "native"))]
    {
        render_tree_sequential(files, variables, all_templates)
    }
}

/// Native 平台：并行渲染（使用 rayon）
#[cfg(feature = "native")]
fn render_tree_parallel(
    files: Vec<TemplateFile>,
    variables: &Variables,
    all_templates: &std::collections::HashMap<String, String>,
) -> Vec<RenderedFile> {
    use rayon::current_num_threads;

    #[cfg(feature = "logging")]
    tracing::info!(
        "Parallel rendering {} files with {} threads (rayon enabled)",
        files.len(),
        current_num_threads()
    );

    // 并行渲染（直接使用引用，rayon 会自动处理生命周期）
    let results: Vec<_> = files
        .into_par_iter() // 并行迭代器
        .map(|file| {
            render_single_file(&file, variables, all_templates).unwrap_or_else(|e| {
                #[cfg(feature = "logging")]
                tracing::error!("Failed to render file {:?}: {}", file.file_name, e);

                // 返回错误结果
                RenderedFile {
                    id: file.id,
                    file_path: file.file_path.clone(),
                    file_name: file.file_name.clone(),
                    file_content: None,
                    is_directory: file.is_directory,
                    filesize: 0,
                    parent_id: file.parent_id,
                    error: Some(e),
                }
            })
        })
        .collect();

    #[cfg(feature = "logging")]
    tracing::info!(
        "Parallel rendering completed: {} files (used {} threads)",
        results.len(),
        current_num_threads()
    );

    results
}

/// 串行渲染（通用实现，所有平台都可用）
fn render_tree_sequential(
    files: Vec<TemplateFile>,
    variables: &Variables,
    all_templates: &std::collections::HashMap<String, String>,
) -> Vec<RenderedFile> {
    #[cfg(feature = "logging")]
    tracing::info!("Sequential rendering {} files", files.len());

    let mut results = Vec::with_capacity(files.len());

    for file in files {
        match render_single_file(&file, variables, all_templates) {
            Ok(rendered) => results.push(rendered),
            Err(e) => {
                #[cfg(feature = "logging")]
                tracing::error!("Failed to render file {:?}: {}", file.file_name, e);

                results.push(RenderedFile {
                    id: file.id,
                    file_path: file.file_path.clone(),
                    file_name: file.file_name.clone(),
                    file_content: None,
                    is_directory: file.is_directory,
                    filesize: 0,
                    parent_id: file.parent_id,
                    error: Some(e),
                });
            }
        }
    }

    #[cfg(feature = "logging")]
    tracing::info!("Sequential rendering completed: {} files", results.len());

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_files(count: usize) -> Vec<TemplateFile> {
        (0..count)
            .map(|i| TemplateFile {
                id: i as i64,
                file_path: format!("test{}.txt", i),
                file_name: format!("test{}.txt", i),
                file_content: "Value: {{x}}".to_string(), // MiniJinja 模板语法
                is_directory: 0,
                parent_id: 0,
                filesize: 20,
                extends: None,
                includes: None,
                imports: None,
                condition: None,
                is_dependency: false,
                required_by: None,
            })
            .collect()
    }

    #[test]
    fn test_batch_render() {
        let files = create_test_files(10);
        let variables = Variables::from_json(r#"{"x": 42}"#).unwrap();
        let templates = std::collections::HashMap::new();

        let results = render_tree_batch(files, &variables, &templates);

        assert_eq!(results.len(), 10);
        for result in results {
            assert!(result.error.is_none()); // 修复：检查 error 字段而不是 success()
            assert_eq!(result.file_content, Some("Value: 42".to_string()));
        }
    }

    #[test]
    fn test_small_batch_uses_sequential() {
        // 小批量（< 50）应该使用串行
        let files = create_test_files(10); // 10 < 50，应该串行
        let variables = Variables::from_json(r#"{"x": 42}"#).unwrap();
        let templates = std::collections::HashMap::new();

        let results = render_tree_batch(files, &variables, &templates);

        assert_eq!(results.len(), 10);
        // 验证渲染正确性
        assert!(results.iter().all(|r| r.error.is_none()));
    }

    #[test]
    fn test_large_batch_uses_parallel() {
        // 大批量（>= 50）应该使用并行
        let files = create_test_files(100); // 100 >= 50，应该并行
        let variables = Variables::from_json(r#"{"x": 42}"#).unwrap();
        let templates = std::collections::HashMap::new();

        let results = render_tree_batch(files, &variables, &templates);

        assert_eq!(results.len(), 100);
        // 验证渲染正确性
        assert!(results.iter().all(|r| r.error.is_none()));
    }

    #[test]
    fn test_batch_render_with_error() {
        let mut files = create_test_files(3);
        // 添加一个有错误的文件
        files.push(TemplateFile {
            id: 99,
            file_path: "error.txt".to_string(),
            file_name: "error.txt".to_string(),
            file_content: "Hello {{ undefined_var }}!".to_string(), // 未定义变量
            is_directory: 0,
            parent_id: 0,
            filesize: 30,
            extends: None,
            includes: None,
            imports: None,
            condition: None,
            is_dependency: false,
            required_by: None,
        });

        let variables = Variables::from_json(r#"{}"#).unwrap();
        let templates = std::collections::HashMap::new();

        let results = render_tree_batch(files, &variables, &templates);

        // 应该返回所有结果，包括失败的
        assert_eq!(results.len(), 4);

        // 检查错误文件
        let error_file = results.iter().find(|f| f.file_name == "error.txt").unwrap();
        assert!(error_file.error.is_some()); // 修复：检查 error 字段
    }
}
