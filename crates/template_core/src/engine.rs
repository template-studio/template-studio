//! MiniJinja 引擎封装和核心渲染逻辑

use minijinja::{Environment, Error as MiniError, ErrorKind, Value};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, RwLock};

use crate::filters;
use crate::types::{RenderError, RenderResult, Variables};

/// 继承渲染环境缓存容量上限（每个环境包含整棵模板树的已编译模板）
const ENV_CACHE_CAPACITY: usize = 32;

/// 全局 MiniJinja 环境实例（包含过滤器）
pub(crate) static GLOBAL_ENV: Lazy<RwLock<Environment<'static>>> = Lazy::new(|| {
    let mut env = Environment::new();

    // 注册所有自定义过滤器
    filters::register_all_filters(&mut env);

    // 按模板名扩展名决定自动转义：HTML/XML 系转义，其余保持原样。
    // 主模板经 render_named_str 携带真实文件名；无名渲染（文件名/路径等
    // 内部渲染）收到空名不转义，行为与历史一致
    env.set_auto_escape_callback(|name: &str| {
        if name.ends_with(".html") || name.ends_with(".htm") || name.ends_with(".xml") {
            minijinja::AutoEscape::Html
        } else {
            minijinja::AutoEscape::None
        }
    });

    // 启用严格模式：未定义的变量会导致错误
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);

    // 去除模板标签产生的多余空行
    env.set_trim_blocks(true);
    env.set_lstrip_blocks(true);

    #[cfg(feature = "logging")]
    tracing::info!("MiniJinja engine initialized");

    RwLock::new(env)
});

/// 继承渲染环境缓存：模板集哈希 → 已编译好的 Environment
///
/// - key 是全部模板（名称+内容）的哈希，内容变化自然换 key，**天然自失效**，
///   无需与文件监听联动
/// - 命中后直接复用整棵已编译模板树，消除「每次渲染重建 Environment + 重注册过滤器」
///   的 CPU/内存放大（并行渲染下尤为明显）
/// - LRU 封顶防止内存无限增长（此前为只写不读的无上限 HashMap）
static ENV_CACHE: Lazy<Mutex<lru::LruCache<usize, Arc<Environment<'static>>>>> = Lazy::new(|| {
    Mutex::new(lru::LruCache::new(
        std::num::NonZeroUsize::new(ENV_CACHE_CAPACITY).unwrap(),
    ))
});

/// 清理模板缓存
///
/// 手动清理所有缓存的已编译环境。
pub fn clear_template_cache() {
    let mut cache = ENV_CACHE.lock().unwrap();
    let size = cache.len();

    #[cfg(feature = "logging")]
    tracing::info!("Clearing template env cache ({} entries)", size);

    cache.clear();
}

/// 获取缓存统计信息
///
/// 返回当前缓存中的环境数量
pub fn get_cache_size() -> usize {
    ENV_CACHE.lock().unwrap().len()
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
#[allow(clippy::result_large_err)]
pub fn render_string(
    template_content: &str,
    variables: &Variables,
    all_templates: Option<&std::collections::HashMap<String, String>>,
) -> Result<RenderResult, RenderError> {
    // 无名渲染：自动转义回调收到空名，不做转义（文件名/路径等内部渲染与
    // WASM 单文件预览走此入口，保持既有行为）
    render_string_named("", template_content, variables, all_templates)
}

/// 带模板名的渲染：模板名（通常为文件相对路径或文件名）参与自动转义决策——
/// `.html`/`.htm`/`.xml` 结尾时 `{{ var }}` 输出自动 HTML 转义（`| safe` 可豁免），
/// 其余扩展名不转义。文件内容渲染应优先使用此入口；文件名/路径渲染用 render_string。
#[allow(clippy::result_large_err)]
pub fn render_string_named(
    template_name: &str,
    template_content: &str,
    variables: &Variables,
    all_templates: Option<&std::collections::HashMap<String, String>>,
) -> Result<RenderResult, RenderError> {
    let env = GLOBAL_ENV.read().unwrap();

    if let Some(templates) = all_templates {
        render_with_templates(&env, template_name, template_content, variables, templates)
    } else {
        render_simple(&env, template_name, template_content, variables)
    }
}

/// 简单渲染（不支持模板继承）
#[allow(clippy::result_large_err)]
fn render_simple(
    env: &Environment<'_>,
    template_name: &str,
    template_content: &str,
    variables: &Variables,
) -> Result<RenderResult, RenderError> {
    let context = convert_variables(variables);

    match env.render_named_str(template_name, template_content, &context) {
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
///
/// 模板集（名称+内容）哈希为缓存键，命中直接复用已编译好的 Environment；
/// 主模板不注册进环境，经 `render_str` 一次性渲染（extends/include 查找
/// 照常解析到环境中已注册的模板）。
#[allow(clippy::result_large_err)]
fn render_with_templates(
    _env: &Environment<'_>,
    template_name: &str,
    template_content: &str,
    variables: &Variables,
    templates: &HashMap<String, String>,
) -> Result<RenderResult, RenderError> {
    let cache_key = generate_cache_key(templates);

    // 取缓存环境；未命中则构建后放入（放置时再查一次避免并发重复构建）
    let env = {
        let mut cache = ENV_CACHE.lock().unwrap();
        if let Some(env) = cache.get(&cache_key) {
            Arc::clone(env)
        } else {
            drop(cache);

            let mut env = Environment::new();
            env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
            env.set_trim_blocks(true);
            env.set_lstrip_blocks(true);
            filters::register_all_filters(&mut env);

            // 按模板名扩展名决定自动转义：HTML/XML 系转义，其余保持原样。
            // 主模板经 render_named_str 携带真实文件名；无名渲染（文件名/路径等
            // 内部渲染）收到空名不转义，行为与历史一致
            env.set_auto_escape_callback(|name: &str| {
                if name.ends_with(".html") || name.ends_with(".htm") || name.ends_with(".xml") {
                    minijinja::AutoEscape::Html
                } else {
                    minijinja::AutoEscape::None
                }
            });

            // add_template 会借用源字符串，无法装入 'static 缓存环境；
            // 改用 loader 按名加载（minijinja 内部对加载结果做编译缓存），
            // 依赖模板的语法错误在首次被引用时以渲染错误形式暴露
            let owned_templates: HashMap<String, String> = templates.clone();
            env.set_loader(move |name| Ok(owned_templates.get(name).cloned()));

            let env = Arc::new(env);
            let mut cache = ENV_CACHE.lock().unwrap();
            cache.put(cache_key, Arc::clone(&env));
            env
        }
    };

    #[cfg(feature = "logging")]
    tracing::trace!("Inheritance render (env cache key: {})", cache_key);

    let context = convert_variables(variables);
    match env.render_named_str(template_name, template_content, &context) {
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
    fn test_env_cache_reuse_and_invalidation() {
        let templates: HashMap<String, String> = HashMap::from([(
            "base.html".to_string(),
            "<b>{% block c %}{% endblock %}</b>".to_string(),
        )]);
        let vars = Variables::from_json("{}").unwrap();

        // 首次渲染：构建环境并缓存
        let r1 = render_string(
            "{% extends \"base.html\" %}{% block c %}A{% endblock %}",
            &vars,
            Some(&templates),
        )
        .unwrap();
        assert_eq!(r1.content, "<b>A</b>");
        let size_after_first = get_cache_size();
        assert!(size_after_first >= 1);

        // 二次渲染：命中缓存，继承仍正确
        let r2 = render_string(
            "{% extends \"base.html\" %}{% block c %}B{% endblock %}",
            &vars,
            Some(&templates),
        )
        .unwrap();
        assert_eq!(r2.content, "<b>B</b>");
        assert_eq!(get_cache_size(), size_after_first, "命中缓存不应新增条目");

        // 模板内容变化：缓存键变化，新环境生效（自失效验证）
        let templates_v2: HashMap<String, String> = HashMap::from([(
            "base.html".to_string(),
            "<i>{% block c %}{% endblock %}</i>".to_string(),
        )]);
        let r3 = render_string(
            "{% extends \"base.html\" %}{% block c %}C{% endblock %}",
            &vars,
            Some(&templates_v2),
        )
        .unwrap();
        assert_eq!(r3.content, "<i>C</i>", "内容变化后应使用新环境");

        // 清空缓存
        clear_template_cache();
        assert_eq!(get_cache_size(), 0);
    }

    #[test]
    fn test_auto_escape_by_extension() {
        let vars = Variables::from_json(r#"{"v": "<b>&"}"#).unwrap();

        // HTML 系扩展名：自动转义
        let r = render_string_named("page.html", "{{ v }}", &vars, None).unwrap();
        assert_eq!(r.content, "&lt;b&gt;&amp;");

        // | safe 豁免
        let r = render_string_named("page.html", "{{ v | safe }}", &vars, None).unwrap();
        assert_eq!(r.content, "<b>&");

        // 非 HTML 扩展名：不转义
        let r = render_string_named("config.yml", "{{ v }}", &vars, None).unwrap();
        assert_eq!(r.content, "<b>&");

        // 无名渲染（render_string）：不转义（文件名/路径渲染与 WASM 单文件入口）
        let r = render_string("{{ v }}", &vars, None).unwrap();
        assert_eq!(r.content, "<b>&");
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
