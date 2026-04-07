use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use sqlx::mysql::MySqlPool;
use sqlx::postgres::PgPool;
use sqlx::sqlite::SqlitePool;

use crate::database::Database;

/// 数据库状态包装器，用于线程安全的异步访问
pub struct DbState(Arc<Database>);

impl DbState {
    pub fn new(database: Database) -> Self {
        DbState(Arc::new(database))
    }
}

impl Clone for DbState {
    fn clone(&self) -> Self {
        DbState(Arc::clone(&self.0))
    }
}

impl AsRef<Database> for DbState {
    fn as_ref(&self) -> &Database {
        &self.0
    }
}

// ===== 浏览器连接池缓存 =====

pub enum BrowserPool {
    MySQL(MySqlPool),
    PostgreSQL(PgPool),
    SQLite(SqlitePool),
}

pub struct BrowserPoolCache {
    pools: Mutex<HashMap<String, BrowserPool>>,
}

impl BrowserPoolCache {
    pub fn new() -> Self {
        Self { pools: Mutex::new(HashMap::new()) }
    }

    pub async fn get_or_create_mysql(&self, url: &str) -> Result<MySqlPool, String> {
        {
            let pools = self.pools.lock().unwrap();
            if let Some(BrowserPool::MySQL(pool)) = pools.get(url) {
                return Ok(pool.clone());
            }
        }
        let pool = MySqlPool::connect(url).await
            .map_err(|e| format!("连接失败: {}", e))?;
        let mut pools = self.pools.lock().unwrap();
        pools.insert(url.to_string(), BrowserPool::MySQL(pool.clone()));
        Ok(pool)
    }

    pub async fn get_or_create_pg(&self, url: &str) -> Result<PgPool, String> {
        {
            let pools = self.pools.lock().unwrap();
            if let Some(BrowserPool::PostgreSQL(pool)) = pools.get(url) {
                return Ok(pool.clone());
            }
        }
        let pool = PgPool::connect(url).await
            .map_err(|e| format!("连接失败: {}", e))?;
        let mut pools = self.pools.lock().unwrap();
        pools.insert(url.to_string(), BrowserPool::PostgreSQL(pool.clone()));
        Ok(pool)
    }

    pub async fn get_or_create_sqlite(&self, url: &str) -> Result<SqlitePool, String> {
        {
            let pools = self.pools.lock().unwrap();
            if let Some(BrowserPool::SQLite(pool)) = pools.get(url) {
                return Ok(pool.clone());
            }
        }
        let pool = SqlitePool::connect(url).await
            .map_err(|e| format!("连接失败: {}", e))?;
        let mut pools = self.pools.lock().unwrap();
        pools.insert(url.to_string(), BrowserPool::SQLite(pool.clone()));
        Ok(pool)
    }
}
