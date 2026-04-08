use std::sync::Arc;
use tracing::{info, warn};

use crate::client::AiClient;
use crate::types::{ChatRequest, ChatMessage, MessageRole, ToolDefinition};
use crate::tools::{AiTool, ToolError};

/// Agent 执行结果
#[derive(Debug, Clone)]
pub struct AgentResult {
    pub success: bool,
    pub response: String,
    pub tool_calls: Vec<ToolCallRecord>,
    pub iterations: usize,
    pub token_usage: Option<TokenUsage>,
}

#[derive(Debug, Clone)]
pub struct ToolCallRecord {
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub result: String,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// AI Agent - 支持多轮 tool calling
pub struct Agent {
    client: Arc<dyn AiClient>,
    tools: Vec<Box<dyn AiTool>>,
    system_prompt: String,
    max_iterations: usize,
    model: String,
}

impl Agent {
    pub fn new(client: Arc<dyn AiClient>) -> Self {
        Self {
            client,
            tools: Vec::new(),
            system_prompt: String::new(),
            max_iterations: 10,
            model: "deepseek-chat".to_string(),
        }
    }

    pub fn with_system_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = prompt.to_string();
        self
    }

    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    pub fn add_tool(mut self, tool: Box<dyn AiTool>) -> Self {
        self.tools.push(tool);
        self
    }

    pub fn add_tools(mut self, tools: Vec<Box<dyn AiTool>>) -> Self {
        self.tools.extend(tools);
        self
    }

    fn tool_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.iter().map(|t| t.definition()).collect()
    }

    fn find_tool(&self, name: &str) -> Option<&dyn AiTool> {
        self.tools.iter().find(|t| t.name() == name).map(|t| t.as_ref())
    }

    /// 执行 Agent 循环
    pub async fn run(&self, user_message: &str) -> Result<AgentResult, AgentError> {
        let mut messages = Vec::new();

        if !self.system_prompt.is_empty() {
            messages.push(ChatMessage {
                role: MessageRole::System,
                content: self.system_prompt.clone(),
            });
        }

        messages.push(ChatMessage {
            role: MessageRole::User,
            content: user_message.to_string(),
        });

        let definitions = self.tool_definitions();
        let mut call_records = Vec::new();
        let mut total_usage = TokenUsage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        };

        for iteration in 0..self.max_iterations {
            info!("Agent 迭代 {}/{}", iteration + 1, self.max_iterations);

            let request = ChatRequest {
                messages: messages.clone(),
                model: self.model.clone(),
                temperature: Some(0.7),
                max_tokens: Some(4096),
            };

            let response = self.client
                .chat_with_tools(request, definitions.clone())
                .await
                .map_err(|e| AgentError::ApiError(format!("{}", e)))?;

            // 累计 token 使用
            total_usage.prompt_tokens += response.usage.prompt_tokens;
            total_usage.completion_tokens += response.usage.completion_tokens;
            total_usage.total_tokens += response.usage.total_tokens;

            // 检查是否有 tool calls
            if let Some(tool_calls) = &response.tool_calls {
                if tool_calls.is_empty() {
                    return Ok(AgentResult {
                        success: true,
                        response: response.content,
                        tool_calls: call_records,
                        iterations: iteration + 1,
                        token_usage: Some(total_usage),
                    });
                }

                // 添加 assistant 消息（带 tool call 信息）
                let tool_call_summary = tool_calls.iter()
                    .map(|tc| format!("[调用工具: {}]", tc.name))
                    .collect::<Vec<_>>()
                    .join(" ");
                messages.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: if response.content.is_empty() {
                        tool_call_summary
                    } else {
                        format!("{}\n{}", response.content, tool_call_summary)
                    },
                });

                // 执行每个 tool call
                for tool_call in tool_calls {
                    let tool_name = &tool_call.name;
                    let args = &tool_call.arguments;

                    info!("执行工具: {}({})", tool_name, args);

                    let (result_content, success) = match self.find_tool(tool_name) {
                        Some(tool) => match tool.execute(args.clone()).await {
                            Ok(result) => {
                                let content = if result.success {
                                    result.output
                                } else {
                                    result.error.unwrap_or_else(|| "未知错误".to_string())
                                };
                                (content, result.success)
                            }
                            Err(e) => {
                                warn!("工具执行失败: {} - {}", tool_name, e);
                                (format!("工具执行失败: {}", e), false)
                            }
                        },
                        None => {
                            warn!("未找到工具: {}", tool_name);
                            (format!("未找到工具: {}", tool_name), false)
                        }
                    };

                    call_records.push(ToolCallRecord {
                        tool_name: tool_name.clone(),
                        arguments: args.clone(),
                        result: result_content.clone(),
                        success,
                    });

                    // 添加 tool 结果作为 user 消息（简化处理）
                    messages.push(ChatMessage {
                        role: MessageRole::User,
                        content: format!("[工具 {} 的结果]\n{}", tool_name, result_content),
                    });
                }
            } else {
                // 没有 tool calls，返回最终响应
                return Ok(AgentResult {
                    success: true,
                    response: response.content,
                    tool_calls: call_records,
                    iterations: iteration + 1,
                    token_usage: Some(total_usage),
                });
            }
        }

        warn!("Agent 达到最大迭代次数: {}", self.max_iterations);
        Ok(AgentResult {
            success: false,
            response: format!("达到最大迭代次数 ({})，任务未完成", self.max_iterations),
            tool_calls: call_records,
            iterations: self.max_iterations,
            token_usage: Some(total_usage),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("API 错误: {0}")]
    ApiError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("工具错误: {0}")]
    ToolError(#[from] ToolError),
}

/// 创建默认 Agent（带所有内置工具）
pub fn create_default_agent(client: Arc<dyn AiClient>) -> Agent {
    use crate::tools::variable::{AnalyzeVariablesTool, FillVariablesTool};
    use crate::tools::render::{RenderPreviewTool, RenderExportTool};
    use crate::tools::validate::{ValidateSyntaxTool, ValidateVariablesTool};
    use crate::tools::file::EditFileTool;
    use crate::tools::convert::ConvertToTemplateTool;
    use crate::tools::recommend::RecommendTemplateTool;
    use crate::tools::diff::RenderDiffTool;

    let fill_tool = FillVariablesTool::new(client.clone());

    Agent::new(client)
        .with_system_prompt("你是 Template Studio AI 助手，帮助用户管理模板、分析变量、渲染预览和转换项目。")
        .with_max_iterations(10)
        .add_tool(Box::new(AnalyzeVariablesTool))
        .add_tool(Box::new(fill_tool))
        .add_tool(Box::new(RenderPreviewTool))
        .add_tool(Box::new(RenderExportTool))
        .add_tool(Box::new(ValidateSyntaxTool))
        .add_tool(Box::new(ValidateVariablesTool))
        .add_tool(Box::new(EditFileTool))
        .add_tool(Box::new(ConvertToTemplateTool))
        .add_tool(Box::new(RecommendTemplateTool))
        .add_tool(Box::new(RenderDiffTool))
}
