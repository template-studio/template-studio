//! 内置函数和全局函数定义
//! 参考 Sprig 函数定义 Tera 模板引擎的内置函数

use serde::{Deserialize, Serialize};

/// 内置函数分类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinFunctionCategory {
    pub name: String,
    pub description: String,
    pub functions: Vec<BuiltinFunction>,
}

/// 内置函数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinFunction {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub description: String,
    pub params: Vec<BuiltinFunctionParam>,
    #[serde(rename = "returnType")]
    pub return_type: String,
    pub category: String,
    pub example: String,
    pub usage: String,
    #[serde(rename = "insertText")]
    pub insert_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<String>>,
}

/// 函数参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinFunctionParam {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub required: bool,
    pub description: String,
}

/// 内置函数响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinFunctionsResponse {
    pub categories: Vec<BuiltinFunctionCategory>,
    pub total: usize,
}

/// 获取所有内置函数分类
pub fn get_builtin_function_categories() -> Vec<BuiltinFunctionCategory> {
    vec![
        build_string_functions(),
        build_list_functions(),
        build_date_functions(),
        build_math_functions(),
        build_encoding_functions(),
        build_type_conversion_functions(),
        build_default_functions(),
    ]
}

/// 获取内置函数响应（用于 API）
pub fn get_builtin_functions_response() -> BuiltinFunctionsResponse {
    let categories = get_builtin_function_categories();
    let total = categories.iter().map(|c| c.functions.len()).sum();

    BuiltinFunctionsResponse { categories, total }
}

/// 字符串函数
fn build_string_functions() -> BuiltinFunctionCategory {
    BuiltinFunctionCategory {
        name: "字符串函数".to_string(),
        description: "字符串操作和处理函数".to_string(),
        functions: vec![
            BuiltinFunction {
                name: "upper".to_string(),
                display_name: "转大写".to_string(),
                description: "将字符串转换为大写字母".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "string".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "要转换的字符串".to_string(),
                }],
                return_type: "string".to_string(),
                category: "字符串函数".to_string(),
                example: "{{ name | upper }}".to_string(),
                usage: "upper 函数将整个字符串转换为大写字母。支持Unicode字符。".to_string(),
                insert_text: "{{ value | upper }}".to_string(),
                note: Some("Tera 内置过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "lower".to_string(),
                display_name: "转小写".to_string(),
                description: "将字符串转换为小写字母".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "string".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "要转换的字符串".to_string(),
                }],
                return_type: "string".to_string(),
                category: "字符串函数".to_string(),
                example: "{{ name | lower }}".to_string(),
                usage: "lower 函数将整个字符串转换为小写字母。支持Unicode字符。".to_string(),
                insert_text: "{{ value | lower }}".to_string(),
                note: Some("Tera 内置过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "trim".to_string(),
                display_name: "去除空格".to_string(),
                description: "去除字符串两端的空白字符".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "string".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "要处理的字符串".to_string(),
                }],
                return_type: "string".to_string(),
                category: "字符串函数".to_string(),
                example: "{{ text | trim }}".to_string(),
                usage: "trim 函数移除字符串两端的空白字符（空格、制表符、换行符等）。".to_string(),
                insert_text: "{{ value | trim }}".to_string(),
                note: Some("Tera 内置过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "truncate".to_string(),
                display_name: "截断字符串".to_string(),
                description: "将字符串截断到指定长度".to_string(),
                params: vec![
                    BuiltinFunctionParam {
                        name: "string".to_string(),
                        param_type: "string".to_string(),
                        required: true,
                        description: "要截断的字符串".to_string(),
                    },
                    BuiltinFunctionParam {
                        name: "length".to_string(),
                        param_type: "number".to_string(),
                        required: false,
                        description: "最大长度，默认为50".to_string(),
                    },
                ],
                return_type: "string".to_string(),
                category: "字符串函数".to_string(),
                example: "{{ content | truncate(length=100) }}".to_string(),
                usage: "truncate 函数将字符串截断到指定长度，超出部分用 ... 替代。".to_string(),
                insert_text: "{{ value | truncate(length=50) }}".to_string(),
                note: Some("template_core 自定义过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "replace".to_string(),
                display_name: "字符串替换".to_string(),
                description: "替换字符串中的指定内容".to_string(),
                params: vec![
                    BuiltinFunctionParam {
                        name: "string".to_string(),
                        param_type: "string".to_string(),
                        required: true,
                        description: "原字符串".to_string(),
                    },
                    BuiltinFunctionParam {
                        name: "from".to_string(),
                        param_type: "string".to_string(),
                        required: true,
                        description: "要查找的内容".to_string(),
                    },
                    BuiltinFunctionParam {
                        name: "to".to_string(),
                        param_type: "string".to_string(),
                        required: true,
                        description: "替换内容".to_string(),
                    },
                ],
                return_type: "string".to_string(),
                category: "字符串函数".to_string(),
                example: r#"{{ text | replace(from="foo", to="bar") }}"#.to_string(),
                usage: "replace 函数替换字符串中的所有匹配项。".to_string(),
                insert_text: r#"{{ value | replace(from="old", to="new") }}"#.to_string(),
                note: Some("Tera 内置过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "slugify".to_string(),
                display_name: "URL友好化".to_string(),
                description: "将字符串转换为URL友好的格式".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "string".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "要转换的字符串".to_string(),
                }],
                return_type: "string".to_string(),
                category: "字符串函数".to_string(),
                example: "{{ title | slugify }}".to_string(),
                usage: "slugify 函数将字符串转换为URL友好的格式，空格替换为连字符，移除特殊字符。"
                    .to_string(),
                insert_text: "{{ value | slugify }}".to_string(),
                note: Some("需要注册 tera_text_filters".to_string()),
                aliases: None,
            },
        ],
    }
}

/// 列表函数
fn build_list_functions() -> BuiltinFunctionCategory {
    BuiltinFunctionCategory {
        name: "列表函数".to_string(),
        description: "列表和数组操作函数".to_string(),
        functions: vec![
            BuiltinFunction {
                name: "length".to_string(),
                display_name: "获取长度".to_string(),
                description: "获取数组、字符串或对象的长度".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "value".to_string(),
                    param_type: "array|string|object".to_string(),
                    required: true,
                    description: "要获取长度的对象".to_string(),
                }],
                return_type: "number".to_string(),
                category: "列表函数".to_string(),
                example: "{{ items | length }}".to_string(),
                usage: "length 函数获取数组、字符串或其他可迭代对象的长度。".to_string(),
                insert_text: "{{ value | length }}".to_string(),
                note: Some("template_core 自定义过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "first".to_string(),
                display_name: "获取首元素".to_string(),
                description: "获取数组或列表的第一个元素".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "array".to_string(),
                    param_type: "array".to_string(),
                    required: true,
                    description: "要操作的数组".to_string(),
                }],
                return_type: "any".to_string(),
                category: "列表函数".to_string(),
                example: "{% for item in items | first %}{{ item }}{% endfor %}".to_string(),
                usage: "first 函数获取数组或列表的第一个元素。".to_string(),
                insert_text: "{{ value | first }}".to_string(),
                note: Some("Tera 内置过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "last".to_string(),
                display_name: "获取末元素".to_string(),
                description: "获取数组或列表的最后一个元素".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "array".to_string(),
                    param_type: "array".to_string(),
                    required: true,
                    description: "要操作的数组".to_string(),
                }],
                return_type: "any".to_string(),
                category: "列表函数".to_string(),
                example: "{% for item in items | last %}{{ item }}{% endfor %}".to_string(),
                usage: "last 函数获取数组或列表的最后一个元素。".to_string(),
                insert_text: "{{ value | last }}".to_string(),
                note: Some("Tera 内置过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "join".to_string(),
                display_name: "连接元素".to_string(),
                description: "将数组元素用指定分隔符连接成字符串".to_string(),
                params: vec![
                    BuiltinFunctionParam {
                        name: "array".to_string(),
                        param_type: "array".to_string(),
                        required: true,
                        description: "要连接的数组".to_string(),
                    },
                    BuiltinFunctionParam {
                        name: "separator".to_string(),
                        param_type: "string".to_string(),
                        required: false,
                        description: "分隔符，默认为逗号".to_string(),
                    },
                ],
                return_type: "string".to_string(),
                category: "列表函数".to_string(),
                example: "{{ tags | join(separator=\", \") }}".to_string(),
                usage: "join 函数将数组元素用指定分隔符连接成字符串。".to_string(),
                insert_text: "{{ value | join(separator=\", \") }}".to_string(),
                note: Some("Tera 内置过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "sort".to_string(),
                display_name: "排序数组".to_string(),
                description: "对数组进行升序排序".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "array".to_string(),
                    param_type: "array".to_string(),
                    required: true,
                    description: "要排序的数组".to_string(),
                }],
                return_type: "array".to_string(),
                category: "列表函数".to_string(),
                example: "{% for item in items | sort %}{{ item }}{% endfor %}".to_string(),
                usage: "sort 函数对数组进行升序排序。".to_string(),
                insert_text: "{{ value | sort }}".to_string(),
                note: Some("Tera 内置过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "unique".to_string(),
                display_name: "去重".to_string(),
                description: "去除数组中的重复元素".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "array".to_string(),
                    param_type: "array".to_string(),
                    required: true,
                    description: "要去重的数组".to_string(),
                }],
                return_type: "array".to_string(),
                category: "列表函数".to_string(),
                example: "{% for item in items | unique %}{{ item }}{% endfor %}".to_string(),
                usage: "unique 函数去除数组中的重复元素。".to_string(),
                insert_text: "{{ value | unique }}".to_string(),
                note: Some("Tera 内置过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "slice".to_string(),
                display_name: "切片".to_string(),
                description: "获取数组的切片".to_string(),
                params: vec![
                    BuiltinFunctionParam {
                        name: "array".to_string(),
                        param_type: "array".to_string(),
                        required: true,
                        description: "要切片的数组".to_string(),
                    },
                    BuiltinFunctionParam {
                        name: "start".to_string(),
                        param_type: "number".to_string(),
                        required: true,
                        description: "起始索引".to_string(),
                    },
                    BuiltinFunctionParam {
                        name: "end".to_string(),
                        param_type: "number".to_string(),
                        required: false,
                        description: "结束索引（不包含）".to_string(),
                    },
                ],
                return_type: "array".to_string(),
                category: "列表函数".to_string(),
                example: "{{ items | slice(start=0, end=5) }}".to_string(),
                usage: "slice 函数获取数组的指定范围元素。".to_string(),
                insert_text: "{{ value | slice(start=0, end=5) }}".to_string(),
                note: Some("Tera 内置过滤器".to_string()),
                aliases: None,
            },
        ],
    }
}

/// 日期函数
fn build_date_functions() -> BuiltinFunctionCategory {
    BuiltinFunctionCategory {
        name: "日期函数".to_string(),
        description: "日期和时间处理函数".to_string(),
        functions: vec![
            BuiltinFunction {
                name: "date".to_string(),
                display_name: "格式化日期".to_string(),
                description: "格式化日期时间显示".to_string(),
                params: vec![
                    BuiltinFunctionParam {
                        name: "timestamp".to_string(),
                        param_type: "number".to_string(),
                        required: true,
                        description: "Unix时间戳".to_string(),
                    },
                    BuiltinFunctionParam {
                        name: "format".to_string(),
                        param_type: "string".to_string(),
                        required: false,
                        description: "日期格式，默认为 %Y-%m-%d %H:%M:%S".to_string(),
                    },
                ],
                return_type: "string".to_string(),
                category: "日期函数".to_string(),
                example: "{{ timestamp | date(format=\"%Y-%m-%d\") }}".to_string(),
                usage: "date 函数格式化时间戳为可读的日期字符串。支持 strftime 格式化符号。"
                    .to_string(),
                insert_text: "{{ value | date(format=\"%Y-%m-%d\") }}".to_string(),
                note: Some("已实现".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "now".to_string(),
                display_name: "当前时间".to_string(),
                description: "获取当前时间的 Unix 时间戳".to_string(),
                params: vec![],
                return_type: "number".to_string(),
                category: "日期函数".to_string(),
                example: "{% set now = now() %}{{ now | date }}".to_string(),
                usage: "now 函数返回当前时间的 Unix 时间戳（秒）。可与 date 过滤器配合使用。"
                    .to_string(),
                insert_text: "{{ now() }}".to_string(),
                note: Some("已实现".to_string()),
                aliases: None,
            },
        ],
    }
}

/// 数学函数
fn build_math_functions() -> BuiltinFunctionCategory {
    BuiltinFunctionCategory {
        name: "数学函数".to_string(),
        description: "数学计算和数值处理函数".to_string(),
        functions: vec![
            BuiltinFunction {
                name: "abs".to_string(),
                display_name: "绝对值".to_string(),
                description: "获取数值的绝对值".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "number".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    description: "要计算的数值".to_string(),
                }],
                return_type: "number".to_string(),
                category: "数学函数".to_string(),
                example: "{{ value | abs }}".to_string(),
                usage: "abs 函数返回数值的绝对值。".to_string(),
                insert_text: "{{ value | abs }}".to_string(),
                note: Some("Tera 内置过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "round".to_string(),
                display_name: "四舍五入".to_string(),
                description: "对数值进行四舍五入".to_string(),
                params: vec![
                    BuiltinFunctionParam {
                        name: "number".to_string(),
                        param_type: "number".to_string(),
                        required: true,
                        description: "要四舍五入的数值".to_string(),
                    },
                    BuiltinFunctionParam {
                        name: "precision".to_string(),
                        param_type: "number".to_string(),
                        required: false,
                        description: "小数位数，默认为0".to_string(),
                    },
                ],
                return_type: "number".to_string(),
                category: "数学函数".to_string(),
                example: "{{ price | round(precision=2) }}".to_string(),
                usage: "round 函数对数值进行四舍五入到指定的小数位数。".to_string(),
                insert_text: "{{ value | round(precision=2) }}".to_string(),
                note: Some("Tera 内置过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "floor".to_string(),
                display_name: "向下取整".to_string(),
                description: "获取不大于该数值的最大整数".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "number".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    description: "要取整的数值".to_string(),
                }],
                return_type: "number".to_string(),
                category: "数学函数".to_string(),
                example: "{{ value | floor }}".to_string(),
                usage: "floor 函数返回不大于该数值的最大整数。".to_string(),
                insert_text: "{{ value | floor }}".to_string(),
                note: Some("Tera 内置过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "ceil".to_string(),
                display_name: "向上取整".to_string(),
                description: "获取不小于该数值的最小整数".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "number".to_string(),
                    param_type: "number".to_string(),
                    required: true,
                    description: "要取整的数值".to_string(),
                }],
                return_type: "number".to_string(),
                category: "数学函数".to_string(),
                example: "{{ value | ceil }}".to_string(),
                usage: "ceil 函数返回不小于该数值的最小整数。".to_string(),
                insert_text: "{{ value | ceil }}".to_string(),
                note: Some("Tera 内置过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "int".to_string(),
                display_name: "转整数".to_string(),
                description: "将数值转换为整数".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "value".to_string(),
                    param_type: "number|string".to_string(),
                    required: true,
                    description: "要转换的值".to_string(),
                }],
                return_type: "number".to_string(),
                category: "数学函数".to_string(),
                example: "{{ price | int }}".to_string(),
                usage: "int 函数将浮点数或数字字符串转换为整数（截断小数部分）。".to_string(),
                insert_text: "{{ value | int }}".to_string(),
                note: Some("Tera 内置过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "float".to_string(),
                display_name: "转浮点数".to_string(),
                description: "将数值转换为浮点数".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "value".to_string(),
                    param_type: "number|string".to_string(),
                    required: true,
                    description: "要转换的值".to_string(),
                }],
                return_type: "number".to_string(),
                category: "数学函数".to_string(),
                example: "{{ value | float }}".to_string(),
                usage: "float 函数将整数或数字字符串转换为浮点数。".to_string(),
                insert_text: "{{ value | float }}".to_string(),
                note: Some("Tera 内置过滤器".to_string()),
                aliases: None,
            },
        ],
    }
}

/// 编码函数
fn build_encoding_functions() -> BuiltinFunctionCategory {
    BuiltinFunctionCategory {
        name: "编码函数".to_string(),
        description: "数据编码和解码函数".to_string(),
        functions: vec![
            BuiltinFunction {
                name: "base64_encode".to_string(),
                display_name: "Base64编码".to_string(),
                description: "将数据编码为 Base64 格式".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "value".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "要编码的字符串".to_string(),
                }],
                return_type: "string".to_string(),
                category: "编码函数".to_string(),
                example: "{{ data | base64_encode }}".to_string(),
                usage: "base64_encode 函数将字符串编码为 Base64 格式，常用于安全传输数据。"
                    .to_string(),
                insert_text: "{{ value | base64_encode }}".to_string(),
                note: Some("template_core 自定义过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "base64_decode".to_string(),
                display_name: "Base64解码".to_string(),
                description: "解码 Base64 格式数据".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "value".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "要解码的 Base64 字符串".to_string(),
                }],
                return_type: "string".to_string(),
                category: "编码函数".to_string(),
                example: "{{ data | base64_decode }}".to_string(),
                usage: "base64_decode 函数解码 Base64 格式的字符串为原始数据。".to_string(),
                insert_text: "{{ value | base64_decode }}".to_string(),
                note: Some("template_core 自定义过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "json_encode".to_string(),
                display_name: "JSON编码".to_string(),
                description: "将数据转换为 JSON 字符串".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "value".to_string(),
                    param_type: "object|array".to_string(),
                    required: true,
                    description: "要转换的对象或数组".to_string(),
                }],
                return_type: "string".to_string(),
                category: "编码函数".to_string(),
                example: "{{ data | json_encode }}".to_string(),
                usage: "json_encode 函数将对象或数组序列化为 JSON 字符串。".to_string(),
                insert_text: "{{ value | json_encode }}".to_string(),
                note: Some("template_core 自定义过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "url_encode".to_string(),
                display_name: "URL编码".to_string(),
                description: "将字符串进行 URL 编码".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "value".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "要编码的字符串".to_string(),
                }],
                return_type: "string".to_string(),
                category: "编码函数".to_string(),
                example: "{{ url | url_encode }}".to_string(),
                usage: "url_encode 函数将字符串进行 URL 编码，转换特殊字符为 %XX 格式。"
                    .to_string(),
                insert_text: "{{ value | url_encode }}".to_string(),
                note: Some("template_core 自定义过滤器".to_string()),
                aliases: None,
            },
        ],
    }
}

/// 类型转换函数
fn build_type_conversion_functions() -> BuiltinFunctionCategory {
    BuiltinFunctionCategory {
        name: "类型转换".to_string(),
        description: "数据类型转换函数".to_string(),
        functions: vec![
            BuiltinFunction {
                name: "string".to_string(),
                display_name: "转字符串".to_string(),
                description: "将值转换为字符串".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "value".to_string(),
                    param_type: "any".to_string(),
                    required: true,
                    description: "要转换的值".to_string(),
                }],
                return_type: "string".to_string(),
                category: "类型转换".to_string(),
                example: "{{ value | string }}".to_string(),
                usage: "string 函数将任何类型的值转换为字符串表示。".to_string(),
                insert_text: "{{ value | string }}".to_string(),
                note: Some("Tera 内置过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "bool".to_string(),
                display_name: "转布尔值".to_string(),
                description: "将值转换为布尔值".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "value".to_string(),
                    param_type: "any".to_string(),
                    required: true,
                    description: "要转换的值".to_string(),
                }],
                return_type: "bool".to_string(),
                category: "类型转换".to_string(),
                example: "{{ value | bool }}".to_string(),
                usage: "bool 函数将值转换为布尔值（非零、非空、非null为true）。".to_string(),
                insert_text: "{{ value | bool }}".to_string(),
                note: Some("Tera 内置过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "number_format".to_string(),
                display_name: "格式化数字".to_string(),
                description: "格式化数字显示，添加千位分隔符".to_string(),
                params: vec![
                    BuiltinFunctionParam {
                        name: "value".to_string(),
                        param_type: "number".to_string(),
                        required: true,
                        description: "要格式化的数字".to_string(),
                    },
                    BuiltinFunctionParam {
                        name: "decimals".to_string(),
                        param_type: "number".to_string(),
                        required: false,
                        description: "小数位数".to_string(),
                    },
                    BuiltinFunctionParam {
                        name: "delimiter".to_string(),
                        param_type: "string".to_string(),
                        required: false,
                        description: "千位分隔符，默认为逗号".to_string(),
                    },
                ],
                return_type: "string".to_string(),
                category: "类型转换".to_string(),
                example: "{{ price | number_format(decimals=2) }}".to_string(),
                usage: "number_format 函数格式化数字显示，添加千位分隔符并指定小数位数。"
                    .to_string(),
                insert_text: "{{ value | number_format(decimals=2) }}".to_string(),
                note: Some("已实现".to_string()),
                aliases: None,
            },
        ],
    }
}

/// 默认值函数
fn build_default_functions() -> BuiltinFunctionCategory {
    BuiltinFunctionCategory {
        name: "默认值".to_string(),
        description: "处理空值和默认值的函数".to_string(),
        functions: vec![
            BuiltinFunction {
                name: "default".to_string(),
                display_name: "默认值".to_string(),
                description: "当值未定义或为空时使用默认值".to_string(),
                params: vec![
                    BuiltinFunctionParam {
                        name: "value".to_string(),
                        param_type: "any".to_string(),
                        required: true,
                        description: "要检查的值".to_string(),
                    },
                    BuiltinFunctionParam {
                        name: "default_value".to_string(),
                        param_type: "any".to_string(),
                        required: true,
                        description: "默认值".to_string(),
                    },
                ],
                return_type: "any".to_string(),
                category: "默认值".to_string(),
                example: "{{ user.name | default(value=\"匿名\") }}".to_string(),
                usage: "default 函数当变量未定义或为空时，返回指定的默认值。".to_string(),
                insert_text: "{{ value | default(value=\"默认\") }}".to_string(),
                note: Some("template_core 自定义过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "escape".to_string(),
                display_name: "HTML转义".to_string(),
                description: "转义 HTML 特殊字符".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "value".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "要转义的字符串".to_string(),
                }],
                return_type: "string".to_string(),
                category: "默认值".to_string(),
                example: "{{ content | escape }}".to_string(),
                usage: "escape 函数转义 HTML 特殊字符（<, >, &, \", '），防止 XSS 攻击。.html/.htm/.xml 模板中 {{ }} 默认已自动转义；需要原样输出 HTML 时使用 safe。".to_string(),
                insert_text: "{{ value | escape }}".to_string(),
                note: Some("HTML/XML 模板默认自动转义，safe 可豁免".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "safe".to_string(),
                display_name: "安全输出".to_string(),
                description: "输出不转义的原始内容".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "value".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "包含 HTML 的字符串".to_string(),
                }],
                return_type: "string".to_string(),
                category: "默认值".to_string(),
                example: "{{ html_content | safe }}".to_string(),
                usage: "safe 过滤器输出原始内容，不进行 HTML 转义。使用时需确保内容安全。".to_string(),
                insert_text: "{{ value | safe }}".to_string(),
                note: Some("Tera 内置过滤器".to_string()),
                aliases: None,
            },
            BuiltinFunction {
                name: "reverse".to_string(),
                display_name: "反转".to_string(),
                description: "反转字符串或数组".to_string(),
                params: vec![BuiltinFunctionParam {
                    name: "value".to_string(),
                    param_type: "string|array".to_string(),
                    required: true,
                    description: "要反转的值".to_string(),
                }],
                return_type: "string|array".to_string(),
                category: "默认值".to_string(),
                example: "{{ text | reverse }}".to_string(),
                usage: "reverse 函数反转字符串或数组的元素顺序。".to_string(),
                insert_text: "{{ value | reverse }}".to_string(),
                note: Some("template_core 自定义过滤器".to_string()),
                aliases: None,
            },
        ],
    }
}
