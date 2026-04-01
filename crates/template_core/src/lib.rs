//! # Template Studio Template Core
//!
//! 独立的模板渲染核心库，提供：
//! - Tera 模板引擎封装
//! - 自定义过滤器注册
//! - 内置函数/全局函数
//! - 纯字符串渲染（无文件系统依赖）
//!
//! ## 特性
//!
//! - ✨ **无文件系统依赖** - 可编译到 WASM
//! - 🚀 **高性能** - 单例引擎，预注册过滤器
//! - 🔧 **易扩展** - 模块化过滤器和内置函数
//! - 🧪 **易测试** - 纯函数设计
//!
//! ## 使用示例
//!
//! ### 渲染单个模板字符串
//!
//! ```rust
//! use template_studio_template_core::{render_string, Variables};
//!
//! let template = "Hello {{ name }}!";
//! let variables = Variables::from_json(r#"{ "name": "World" }"#).unwrap();
//! let result = render_string(template, &variables, None).unwrap();
//! assert_eq!(result.content, "Hello World!");
//! assert!(result.success);
//! ```
//!
//! ### 渲染文件树
//!
//! ```rust
//! use template_studio_template_core::{render_tree, TemplateFile};
//! # use template_studio_template_core::Variables;
//!
//! let tree = vec![
//!     TemplateFile {
//!         id: 1,
//!         file_path: "README.md".to_string(),
//!         file_name: "README.md".to_string(),
//!         file_content: "# {{ projectName }}".to_string(),
//!         is_directory: 0,
//!         parent_id: 0,
//!         filesize: 20,
//!         extends: None,
//!         includes: None,
//!         imports: None,
//!         condition: None,
//!         is_dependency: false,
//!         required_by: None,
//!     }
//! ];
//!
//! let variables = Variables::from_json(r#"{"projectName": "test"}"#).unwrap();
//! let rendered = render_tree(tree, &variables).unwrap();
//! ```

mod engine;
mod filters;
mod types;
mod builtin;
mod parallel;  // 批量渲染优化
pub mod tree;
pub mod conditions;
pub mod dependency_analyzer;
pub mod tree_builder;

// 重新导出公共类型
pub use types::{RenderResult, RenderError, Variables, FilterInfo};

// 导出文件树渲染相关类型和函数
pub use tree::{TemplateFile, RenderedFile, render_tree, IncludeDependency};

// 导出批量渲染
pub use parallel::render_tree_batch;

// 导出依赖分析相关类型和函数
pub use dependency_analyzer::{
    TeraDependencyAnalyzer,
    FileDependencies,
    ImportDependency,
};

// 导出树构建器相关类型和函数
pub use tree_builder::TreeBuilder;

// 导出条件管理相关类型和函数
pub use conditions::{
    ConditionType,
    Operator,
    SwitchCase,
    Condition,
    FileCondition,
    ConditionsYaml,
};

// 导出内置函数相关类型和函数
pub use builtin::{
    BuiltinFunctionCategory,
    BuiltinFunction,
    BuiltinFunctionParam,
    BuiltinFunctionsResponse,
    get_builtin_function_categories,
    get_builtin_functions_response,
};

// 导出渲染函数
pub use engine::{render_string, clear_template_cache, get_cache_size};

/// 预注册所有内置过滤器和函数
///
/// 通常在应用启动时调用一次即可
pub fn initialize() {
    engine::initialize_engine();
}

/// 获取所有可用的过滤器列表
pub fn get_available_filters() -> Vec<FilterInfo> {
    filters::get_all_filters()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_render() {
        let template = "Hello {{ name }}!";
        let variables = Variables::from_json(r#"{"name": "World"}"#).unwrap();
        let result = render_string(template, &variables, None).unwrap();
        assert_eq!(result.content, "Hello World!");
        assert!(result.success);
    }

    #[test]
    fn test_base64_filter() {
        let template = "{{ value | base64_encode }}";
        let variables = Variables::from_json(r#"{"value": "123"}"#).unwrap();
        let result = render_string(template, &variables, None).unwrap();
        assert_eq!(result.content, "MTIz");
    }

    #[test]
    fn test_error_line_number() {
        // 测试错误行号是否正确（第 8 行）
        let template = "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\n{{ sy }}\nline 9";
        let variables = Variables::from_json(r#"{"test": "value"}"#).unwrap();
        let result = render_string(template, &variables, None).unwrap();

        assert!(!result.success);
        assert!(result.error.is_some());

        let error = result.error.unwrap();
        assert_eq!(error.line, Some(8)); // 应该是第 8 行
        assert!(!error.message.contains("while rendering")); // 不应该包含模板名称
    }
}
