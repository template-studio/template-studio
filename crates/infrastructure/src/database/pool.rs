use crate::config::settings::DatabaseConfig;
use sqlx::{MySql, Pool};

/// 数据库连接池管理器
pub struct DatabasePool {
    pool: Pool<MySql>,
}

impl DatabasePool {
    /// 创建新的连接池
    pub async fn new(config: &DatabaseConfig) -> anyhow::Result<Self> {
        let pool = sqlx::MySqlPool::connect(&config.url).await?;

        Ok(Self { pool })
    }

    /// 获取连接池
    pub fn get_pool(&self) -> &Pool<MySql> {
        &self.pool
    }

    /// 运行数据库迁移
    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        // 检查var_preset表是否存在
        let table_exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM information_schema.tables WHERE table_schema = DATABASE() AND table_name = 'var_preset'"
        )
        .fetch_one(&self.pool)
        .await?;

        if table_exists {
            tracing::info!("var_preset表已存在，跳过创建");
        } else {
            tracing::warn!("var_preset表不存在，请手动创建表结构");
        }

        Ok(())
    }

    /// 健康检查
    pub async fn health_check(&self) -> bool {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .is_ok()
    }

    /// 关闭连接池
    pub async fn close(self) {
        self.pool.close().await;
    }
}

