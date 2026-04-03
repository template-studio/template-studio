//! MiniJinja 引擎封装和核心渲染逻辑

use once_cell::sync::Lazy;
use std::sync::RwLock;
use std::collections::HashMap;
use minijinja::{Environment, Error as MiniError, Value, ErrorKind};
use std::hash::{Hash, Hasher};

use crate::types::{RenderResult, RenderError, Variables};
use crate::filters;

/// 全局 MiniJinja 环境实例（包含过滤器）
pub(crate) static GLOBAL_ENV: Lazy<RwLock<Environment<'static>>> = Lazy::new(|| {
    let mut env = Environment::new();

    // 注册所有自定义过滤器
    filters::register_all_filters(&mut env);

    // 启用严格模式：未定义的变量会导致错误
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);

    // 去除模板标签产生的多余空行
    env.set_trim_blocks(true);
    env.set_lstrip_blocks(true);

    #[cfg(feature = "logging")]
    tracing::info!("MiniJinja engine initialized");

    RwLock::new(env)
});

/// 模板缓存：缓存已编译的模板
/// Key: 模板内容的哈希值
/// Value: 编译后的模板源码
static TEMPLATE_CACHE: Lazy<RwLock<HashMap<usize, String>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

/// 清理模板缓存
///
/// 手动清理所有缓存的模板。
pub fn clear_template_cache() {
    let mut cache = TEMPLATE_CACHE.write().unwrap();
    let size = cache.len();

    #[cfg(feature = "logging")]
    tracing::info!("Clearing template cache ({} entries)", size);

    cache.clear();
}

/// 获取缓存统计信息
///
/// 返回当前缓存中的模板数量
pub fn get_cache_size() -> usize {
    TEMPLATE_CACHE.read().unwrap().len()
}

/// 初始化引擎（可选，通常在应用启动时调用一次）
pub fn initialize_engine() {
    // GLOBAL_ENV 在首次访问时会自动初始化
    #[cfg(feature = "logging")]
    tracing::info!("Template core engine initialized");
}

/// 核心渲染函数 - 渲染模板字符串（底层 API）
///
/// 这是**底层渲染 API**，用于直接渲染模板字符串内容。
///
/// # 参数
///
/// * `template_content` - 模板内容字符串
/// * `variables` - 渲染变量
/// * `all_templates` - 可选的所有模板映射（用于模板继承）
///
/// # 返回
///
/// 返回 `RenderResult`，包含渲染后的内容和状态
pub fn render_string(
    template_content: &str,
    variables: &Variables,
    all_templates: Option<&std::collections::HashMap<String, String>>,
) -> Result<RenderResult, RenderError> {
    let env = GLOBAL_ENV.read().unwrap();

    // 如果提供了所有模板的映射，使用支持继承的渲染
    if let Some(templates) = all_templates {
        render_with_templates(&env, template_content, variables, templates)
    } else {
        render_simple(&env, template_content, variables)
    }
}

/// 简单渲染（不支持模板继承）
fn render_simple(
    _env: &Environment<'_>,
    template_content: &str,
    variables: &Variables,
) -> Result<RenderResult, RenderError> {
    // 直接使用全局环境（已经配置好过滤器和严格模式）
    let env = GLOBAL_ENV.read().unwrap();
    let context = convert_variables(variables);

    match env.render_str(template_content, &context) {
        Ok(content) => Ok(RenderResult {
            content,
            success: true,
            error: None,
            variables: variables.as_value().clone(),
        }),
        Err(e) => Ok(RenderResult {
            content: String::new(),
            success: false,
            error: Some(parse_minijinja_error(&e, template_content)),
            variables: variables.as_value().clone(),
        }),
    }
}

/// 支持模板继承的渲染
fn render_with_templates(
    _env: &Environment<'_>,
    template_content: &str,
    variables: &Variables,
    templates: &HashMap<String, String>,
) -> Result<RenderResult, RenderError> {
    // 生成缓存键
    let cache_key = generate_cache_key(templates);

    // 检查缓存
    {
        let cache = TEMPLATE_CACHE.read().unwrap();
        if let Some(_) = cache.get(&cache_key) {
            #[cfg(feature = "logging")]
            tracing::debug!("Template cache hit (hash: {})", cache_key);
        }
    }

    // 创建新环境（MiniJinja Environment 不支持直接 clone）
    let mut env = Environment::new();

    // 复制严格模式配置
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);

    // 去除模板标签产生的多余空行
    env.set_trim_blocks(true);
    env.set_lstrip_blocks(true);

    // 注册过滤器（这是必要的，因为每个 Environment 实例需要自己的过滤器）
    #[cfg(feature = "logging")]
    tracing::debug!("Registering filters for template inheritance environment");

    filters::register_all_filters(&mut env);

    // 加载所有模板
    for (name, content) in templates {
        if let Err(e) = env.add_template(name, content) {
            return Ok(RenderResult {
                content: String::new(),
                success: false,
                error: Some(RenderError {
                    error_type: "template_error".to_string(),
                    message: format!("加载模板 '{}' 失败: {}", name, e),
                    line: None,
                    column: None,
                    context: None,
                    suggestion: None,
                }),
                variables: variables.as_value().clone(),
            });
        }
    }

    // 添加主模板（使用特殊名称）
    let main_template_name = "__main__";
    if let Err(e) = env.add_template(main_template_name, template_content) {
        return Ok(RenderResult {
            content: String::new(),
            success: false,
            error: Some(parse_minijinja_error(&e, template_content)),
            variables: variables.as_value().clone(),
        });
    }

    // 写入缓存
    {
        let mut cache = TEMPLATE_CACHE.write().unwrap();
        cache.insert(cache_key, template_content.to_string());
    }

    // 渲染
    let context = convert_variables(variables);
    match env.get_template(main_template_name) {
        Ok(tpl) => {
            match tpl.render(&context) {
                Ok(content) => Ok(RenderResult {
                    content,
                    success: true,
                    error: None,
                    variables: variables.as_value().clone(),
                }),
                Err(e) => Ok(RenderResult {
                    content: String::new(),
                    success: false,
                    error: Some(parse_minijinja_error(&e, template_content)),
                    variables: variables.as_value().clone(),
                }),
            }
        }
        Err(e) => Ok(RenderResult {
            content: String::new(),
            success: false,
            error: Some(parse_minijinja_error(&e, template_content)),
            variables: variables.as_value().clone(),
        }),
    }
}

/// 生成缓存键（基于所有模板内容）
fn generate_cache_key(templates: &HashMap<String, String>) -> usize {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    let mut sorted_templates: Vec<_> = templates.iter().collect();
    sorted_templates.sort_by_key(|(k, _)| *k);

    for (name, content) in sorted_templates {
        name.hash(&mut hasher);
        content.hash(&mut hasher);
    }

    hasher.finish() as usize
}

/// 将 Variables 转换为 MiniJinja context
fn convert_variables(variables: &Variables) -> HashMap<String, Value> {
    let mut context = HashMap::new();

    if let Some(obj) = variables.as_value().as_object() {
        for (key, value) in obj {
            context.insert(key.clone(), convert_json_value(value));
        }
    }

    context
}

/// 将 JSON 值转换为 MiniJinja Value
fn convert_json_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::from(()),
        serde_json::Value::Bool(b) => Value::from(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::from(i)
            } else if let Some(f) = n.as_f64() {
                Value::from(f)
            } else {
                Value::from(n.to_string())
            }
        }
        serde_json::Value::String(s) => Value::from(s.as_str()),
        serde_json::Value::Array(arr) => {
            Value::from(arr.iter().map(convert_json_value).collect::<Vec<_>>())
        }
        serde_json::Value::Object(obj) => {
            let map: std::collections::HashMap<String, Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), convert_json_value(v)))
                .collect();
            Value::from(map)
        }
    }
}

/// 解析 MiniJinja 错误为 RenderError
fn parse_minijinja_error(error: &MiniError, template_content: &str) -> RenderError {
    let line = error.line();

    #[cfg(feature = "logging")]
    tracing::debug!("MiniJinja error at line {:?}: {}", line, error);

    RenderError {
        error_type: error_kind_to_string(&error.kind()),
        message: error.to_string(),
        line,
        column: None, // MiniJinja 不直接提供列号
        context: extract_error_context(template_content, line),
        suggestion: None,
    }
}

/// 提取错误上下文
fn extract_error_context(template_content: &str, line: Option<usize>) -> Option<String> {
    if let Some(line_num) = line {
        let lines: Vec<&str> = template_content.lines().collect();
        if line_num > 0 && line_num <= lines.len() {
            let context_line = lines[line_num - 1];
            return Some(format!("Line {}: {}", line_num, context_line));
        }
    }
    None
}

/// 将 MiniJinja 错误类型转换为字符串
fn error_kind_to_string(kind: &ErrorKind) -> String {
    match kind {
        ErrorKind::SyntaxError => "syntax_error".to_string(),
        ErrorKind::TemplateNotFound => "template_not_found".to_string(),
        ErrorKind::UndefinedError => "undefined_error".to_string(),
        ErrorKind::MissingArgument => "missing_argument".to_string(),
        ErrorKind::BadSerialization => "bad_serialization".to_string(),
        ErrorKind::InvalidOperation => "invalid_operation".to_string(),
        ErrorKind::UnknownMethod => "unknown_method".to_string(),
        _ => "unknown_error".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_render() {
        let template = "Hello {{ name }}!";
        let variables = Variables::from_json(r#"{"name": "World"}"#).unwrap();
        let result = render_string(template, &variables, None).unwrap();
        assert_eq!(result.content, "Hello World!");
        assert!(result.success);
    }

    #[test]
    fn test_render_with_variables() {
        let template = "{{ x }} + {{ y }} = {{ x + y }}";
        let variables = Variables::from_json(r#"{"x": 10, "y": 20}"#).unwrap();
        let result = render_string(template, &variables, None).unwrap();
        assert_eq!(result.content, "10 + 20 = 30");
    }

    #[test]
    fn test_render_error() {
        let template = "Hello {{ undefined_var }}!";
        let variables = Variables::from_json(r#"{}"#).unwrap();
        let result = render_string(template, &variables, None).unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
    }
}
