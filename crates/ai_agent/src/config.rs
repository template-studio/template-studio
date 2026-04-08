use serde::{Deserialize, Serialize};

/// AI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// 提供商名称（deepseek, glm, openai, custom）
    pub provider: String,
    /// 模型名称
    pub model: String,
    /// API Key
    pub api_key: String,
    /// 自定义 API 基础 URL（可选）
    pub base_url: Option<String>,
}

impl AiConfig {
    /// 获取 API 基础 URL
    pub fn get_base_url(&self) -> String {
        if let Some(url) = &self.base_url {
            return url.clone();
        }
        match self.provider.as_str() {
            "deepseek" => "https://api.deepseek.com/v1".to_string(),
            "openai" => "https://api.openai.com/v1".to_string(),
            "glm" => "https://open.bigmodel.cn/api/paas/v4".to_string(),
            _ => "http://localhost:11434/v1".to_string(),
        }
    }

    /// 获取聊天完成端点
    pub fn chat_endpoint(&self) -> String {
        format!("{}/chat/completions", self.get_base_url())
    }
}
