use crate::config::settings::DatabaseConfig;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};
use std::time::Duration;

/// 数据库连接池管理器
pub struct DatabasePool {
    pool: Pool<MySql>,
}

impl DatabasePool {
    /// 创建新的连接池
    ///
    /// 池参数显式化：`max_connections` 来自配置（此前 `MySqlPool::connect` 走默认值，
    /// 配置项被静默忽略）；获取连接 5s 超时避免请求无限堆积；连接 30min 最大寿命、
    /// 10min 空闲回收，降低长连接被服务端/中间设备静默断掉后首次使用的失败率。
    pub async fn new(config: &DatabaseConfig) -> anyhow::Result<Self> {
        let pool = MySqlPoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(Duration::from_secs(5))
            .max_lifetime(Duration::from_secs(30 * 60))
            .idle_timeout(Duration::from_secs(10 * 60))
            .connect(&config.url)
            .await?;

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

