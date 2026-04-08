pub mod variable;
pub mod render;
pub mod validate;
pub mod file;
pub mod convert;
pub mod recommend;
pub mod diff;

use async_trait::async_trait;
use crate::types::{ToolDefinition, ToolResult};

/// AI 工具 trait
#[async_trait]
pub trait AiTool: Send + Sync {
    /// 工具名称
    fn name(&self) -> &str;

    /// 工具描述
    fn description(&self) -> &str;

    /// 参数 JSON Schema
    fn parameters(&self) -> serde_json::Value;

    /// 获取工具定义
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: self.parameters(),
        }
    }

    /// 执行工具
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError>;
}

/// 工具错误
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("参数错误: {0}")]
    InvalidArgument(String),

    #[error("执行失败: {0}")]
    ExecutionFailed(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),
}
