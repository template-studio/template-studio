use serde::{Deserialize, Serialize};

/// AI 聊天请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

/// 消息角色
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// AI 聊天响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
    pub usage: Usage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// 流式响应块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunk {
    pub delta: String,
    pub finish_reason: Option<String>,
}

/// Token 使用量
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// 工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// 工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 变量 Schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VariableSchema {
    pub name: String,
    #[serde(rename = "type")]
    pub var_type: String,
    pub title: String,
    pub description: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// 变量来源：regex（直接提取）或 inferred（AI 推断）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// AI 推断的建议值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<serde_json::Value>,
}

/// 变量分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableAnalysisResult {
    pub template: String,
    pub variables: Vec<VariableSchema>,
    pub total: usize,
    pub auto_inferred: usize,
}

/// 变量填充结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableFillResult {
    pub template: String,
    pub project: String,
    pub filled: serde_json::Value,
    pub confidence: f32,
    pub ai_reasoning: String,
}

/// Agent 运行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub response: String,
    pub iterations: usize,
    pub tool_calls_made: Vec<ToolCall>,
    pub usage: Usage,
}
