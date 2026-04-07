#![allow(clippy::too_many_arguments, clippy::type_complexity)]

pub mod models;
pub use models::*;

mod project;
mod datasource;
mod table;
mod column;
mod language;
mod type_mapping;
mod preferences;
mod ai;
pub(crate) mod import;
mod migrations;

use sqlx::SqlitePool;
use std::path::PathBuf;
use std::fs;
use dirs::home_dir;

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
        let db_path = get_database_path()
            .map_err(sqlx::Error::Io)?;

        println!("初始化数据库: {:?}", db_path);
        println!("数据库文件存在: {}", db_path.exists());

        // 创建数据库连接字符串
        // 在 Windows 上，需要使用 sqlite:// 前缀，并确保路径使用正斜杠
        // mode=rwc 表示 read-write-create，允许创建数据库文件
        let path_str = db_path.to_string_lossy().replace('\\', "/");
        let connection_string = format!("sqlite://{}?mode=rwc", path_str);

        println!("连接字符串: {}", connection_string);

        // 创建连接池
        let pool = SqlitePool::connect(&connection_string).await?;

        // 设置数据库优化参数（PRAGMA 语句）
        sqlx::query("PRAGMA journal_mode = WAL")       // Write-Ahead Logging 模式
            .execute(&pool)
            .await?;

        sqlx::query("PRAGMA synchronous = NORMAL")      // 正常同步模式
            .execute(&pool)
            .await?;

        sqlx::query("PRAGMA cache_size = -64000")       // 64MB 缓存
            .execute(&pool)
            .await?;

        sqlx::query("PRAGMA foreign_keys = ON")         // 启用外键约束
            .execute(&pool)
            .await?;

        sqlx::query("PRAGMA temp_store = MEMORY")       // 临时表存储在内存中
            .execute(&pool)
            .await?;

        let db = Database { pool };

        // 运行迁移
        db.run_migrations().await?;

        Ok(db)
    }

    /// 获取数据库连接池的引用
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
