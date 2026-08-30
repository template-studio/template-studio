use crate::database::import::{
    fetch_mysql_tables, fetch_postgresql_tables, fetch_sqlite_tables, import_single_table,
};
use crate::database::TestConnectionParams;
use crate::ddl::{generate_create_table_ddl, PushColumnDef};
use crate::state::{BrowserPoolCache, DbState};
use sqlx::Row;

/// 列出数据库中的表
#[tauri::command]
pub async fn cmd_list_database_tables(params: TestConnectionParams) -> Result<String, String> {
    use sqlx::mysql::MySqlPool;
    use sqlx::postgres::PgPool;
    use sqlx::sqlite::SqlitePool;

    let db_type = params.type_.clone();

    match db_type.as_str() {
        "mysql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(3306);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_default();

            // 如果没有指定数据库，先连接到服务器获取数据库列表
            if database.is_empty() {
                let url = format!("mysql://{}:{}@{}:{}", username, password, host, port);
                let pool = MySqlPool::connect(&url)
                    .await
                    .map_err(|e| format!("连接失败: {}", e))?;

                let rows = sqlx::query("SHOW DATABASES")
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| format!("查询失败: {}", e))?;

                let databases: Vec<String> =
                    rows.iter().map(|row| row.get::<String, _>(0)).collect();

                pool.close().await;
                return serde_json::to_string(&databases).map_err(|e| format!("序列化失败: {}", e));
            }

            let url = format!(
                "mysql://{}:{}@{}:{}/{}",
                username, password, host, port, database
            );
            let pool = MySqlPool::connect(&url)
                .await
                .map_err(|e| format!("连接失败: {}", e))?;

            let rows = sqlx::query("SHOW TABLES")
                .fetch_all(&pool)
                .await
                .map_err(|e| format!("查询失败: {}", e))?;

            let tables: Vec<String> = rows.iter().map(|row| row.get::<String, _>(0)).collect();

            pool.close().await;
            serde_json::to_string(&tables).map_err(|e| format!("序列化失败: {}", e))
        }
        "postgresql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(5432);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_else(|| "postgres".to_string());

            let url = format!(
                "postgres://{}:{}@{}:{}/{}",
                username, password, host, port, database
            );
            let pool = PgPool::connect(&url)
                .await
                .map_err(|e| format!("连接失败: {}", e))?;

            let rows = sqlx::query(
                "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'",
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("查询失败: {}", e))?;

            let tables: Vec<String> = rows.iter().map(|row| row.get::<String, _>(0)).collect();

            pool.close().await;
            serde_json::to_string(&tables).map_err(|e| format!("序列化失败: {}", e))
        }
        "sqlite" => {
            let sqlite_file = params.sqlite_file.unwrap_or_default();
            let url = format!("sqlite:{}", sqlite_file);
            let pool = SqlitePool::connect(&url)
                .await
                .map_err(|e| format!("连接失败: {}", e))?;

            let rows = sqlx::query(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("查询失败: {}", e))?;

            let tables: Vec<String> = rows.iter().map(|row| row.get::<String, _>(0)).collect();

            pool.close().await;
            serde_json::to_string(&tables).map_err(|e| format!("序列化失败: {}", e))
        }
        _ => Err(format!("不支持的数据库类型: {}", db_type)),
    }
}

/// 获取表的列信息
#[tauri::command]
pub async fn cmd_get_table_columns(
    params: TestConnectionParams,
    table_name: String,
    pool_cache: tauri::State<'_, BrowserPoolCache>,
) -> Result<String, String> {
    let db_type = params.type_.clone();

    match db_type.as_str() {
        "mysql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(3306);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_default();

            let url = format!(
                "mysql://{}:{}@{}:{}/{}",
                username, password, host, port, database
            );
            let pool = pool_cache.get_or_create_mysql(&url).await?;

            let rows = sqlx::query(
                "SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE, COLUMN_KEY, COLUMN_DEFAULT, COLUMN_COMMENT \
                 FROM INFORMATION_SCHEMA.COLUMNS \
                 WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? \
                 ORDER BY ORDINAL_POSITION"
            )
            .bind(&database)
            .bind(&table_name)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("查询失败: {}", e))?;

            let columns: Vec<serde_json::Value> = rows
                .iter()
                .map(|row| {
                    serde_json::json!({
                        "name": row.get::<String, _>(0),
                        "type": row.get::<String, _>(1),
                        "nullable": row.get::<String, _>(2) == "YES",
                        "key": row.get::<String, _>(3),
                        "default": row.get::<Option<String>, _>(4),
                        "comment": row.get::<Option<String>, _>(5)
                    })
                })
                .collect();

            serde_json::to_string(&columns).map_err(|e| format!("序列化失败: {}", e))
        }
        "postgresql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(5432);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_else(|| "postgres".to_string());

            let url = format!(
                "postgres://{}:{}@{}:{}/{}",
                username, password, host, port, database
            );
            let pool = pool_cache.get_or_create_pg(&url).await?;

            let rows = sqlx::query(
                "SELECT column_name, data_type, is_nullable, \
                 CASE WHEN constraint_type = 'PRIMARY KEY' THEN 'PRI' ELSE '' END as column_key, \
                 column_default, '' as column_comment \
                 FROM information_schema.columns c \
                 LEFT JOIN information_schema.key_column_usage k \
                   ON c.table_name = k.table_name AND c.column_name = k.column_name \
                 LEFT JOIN information_schema.table_constraints t \
                   ON k.constraint_name = t.constraint_name AND t.constraint_type = 'PRIMARY KEY' \
                 WHERE c.table_schema = 'public' AND c.table_name = $1 \
                 ORDER BY c.ordinal_position",
            )
            .bind(&table_name)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("查询失败: {}", e))?;

            let columns: Vec<serde_json::Value> = rows
                .iter()
                .map(|row| {
                    serde_json::json!({
                        "name": row.get::<String, _>(0),
                        "type": row.get::<String, _>(1),
                        "nullable": row.get::<String, _>(2) == "YES",
                        "key": row.get::<String, _>(3),
                        "default": row.get::<Option<String>, _>(4),
                        "comment": row.get::<String, _>(5)
                    })
                })
                .collect();

            serde_json::to_string(&columns).map_err(|e| format!("序列化失败: {}", e))
        }
        "sqlite" => {
            let sqlite_file = params.sqlite_file.unwrap_or_default();
            let url = format!("sqlite:{}", sqlite_file);
            let pool = pool_cache.get_or_create_sqlite(&url).await?;

            let rows = sqlx::query(&format!(
                "PRAGMA table_info('{}')",
                table_name.replace('\'', "''")
            ))
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("查询失败: {}", e))?;

            let columns: Vec<serde_json::Value> = rows
                .iter()
                .map(|row| {
                    serde_json::json!({
                        "name": row.get::<String, _>(1),
                        "type": row.get::<String, _>(2),
                        "nullable": !row.get::<bool, _>(3),
                        "key": if row.get::<bool, _>(5) { "PRI" } else { "" },
                        "default": row.get::<Option<String>, _>(4),
                        "comment": ""
                    })
                })
                .collect();

            serde_json::to_string(&columns).map_err(|e| format!("序列化失败: {}", e))
        }
        _ => Err(format!("不支持的数据库类型: {}", db_type)),
    }
}

/// 在远程数据库执行 SQL（用于同步本地表到远程）
#[tauri::command]
pub async fn cmd_execute_sql_on_remote(
    params: TestConnectionParams,
    sql: String,
    pool_cache: tauri::State<'_, BrowserPoolCache>,
) -> Result<String, String> {
    let db_type = params.type_.clone();

    match db_type.as_str() {
        "mysql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(3306);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_default();
            let url = format!(
                "mysql://{}:{}@{}:{}/{}",
                username, password, host, port, database
            );
            let pool = pool_cache.get_or_create_mysql(&url).await?;
            sqlx::query(&sql)
                .execute(&pool)
                .await
                .map_err(|e| format!("执行失败: {}", e))?;
        }
        "postgresql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(5432);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_else(|| "postgres".to_string());
            let url = format!(
                "postgres://{}:{}@{}:{}/{}",
                username, password, host, port, database
            );
            let pool = pool_cache.get_or_create_pg(&url).await?;
            sqlx::query(&sql)
                .execute(&pool)
                .await
                .map_err(|e| format!("执行失败: {}", e))?;
        }
        "sqlite" => {
            let sqlite_file = params.sqlite_file.unwrap_or_default();
            let url = format!("sqlite:{}", sqlite_file);
            let pool = pool_cache.get_or_create_sqlite(&url).await?;
            sqlx::query(&sql)
                .execute(&pool)
                .await
                .map_err(|e| format!("执行失败: {}", e))?;
        }
        _ => return Err(format!("不支持的数据库类型: {}", db_type)),
    }

    Ok("ok".to_string())
}

/// 推送表结构到远程数据库
#[tauri::command]
pub async fn cmd_push_table_to_remote(
    params: TestConnectionParams,
    table_name: String,
    table_engine: Option<String>,
    table_comment: Option<String>,
    columns: Vec<PushColumnDef>,
    pool_cache: tauri::State<'_, BrowserPoolCache>,
) -> Result<String, String> {
    let sql = generate_create_table_ddl(
        &table_name,
        table_engine.as_deref(),
        table_comment.as_deref(),
        &columns,
    );
    let db_type = params.type_.clone();
    match db_type.as_str() {
        "mysql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(3306);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_default();
            let url = format!(
                "mysql://{}:{}@{}:{}/{}",
                username, password, host, port, database
            );
            let pool = pool_cache.get_or_create_mysql(&url).await?;
            sqlx::query(&sql)
                .execute(&pool)
                .await
                .map_err(|e| format!("执行失败: {}", e))?;
        }
        "postgresql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(5432);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_else(|| "postgres".to_string());
            let url = format!(
                "postgres://{}:{}@{}:{}/{}",
                username, password, host, port, database
            );
            let pool = pool_cache.get_or_create_pg(&url).await?;
            sqlx::query(&sql)
                .execute(&pool)
                .await
                .map_err(|e| format!("执行失败: {}", e))?;
        }
        "sqlite" => {
            let sqlite_file = params.sqlite_file.unwrap_or_default();
            let url = format!("sqlite:{}", sqlite_file);
            let pool = pool_cache.get_or_create_sqlite(&url).await?;
            sqlx::query(&sql)
                .execute(&pool)
                .await
                .map_err(|e| format!("执行失败: {}", e))?;
        }
        _ => return Err(format!("不支持的数据库类型: {}", db_type)),
    }

    Ok("ok".to_string())
}

/// 查询表数据（带分页，使用连接池缓存 + 快速行数估算 + 并行查询）
#[tauri::command]
pub async fn cmd_query_table_data(
    params: TestConnectionParams,
    table_name: String,
    limit: i64,
    offset: i64,
    pool_cache: tauri::State<'_, BrowserPoolCache>,
) -> Result<String, String> {
    let db_type = params.type_.clone();

    match db_type.as_str() {
        "mysql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(3306);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_default();

            let url = format!(
                "mysql://{}:{}@{}:{}/{}",
                username, password, host, port, database
            );
            let pool = pool_cache.get_or_create_mysql(&url).await?;

            // 快速行数估算（MySQL: 从 information_schema 获取估计值，避免全表扫描）
            let est_sql = "SELECT TABLE_ROWS FROM INFORMATION_SCHEMA.TABLES \
                           WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?";
            let est_total: i64 = sqlx::query_scalar(est_sql)
                .bind(&database)
                .bind(&table_name)
                .fetch_one(&pool)
                .await
                .unwrap_or(0);

            // 估算值为 0 时回退到 COUNT(*)（小表或统计信息未更新）
            let total = if est_total <= 0 {
                let count_sql = format!("SELECT COUNT(*) FROM `{}`", table_name.replace('`', "``"));
                sqlx::query_scalar(&count_sql)
                    .fetch_one(&pool)
                    .await
                    .unwrap_or(0)
            } else {
                est_total
            };

            // 获取列名
            let col_rows = sqlx::query(
                "SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS \
                 WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? ORDER BY ORDINAL_POSITION",
            )
            .bind(&database)
            .bind(&table_name)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("获取列信息失败: {}", e))?;

            let columns: Vec<String> = col_rows.iter().map(|row| row.get::<String, _>(0)).collect();

            if columns.is_empty() {
                return serde_json::to_string(&serde_json::json!({
                    "columns": [], "rows": [], "total": total
                }))
                .map_err(|e| format!("序列化失败: {}", e));
            }

            // 用 CAST 将所有列转为字符串，避免类型转换问题
            let cast_cols: Vec<String> = columns
                .iter()
                .map(|c| {
                    format!(
                        "CAST(`{}` AS CHAR) AS `{}`",
                        c.replace('`', "``"),
                        c.replace('`', "``")
                    )
                })
                .collect();
            let data_sql = format!(
                "SELECT {} FROM `{}` LIMIT {} OFFSET {}",
                cast_cols.join(", "),
                table_name.replace('`', "``"),
                limit,
                offset
            );
            let rows = sqlx::query(&data_sql)
                .fetch_all(&pool)
                .await
                .map_err(|e| format!("查询失败: {}", e))?;

            let data: Vec<Vec<serde_json::Value>> = rows
                .iter()
                .map(|row| {
                    (0..columns.len())
                        .map(|i| match row.try_get::<Option<String>, _>(i) {
                            Ok(Some(v)) => serde_json::Value::String(v),
                            _ => serde_json::Value::Null,
                        })
                        .collect()
                })
                .collect();

            serde_json::to_string(&serde_json::json!({
                "columns": columns,
                "rows": data,
                "total": total
            }))
            .map_err(|e| format!("序列化失败: {}", e))
        }
        "postgresql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(5432);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_else(|| "postgres".to_string());

            let url = format!(
                "postgres://{}:{}@{}:{}/{}",
                username, password, host, port, database
            );
            let pool = pool_cache.get_or_create_pg(&url).await?;

            // 快速行数估算（PostgreSQL: pg_class.reltuples，避免 COUNT(*) 全表扫描）
            let est_total: i64 = sqlx::query_scalar(
                "SELECT COALESCE(reltuples::bigint, 0) FROM pg_class WHERE relname = $1",
            )
            .bind(&table_name)
            .fetch_one(&pool)
            .await
            .unwrap_or(0);

            // 估算值为 0 时回退到 COUNT(*)
            let total = if est_total <= 0 {
                let count_sql = format!(
                    "SELECT COUNT(*) FROM \"{}\"",
                    table_name.replace('"', "\"\"")
                );
                sqlx::query_scalar(&count_sql)
                    .fetch_one(&pool)
                    .await
                    .unwrap_or(0)
            } else {
                est_total
            };

            // 获取列名
            let col_rows = sqlx::query(
                "SELECT column_name FROM information_schema.columns \
                 WHERE table_schema = 'public' AND table_name = $1 ORDER BY ordinal_position",
            )
            .bind(&table_name)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("获取列信息失败: {}", e))?;

            let columns: Vec<String> = col_rows.iter().map(|row| row.get::<String, _>(0)).collect();

            if columns.is_empty() {
                return serde_json::to_string(&serde_json::json!({
                    "columns": [], "rows": [], "total": total
                }))
                .map_err(|e| format!("序列化失败: {}", e));
            }

            // 用 CAST 将所有列转为 TEXT
            let cast_cols: Vec<String> = columns
                .iter()
                .map(|c| {
                    format!(
                        "\"{}\"::TEXT AS \"{}\"",
                        c.replace('"', "\"\""),
                        c.replace('"', "\"\"")
                    )
                })
                .collect();
            let data_sql = format!(
                "SELECT {} FROM \"{}\" LIMIT {} OFFSET {}",
                cast_cols.join(", "),
                table_name.replace('"', "\"\""),
                limit,
                offset
            );
            let rows = sqlx::query(&data_sql)
                .fetch_all(&pool)
                .await
                .map_err(|e| format!("查询失败: {}", e))?;

            let data: Vec<Vec<serde_json::Value>> = rows
                .iter()
                .map(|row| {
                    (0..columns.len())
                        .map(|i| match row.try_get::<Option<String>, _>(i) {
                            Ok(Some(v)) => serde_json::Value::String(v),
                            _ => serde_json::Value::Null,
                        })
                        .collect()
                })
                .collect();

            serde_json::to_string(&serde_json::json!({
                "columns": columns,
                "rows": data,
                "total": total
            }))
            .map_err(|e| format!("序列化失败: {}", e))
        }
        "sqlite" => {
            let sqlite_file = params.sqlite_file.unwrap_or_default();
            let url = format!("sqlite:{}", sqlite_file);
            let pool = pool_cache.get_or_create_sqlite(&url).await?;

            let count_sql = format!(
                "SELECT COUNT(*) FROM \"{}\"",
                table_name.replace('"', "\"\"")
            );
            let total: i64 = sqlx::query_scalar(&count_sql)
                .fetch_one(&pool)
                .await
                .map_err(|e| format!("查询总数失败: {}", e))?;

            // 获取列名
            let col_rows = sqlx::query(&format!(
                "PRAGMA table_info('{}')",
                table_name.replace('\'', "''")
            ))
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("获取列信息失败: {}", e))?;

            let columns: Vec<String> = col_rows.iter().map(|row| row.get::<String, _>(1)).collect();

            if columns.is_empty() {
                return serde_json::to_string(&serde_json::json!({
                    "columns": [], "rows": [], "total": total
                }))
                .map_err(|e| format!("序列化失败: {}", e));
            }

            // 用 CAST 将所有列转为 TEXT
            let cast_cols: Vec<String> = columns
                .iter()
                .map(|c| {
                    format!(
                        "CAST(\"{}\" AS TEXT) AS \"{}\"",
                        c.replace('"', "\"\""),
                        c.replace('"', "\"\"")
                    )
                })
                .collect();
            let data_sql = format!(
                "SELECT {} FROM \"{}\" LIMIT {} OFFSET {}",
                cast_cols.join(", "),
                table_name.replace('"', "\"\""),
                limit,
                offset
            );
            let rows = sqlx::query(&data_sql)
                .fetch_all(&pool)
                .await
                .map_err(|e| format!("查询失败: {}", e))?;

            let data: Vec<Vec<serde_json::Value>> = rows
                .iter()
                .map(|row| {
                    (0..columns.len())
                        .map(|i| match row.try_get::<Option<String>, _>(i) {
                            Ok(Some(v)) => serde_json::Value::String(v),
                            _ => serde_json::Value::Null,
                        })
                        .collect()
                })
                .collect();

            serde_json::to_string(&serde_json::json!({
                "columns": columns,
                "rows": data,
                "total": total
            }))
            .map_err(|e| format!("序列化失败: {}", e))
        }
        _ => Err(format!("不支持的数据库类型: {}", db_type)),
    }
}

/// 获取数据库连接状态信息
#[tauri::command]
pub async fn cmd_get_connection_status(
    params: TestConnectionParams,
    pool_cache: tauri::State<'_, BrowserPoolCache>,
) -> Result<String, String> {
    let db_type = params.type_.clone();
    let start = std::time::Instant::now();

    match db_type.as_str() {
        "mysql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(3306);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_default();

            // 构建连接 URL（无数据库时不带路径）
            let url = if database.is_empty() {
                format!("mysql://{}:{}@{}:{}", username, password, host, port)
            } else {
                format!(
                    "mysql://{}:{}@{}:{}/{}",
                    username, password, host, port, database
                )
            };
            let pool = pool_cache.get_or_create_mysql(&url).await?;
            let latency = start.elapsed().as_millis();

            // 获取服务器版本
            let version: String = sqlx::query_scalar("SELECT VERSION()")
                .fetch_one(&pool)
                .await
                .unwrap_or_else(|_| "未知".to_string());

            // 获取活跃连接数
            let active_connections: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM information_schema.processlist")
                    .fetch_one(&pool)
                    .await
                    .unwrap_or(0);

            // 获取最大连接数
            let max_connections: i64 = sqlx::query("SHOW VARIABLES LIKE 'max_connections'")
                .fetch_optional(&pool)
                .await
                .ok()
                .flatten()
                .map(|row| row.get::<String, _>(1).parse::<i64>().unwrap_or(0))
                .unwrap_or(0);

            // 获取运行时间
            let uptime: i64 = sqlx::query("SHOW VARIABLES LIKE 'uptime'")
                .fetch_optional(&pool)
                .await
                .ok()
                .flatten()
                .map(|row| row.get::<String, _>(1).parse::<i64>().unwrap_or(0))
                .unwrap_or(0);

            // 数据库特定信息（仅当指定了数据库时查询）
            let (db_size, table_count) = if !database.is_empty() {
                let size: String = sqlx::query_scalar(
                    "SELECT CONCAT(ROUND(SUM(data_length + index_length) / 1024 / 1024, 2), ' MB') \
                     FROM information_schema.tables WHERE table_schema = ?"
                )
                .bind(&database)
                .fetch_one(&pool)
                .await
                .unwrap_or_else(|_| "未知".to_string());

                let count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = ?",
                )
                .bind(&database)
                .fetch_one(&pool)
                .await
                .unwrap_or(0);

                (size, count)
            } else {
                ("未知".to_string(), 0)
            };

            serde_json::to_string(&serde_json::json!({
                "status": "connected",
                "type": "MySQL",
                "version": version,
                "host": host,
                "port": port,
                "database": if database.is_empty() { None::<String> } else { Some(database) },
                "latency_ms": latency,
                "active_connections": active_connections,
                "max_connections": max_connections,
                "uptime_seconds": uptime,
                "database_size": db_size,
                "table_count": table_count,
                "pool_size": pool.size(),
                "pool_idle": pool.num_idle(),
            }))
            .map_err(|e| format!("序列化失败: {}", e))
        }
        "postgresql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(5432);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_else(|| "postgres".to_string());

            let url = format!(
                "postgres://{}:{}@{}:{}/{}",
                username, password, host, port, database
            );
            let pool = pool_cache.get_or_create_pg(&url).await?;
            let latency = start.elapsed().as_millis();

            let version: String = sqlx::query_scalar("SELECT version()")
                .fetch_one(&pool)
                .await
                .unwrap_or_else(|_| "未知".to_string());

            let active_connections: i64 =
                sqlx::query_scalar("SELECT count(*) FROM pg_stat_activity WHERE state = 'active'")
                    .fetch_one(&pool)
                    .await
                    .unwrap_or(0);

            let max_connections: i64 = sqlx::query_scalar::<_, String>("SHOW max_connections")
                .fetch_one(&pool)
                .await
                .ok()
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(0);

            let db_size: String = sqlx::query_scalar("SELECT pg_size_pretty(pg_database_size($1))")
                .bind(&database)
                .fetch_one(&pool)
                .await
                .unwrap_or_else(|_| "未知".to_string());

            let table_count: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM information_schema.tables WHERE table_schema = 'public'",
            )
            .fetch_one(&pool)
            .await
            .unwrap_or(0);

            serde_json::to_string(&serde_json::json!({
                "status": "connected",
                "type": "PostgreSQL",
                "version": version,
                "host": host,
                "port": port,
                "database": database,
                "latency_ms": latency,
                "active_connections": active_connections,
                "max_connections": max_connections,
                "database_size": db_size,
                "table_count": table_count,
                "pool_size": pool.size(),
                "pool_idle": pool.num_idle(),
            }))
            .map_err(|e| format!("序列化失败: {}", e))
        }
        "sqlite" => {
            let sqlite_file = params.sqlite_file.unwrap_or_default();
            let url = format!("sqlite:{}", sqlite_file);
            let pool = pool_cache.get_or_create_sqlite(&url).await?;
            let latency = start.elapsed().as_millis();

            let version: String = sqlx::query_scalar("SELECT sqlite_version()")
                .fetch_one(&pool)
                .await
                .unwrap_or_else(|_| "未知".to_string());

            let table_count: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
            )
            .fetch_one(&pool)
            .await
            .unwrap_or(0);

            // 获取文件大小
            let file_size = std::fs::metadata(&sqlite_file)
                .map(|m| {
                    let bytes = m.len();
                    if bytes > 1024 * 1024 {
                        format!("{:.2} MB", bytes as f64 / 1024.0 / 1024.0)
                    } else if bytes > 1024 {
                        format!("{:.2} KB", bytes as f64 / 1024.0)
                    } else {
                        format!("{} B", bytes)
                    }
                })
                .unwrap_or_else(|_| "未知".to_string());

            serde_json::to_string(&serde_json::json!({
                "status": "connected",
                "type": "SQLite",
                "version": version,
                "file": sqlite_file,
                "latency_ms": latency,
                "database_size": file_size,
                "table_count": table_count,
                "pool_size": pool.size(),
                "pool_idle": pool.num_idle(),
            }))
            .map_err(|e| format!("序列化失败: {}", e))
        }
        _ => Err(format!("不支持的数据库类型: {}", db_type)),
    }
}

// ===== 表结构读取和导入命令 =====

/// 读取 MySQL 数据库的表列表
#[tauri::command]
pub async fn cmd_fetch_mysql_tables(
    datasource_id: i64,
    database_name: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    // 获取数据源信息
    let datasource = db
        .get_datasource(datasource_id)
        .await
        .map_err(|e| format!("获取数据源失败: {}", e))?
        .ok_or_else(|| "数据源不存在".to_string())?;

    fetch_mysql_tables(db.pool(), &datasource, &database_name).await
}

/// 读取 PostgreSQL 数据库的表列表
#[tauri::command]
pub async fn cmd_fetch_postgresql_tables(
    datasource_id: i64,
    database_name: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    // 获取数据源信息
    let datasource = db
        .get_datasource(datasource_id)
        .await
        .map_err(|e| format!("获取数据源失败: {}", e))?
        .ok_or_else(|| "数据源不存在".to_string())?;

    fetch_postgresql_tables(db.pool(), &datasource, &database_name).await
}

/// 读取 SQLite 数据库的表列表
#[tauri::command]
pub async fn cmd_fetch_sqlite_tables(
    datasource_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    // 获取数据源信息
    let datasource = db
        .get_datasource(datasource_id)
        .await
        .map_err(|e| format!("获取数据源失败: {}", e))?
        .ok_or_else(|| "数据源不存在".to_string())?;

    fetch_sqlite_tables(db.pool(), &datasource).await
}

/// 导入单个表
#[tauri::command]
pub async fn cmd_import_single_table(
    project_id: i64,
    datasource_id: i64,
    database_name: String,
    table_name: String,
    table_comment: Option<String>,
    table_type: String,
    engine: Option<String>,
    row_count: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    // 获取数据源信息
    let datasource = db
        .get_datasource(datasource_id)
        .await
        .map_err(|e| format!("获取数据源失败: {}", e))?
        .ok_or_else(|| "数据源不存在".to_string())?;

    import_single_table(
        db.pool(),
        project_id,
        &datasource,
        &database_name,
        &table_name,
        table_comment.as_deref(),
        &table_type,
        engine.as_deref(),
        row_count,
    )
    .await
}
