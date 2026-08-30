use super::{Database, Datasource, TestConnectionParams};
use sqlx::Row;

/// 凭据加密错误 → sqlx::Error（数据库层的存储加密属 IO 范畴错误）
fn sqlx_encrypt_err(e: String) -> sqlx::Error {
    sqlx::Error::Io(std::io::Error::other(format!("凭据加密失败: {}", e)))
}

impl Database {
    /// ===== 数据源操作 =====
    /// 创建数据源
    pub async fn create_datasource(
        &self,
        name: &str,
        type_: &str,
        host: Option<&str>,
        port: Option<u16>,
        username: Option<&str>,
        password: Option<&str>,
        database: Option<&str>,
        sqlite_file: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO datasources (name, type, host, port, username, password, database, sqlite_file)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
        )
        .bind(name)
        .bind(type_)
        .bind(host)
        .bind(port.map(|p| p as i32))
        .bind(username)
        .bind(password.map(crate::database::credential::encrypt).transpose().map_err(sqlx_encrypt_err)?)
        .bind(database)
        .bind(sqlite_file)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 获取所有数据源
    pub async fn get_all_datasources(&self) -> Result<Vec<Datasource>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, name, type, host, port, username, password, database, sqlite_file, is_active, created_at, updated_at
             FROM datasources
             ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let datasources = rows
            .into_iter()
            .map(|row| Datasource {
                id: row.get("id"),
                name: row.get("name"),
                type_: row.get("type"),
                host: row.try_get("host").ok(),
                port: row.try_get("port").ok(),
                username: row.try_get("username").ok(),
                password: row
                    .try_get::<Option<String>, _>("password")
                    .ok()
                    .flatten()
                    .map(|p| crate::database::credential::decrypt(&p).unwrap_or_default()),
                database: row.try_get("database").ok(),
                sqlite_file: row.try_get("sqlite_file").ok(),
                is_active: row.get::<i32, _>("is_active") == 1,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(datasources)
    }

    /// 根据 ID 获取单个数据源
    pub async fn get_datasource(&self, id: i64) -> Result<Option<Datasource>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, type, host, port, username, password, database, sqlite_file, is_active, created_at, updated_at
             FROM datasources
             WHERE id = ?1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(Datasource {
                id: row.get("id"),
                name: row.get("name"),
                type_: row.get("type"),
                host: row.try_get("host").ok(),
                port: row.try_get("port").ok(),
                username: row.try_get("username").ok(),
                password: row
                    .try_get::<Option<String>, _>("password")
                    .ok()
                    .flatten()
                    .map(|p| crate::database::credential::decrypt(&p).unwrap_or_default()),
                database: row.try_get("database").ok(),
                sqlite_file: row.try_get("sqlite_file").ok(),
                is_active: row.get::<i32, _>("is_active") == 1,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })),
            None => Ok(None),
        }
    }

    /// 更新数据源
    pub async fn update_datasource(
        &self,
        id: i64,
        name: &str,
        type_: &str,
        host: Option<&str>,
        port: Option<u16>,
        username: Option<&str>,
        password: Option<&str>,
        database: Option<&str>,
        sqlite_file: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE datasources
             SET name = ?1, type = ?2, host = ?3, port = ?4, username = ?5, password = ?6, database = ?7, sqlite_file = ?8, updated_at = datetime('now')
             WHERE id = ?9"
        )
        .bind(name)
        .bind(type_)
        .bind(host)
        .bind(port.map(|p| p as i32))
        .bind(username)
        .bind(password.map(crate::database::credential::encrypt).transpose().map_err(sqlx_encrypt_err)?)
        .bind(database)
        .bind(sqlite_file)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除数据源
    pub async fn delete_datasource(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM datasources WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 测试数据库连接
    pub async fn test_datasource_connection(
        params: TestConnectionParams,
    ) -> Result<String, String> {
        match params.type_.as_str() {
            "mysql" => {
                // MySQL 连接时可以不指定数据库
                let database = params.database.as_deref().unwrap_or("");

                let connection_string = if database.is_empty() {
                    // 不指定数据库，只连接服务器
                    format!(
                        "mysql://{}:{}@{}:{}",
                        params.username.as_deref().unwrap_or(""),
                        params.password.as_deref().unwrap_or(""),
                        params.host.as_deref().unwrap_or("localhost"),
                        params.port.unwrap_or(3306)
                    )
                } else {
                    // 指定数据库
                    format!(
                        "mysql://{}:{}@{}:{}/{}",
                        params.username.as_deref().unwrap_or(""),
                        params.password.as_deref().unwrap_or(""),
                        params.host.as_deref().unwrap_or("localhost"),
                        params.port.unwrap_or(3306),
                        database
                    )
                };

                sqlx::mysql::MySqlPool::connect(&connection_string)
                    .await
                    .map(|_| {
                        if database.is_empty() {
                            "MySQL 服务器连接成功".to_string()
                        } else {
                            format!("MySQL 数据库 '{}' 连接成功", database)
                        }
                    })
                    .map_err(|e| format!("MySQL 连接失败: {}", e))
            }
            "postgresql" => {
                // PostgreSQL 需要指定初始数据库（通常使用 postgres）
                let database = params.database.as_deref().unwrap_or("postgres");

                let connection_string = format!(
                    "postgresql://{}:{}@{}:{}/{}",
                    params.username.as_deref().unwrap_or(""),
                    params.password.as_deref().unwrap_or(""),
                    params.host.as_deref().unwrap_or("localhost"),
                    params.port.unwrap_or(5432),
                    database
                );

                sqlx::postgres::PgPool::connect(&connection_string)
                    .await
                    .map(|_| format!("PostgreSQL 数据库 '{}' 连接成功", database))
                    .map_err(|e| format!("PostgreSQL 连接失败: {}", e))
            }
            "sqlite" => {
                // 检查文件是否存在
                let sqlite_file = params
                    .sqlite_file
                    .as_deref()
                    .ok_or_else(|| "SQLite 文件路径未指定".to_string())?;

                if !std::path::Path::new(sqlite_file).exists() {
                    return Err(format!("SQLite 文件不存在: {}", sqlite_file));
                }

                sqlx::sqlite::SqlitePool::connect(&format!("sqlite:{}", sqlite_file))
                    .await
                    .map(|_| "SQLite 连接成功".to_string())
                    .map_err(|e| format!("SQLite 连接失败: {}", e))
            }
            _ => Err(format!("不支持的数据库类型: {}", params.type_)),
        }
    }
}
