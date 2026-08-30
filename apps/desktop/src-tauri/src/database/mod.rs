#![allow(clippy::too_many_arguments, clippy::type_complexity)]

pub mod models;
pub use models::*;

mod ai;
mod column;
mod datasource;
pub mod credential;
pub(crate) mod import;
mod language;
mod migrations;
mod preferences;
mod project;
mod table;
mod type_mapping;

use dirs::home_dir;
use sqlx::SqlitePool;
use std::fs;
use std::path::PathBuf;

/// 数据库路径
pub fn get_database_path() -> Result<PathBuf, std::io::Error> {
    // C:\Users\{user}\.cicbyte\template_studio\db\desktop.db
    let mut db_dir = home_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "无法获取用户主目录"))?;
    db_dir.push(".cicbyte");
    db_dir.push("template_studio");
    db_dir.push("db");

    // 确保目录存在
    fs::create_dir_all(&db_dir)?;

    db_dir.push("desktop.db");
    Ok(db_dir)
}

/// 数据库连接池
pub struct Database {
    pub(crate) pool: SqlitePool,
}

impl Database {
    /// 初始化数据库连接池（如果不存在则创建）
    pub async fn init() -> Result<Self, sqlx::Error> {
        let db_path = get_database_path().map_err(sqlx::Error::Io)?;

        println!("初始化数据库: {:?}", db_path);
        println!("数据库文件存在: {}", db_path.exists());

        // PRAGMA 通过连接选项下发：pool.execute 只作用于单个连接，
        // synchronous/foreign_keys/cache_size 等按连接生效的参数会对其余连接失效。
        // busy_timeout 避免 WAL 下多连接并发写直接报 database is locked。
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .foreign_keys(true)
            .busy_timeout(std::time::Duration::from_secs(5))
            .pragma("cache_size", "-64000") // 64MB 页缓存
            .pragma("temp_store", "MEMORY"); // 临时表放内存

        // 创建连接池
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(10)
            .connect_with(options)
            .await?;

        let db = Database { pool };

        // 运行迁移
        db.run_migrations().await?;

        Ok(db)
    }

    /// 用已有连接池构造（测试用：对指定库文件执行迁移验证）
    pub fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 获取数据库连接池的引用
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
