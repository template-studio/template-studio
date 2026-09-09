//! 统一 AI 执行层。
//!
//! 基于 rig-core(锁定 =0.42)的多 provider 抽象,桌面端所有 AI 调用收敛到此模块,
//! 对外仅暴露 `chat`(P0)/ `structured`(P1 接 schemars)/ `stream`(P1 接 Tauri event)。
//! rig 的破坏性变更影响面被限制在本文件内。

use rig_core::client::CompletionClient;
use rig_core::completion::{AssistantContent, CompletionModel, Message};

/// 接入协议。决定 rig provider client 的构造方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// OpenAI 兼容 /chat/completions(deepseek/glm/openai/longcat/mimo/cherry-studio 等)
    OpenAiCompatible,
    Anthropic,
    Gemini,
    Ollama,
}

impl Protocol {
    /// 库里旧数据/未知值一律回落 OpenAI 兼容协议(迁移默认值同源)。
    pub fn parse(s: &str) -> Self {
        match s {
            "anthropic" => Self::Anthropic,
            "gemini" => Self::Gemini,
            "ollama" => Self::Ollama,
            _ => Self::OpenAiCompatible,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAiCompatible => "openai_compatible",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
            Self::Ollama => "ollama",
        }
    }
}

/// 一次 AI 调用的目标:由 ai_providers 行(协议/密钥/端点)+ 模型名解析而来。
pub struct CallTarget {
    pub protocol: Protocol,
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: String,
    pub temperature: f64,
    pub max_tokens: u64,
}

/// 从 ai_providers 的 JSON 行 + 模型名构造调用目标。
/// `provider` 为 `get_ai_provider` 返回值(含解密后的 apiKey)。
pub fn call_target_from_provider(
    provider: &serde_json::Value,
    model: &str,
    protocol: Protocol,
) -> Result<CallTarget, String> {
    let api_key = provider["apiKey"]
        .as_str()
        .filter(|k| !k.is_empty())
        .ok_or_else(|| "请先配置 API 密钥".to_string())?
        .to_string();
    let base_url = provider["apiEndpoint"]
        .as_str()
        .filter(|u| !u.is_empty())
        .map(str::to_string);
    Ok(CallTarget {
        protocol,
        api_key,
        base_url,
        model: model.to_string(),
        temperature: provider["temperature"].as_f64().unwrap_or(0.7),
        max_tokens: provider["maxTokens"].as_i64().unwrap_or(4096).max(1) as u64,
    })
}

/// OpenAI 风格历史 [{role, content}] → rig Message。
/// system 消息收敛进 preamble 由调用方处理,此处只转 user/assistant。
fn history_to_messages(history: &[serde_json::Value]) -> Vec<Message> {
    history
        .iter()
        .filter_map(|m| {
            let content = m.get("content")?.as_str()?;
            match m.get("role")?.as_str()? {
                "assistant" => Some(Message::assistant(content)),
                "system" => None,
                _ => Some(Message::user(content)),
            }
        })
        .collect()
}

/// 从回复 choice 中提取纯文本。
fn extract_text(choice: Vec<AssistantContent>) -> Result<String, String> {
    choice
        .into_iter()
        .find_map(|c| match c {
            AssistantContent::Text(t) => Some(t.text),
            _ => None,
        })
        .ok_or_else(|| "AI 返回中不包含文本内容".to_string())
}

/// 单轮/多轮对话。`system` 进 preamble,`history` 为 prior 轮次,`prompt` 为本轮输入。
pub async fn chat(
    target: &CallTarget,
    system: Option<&str>,
    prompt: &str,
    history: &[serde_json::Value],
) -> Result<String, String> {
    match target.protocol {
        Protocol::OpenAiCompatible => {
            let client = openai_client(target)?;
            let model = client.completion_model(target.model.clone());
            run_chat(&model, target, system, prompt, history).await
        }
        Protocol::Anthropic => {
            let client = anthropic_client(target)?;
            let model = client.completion_model(target.model.clone());
            run_chat(&model, target, system, prompt, history).await
        }
        Protocol::Gemini => {
            let client = gemini_client(target)?;
            let model = client.completion_model(target.model.clone());
            run_chat(&model, target, system, prompt, history).await
        }
        Protocol::Ollama => {
            let client = ollama_client(target)?;
            let model = client.completion_model(target.model.clone());
            run_chat(&model, target, system, prompt, history).await
        }
    }
}

/// OpenAI 风格整包消息入口(前端传来的原样数组):
/// system 消息合并进 preamble,最后一条 user 消息作为本轮 prompt,其余进 history。
pub async fn chat_openai_style(
    target: &CallTarget,
    messages: &[serde_json::Value],
) -> Result<String, String> {
    if messages.is_empty() {
        return Err("messages 不能为空".to_string());
    }

    let system: Vec<&str> = messages
        .iter()
        .filter(|m| m.get("role").and_then(|r| r.as_str()) == Some("system"))
        .filter_map(|m| m.get("content").and_then(|c| c.as_str()))
        .collect();
    let preamble = if system.is_empty() {
        None
    } else {
        Some(system.join("\n\n"))
    };

    let (prompt, history) = match messages.last() {
        Some(last) if last.get("role").and_then(|r| r.as_str()) == Some("user") => (
            last.get("content").and_then(|c| c.as_str()).unwrap_or(""),
            &messages[..messages.len() - 1],
        ),
        _ => ("", messages),
    };

    chat(target, preamble.as_deref(), prompt, history).await
}

async fn run_chat<M: CompletionModel + Clone>(
    model: &M,
    target: &CallTarget,
    system: Option<&str>,
    prompt: &str,
    history: &[serde_json::Value],
) -> Result<String, String> {
    let mut req = model
        .completion_request(prompt.to_string())
        .temperature(target.temperature)
        .max_tokens(target.max_tokens)
        .messages(history_to_messages(history));
    if let Some(sys) = system {
        req = req.preamble(sys.to_string());
    }
    let resp = model
        .completion(req.build())
        .await
        .map_err(|e| format!("AI 调用失败: {}", e))?;
    extract_text(resp.choice)
}

// ---- 各协议 client 构造 ----

pub(crate) fn openai_client(target: &CallTarget) -> Result<rig_core::providers::openai::Client, String> {
    let mut b = rig_core::providers::openai::Client::builder().api_key(target.api_key.clone());
    if let Some(u) = &target.base_url {
        b = b.base_url(u.clone());
    }
    b.build().map_err(|e| format!("OpenAI 兼容客户端构造失败: {}", e))
}

fn anthropic_client(target: &CallTarget) -> Result<rig_core::providers::anthropic::Client, String> {
    let mut b = rig_core::providers::anthropic::Client::builder().api_key(target.api_key.clone());
    if let Some(u) = &target.base_url {
        b = b.base_url(u.clone());
    }
    b.build().map_err(|e| format!("Anthropic 客户端构造失败: {}", e))
}

fn gemini_client(target: &CallTarget) -> Result<rig_core::providers::gemini::Client, String> {
    let mut b = rig_core::providers::gemini::Client::builder().api_key(target.api_key.clone());
    if let Some(u) = &target.base_url {
        b = b.base_url(u.clone());
    }
    b.build().map_err(|e| format!("Gemini 客户端构造失败: {}", e))
}

fn ollama_client(target: &CallTarget) -> Result<rig_core::providers::ollama::Client, String> {
    let mut b = rig_core::providers::ollama::Client::builder()
        .api_key(rig_core::providers::ollama::OllamaApiKey::from("ollama"));
    if let Some(u) = &target.base_url {
        b = b.base_url(u.clone());
    }
    b.build().map_err(|e| format!("Ollama 客户端构造失败: {}", e))
}
