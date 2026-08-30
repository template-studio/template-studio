use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub user: UserConfig,
    pub storage: StorageConfig,
    /// AI 配置（可选——未配置时 AI 命令回退环境变量 AI_API_KEY/AI_BASE_URL）
    #[serde(default)]
    pub ai: Option<AiSection>,
}

/// CLI 本地持久化的 AI 配置段（ai config set 写入）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSection {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    #[serde(default)]
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub url: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub author: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub template_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        // 默认模板存储路径
        // Windows: C:\Users\{user}\.cicbyte\template_studio\data\templates
        // Linux/Mac: ~/.cicbyte/template_studio/data/templates
        let template_path = if cfg!(windows) {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".cicbyte")
                .join("template_studio")
                .join("data")
                .join("templates")
        } else {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".cicbyte")
                .join("template_studio")
                .join("data")
                .join("templates")
        };

        Self {
            server: ServerConfig {
                url: "http://127.0.0.1:8080".to_string(),
                api_key: String::new(),
            },
            user: UserConfig {
                author: None,
                email: None,
            },
            storage: StorageConfig { template_path },
            ai: None,
        }
    }
}

impl Config {
    pub fn load(custom_path: Option<String>) -> Result<Self> {
        let config_path = if let Some(ref path) = custom_path {
            PathBuf::from(path)
        } else {
            // 默认配置路径: ~/.cicbyte/template_studio/config/config.toml
            let home_dir = dirs::home_dir().context("无法确定用户主目录")?;

            let config_dir = home_dir
                .join(".cicbyte")
                .join("template_studio")
                .join("config");

            // 确保配置目录存在
            std::fs::create_dir_all(&config_dir).context("创建配置目录失败")?;

            config_dir.join("config.toml")
        };

        // 如果配置文件不存在，创建默认配置
        if !config_path.exists() {
            info!("创建默认配置文件: {:?}", config_path);
            let default_config = Config::default();

            // 确保模板存储目录存在
            std::fs::create_dir_all(&default_config.storage.template_path)
                .context("创建模板存储目录失败")?;

            // 序列化配置为TOML格式
            let config_str = toml::to_string_pretty(&default_config).context("序列化配置失败")?;

            std::fs::write(&config_path, config_str).context("写入配置文件失败")?;

            return Ok(default_config);
        }

        // 读取配置文件
        let config_str = std::fs::read_to_string(&config_path).context("读取配置文件失败")?;

        let config: Config = toml::from_str(&config_str).context("解析配置文件失败")?;

        Ok(config)
    }

    #[allow(dead_code)]
    pub fn save(&self) -> Result<()> {
        let home_dir = dirs::home_dir().context("无法确定用户主目录")?;

        let config_dir = home_dir
            .join(".cicbyte")
            .join("template_studio")
            .join("config");

        std::fs::create_dir_all(&config_dir).context("创建配置目录失败")?;

        // 确保模板存储目录存在
        std::fs::create_dir_all(&self.storage.template_path).context("创建模板存储目录失败")?;

        let config_path = config_dir.join("config.toml");
        let config_str = toml::to_string_pretty(self).context("序列化配置失败")?;

        std::fs::write(&config_path, config_str).context("写入配置文件失败")?;

        Ok(())
    }
}
