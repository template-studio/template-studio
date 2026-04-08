use async_trait::async_trait;
use futures::stream::Stream;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use crate::config::AiConfig;
use crate::types::*;

/// AI 客户端 trait
#[async_trait]
pub trait AiClient: Send + Sync {
    /// 普通聊天
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, AiError>;

    /// 带工具的聊天
    async fn chat_with_tools(
        &self,
        request: ChatRequest,
        tools: Vec<ToolDefinition>,
    ) -> Result<ChatResponse, AiError>;

    /// 流式聊天
    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, AiError>> + Send>>, AiError>;
}

/// AI 错误
#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("HTTP 请求失败: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON 解析失败: {0}")]
    Json(#[from] serde_json::Error),

    #[error("API 错误: {status} - {message}")]
    Api { status: u16, message: String },

    #[error("配置错误: {0}")]
    Config(String),

    #[error("流式响应错误: {0}")]
    Stream(String),
}

/// OpenAI 兼容客户端
pub struct OpenAiClient {
    config: AiConfig,
    http: Client,
}

impl OpenAiClient {
    pub fn new(config: AiConfig) -> Self {
        Self {
            config,
            http: Client::new(),
        }
    }
}

/// OpenAI API 请求格式
#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAiTool>>,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAiTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAiFunction,
}

#[derive(Debug, Serialize)]
struct OpenAiFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

/// OpenAI API 响应格式
#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    model: String,
    usage: OpenAiUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoiceMessage {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<OpenAiToolCall>>,
}

#[derive(Debug, Deserialize)]
struct OpenAiToolCall {
    id: String,
    function: OpenAiToolCallFunction,
}

#[derive(Debug, Deserialize)]
struct OpenAiToolCallFunction {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// 流式响应格式
#[derive(Debug, Deserialize)]
struct OpenAiStreamResponse {
    choices: Vec<OpenAiStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChoice {
    delta: OpenAiStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamDelta {
    content: Option<String>,
}

fn to_openai_messages(messages: &[ChatMessage]) -> Vec<OpenAiMessage> {
    messages
        .iter()
        .map(|m| OpenAiMessage {
            role: match m.role {
                MessageRole::System => "system".to_string(),
                MessageRole::User => "user".to_string(),
                MessageRole::Assistant => "assistant".to_string(),
            },
            content: m.content.clone(),
        })
        .collect()
}

fn to_openai_tools(tools: &[ToolDefinition]) -> Vec<OpenAiTool> {
    tools
        .iter()
        .map(|t| OpenAiTool {
            tool_type: "function".to_string(),
            function: OpenAiFunction {
                name: t.name.clone(),
                description: t.description.clone(),
                parameters: t.parameters.clone(),
            },
        })
        .collect()
}

fn from_openai_response(resp: OpenAiResponse) -> ChatResponse {
    let choice = resp.choices.into_iter().next().unwrap_or_default();
    let tool_calls = choice.message.tool_calls.map(|calls| {
        calls
            .into_iter()
            .map(|tc| ToolCall {
                id: tc.id,
                name: tc.function.name,
                arguments: serde_json::from_str(&tc.function.arguments)
                    .unwrap_or(serde_json::Value::Null),
            })
            .collect()
    });

    ChatResponse {
        content: choice.message.content.unwrap_or_default(),
        model: resp.model,
        usage: Usage {
            prompt_tokens: resp.usage.prompt_tokens,
            completion_tokens: resp.usage.completion_tokens,
            total_tokens: resp.usage.total_tokens,
        },
        tool_calls,
    }
}

impl Default for OpenAiChoice {
    fn default() -> Self {
        Self {
            message: OpenAiChoiceMessage {
                content: None,
                tool_calls: None,
            },
        }
    }
}

#[async_trait]
impl AiClient for OpenAiClient {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, AiError> {
        let req = OpenAiRequest {
            model: self.config.model.clone(),
            messages: to_openai_messages(&request.messages),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            tools: None,
            stream: false,
        };

        let resp = self
            .http
            .post(self.config.chat_endpoint())
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(AiError::Api {
                status,
                message: body,
            });
        }

        let openai_resp: OpenAiResponse = resp.json().await?;
        Ok(from_openai_response(openai_resp))
    }

    async fn chat_with_tools(
        &self,
        request: ChatRequest,
        tools: Vec<ToolDefinition>,
    ) -> Result<ChatResponse, AiError> {
        let req = OpenAiRequest {
            model: self.config.model.clone(),
            messages: to_openai_messages(&request.messages),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            tools: Some(to_openai_tools(&tools)),
            stream: false,
        };

        let resp = self
            .http
            .post(self.config.chat_endpoint())
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(AiError::Api {
                status,
                message: body,
            });
        }

        let openai_resp: OpenAiResponse = resp.json().await?;
        Ok(from_openai_response(openai_resp))
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, AiError>> + Send>>, AiError> {
        let req = OpenAiRequest {
            model: self.config.model.clone(),
            messages: to_openai_messages(&request.messages),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            tools: None,
            stream: true,
        };

        let resp = self
            .http
            .post(self.config.chat_endpoint())
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(AiError::Api {
                status,
                message: body,
            });
        }

        let stream = resp.bytes_stream();
        let mapped = stream.filter_map(|chunk_result| async move {
            match chunk_result {
                Ok(chunk) => {
                    let text = String::from_utf8_lossy(&chunk);
                    for line in text.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data == "[DONE]" {
                                return None;
                            }
                            if let Ok(resp) = serde_json::from_str::<OpenAiStreamResponse>(data) {
                                if let Some(choice) = resp.choices.into_iter().next() {
                                    return Some(Ok(ChatChunk {
                                        delta: choice.delta.content.unwrap_or_default(),
                                        finish_reason: choice.finish_reason,
                                    }));
                                }
                            }
                        }
                    }
                    None
                }
                Err(e) => Some(Err(AiError::Http(e))),
            }
        });

        Ok(Box::pin(mapped))
    }
}
