use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub git: GitConfig,
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    /// 显式放行的 CORS 来源（生产部署必须配置；未配置时仅放行 localhost 开发来源）
    #[serde(default)]
    pub cors_origins: Option<Vec<String>>,
}

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub r#type: String,
    pub base_path: PathBuf,
}

/// Git配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub auto_init: bool,
    pub default_branch: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                cors_origins: None,
            },
            database: DatabaseConfig {
                url: "mysql://root:password@localhost/template_studio".to_string(),
                max_connections: 10,
            },
            storage: StorageConfig {
                r#type: "local".to_string(),
                base_path: PathBuf::from("./data"),
            },
            git: GitConfig {
                auto_init: true,
                default_branch: "main".to_string(),
            },
        }
    }
}

/// 加载配置
pub fn load_config() -> anyhow::Result<AppConfig> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name("config/config"))
        .add_source(config::File::with_name("config/config.dev").required(false))
        .add_source(config::Environment::with_prefix("APP"))
        .build()?;

    let config: AppConfig = settings.try_deserialize()?;
    Ok(config)
}
