//! 自定义过滤器（MiniJinja 版本）

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
        value.to_string().len()
    };
    Ok(Value::from(len as i64))
}

/// 字符串截断过滤器
fn truncate(value: &Value, args: &[Value]) -> Result<Value, MiniError> {
    let default_length = 50;
    let length = if !args.is_empty() && args[0].is_number() {
        args[0].as_usize().unwrap_or(default_length)
    } else {
        default_length
    };

    if value.is_undefined() || value.is_none() {
        return Ok(Value::from(""));
    }

    if let Some(s) = value.as_str() {
        if s.len() > length {
            return Ok(Value::from(format!("{}...", &s[..length])));
        } else {
            return Ok(value.clone());
        }
    }

    Ok(Value::from(value.to_string()))
}

/// 默认值过滤器
fn default_filter(value: &Value, args: &[Value]) -> Result<Value, MiniError> {
    if value.is_undefined() || value.is_none() {
        if !args.is_empty() {
            return Ok(args[0].clone());
        } else {
            return Ok(Value::from(""));
        }
    }

    Ok(value.clone())
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
            description: "截断字符串到指定长度".to_string(),
            example: r#"{{ long_text | truncate(10) }}"#.to_string(),
        },
        super::FilterInfo {
            name: "default".to_string(),
            description: "当值为空时使用默认值".to_string(),
            example: r#"{{ null | default("N/A") }}"#.to_string(),
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

    #[cfg(feature = "logging")]
    tracing::debug!("Registered {} custom filters", 11);
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
