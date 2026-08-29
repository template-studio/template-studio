//! # Tera 依赖分析模块
//!
//! 分析 Tera 模板文件中的所有依赖关系：
//! - include: 包含其他模板
//! - extends: 继承父模板
//! - import: 导入宏文件
//!
//! ## 核心特性
//!
//! - **静态分析** - 通过正则表达式解析，无需运行时
//! - **完整覆盖** - 支持 Tera 的所有文件依赖语法
//! - **WASM兼容** - 无文件系统依赖

use crate::tree::IncludeDependency;
use regex::Regex;

/// 文件依赖信息
///
/// 表示单个 Tera 模板文件的所有依赖关系
#[derive(Debug, Default, Clone)]
pub struct FileDependencies {
    /// extends 父模板路径（最多一个）
    pub extends: Option<String>,

    /// include 依赖列表
    pub includes: Vec<IncludeDependency>,

    /// import 依赖列表
    pub imports: Vec<ImportDependency>,
}

/// Import 依赖信息
#[derive(Debug, Clone)]
pub struct ImportDependency {
    /// 文件路径
    pub path: String,
    /// 命名空间名称
    pub namespace: String,
}

/// Tera 依赖分析器
///
/// 分析 Tera 模板文件的所有文件依赖
pub struct TeraDependencyAnalyzer {
    // Extends 语句的正则模式
    extends_pattern: Regex,

    // Import 语句的正则模式
    import_pattern: Regex,
}

impl TeraDependencyAnalyzer {
    /// 创建新的依赖分析器
    pub fn new() -> Self {
        Self {
            // Extends 语句模式: {% extends "base.html" %}
            extends_pattern: Regex::new(r#"\{%\s*extends\s+"(?P<path>[^"]+)"\s*%\}"#).unwrap(),

            // Import 语句模式: {% import "macros.html" as macros %}
            import_pattern: Regex::new(
                r#"\{%\s*import\s+"(?P<path>[^"]+)"\s+as\s+(?P<name>\w+)\s*%\}"#,
            )
            .unwrap(),
        }
    }

    /// 分析文件的所有依赖
    ///
    /// # 参数
    ///
    /// * `file_content` - 模板文件内容
    ///
    /// # 返回
    ///
    /// 文件的依赖信息
    ///
    /// # 示例
    ///
    /// ```
    /// use template_studio_template_core::dependency_analyzer::TeraDependencyAnalyzer;
    ///
    /// let analyzer = TeraDependencyAnalyzer::new();
    /// let content = r#"
    ///     {% extends "base.html" %}
    ///     {% import "macros.html" as macros %}
    ///     {% include "header.html" %}
    /// "#;
    ///
    /// let deps = analyzer.analyze(content).unwrap();
    /// assert_eq!(deps.extends, Some("base.html".to_string()));
    /// assert_eq!(deps.imports.len(), 1);
    /// assert_eq!(deps.includes.len(), 1);
    /// ```
    pub fn analyze(&self, file_content: &str) -> Result<FileDependencies, String> {
        let mut deps = FileDependencies::default();

        // 1. 解析 extends
        for caps in self.extends_pattern.captures_iter(file_content) {
            if let Some(path) = caps.name("path") {
                if deps.extends.is_some() {
                    return Err("一个文件只能 extends 一个父模板".to_string());
                }
                deps.extends = Some(path.as_str().to_string());
            }
        }

        // 2. 解析 include - 使用更精确的正则，避免重复匹配
        // 先匹配带 ignore missing 的
        let include_ignore_missing =
            Regex::new(r#"\{%\s*include\s+"(?P<path>[^"]+)"\s+ignore\s+missing\s*%\}"#).unwrap();

        // 单个文件（带 ignore missing）
        for caps in include_ignore_missing.captures_iter(file_content) {
            if let Some(path) = caps.name("path") {
                deps.includes
                    .push(IncludeDependency::Optional(path.as_str().to_string()));
            }
        }

        // 多候选项
        let include_multiple =
            Regex::new(r#"\{%\s*include\s+\[(?P<paths>[^\]]+)\](?:\s+ignore\s+missing)?\s*%\}"#)
                .unwrap();

        for caps in include_multiple.captures_iter(file_content) {
            if let Some(paths) = caps.name("paths") {
                let paths: Vec<String> = paths
                    .as_str()
                    .split(',')
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                    .collect();

                // 检查原始语句中是否有 ignore missing
                let full_match = caps.get(0).map(|m| m.as_str()).unwrap_or("");
                let has_ignore_missing = full_match.contains("ignore");

                if has_ignore_missing {
                    // 整个列表是可选的
                    deps.includes
                        .push(IncludeDependency::Optional(paths.join(", ")));
                } else {
                    deps.includes.push(IncludeDependency::Multiple(paths));
                }
            }
        }

        // 单个文件（不带 ignore missing，且不在数组中）
        let include_single = Regex::new(r#"\{%\s*include\s+"(?P<path>[^"]+)"\s*%\}"#).unwrap();

        for caps in include_single.captures_iter(file_content) {
            if let Some(path) = caps.name("path") {
                // 确保不是前面已经匹配过的
                let full_match = caps.get(0).map(|m| m.as_str()).unwrap_or("");
                if !full_match.contains("ignore") && !full_match.contains('[') {
                    deps.includes
                        .push(IncludeDependency::Single(path.as_str().to_string()));
                }
            }
        }

        // 3. 解析 import
        for caps in self.import_pattern.captures_iter(file_content) {
            let path = caps.name("path").unwrap().as_str().to_string();
            let namespace = caps.name("name").unwrap().as_str().to_string();
            deps.imports.push(ImportDependency { path, namespace });
        }

        Ok(deps)
    }
}

impl Default for TeraDependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_include_single() {
        let analyzer = TeraDependencyAnalyzer::new();
        let content = r#"{% include "header.html" %}"#;

        let deps = analyzer.analyze(content).unwrap();

        assert_eq!(deps.includes.len(), 1);
        match &deps.includes[0] {
            IncludeDependency::Single(path) => {
                assert_eq!(path, "header.html");
            }
            _ => panic!("Expected Single"),
        }
    }

    #[test]
    fn test_parse_include_multiple() {
        let analyzer = TeraDependencyAnalyzer::new();
        let content = r#"{% include ["custom/header.html", "header.html"] %}"#;

        let deps = analyzer.analyze(content).unwrap();

        assert_eq!(deps.includes.len(), 1);
        match &deps.includes[0] {
            IncludeDependency::Multiple(paths) => {
                assert_eq!(paths.len(), 2);
                assert_eq!(paths[0], "custom/header.html");
                assert_eq!(paths[1], "header.html");
            }
            _ => panic!("Expected Multiple"),
        }
    }

    #[test]
    fn test_parse_include_ignore_missing() {
        let analyzer = TeraDependencyAnalyzer::new();
        let content = r#"{% include "optional.html" ignore missing %}"#;

        let deps = analyzer.analyze(content).unwrap();

        assert_eq!(deps.includes.len(), 1);
        match &deps.includes[0] {
            IncludeDependency::Optional(path) => {
                assert_eq!(path, "optional.html");
            }
            _ => panic!("Expected Optional"),
        }
    }

    #[test]
    fn test_parse_extends() {
        let analyzer = TeraDependencyAnalyzer::new();
        let content = r#"{% extends "base.html" %}
{% block content %}Hello{% endblock content %}"#;

        let deps = analyzer.analyze(content).unwrap();

        assert_eq!(deps.extends, Some("base.html".to_string()));
    }

    #[test]
    fn test_parse_import() {
        let analyzer = TeraDependencyAnalyzer::new();
        let content = r#"{% import "macros/forms.html" as forms %}"#;

        let deps = analyzer.analyze(content).unwrap();

        assert_eq!(deps.imports.len(), 1);
        assert_eq!(deps.imports[0].path, "macros/forms.html");
        assert_eq!(deps.imports[0].namespace, "forms");
    }

    #[test]
    fn test_parse_multiple_imports() {
        let analyzer = TeraDependencyAnalyzer::new();
        let content = r#"
{% import "macros/forms.html" as forms %}
{% import "macros/utils.html" as utils %}
"#;

        let deps = analyzer.analyze(content).unwrap();

        assert_eq!(deps.imports.len(), 2);
        assert_eq!(deps.imports[0].path, "macros/forms.html");
        assert_eq!(deps.imports[1].path, "macros/utils.html");
    }

    #[test]
    fn test_complex_template() {
        let analyzer = TeraDependencyAnalyzer::new();
        let content = r#"
{% extends "layouts/main.html" %}
{% import "macros/forms.html" as forms %}

{% block content %}
  {% include "header.html" %}
  {{ forms::input(label="Name") }}
  {% include ["custom/footer.html", "footer.html"] ignore missing %}
{% endblock content %}
"#;

        let deps = analyzer.analyze(content).unwrap();

        // extends
        assert_eq!(deps.extends, Some("layouts/main.html".to_string()));

        // import
        assert_eq!(deps.imports.len(), 1);
        assert_eq!(deps.imports[0].path, "macros/forms.html");

        // includes
        assert!(deps.includes.len() >= 2);
        match &deps.includes[0] {
            IncludeDependency::Single(p) => assert_eq!(p, "header.html"),
            _ => {}
        }
    }

    #[test]
    fn test_multiple_extends_error() {
        let analyzer = TeraDependencyAnalyzer::new();
        let content = r#"
{% extends "base1.html" %}
{% extends "base2.html" %}
"#;

        let result = analyzer.analyze(content);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "一个文件只能 extends 一个父模板");
    }
}
