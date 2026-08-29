//! 类型定义

use serde::{Deserialize, Serialize};

/// 渲染变量
#[derive(Debug, Clone, Default)]
pub struct Variables {
    inner: serde_json::Value,
}

impl Variables {
    /// 从 JSON 字符串创建
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json)
            .map(|v| Variables { inner: v })
            .map_err(|e| e.to_string())
    }

    /// 从 serde_json::Value 创建
    pub fn from_value(value: serde_json::Value) -> Self {
        Variables { inner: value }
    }

    /// 获取内部的 JSON Value
    pub fn as_value(&self) -> &serde_json::Value {
        &self.inner
    }
}

/// 渲染结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderResult {
    /// 渲染后的内容
    pub content: String,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果失败）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RenderError>,
    /// 使用的原始变量
    pub variables: serde_json::Value,
}

/// 渲染错误
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenderError {
    /// 错误类型
    #[serde(rename = "type")]
    pub error_type: String,
    /// 错误消息
    pub message: String,
    /// 错误所在行号
    pub line: Option<usize>,
    /// 错误所在列
    pub column: Option<usize>,
    /// 错误上下文（前后几行）
    pub context: Option<String>,
    /// 修复建议
    pub suggestion: Option<String>,
}

impl RenderError {
    /// 创建变量未找到错误
    pub fn variable_not_found(var_name: &str) -> Self {
        RenderError {
            error_type: "variable_error".to_string(),
            message: format!("Variable `{}` not found in context", var_name),
            line: None,
            column: None,
            context: None,
            suggestion: Some(format!(
                "检查变量名 `{}` 是否正确，确保变量在上下文中已定义",
                var_name
            )),
        }
    }

    /// 创建过滤器未找到错误
    pub fn filter_not_found(filter_name: &str) -> Self {
        RenderError {
            error_type: "filter_error".to_string(),
            message: format!("Filter `{}` not found", filter_name),
            line: None,
            column: None,
            context: None,
            suggestion: Some("检查过滤器名称是否正确".to_string()),
        }
    }

    /// 创建语法错误
    pub fn syntax_error(msg: &str) -> Self {
        RenderError {
            error_type: "parse_error".to_string(),
            message: msg.to_string(),
            line: None,
            column: None,
            context: None,
            suggestion: Some("检查模板语法是否正确".to_string()),
        }
    }
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error_type, self.message)
    }
}

impl std::error::Error for RenderError {}

/// 过滤器信息
#[derive(Debug, Clone)]
pub struct FilterInfo {
    /// 过滤器名称
    pub name: String,
    /// 过滤器描述
    pub description: String,
    /// 示例
    pub example: String,
}
