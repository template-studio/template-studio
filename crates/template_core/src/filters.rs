//! 自定义过滤器（MiniJinja 版本）

use minijinja::value::Kwargs;
use minijinja::{Error as MiniError, ErrorKind, Value};

/// Base64 编码过滤器
fn base64_encode(value: &Value, _args: &[Value]) -> Result<Value, MiniError> {
    use base64::{engine::general_purpose, Engine as _};

    // 检查是否为 undefined
    if value.is_undefined() || value.is_none() {
        return Ok(Value::from(""));
    }

    // 使用 as_str() 获取字符串
    let s = if let Some(str_val) = value.as_str() {
        str_val
    } else {
        // 如果不是字符串，转换为字符串
        &value.to_string()
    };

    let encoded = general_purpose::STANDARD.encode(s.as_bytes());
    Ok(Value::from(encoded))
}

/// Base64 解码过滤器
fn base64_decode(value: &Value, _args: &[Value]) -> Result<Value, MiniError> {
    use base64::{engine::general_purpose, Engine as _};

    if value.is_undefined() || value.is_none() {
        return Ok(Value::from(""));
    }

    // 只处理字符串类型
    if let Some(s) = value.as_str() {
        match general_purpose::STANDARD.decode(s) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(decoded) => Ok(Value::from(decoded)),
                Err(_) => Ok(Value::from(format!("<binary data: {} bytes>", s.len()))),
            },
            Err(e) => Ok(Value::from(format!("<base64 decode error: {}>", e))),
        }
    } else {
        Ok(Value::from("<unsupported type>"))
    }
}

/// JSON 序列化过滤器
fn json_encode(value: &Value, _args: &[Value]) -> Result<Value, MiniError> {
    // 将 MiniJinja Value 转回 serde_json::Value
    let json_value = convert_minijinja_to_json(value)?;
    match serde_json::to_string_pretty(&json_value) {
        Ok(json) => Ok(Value::from(json)),
        Err(e) => Ok(Value::from(format!("<json encode error: {}>", e))),
    }
}

/// URL 编码过滤器
fn url_encode(value: &Value, _args: &[Value]) -> Result<Value, MiniError> {
    if value.is_undefined() || value.is_none() {
        return Ok(Value::from(""));
    }

    let s = value.to_string();
    let encoded =
        percent_encoding::utf8_percent_encode(&s, percent_encoding::NON_ALPHANUMERIC).to_string();
    Ok(Value::from(encoded))
}

/// 字符串转大写过滤器
fn uppercase(value: &Value, _args: &[Value]) -> Result<Value, MiniError> {
    if value.is_undefined() || value.is_none() {
        return Ok(Value::from(""));
    }

    let s = value.to_string();
    Ok(Value::from(s.to_uppercase()))
}

/// 字符串转小写过滤器
fn lowercase(value: &Value, _args: &[Value]) -> Result<Value, MiniError> {
    if value.is_undefined() || value.is_none() {
        return Ok(Value::from(""));
    }

    let s = value.to_string();
    Ok(Value::from(s.to_lowercase()))
}

/// 字符串反转过滤器
fn reverse(value: &Value, _args: &[Value]) -> Result<Value, MiniError> {
    if value.is_undefined() || value.is_none() {
        return Ok(Value::from(""));
    }

    // 处理字符串
    if let Some(s) = value.as_str() {
        let reversed: String = s.chars().rev().collect();
        return Ok(Value::from(reversed));
    }

    // 处理可迭代对象
    if let Ok(iter) = value.try_iter() {
        let items: Vec<Value> = iter.collect();
        let reversed: Vec<Value> = items.into_iter().rev().collect();
        return Ok(Value::from(reversed));
    }

    Ok(Value::from(value.to_string()))
}

/// 获取长度过滤器
fn length(value: &Value, _args: &[Value]) -> Result<Value, MiniError> {
    let len = if value.is_undefined() || value.is_none() {
        0
    } else if let Some(len_val) = value.len() {
        len_val
    } else {
        // 无法直接取长度的值按字符数计（按字节计会把中文等多字节字符算多）
        value.to_string().chars().count()
    };
    Ok(Value::from(len as i64))
}

/// 字符串截断过滤器
///
/// 支持位置与关键字两种传参：`{{ s | truncate(5) }}`、`{{ s | truncate(length=5) }}`、
/// `{{ s | truncate(length=5, end="…") }}`。
/// 按字符截断，多字节字符（中文等）不会因字节边界切片而 panic。
fn truncate(value: &Value, length: Option<usize>, kwargs: Kwargs) -> Result<Value, MiniError> {
    let _ = &kwargs; // 兼容旧版 minijinja：无 assert_unused，未知关键字参数静默忽略
    if value.is_undefined() || value.is_none() {
        return Ok(Value::from(""));
    }

    let Some(s) = value.as_str() else {
        return Ok(Value::from(value.to_string()));
    };

    // 位置参数与关键字参数（length=5）都支持；此版本 minijinja 的 kwarg 不自动绑定同名参数
    let length_kw: Option<usize> = kwargs.get("length").ok().flatten();
    let max = length.or(length_kw).unwrap_or(50);
    let end_str: String = kwargs
        .get("end")
        .ok()
        .flatten()
        .unwrap_or_else(|| "...".to_string());

    if s.chars().count() <= max {
        return Ok(value.clone());
    }
    let truncated: String = s.chars().take(max).collect();
    Ok(Value::from(format!("{}{}", truncated, end_str)))
}

/// 默认值过滤器
///
/// 与 Jinja2 语义对齐：默认仅在值为 undefined/none 时启用默认值；
/// `boolean=true` 时对一切 falsy 值（空串、0、false）也启用默认值。
fn default_filter(
    value: &Value,
    default_value: Option<Value>,
    boolean: Option<bool>,
    kwargs: Kwargs,
) -> Result<Value, MiniError> {
    let _ = &kwargs; // 兼容旧版 minijinja：无 assert_unused，未知关键字参数静默忽略
    let use_default =
        value.is_undefined() || value.is_none() || (boolean.unwrap_or(false) && !value.is_true());

    if use_default {
        Ok(default_value.unwrap_or_else(|| Value::from("")))
    } else {
        Ok(value.clone())
    }
}

/// 当前时间 Unix 时间戳（秒）——注册为函数 `{{ now() }}`
fn now_fn(_args: &[Value]) -> Result<Value, MiniError> {
    Ok(Value::from(chrono::Utc::now().timestamp()))
}

/// 格式化日期过滤器：`{{ timestamp | date }}` 或 `{{ timestamp | date(format="%Y-%m-%d") }}`
fn date_filter(value: &Value, format: Option<String>, kwargs: Kwargs) -> Result<Value, MiniError> {
    let _ = &kwargs; // 兼容旧版 minijinja：无 assert_unused，未知关键字参数静默忽略
    let ts = i64::try_from(value.clone()).map_err(|_| {
        MiniError::new(
            ErrorKind::InvalidOperation,
            "date 过滤器需要 Unix 时间戳（秒）",
        )
    })?;
    let dt = chrono::DateTime::from_timestamp(ts, 0).ok_or_else(|| {
        MiniError::new(ErrorKind::InvalidOperation, format!("无效的时间戳: {}", ts))
    })?;
    let format_kw: Option<String> = kwargs.get("format").ok().flatten();
    let fmt = format
        .or(format_kw)
        .unwrap_or_else(|| "%Y-%m-%d %H:%M:%S".to_string());
    // 按服务器本地时区渲染（面向中文用户的产品语义）
    Ok(Value::from(
        dt.with_timezone(&chrono::Local).format(&fmt).to_string(),
    ))
}

/// 数字千位分隔过滤器：`{{ n | number_format }}` 或 `{{ n | number_format(decimals=0) }}`
fn number_format_filter(
    value: &Value,
    decimals: Option<usize>,
    kwargs: Kwargs,
) -> Result<Value, MiniError> {
    let _ = &kwargs; // 兼容旧版 minijinja：无 assert_unused，未知关键字参数静默忽略
    let n = f64::try_from(value.clone())
        .map_err(|_| MiniError::new(ErrorKind::InvalidOperation, "number_format 需要数字"))?;
    let decimals_kw: Option<usize> = kwargs.get("decimals").ok().flatten();
    let d = decimals.or(decimals_kw).unwrap_or(2);
    let formatted = format!("{:.*}", d, n);
    // 手动加千位分隔符（整数部分每三位一个逗号）
    let (int_part, frac) = match formatted.split_once('.') {
        Some((i, f)) => (i.to_string(), Some(f.to_string())),
        None => (formatted, None),
    };
    let negative = int_part.starts_with('-');
    let digits = int_part.trim_start_matches('-');
    let mut grouped = String::new();
    for (idx, ch) in digits.chars().enumerate() {
        if idx > 0 && (digits.len() - idx) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    let mut result = String::new();
    if negative {
        result.push('-');
    }
    result.push_str(&grouped);
    if let Some(f) = frac {
        result.push('.');
        result.push_str(&f);
    }
    Ok(Value::from(result))
}

/// URL 友好化过滤器：`{{ title | slugify }}`——小写、非字母数字折叠为连字符
fn slugify_filter(value: &Value, _args: &[Value]) -> Result<Value, MiniError> {
    let s = if let Some(str_val) = value.as_str() {
        str_val
    } else {
        &value.to_string()
    };
    let mut out = String::with_capacity(s.len());
    let mut last_dash = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_matches('-');
    Ok(Value::from(trimmed))
}

/// 获取所有过滤器信息
pub fn get_all_filters() -> Vec<super::FilterInfo> {
    vec![
        super::FilterInfo {
            name: "base64_encode".to_string(),
            description: "将字符串编码为 Base64 格式".to_string(),
            example: r#"{{ "hello" | base64_encode }} → "aGVsbG8=""#.to_string(),
        },
        super::FilterInfo {
            name: "base64_decode".to_string(),
            description: "解码 Base64 格式的字符串".to_string(),
            example: r#"{{ "aGVsbG8=" | base64_decode }} → "hello""#.to_string(),
        },
        super::FilterInfo {
            name: "json_encode".to_string(),
            description: "将值序列化为格式化的 JSON 字符串".to_string(),
            example: r#"{{ obj | json_encode }}"#.to_string(),
        },
        super::FilterInfo {
            name: "url_encode".to_string(),
            description: "URL 编码字符串".to_string(),
            example: r#"{{ "hello world" | url_encode }} → "hello%20world""#.to_string(),
        },
        super::FilterInfo {
            name: "uppercase".to_string(),
            description: "转换为大写".to_string(),
            example: r#"{{ "hello" | uppercase }} → "HELLO""#.to_string(),
        },
        super::FilterInfo {
            name: "lowercase".to_string(),
            description: "转换为小写".to_string(),
            example: r#"{{ "HELLO" | lowercase }} → "hello""#.to_string(),
        },
        super::FilterInfo {
            name: "reverse".to_string(),
            description: "反转字符串或数组".to_string(),
            example: r#"{{ "hello" | reverse }} → "olleh""#.to_string(),
        },
        super::FilterInfo {
            name: "length".to_string(),
            description: "获取字符串或数组长度".to_string(),
            example: r#"{{ "hello" | length }} → 5"#.to_string(),
        },
        super::FilterInfo {
            name: "truncate".to_string(),
            description: "截断字符串到指定字符数（支持 length/end 关键字参数，多字节安全）"
                .to_string(),
            example: r#"{{ long_text | truncate(length=10) }}"#.to_string(),
        },
        super::FilterInfo {
            name: "default".to_string(),
            description: "当值为空时使用默认值；default(\"x\", true) 对空串/0/false 也生效"
                .to_string(),
            example: r#"{{ "" | default("N/A", true) }}"#.to_string(),
        },
        super::FilterInfo {
            name: "date".to_string(),
            description: "将 Unix 时间戳（秒）格式化为日期时间字符串".to_string(),
            example: r#"{{ now() | date(format="%Y-%m-%d") }}"#.to_string(),
        },
        super::FilterInfo {
            name: "number_format".to_string(),
            description: "数字添加千位分隔符（可选 decimals 指定小数位）".to_string(),
            example: r#"{{ 1234567.891 | number_format(decimals=2) }} → 1,234,567.89"#.to_string(),
        },
        super::FilterInfo {
            name: "slugify".to_string(),
            description: "转换为 URL 友好格式：小写、非字母数字折叠为连字符".to_string(),
            example: r#"{{ "Hello World! 你好" | slugify }}"#.to_string(),
        },
    ]
}

/// 注册所有自定义过滤器到 MiniJinja
pub(super) fn register_all_filters(env: &mut minijinja::Environment<'_>) {
    env.add_filter("base64_encode", base64_encode);
    env.add_filter("base64_decode", base64_decode);
    env.add_filter("json_encode", json_encode);
    env.add_filter("url_encode", url_encode);
    env.add_filter("upper", uppercase); // MiniJinja 内置的是 upper，但我们还是注册
    env.add_filter("uppercase", uppercase); // 兼容 Tera
    env.add_filter("lower", lowercase);
    env.add_filter("lowercase", lowercase); // 兼容 Tera
    env.add_filter("reverse", reverse);
    env.add_filter("length", length);
    env.add_filter("truncate", truncate);
    env.add_filter("default", default_filter);
    env.add_filter("date", date_filter);
    env.add_filter("number_format", number_format_filter);
    env.add_filter("slugify", slugify_filter);
    env.add_function("now", now_fn);

    #[cfg(feature = "logging")]
    tracing::debug!("Registered custom filters and functions");
}

/// 辅助函数：将 MiniJinja Value 转换为 serde_json::Value
fn convert_minijinja_to_json(value: &Value) -> Result<serde_json::Value, MiniError> {
    // 处理 undefined 和 none
    if value.is_undefined() || value.is_none() {
        return Ok(serde_json::Value::Null);
    }

    // 处理数字（使用 TryFrom）
    if value.is_number() {
        // 尝试转换为各种数值类型
        if let Ok(b) = bool::try_from(value.clone()) {
            return Ok(serde_json::Value::Bool(b));
        }
        if let Ok(i) = i64::try_from(value.clone()) {
            return Ok(serde_json::Value::Number(serde_json::Number::from(i)));
        }
        if let Ok(u) = u64::try_from(value.clone()) {
            return Ok(serde_json::Value::Number(serde_json::Number::from(u)));
        }
        if let Ok(f) = f64::try_from(value.clone()) {
            if let Some(n) = serde_json::Number::from_f64(f) {
                return Ok(serde_json::Value::Number(n));
            }
        }
        return Err(MiniError::new(
            ErrorKind::BadSerialization,
            "Cannot convert number",
        ));
    }

    // 处理字符串
    if let Some(s) = value.as_str() {
        return Ok(serde_json::Value::String(s.to_string()));
    }

    // 处理数组/序列
    if let Ok(iter) = value.try_iter() {
        let vec: Result<Vec<_>, _> = iter.map(|v| convert_minijinja_to_json(&v)).collect();
        return Ok(serde_json::Value::Array(vec?));
    }

    // 处理对象
    if let Some(_obj) = value.as_object() {
        let mut map = serde_json::Map::new();
        if let Ok(iter) = value.try_iter() {
            for key_result in iter {
                if let Ok(val) = value.get_item(&key_result) {
                    if let Some(key_str) = key_result.as_str() {
                        map.insert(key_str.to_string(), convert_minijinja_to_json(&val)?);
                    }
                }
            }
        }
        return Ok(serde_json::Value::Object(map));
    }

    // 尝试通过序列化转换其他类型
    match serde_json::to_value(value.to_string()) {
        Ok(v) => Ok(v),
        Err(_) => Err(MiniError::new(
            ErrorKind::BadSerialization,
            "Cannot convert value",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minijinja::Environment;

    fn test_env() -> Environment<'static> {
        let mut env = Environment::new();
        register_all_filters(&mut env);
        env
    }

    fn render(env: &Environment, template: &str) -> String {
        env.render_str(template, minijinja::context! {}).unwrap()
    }

    #[test]
    fn test_truncate_named_and_positional() {
        let env = test_env();
        // 关键字参数（此前实测无效的形态）
        assert_eq!(
            render(&env, r#"{{ "hello world" | truncate(length=5) }}"#),
            "hello..."
        );
        // 位置参数
        assert_eq!(
            render(&env, r#"{{ "hello world" | truncate(5) }}"#),
            "hello..."
        );
        // 不超长原样返回
        assert_eq!(render(&env, r#"{{ "hi" | truncate(5) }}"#), "hi");
        // 自定义结尾
        assert_eq!(
            render(&env, r#"{{ "hello world" | truncate(length=5, end="…") }}"#),
            "hello…"
        );
    }

    #[test]
    fn test_truncate_multibyte_safe() {
        let env = test_env();
        // 中文按字符截断，不因字节边界 panic
        assert_eq!(
            render(&env, r#"{{ "你好世界模板" | truncate(length=3) }}"#),
            "你好世..."
        );
    }

    #[test]
    fn test_default_boolean_semantics() {
        let env = test_env();
        // 空串默认不替换
        assert_eq!(render(&env, r#"{{ "" | default("x") }}"#), "");
        // boolean=true 对空串生效（此前实测返回空串的 bug）
        assert_eq!(render(&env, r#"{{ "" | default("x", true) }}"#), "x");
        // 对 0 生效
        assert_eq!(render(&env, r#"{{ 0 | default("x", true) }}"#), "x");
        // 非空值不受影响
        assert_eq!(render(&env, r#"{{ "v" | default("x", true) }}"#), "v");
    }

    #[test]
    fn test_length_multibyte() {
        let env = test_env();
        assert_eq!(render(&env, r#"{{ "你好" | length }}"#), "2");
    }

    #[test]
    fn test_now_and_date() {
        let env = test_env();
        let now: i64 = render(&env, "{{ now() }}").parse().unwrap();
        assert!(now > 1_700_000_000);
        // date 过滤器按格式化输出（1753920000 = 2025-07-31T00:00:00Z，
        // 本地时区 UTC..+8 内日期一致，避免测试受时区环境影响）
        assert_eq!(
            render(&env, r#"{{ 1753920000 | date(format="%Y-%m-%d") }}"#),
            "2025-07-31"
        );
        let default_fmt = render(&env, r#"{{ 1753920000 | date }}"#);
        assert!(
            default_fmt.starts_with("2025-07-31"),
            "默认格式应以日期开头: {}",
            default_fmt
        );
    }

    #[test]
    fn test_number_format() {
        let env = test_env();
        assert_eq!(
            render(&env, r#"{{ 1234567.891 | number_format(decimals=2) }}"#),
            "1,234,567.89"
        );
        assert_eq!(
            render(&env, r#"{{ 1234567 | number_format(decimals=0) }}"#),
            "1,234,567"
        );
        assert_eq!(
            render(&env, r#"{{ -1234567.5 | number_format(decimals=1) }}"#),
            "-1,234,567.5"
        );
    }

    #[test]
    fn test_slugify() {
        let env = test_env();
        assert_eq!(
            render(&env, r#"{{ "Hello World!" | slugify }}"#),
            "hello-world"
        );
        // 中文字符折叠为单个连字符
        assert_eq!(render(&env, r#"{{ "你好 World" | slugify }}"#), "world");
        assert_eq!(
            render(&env, r#"{{ "  --Multiple   Spaces--  " | slugify }}"#),
            "multiple-spaces"
        );
    }
}
