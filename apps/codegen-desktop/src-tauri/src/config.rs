use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub storage: StorageConfig,
    pub web_server: WebServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub template_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebServerConfig {
    pub api_url: String,
    pub api_key: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        // 默认模板存储路径（与CLI共享）
        // Windows: C:\Users\{user}\.ciclebyte\template_studio_rust\data\templates
        // Linux/Mac: ~/.ciclebyte/template_studio_rust/data/templates
        let template_path = if cfg!(windows) {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".ciclebyte")
                .join("template_studio_rust")
                .join("data")
                .join("templates")
        } else {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".ciclebyte")
                .join("template_studio_rust")
                .join("data")
                .join("templates")
        };

        Self {
            storage: StorageConfig { template_path },
            web_server: WebServerConfig {
                api_url: "http://127.0.0.1:8080".to_string(),
                api_key: None,
            },
        }
    }
}

impl Config {
    /// 获取配置文件路径
    fn get_config_path() -> Result<PathBuf> {
        let config_dir = if cfg!(windows) {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".ciclebyte")
                .join("template_studio_rust")
                .join("config")
        } else {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".ciclebyte")
                .join("template_studio_rust")
                .join("config")
        };

        // 确保配置目录存在
        std::fs::create_dir_all(&config_dir)
            .context("创建配置目录失败")?;

        Ok(config_dir.join("codegen-desktop.yaml"))
    }

    /// 从文件加载配置
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        // 如果配置文件不存在，返回默认配置
        if !config_path.exists() {
            let config = Self::default();

            // 保存默认配置到文件
            config.save()?;

            // 确保模板存储目录存在
            std::fs::create_dir_all(&config.storage.template_path)
                .context("创建模板存储目录失败")?;

            return Ok(config);
        }

        // 读取配置文件
        let content = std::fs::read_to_string(&config_path)
            .context("读取配置文件失败")?;

        // 解析 YAML
        let config: Config = serde_yaml::from_str(&content)
            .context("解析配置文件失败")?;

        // 确保模板存储目录存在
        std::fs::create_dir_all(&config.storage.template_path)
            .context("创建模板存储目录失败")?;

        Ok(config)
    }

    /// 保存配置到文件
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;

        let yaml = serde_yaml::to_string(self)
            .context("序列化配置失败")?;

        std::fs::write(&config_path, yaml)
            .context("写入配置文件失败")?;

        Ok(())
    }

    pub fn get_template_path(&self, template_id: &str, version: &str) -> PathBuf {
        // 目录结构: templates/{template_id}/{version}/
        // version 参数是实际的版本号（如 "1.0.0"）
        // 前端会自动选择 is_latest=true 的版本，与 CLI 保持一致
        self.storage.template_path
            .join(template_id)
            .join(version)
    }
}
