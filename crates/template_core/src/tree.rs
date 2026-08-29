//! # 文件树渲染模块
//!
//! 提供文件树渲染功能，支持WASM编译。
//!
//! ## 核心特性
//!
//! - **无文件系统依赖** - 所有数据通过参数传递
//! - **纯函数设计** - 易于测试和WASM编译
//! - **容错处理** - 单个文件渲染失败不影响其他文件

use crate::conditions::Condition;
use crate::render_string;
use crate::types::{RenderError, Variables};
use serde::{Deserialize, Serialize};

/// Include 依赖类型
///
/// 表示 Tera 模板中不同形式的 include 语句
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncludeDependency {
    /// 单个文件: {% include "header.html" %}
    Single(String),

    /// 多候选项（按优先级查找第一个存在的）: {% include ["custom/header.html", "header.html"] %}
    Multiple(Vec<String>),

    /// 可选文件（文件不存在时忽略）: {% include "optional.html" ignore missing %}
    Optional(String),
}

/// 模板文件节点（输入）
///
/// 表示模板目录中的一个文件或目录。
/// 扁平化结构，通过 `parent_id` 建立父子关系。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    /// 节点ID
    pub id: i64,

    /// 相对路径（如 "src/main.go"）
    pub file_path: String,

    /// 文件名（如 "main.go"）
    pub file_name: String,

    /// 文件内容（模板字符串，目录为空字符串）
    pub file_content: String,

    /// 是否为目录（1=目录，0=文件）
    pub is_directory: i32,

    /// 父节点ID（根节点为0）
    pub parent_id: i64,

    /// 文件大小（字节）
    pub filesize: i32,

    // ==================== 新增：依赖关系字段 ====================
    /// 继承的父模板路径（extends）: {% extends "base.html" %}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,

    /// 包含的文件依赖列表（include）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub includes: Option<Vec<IncludeDependency>>,

    /// 导入的宏文件路径列表（import）: {% import "macros.html" as macros %}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imports: Option<Vec<String>>,

    /// 文件生成条件
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<Condition>,

    /// 是否为依赖文件（被其他文件 include/import/extends）
    #[serde(default)]
    pub is_dependency: bool,

    /// 被哪些文件依赖（文件ID列表）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_by: Option<Vec<i64>>,
}

/// 渲染后的文件节点（输出）
///
/// 表示渲染后的文件或目录。
/// 扁平化结构，通过 `parent_id` 建立父子关系。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedFile {
    /// 节点ID（与TemplateFile对应）
    pub id: i64,

    /// 渲染后的文件路径
    pub file_path: String,

    /// 渲染后的文件名
    pub file_name: String,

    /// 渲染后的文件内容（目录为None）
    pub file_content: Option<String>,

    /// 是否为目录（1=目录，0=文件）
    pub is_directory: i32,

    /// 文件大小（字节，渲染失败时为0）
    pub filesize: i32,

    /// 父节点ID（根节点为0）
    pub parent_id: i64,

    /// 渲染错误信息（仅当渲染失败时有值）
    pub error: Option<RenderError>,
}

/// 渲染文件树
///
/// # 参数
///
/// * `files` - 模板文件列表（扁平结构）
/// * `variables` - 模板变量
///
/// # 返回
///
/// 渲染后的文件列表（扁平结构，通过 parent_id 建立关系）
///
/// # 特性
///
/// - **纯函数** - 无副作用，可缓存
/// - **容错处理** - 单个文件渲染失败不影响其他文件
/// - **WASM兼容** - 无文件系统依赖
///
/// # 示例
///
/// ```no_run
/// use template_studio_template_core::{render_tree, TemplateFile};
/// use template_studio_template_core::Variables;
///
/// let files = vec![
///     TemplateFile {
///         id: 1,
///         file_path: "README.md".to_string(),
///         file_name: "README.md".to_string(),
///         file_content: "# {{ projectName }}".to_string(),
///         is_directory: 0,
///         parent_id: 0,
///         filesize: 20,
///         extends: None,
///         includes: None,
///         imports: None,
///         condition: None,
///         is_dependency: false,
///         required_by: None,
///     }
/// ];
///
/// let variables = Variables::from_json(r#"{"projectName": "MyApp"}"#).unwrap();
/// let result = render_tree(files, &variables).unwrap();
///
/// assert_eq!(result[0].file_content, Some("# MyApp".to_string()));
/// ```
///
/// 渲染文件树（高层 API）
///
/// 这是**文件树渲染的统一入口**，用于渲染完整的模板文件树。
///
/// - 如果只需要渲染单个模板字符串，请使用 `render_string()`
/// - 此函数会自动处理模板继承（`{% extends %}`）、包含（`{% include %}`）等依赖
///
/// # 参数
///
/// * `tree` - 完整的文件树（包含所有依赖文件和条件信息）
/// * `variables` - 渲染变量
///
/// # 返回
///
/// 渲染后的文件树（扁平结构，通过 parent_id 建立关系）
///
/// # 特性
///
/// - **高层 API** - 文件树渲染的统一入口
/// - **条件渲染** - 根据文件条件自动过滤
/// - **容错处理** - 单个文件渲染失败不影响其他文件
/// - **模板继承** - 自动处理 HTML 模板的 `{% extends %}` 依赖
/// - **WASM 兼容** - 无文件系统依赖
/// - **上下文完整** - 包含所有 include/extends/import 依赖
///
/// # 示例
///
/// ```no_run
/// use template_studio_template_core::{render_tree, TreeBuilder, TemplateFile};
/// use template_studio_template_core::Variables;
///
/// // 1. 准备变量
/// let variables = Variables::from_json(r#"{"name": "World"}"#).unwrap();
///
/// // 2. 构建文件树（包含所有依赖）
/// let builder = TreeBuilder::new();
/// let files = vec![/* 初始文件列表 */];
/// let tree = builder.build_complete_tree(files).unwrap();
///
/// // 3. 根据条件过滤文件
/// let filtered = builder.filter_by_conditions(tree, &variables);
///
/// // 4. 渲染
/// let rendered = render_tree(filtered, &variables).unwrap();
/// ```
#[allow(clippy::result_large_err)]
pub fn render_tree(
    tree: Vec<TemplateFile>,
    variables: &Variables,
) -> Result<Vec<RenderedFile>, RenderError> {
    // 构建所有模板文件的映射（用于模板继承），双键注册语义见 build_template_map
    let all_templates = build_template_map(
        tree.iter()
            .filter(|f| f.is_directory == 0 && !f.file_content.is_empty())
            .map(|f| (f.file_path.clone(), f.file_content.clone())),
    );

    // 使用批量渲染（自动选择最优策略：Native 并行 / WASM 串行）
    let results = crate::parallel::render_tree_batch(tree, variables, &all_templates);

    Ok(results)
}

/// 由 (相对路径, 内容) 序列构建双键模板映射：
/// - 相对路径为主键：`{% extends "layouts/base.html" %}` 按路径引用可解析，
///   跨目录同名文件天然消歧
/// - basename 为兼容键，仅在全集合唯一时注册：保持 `extends "base.html"` 简写可用；
///   同名冲突时跳过（明确报找不到，优于随机覆盖）
///
/// 整树渲染（render_tree）与单文件预览（render_file_from_path）共用此函数，
/// 保证「预览结果 = 最终渲染结果」。
pub fn build_template_map<I: IntoIterator<Item = (String, String)>>(
    entries: I,
) -> std::collections::HashMap<String, String> {
    let entries: Vec<(String, String)> = entries.into_iter().collect();

    let basename_of = |path: &str| path.rsplit('/').next().unwrap_or(path).to_string();

    let mut basename_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for (path, _) in &entries {
        *basename_counts.entry(basename_of(path)).or_insert(0) += 1;
    }

    let mut map: std::collections::HashMap<String, String> =
        std::collections::HashMap::with_capacity(entries.len() * 2);
    for (path, content) in entries {
        let base = basename_of(&path);
        map.insert(path, content.clone());
        if basename_counts[&base] == 1 {
            map.entry(base).or_insert(content);
        }
    }
    map
}

/// 渲染单个文件（内部实现细节）
///
/// 此函数是 `render_tree()` 的内部实现细节，但也被 parallel 模块使用
///
/// # 参数
///
/// * `file` - 单个模板文件节点
/// * `variables` - 模板变量
/// * `all_templates` - 所有模板文件的映射（用于模板继承）
#[allow(clippy::result_large_err)]
pub(crate) fn render_single_file(
    file: &TemplateFile,
    variables: &Variables,
    all_templates: &std::collections::HashMap<String, String>,
) -> Result<RenderedFile, RenderError> {
    // 渲染文件名（目录和文件都需要渲染）
    let rendered_name = render_string(&file.file_name, variables, None)?;

    // 渲染文件路径（目录和文件都需要渲染）
    let rendered_path = render_string(&file.file_path, variables, None)?;

    // 目录不渲染内容
    if file.is_directory == 1 {
        return Ok(RenderedFile {
            id: file.id,
            file_path: rendered_path.content,
            file_name: rendered_name.content,
            file_content: None,
            is_directory: 1,
            filesize: 0,
            parent_id: file.parent_id,
            error: None,
        });
    }

    // 渲染文件内容：所有文件统一走支持继承的渲染（传递整棵模板树）。
    // MiniJinja 对任意模板支持 extends/include 语法，不再按 .html 扩展名分流；
    // 性能由引擎侧的环境缓存（模板集哈希复用）吸收
    let render_result = render_string(&file.file_content, variables, Some(all_templates))?;

    // 检查渲染是否成功
    if !render_result.success {
        // 返回错误信息
        return Err(render_result.error.unwrap_or_else(|| RenderError {
            error_type: "render_error".to_string(),
            message: "Unknown rendering error".to_string(),
            line: None,
            column: None,
            context: None,
            suggestion: None,
        }));
    }

    // 计算文件大小
    let filesize = render_result.content.len() as i32;

    Ok(RenderedFile {
        id: file.id,
        file_path: rendered_path.content,
        file_name: rendered_name.content,
        file_content: Some(render_result.content),
        is_directory: 0,
        filesize,
        parent_id: file.parent_id,
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_file(id: i64, path: &str, content: &str) -> TemplateFile {
        TemplateFile {
            id,
            file_path: path.to_string(),
            file_name: path.rsplit('/').next().unwrap_or(path).to_string(),
            file_content: content.to_string(),
            is_directory: 0,
            parent_id: 0,
            filesize: content.len() as i32,
            extends: None,
            includes: None,
            imports: None,
            condition: None,
            is_dependency: false,
            required_by: None,
        }
    }

    #[test]
    fn test_inheritance_by_path_and_basename() {
        let vars = crate::types::Variables::from_json("{}").unwrap();
        let tree = vec![
            mk_file(1, "layouts/base.html", "<b>{% block c %}{% endblock %}</b>"),
            mk_file(
                2,
                "pages/by_path.html",
                "{% extends \"layouts/base.html\" %}{% block c %}P{% endblock %}",
            ),
            mk_file(
                3,
                "pages/by_name.html",
                "{% extends \"base.html\" %}{% block c %}N{% endblock %}",
            ),
        ];
        let out = render_tree(tree, &vars).unwrap();
        assert_eq!(
            out[1].file_content.as_deref(),
            Some("<b>P</b>"),
            "路径引用应可解析"
        );
        assert_eq!(
            out[2].file_content.as_deref(),
            Some("<b>N</b>"),
            "basename 简写应保持可用"
        );
    }

    #[test]
    fn test_same_name_cross_dir_no_silent_overwrite() {
        let vars = crate::types::Variables::from_json("{}").unwrap();
        let tree = vec![
            mk_file(1, "a/base.html", "<A>{% block c %}{% endblock %}</A>"),
            mk_file(2, "b/base.html", "<B>{% block c %}{% endblock %}</B>"),
            // 同名冲突：basename 简写不可用（明确失败，不随机继承某一个）
            mk_file(
                3,
                "pages/ambiguous.html",
                "{% extends \"base.html\" %}{% block c %}X{% endblock %}",
            ),
            // 路径引用精确消歧
            mk_file(
                4,
                "pages/precise.html",
                "{% extends \"b/base.html\" %}{% block c %}Y{% endblock %}",
            ),
        ];
        let out = render_tree(tree, &vars).unwrap();
        assert!(!out[2].file_content.as_deref().unwrap_or("").contains("<A>"));
        assert!(!out[2].file_content.as_deref().unwrap_or("").contains("<B>"));
        assert_eq!(
            out[3].file_content.as_deref(),
            Some("<B>Y</B>"),
            "路径引用应精确消歧"
        );
    }

    #[test]
    fn test_inheritance_for_non_html_files() {
        let vars = crate::types::Variables::from_json("{}").unwrap();
        let tree = vec![
            mk_file(1, "fragments/header.txt", "HEADER[{{ name }}]"),
            mk_file(
                2,
                "notes/readme.txt",
                "{% include \"fragments/header.txt\" %} body",
            ),
        ];
        let vars = crate::types::Variables::from_json(r#"{"name":"n"}"#).unwrap();
        let out = render_tree(tree, &vars).unwrap();
        assert_eq!(
            out[1].file_content.as_deref(),
            Some("HEADER[n] body"),
            "非 HTML 文件也应支持 include/extends"
        );
    }

    #[test]
    fn test_render_single_file() {
        let files = vec![TemplateFile {
            id: 1,
            file_path: "README.md".to_string(),
            file_name: "README.md".to_string(),
            file_content: "# {{ projectName }}".to_string(),
            is_directory: 0,
            parent_id: 0,
            filesize: 20,
            extends: None,
            includes: None,
            imports: None,
            condition: None,
            is_dependency: false,
            required_by: None,
        }];

        let variables = Variables::from_json(r#"{"projectName": "test"}"#).unwrap();
        let result = render_tree(files, &variables).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_content, Some("# test".to_string()));
        assert_eq!(result[0].error, None);
    }

    #[test]
    fn test_render_directory() {
        let files = vec![TemplateFile {
            id: 1,
            file_path: "src".to_string(),
            file_name: "src".to_string(),
            file_content: "".to_string(),
            is_directory: 1,
            parent_id: 0,
            filesize: 0,
            extends: None,
            includes: None,
            imports: None,
            condition: None,
            is_dependency: false,
            required_by: None,
        }];

        let variables = Variables::from_json(r#"{}"#).unwrap();
        let result = render_tree(files, &variables).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_content, None);
        assert_eq!(result[0].is_directory, 1);
    }

    #[test]
    fn test_render_with_error() {
        // 测试语法错误的情况
        let files = vec![TemplateFile {
            id: 1,
            file_path: "test.txt".to_string(),
            file_name: "test.txt".to_string(),
            file_content: "Hello {{ undefined_var }!".to_string(), // 语法错误：缺少右括号
            is_directory: 0,
            parent_id: 0,
            filesize: 30,
            extends: None,
            includes: None,
            imports: None,
            condition: None,
            is_dependency: false,
            required_by: None,
        }];

        let variables = Variables::from_json(r#"{}"#).unwrap();
        let result = render_tree(files, &variables).unwrap();

        // 应该返回结果，但带有错误信息
        assert_eq!(result.len(), 1);
        assert!(result[0].error.is_some());
        assert_eq!(result[0].file_content, None);
    }

    #[test]
    fn test_render_file_name_and_path() {
        let files = vec![TemplateFile {
            id: 1,
            file_path: "src/{{ language }}/main.go".to_string(),
            file_name: "main.{{ ext }}".to_string(),
            file_content: "package main".to_string(),
            is_directory: 0,
            parent_id: 0,
            filesize: 20,
            extends: None,
            includes: None,
            imports: None,
            condition: None,
            is_dependency: false,
            required_by: None,
        }];

        let variables = Variables::from_json(r#"{"language": "go", "ext": "go"}"#).unwrap();
        let result = render_tree(files, &variables).unwrap();

        assert_eq!(result[0].file_path, "src/go/main.go");
        assert_eq!(result[0].file_name, "main.go");
    }

    #[test]
    fn test_template_inheritance() {
        // 测试模板继承功能
        let files = vec![
            TemplateFile {
                id: 1,
                file_path: "base.html".to_string(),
                file_name: "base.html".to_string(),
                file_content: r#"<html>
<head>
    <title>{% block title %}默认标题{% endblock %}</title>
</head>
<body>
    {% block content %}{% endblock %}
</body>
</html>"#
                    .to_string(),
                is_directory: 0,
                parent_id: 0,
                filesize: 100,
                extends: None,
                includes: None,
                imports: None,
                condition: None,
                is_dependency: false,
                required_by: None,
            },
            TemplateFile {
                id: 2,
                file_path: "page.html".to_string(),
                file_name: "page.html".to_string(),
                file_content: r#"{% extends "base.html" %}

{% block title %}我的页面{% endblock %}

{% block content %}
    <h1>欢迎</h1>
{% endblock %}"#
                    .to_string(),
                is_directory: 0,
                parent_id: 0,
                filesize: 100,
                extends: None,
                includes: None,
                imports: None,
                condition: None,
                is_dependency: false,
                required_by: None,
            },
        ];

        let variables = Variables::from_json(r#"{}"#).unwrap();
        let result = render_tree(files, &variables).unwrap();

        // 应该渲染两个文件
        assert_eq!(result.len(), 2);

        // page.html 应该正确继承 base.html
        let page_html = result.iter().find(|f| f.file_name == "page.html").unwrap();
        assert!(page_html.file_content.is_some());
        let content = page_html.file_content.as_ref().unwrap();

        // 验证继承是否成功
        assert!(content.contains("<html>"));
        assert!(content.contains("<title>我的页面</title>"));
        assert!(content.contains("<h1>欢迎</h1>"));
        assert!(content.contains("</body>"));
        assert!(content.contains("</html>"));
    }
}
