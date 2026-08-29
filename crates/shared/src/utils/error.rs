use thiserror::Error;

/// 应用错误类型
#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("验证错误: {0}")]
    Validation(String),

    #[error("未找到记录: {0}")]
    NotFound(String),

    #[error("权限不足: {0}")]
    Forbidden(String),

    #[error("重复记录: {0}")]
    Duplicate(String),

    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git错误: {0}")]
    Git(String),

    #[error("模板渲染错误: {0}")]
    TemplateRender(String),

    #[error("配置错误: {0}")]
    Config(String),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("内部服务器错误: {0}")]
    Internal(String),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl AppError {
    /// 获取错误代码
    pub fn error_code(&self) -> i32 {
        match self {
            AppError::Database(_) => 5001,
            AppError::Validation(_) => 4000,
            AppError::NotFound(_) => 4004,
            AppError::Forbidden(_) => 4003,
            AppError::Duplicate(_) => 4009,
            AppError::Io(_) => 5002,
            AppError::Git(_) => 5003,
            AppError::TemplateRender(_) => 5004,
            AppError::Config(_) => 5005,
            AppError::Serialization(_) => 5006,
            AppError::Internal(_) => 5000,
        }
    }
}
