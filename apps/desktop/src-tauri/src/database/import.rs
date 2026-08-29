use super::Datasource;
use sqlx::{Row, SqlitePool};

// ===== 数据库特定的表信息结构 =====

#[derive(Debug, sqlx::FromRow)]
struct MySQLTableInfo {
    name: String,
    comment: Option<String>,
    table_type: String,
    engine: Option<String>,
    row_count: Option<i64>,
}

#[derive(Debug, sqlx::FromRow)]
struct MySQLColumnInfo {
    name: String,
    data_type: String,
    length: Option<i64>,
    is_nullable: Option<String>,
    column_key: Option<String>,
    default_value: Option<String>,
    comment: Option<String>,
    ordinal_position: Option<u32>,
}

#[derive(Debug, sqlx::FromRow)]
struct PgTableInfo {
    name: String,
    comment: Option<String>,
    table_type: String,
    engine: Option<String>,
    row_count: Option<i64>,
}

#[derive(Debug, sqlx::FromRow)]
struct PgColumnInfo {
    name: String,
    data_type: String,
    length: Option<i64>,
    is_nullable: Option<String>,
    default_value: Option<String>,
    comment: Option<String>,
    ordinal_position: Option<i32>,
}

#[derive(Debug, sqlx::FromRow)]
struct SQLiteTableInfo {
    name: String,
    table_type: String,
}

#[derive(Debug, sqlx::FromRow)]
struct SQLiteColumnInfo {
    cid: i32,
    name: String,
    data_type: String,
    not_null: i32,
    default_value: Option<String>,
    pk: i32,
}

/// 从数据源导入表结构
pub async fn import_tables_from_datasource(
    pool: &SqlitePool,
    project_id: i64,
    datasource_id: i64,
    database_name: &str,
) -> Result<String, String> {
    // 1. 获取数据源配置
    let datasource = sqlx::query_as::<_, Datasource>(
        "SELECT id, name, type, host, port, username, password, database, sqlite_file, is_active, created_at, updated_at
         FROM datasources WHERE id = ?1"
    )
    .bind(datasource_id)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("获取数据源失败: {}", e))?;

    // 2. 根据数据库类型执行导入
    match datasource.type_.as_str() {
        "mysql" => import_mysql_tables(pool, project_id, &datasource, database_name).await,
        "postgresql" => {
            import_postgresql_tables(pool, project_id, &datasource, database_name).await
        }
        "sqlite" => import_sqlite_tables(pool, project_id, &datasource).await,
        _ => Err(format!("不支持的数据库类型: {}", datasource.type_)),
    }
}

/// 从 MySQL 导入表结构
async fn import_mysql_tables(
    pool: &SqlitePool,
    project_id: i64,
    datasource: &Datasource,
    database_name: &str,
) -> Result<String, String> {
    // 连接到 MySQL 数据库
    let connection_string = format!(
        "mysql://{}:{}@{}:{}/{}",
        datasource.username.as_ref().ok_or("MySQL 用户名未配置")?,
        datasource.password.as_ref().ok_or("MySQL 密码未配置")?,
        datasource.host.as_ref().ok_or("MySQL 主机未配置")?,
        datasource.port.unwrap_or(3306),
        database_name
    );

    let mysql_pool = sqlx::mysql::MySqlPool::connect(&connection_string)
        .await
        .map_err(|e| format!("MySQL 连接失败: {}", e))?;

    // 查询所有表
    let tables: Vec<MySQLTableInfo> = sqlx::query_as(
        "SELECT
            TABLE_NAME as name,
            TABLE_COMMENT as comment,
            TABLE_TYPE as table_type,
            ENGINE as engine,
            TABLE_ROWS as row_count
        FROM information_schema.TABLES
        WHERE TABLE_SCHEMA = ? AND TABLE_TYPE IN ('BASE TABLE', 'VIEW')
        ORDER BY TABLE_NAME",
    )
    .bind(database_name)
    .fetch_all(&mysql_pool)
    .await
    .map_err(|e| format!("查询表列表失败: {}", e))?;

    if tables.is_empty() {
        return Ok("数据库中没有找到任何表".to_string());
    }

    let mut imported_count = 0;

    // 导入每张表
    for table_info in tables {
        // 创建表记录
        let table_id: i64 = sqlx::query_scalar(
            "INSERT INTO db_tables (project_id, name, comment, engine, table_type, row_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             RETURNING id",
        )
        .bind(project_id)
        .bind(&table_info.name)
        .bind(&table_info.comment)
        .bind(&table_info.engine)
        .bind(if table_info.table_type == "BASE TABLE" {
            "table"
        } else {
            "view"
        })
        .bind(table_info.row_count.map(|c| c as i32).unwrap_or(0))
        .fetch_one(pool)
        .await
        .map_err(|e| format!("创建表记录失败: {}", e))?;

        // 查询列信息
        let columns: Vec<MySQLColumnInfo> = sqlx::query_as(
            "SELECT
                COLUMN_NAME as name,
                DATA_TYPE as data_type,
                CHARACTER_MAXIMUM_LENGTH as length,
                IS_NULLABLE as is_nullable,
                COLUMN_KEY as column_key,
                COLUMN_DEFAULT as default_value,
                COLUMN_COMMENT as comment,
                ORDINAL_POSITION as ordinal_position
            FROM information_schema.COLUMNS
            WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?
            ORDER BY ORDINAL_POSITION",
        )
        .bind(database_name)
        .bind(&table_info.name)
        .fetch_all(&mysql_pool)
        .await
        .map_err(|e| format!("查询列信息失败: {}", e))?;

        // 插入列记录
        for col in columns {
            let is_primary_key = col.column_key.as_deref() == Some("PRI");
            let is_unique = col.column_key.as_deref() == Some("UNI")
                || col.column_key.as_deref() == Some("PRI");
            let is_nullable = col.is_nullable.as_deref() == Some("YES");

            sqlx::query(
                "INSERT INTO db_columns (table_id, name, data_type, length, is_nullable, is_primary_key, is_unique, default_value, comment, ordinal_position)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
            )
            .bind(table_id)
            .bind(&col.name)
            .bind(&col.data_type)
            .bind(col.length)
            .bind(is_nullable as i32)
            .bind(is_primary_key as i32)
            .bind(is_unique as i32)
            .bind(&col.default_value)
            .bind(&col.comment)
            .bind(col.ordinal_position.map(|p| p as i32).unwrap_or(0))
            .execute(pool)
            .await
            .map_err(|e| format!("创建列记录失败: {}", e))?;
        }

        // 更新表的列计数
        let column_count: i32 =
            sqlx::query_scalar("SELECT COUNT(*) FROM db_columns WHERE table_id = ?1")
                .bind(table_id)
                .fetch_one(pool)
                .await
                .map_err(|e| format!("查询列计数失败: {}", e))?;

        sqlx::query("UPDATE db_tables SET column_count = ?1 WHERE id = ?2")
            .bind(column_count)
            .bind(table_id)
            .execute(pool)
            .await
            .map_err(|e| format!("更新列计数失败: {}", e))?;

        imported_count += 1;
    }

    // 更新项目的表计数
    sqlx::query("UPDATE projects SET table_count = (SELECT COUNT(*) FROM db_tables WHERE project_id = ?1), updated_at = datetime('now') WHERE id = ?1")
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(|e| format!("更新项目表计数失败: {}", e))?;

    Ok(format!("成功导入 {} 张表", imported_count))
}

/// 从 PostgreSQL 导入表结构
async fn import_postgresql_tables(
    pool: &SqlitePool,
    project_id: i64,
    datasource: &Datasource,
    database_name: &str,
) -> Result<String, String> {
    // 连接到 PostgreSQL 数据库
    let connection_string = format!(
        "postgresql://{}:{}@{}:{}/{}",
        datasource
            .username
            .as_ref()
            .ok_or("PostgreSQL 用户名未配置")?,
        datasource
            .password
            .as_ref()
            .ok_or("PostgreSQL 密码未配置")?,
        datasource.host.as_ref().ok_or("PostgreSQL 主机未配置")?,
        datasource.port.unwrap_or(5432),
        database_name
    );

    let pg_pool = sqlx::postgres::PgPool::connect(&connection_string)
        .await
        .map_err(|e| format!("PostgreSQL 连接失败: {}", e))?;

    // 查询所有表
    let tables: Vec<PgTableInfo> = sqlx::query_as(
        "SELECT
            t.table_name as name,
            obj_description((t.table_schema||'.'||t.table_name)::regclass, 'pg_class') as comment,
            CASE WHEN c.relkind = 'r' THEN 'table' WHEN c.relkind = 'v' THEN 'view' ELSE 'table' END as table_type,
            NULL::text as engine,
            0::bigint as row_count
        FROM information_schema.tables t
        JOIN pg_class c ON c.relname = t.table_name
        WHERE t.table_schema = 'public' AND t.table_type IN ('BASE TABLE', 'VIEW')
        ORDER BY t.table_name"
    )
    .fetch_all(&pg_pool)
    .await
    .map_err(|e| format!("查询表列表失败: {}", e))?;

    if tables.is_empty() {
        return Ok("数据库中没有找到任何表".to_string());
    }

    let mut imported_count = 0;

    // 导入每张表
    for table_info in tables {
        // 创建表记录
        let table_id: i64 = sqlx::query_scalar(
            "INSERT INTO db_tables (project_id, name, comment, engine, table_type, row_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             RETURNING id",
        )
        .bind(project_id)
        .bind(&table_info.name)
        .bind(&table_info.comment)
        .bind(&table_info.engine)
        .bind(&table_info.table_type)
        .bind(table_info.row_count.map(|c| c as i32).unwrap_or(0))
        .fetch_one(pool)
        .await
        .map_err(|e| format!("创建表记录失败: {}", e))?;

        // 查询列信息
        let columns: Vec<PgColumnInfo> = sqlx::query_as(
            "SELECT
                column_name as name,
                data_type as data_type,
                character_maximum_length as length,
                is_nullable as is_nullable,
                column_default as default_value,
                col_description((table_schema||'.'||table_name)::regclass::oid, ordinal_position) as comment,
                ordinal_position as ordinal_position
            FROM information_schema.columns
            WHERE table_schema = 'public' AND table_name = $1
            ORDER BY ordinal_position"
        )
        .bind(&table_info.name)
        .fetch_all(&pg_pool)
        .await
        .map_err(|e| format!("查询列信息失败: {}", e))?;

        // 查询主键信息
        let primary_keys: Vec<String> = sqlx::query_scalar(
            "SELECT a.attname
             FROM pg_index i
             JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey)
             WHERE i.indrelid = $1::regclass AND i.indisprimary",
        )
        .bind(format!("public.{}", table_info.name))
        .fetch_all(&pg_pool)
        .await
        .map_err(|e| format!("查询主键失败: {}", e))?;

        // 插入列记录
        for col in columns {
            let is_primary_key = primary_keys.contains(&col.name);
            let is_nullable = col.is_nullable.as_deref() == Some("YES");

            sqlx::query(
                "INSERT INTO db_columns (table_id, name, data_type, length, is_nullable, is_primary_key, is_unique, default_value, comment, ordinal_position)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
            )
            .bind(table_id)
            .bind(&col.name)
            .bind(&col.data_type)
            .bind(col.length)
            .bind(is_nullable as i32)
            .bind(is_primary_key as i32)
            .bind(0) // PostgreSQL unique 需要额外查询，这里简化处理
            .bind(&col.default_value)
            .bind(&col.comment)
            .bind(col.ordinal_position.unwrap_or(0))
            .execute(pool)
            .await
            .map_err(|e| format!("创建列记录失败: {}", e))?;
        }

        // 更新表的列计数
        let column_count: i32 =
            sqlx::query_scalar("SELECT COUNT(*) FROM db_columns WHERE table_id = ?1")
                .bind(table_id)
                .fetch_one(pool)
                .await
                .map_err(|e| format!("查询列计数失败: {}", e))?;

        sqlx::query("UPDATE db_tables SET column_count = ?1 WHERE id = ?2")
            .bind(column_count)
            .bind(table_id)
            .execute(pool)
            .await
            .map_err(|e| format!("更新列计数失败: {}", e))?;

        imported_count += 1;
    }

    // 更新项目的表计数
    sqlx::query("UPDATE projects SET table_count = (SELECT COUNT(*) FROM db_tables WHERE project_id = ?1), updated_at = datetime('now') WHERE id = ?1")
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(|e| format!("更新项目表计数失败: {}", e))?;

    Ok(format!("成功导入 {} 张表", imported_count))
}

/// 从 SQLite 导入表结构
async fn import_sqlite_tables(
    pool: &SqlitePool,
    project_id: i64,
    datasource: &Datasource,
) -> Result<String, String> {
    // 连接到 SQLite 数据库
    let sqlite_file = datasource
        .sqlite_file
        .as_ref()
        .ok_or("SQLite 文件路径未配置")?;

    if !std::path::Path::new(sqlite_file).exists() {
        return Err(format!("SQLite 文件不存在: {}", sqlite_file));
    }

    let sqlite_pool = sqlx::sqlite::SqlitePool::connect(&format!("sqlite://{}", sqlite_file))
        .await
        .map_err(|e| format!("SQLite 连接失败: {}", e))?;

    // 查询所有表
    let tables: Vec<SQLiteTableInfo> = sqlx::query_as(
        "SELECT name, type
         FROM sqlite_master
         WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%'
         ORDER BY name",
    )
    .fetch_all(&sqlite_pool)
    .await
    .map_err(|e| format!("查询表列表失败: {}", e))?;

    if tables.is_empty() {
        return Ok("数据库中没有找到任何表".to_string());
    }

    let mut imported_count = 0;

    // 导入每张表
    for table_info in tables {
        // 创建表记录
        let table_id: i64 = sqlx::query_scalar(
            "INSERT INTO db_tables (project_id, name, comment, engine, table_type, row_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             RETURNING id",
        )
        .bind(project_id)
        .bind(&table_info.name)
        .bind(None::<&str>)
        .bind(Some("SQLite"))
        .bind(if table_info.table_type == "table" {
            "table"
        } else {
            "view"
        })
        .bind(0)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("创建表记录失败: {}", e))?;

        // 使用 PRAGMA 获取表结构
        let pragma_sql = format!("PRAGMA table_info({})", table_info.name);
        let columns: Vec<SQLiteColumnInfo> = sqlx::query_as(&pragma_sql)
            .fetch_all(&sqlite_pool)
            .await
            .map_err(|e| format!("查询列信息失败: {}", e))?;

        // 插入列记录
        for col in columns {
            let is_primary_key = col.pk > 0;

            sqlx::query(
                "INSERT INTO db_columns (table_id, name, data_type, length, is_nullable, is_primary_key, is_unique, default_value, comment, ordinal_position)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
            )
            .bind(table_id)
            .bind(&col.name)
            .bind(&col.data_type)
            .bind(None::<i64>)
            .bind((col.not_null == 0) as i32)
            .bind(is_primary_key as i32)
            .bind(0)
            .bind(&col.default_value)
            .bind(None::<&str>)
            .bind(col.cid)
            .execute(pool)
            .await
            .map_err(|e| format!("创建列记录失败: {}", e))?;
        }

        // 更新表的列计数
        let column_count: i32 =
            sqlx::query_scalar("SELECT COUNT(*) FROM db_columns WHERE table_id = ?1")
                .bind(table_id)
                .fetch_one(pool)
                .await
                .map_err(|e| format!("查询列计数失败: {}", e))?;

        sqlx::query("UPDATE db_tables SET column_count = ?1 WHERE id = ?2")
            .bind(column_count)
            .bind(table_id)
            .execute(pool)
            .await
            .map_err(|e| format!("更新列计数失败: {}", e))?;

        imported_count += 1;
    }

    // 更新项目的表计数
    sqlx::query("UPDATE projects SET table_count = (SELECT COUNT(*) FROM db_tables WHERE project_id = ?1), updated_at = datetime('now') WHERE id = ?1")
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(|e| format!("更新项目表计数失败: {}", e))?;

    Ok(format!("成功导入 {} 张表", imported_count))
}

/// 读取 MySQL 数据库的表列表
pub async fn fetch_mysql_tables(
    _pool: &SqlitePool,
    datasource: &Datasource,
    database_name: &str,
) -> Result<String, String> {
    // 连接到 MySQL 数据库
    let connection_string = format!(
        "mysql://{}:{}@{}:{}/{}",
        datasource.username.as_ref().ok_or("MySQL 用户名未配置")?,
        datasource.password.as_ref().ok_or("MySQL 密码未配置")?,
        datasource.host.as_ref().ok_or("MySQL 主机未配置")?,
        datasource.port.unwrap_or(3306),
        database_name
    );

    let mysql_pool = sqlx::mysql::MySqlPool::connect(&connection_string)
        .await
        .map_err(|e| format!("MySQL 连接失败: {}", e))?;

    // 查询所有表
    let tables: Vec<serde_json::Value> = sqlx::query(
        "SELECT
            TABLE_NAME as name,
            TABLE_COMMENT as comment,
            TABLE_TYPE as table_type,
            ENGINE as engine,
            CAST(TABLE_ROWS AS SIGNED) as row_count
        FROM information_schema.TABLES
        WHERE TABLE_SCHEMA = ? AND TABLE_TYPE IN ('BASE TABLE', 'VIEW')
        ORDER BY TABLE_NAME",
    )
    .bind(database_name)
    .fetch_all(&mysql_pool)
    .await
    .map_err(|e| format!("查询表列表失败: {}", e))?
    .into_iter()
    .map(|row| {
        serde_json::json!({
            "name": row.get::<String, _>("name"),
            "comment": row.get::<Option<String>, _>("comment"),
            "table_type": row.get::<String, _>("table_type"),
            "engine": row.get::<Option<String>, _>("engine"),
            "row_count": row.get::<Option<i64>, _>("row_count")
        })
    })
    .collect();

    let json = serde_json::to_string(&tables).map_err(|e| format!("JSON 序列化失败: {}", e))?;

    // 关闭连接
    mysql_pool.close().await;

    Ok(json)
}

/// 读取 PostgreSQL 数据库的表列表
pub async fn fetch_postgresql_tables(
    _pool: &SqlitePool,
    datasource: &Datasource,
    database_name: &str,
) -> Result<String, String> {
    // 连接到 PostgreSQL 数据库
    let connection_string = format!(
        "postgresql://{}:{}@{}:{}/{}",
        datasource
            .username
            .as_ref()
            .ok_or("PostgreSQL 用户名未配置")?,
        datasource
            .password
            .as_ref()
            .ok_or("PostgreSQL 密码未配置")?,
        datasource.host.as_ref().ok_or("PostgreSQL 主机未配置")?,
        datasource.port.unwrap_or(5432),
        database_name
    );

    let pg_pool = sqlx::postgres::PgPool::connect(&connection_string)
        .await
        .map_err(|e| format!("PostgreSQL 连接失败: {}", e))?;

    // 查询所有表
    let tables: Vec<serde_json::Value> = sqlx::query(
        "SELECT
            t.table_name as name,
            obj_description((t.table_schema||'.'||t.table_name)::regclass, 'pg_class') as comment,
            CASE c.relkind
                WHEN 'r' THEN 'table'
                WHEN 'v' THEN 'view'
                ELSE 'table'
            END as table_type,
            NULL::text as engine,
            COALESCE((SELECT reltuples::bigint FROM pg_class WHERE oid = (t.table_schema||'.'||t.table_name)::regclass), 0) as row_count
        FROM information_schema.tables t
        JOIN pg_class c ON c.relname = t.table_name
        WHERE t.table_schema = 'public' AND t.table_type IN ('BASE TABLE', 'VIEW')
        ORDER BY t.table_name"
    )
    .fetch_all(&pg_pool)
    .await
        .map_err(|e| format!("查询表列表失败: {}", e))?
    .into_iter()
        .map(|row| {
            serde_json::json!({
                "name": row.get::<String, _>("name"),
                "comment": row.get::<Option<String>, _>("comment"),
                "table_type": row.get::<String, _>("table_type"),
                "engine": None::<String>,
                "row_count": row.get::<i64, _>("row_count")
            })
        })
        .collect();

    let json = serde_json::to_string(&tables).map_err(|e| format!("JSON 序列化失败: {}", e))?;

    // 关闭连接
    pg_pool.close().await;

    Ok(json)
}

/// 读取 SQLite 数据库的表列表
pub async fn fetch_sqlite_tables(
    _pool: &SqlitePool,
    datasource: &Datasource,
) -> Result<String, String> {
    let sqlite_file = datasource
        .sqlite_file
        .as_ref()
        .ok_or_else(|| "SQLite 文件路径未指定".to_string())?;

    if !std::path::Path::new(sqlite_file).exists() {
        return Err("SQLite 文件不存在".to_string());
    }

    let connection_string = format!("sqlite:{}", sqlite_file);

    let sqlite_pool = sqlx::sqlite::SqlitePool::connect(&connection_string)
        .await
        .map_err(|e| format!("SQLite 连接失败: {}", e))?;

    // 查询所有表
    let tables: Vec<serde_json::Value> = sqlx::query(
        "SELECT name, type as table_type
        FROM sqlite_master
        WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%'
        ORDER BY name",
    )
    .fetch_all(&sqlite_pool)
    .await
    .map_err(|e| format!("查询表列表失败: {}", e))?
    .into_iter()
    .map(|row| {
        serde_json::json!({
            "name": row.get::<String, _>("name"),
            "comment": None::<String>,
            "table_type": row.get::<String, _>("table_type"),
            "engine": None::<String>,
            "row_count": None::<i64>
        })
    })
    .collect();

    let json = serde_json::to_string(&tables).map_err(|e| format!("JSON 序列化失败: {}", e))?;

    // 关闭连接
    sqlite_pool.close().await;

    Ok(json)
}

/// 导入单个表及其列信息
pub async fn import_single_table(
    pool: &SqlitePool,
    project_id: i64,
    datasource: &Datasource,
    database_name: &str,
    table_name: &str,
    table_comment: Option<&str>,
    table_type: &str,
    engine: Option<&str>,
    row_count: i64,
) -> Result<(), String> {
    match datasource.type_.as_str() {
        "mysql" => {
            import_mysql_single_table(
                pool,
                project_id,
                datasource,
                database_name,
                table_name,
                table_comment,
                table_type,
                engine,
                row_count,
            )
            .await
        }
        "postgresql" => {
            import_postgresql_single_table(
                pool,
                project_id,
                datasource,
                database_name,
                table_name,
                table_comment,
                table_type,
                engine,
                row_count,
            )
            .await
        }
        "sqlite" => {
            import_sqlite_single_table(
                pool,
                project_id,
                datasource,
                table_name,
                table_comment,
                table_type,
                row_count,
            )
            .await
        }
        _ => Err(format!("不支持的数据源类型: {}", datasource.type_)),
    }
}

/// 导入单个 MySQL 表
async fn import_mysql_single_table(
    pool: &SqlitePool,
    project_id: i64,
    datasource: &Datasource,
    database_name: &str,
    table_name: &str,
    table_comment: Option<&str>,
    table_type: &str,
    _engine: Option<&str>,
    _row_count: i64,
) -> Result<(), String> {
    println!("开始导入表: {}", table_name);

    // 连接到 MySQL
    let connection_string = format!(
        "mysql://{}:{}@{}:{}/{}",
        datasource.username.as_ref().ok_or("MySQL 用户名未配置")?,
        datasource.password.as_ref().ok_or("MySQL 密码未配置")?,
        datasource.host.as_ref().ok_or("MySQL 主机未配置")?,
        datasource.port.unwrap_or(3306),
        database_name
    );

    println!("连接字符串已构建");

    let mysql_pool = sqlx::mysql::MySqlPool::connect(&connection_string)
        .await
        .map_err(|e| format!("MySQL 连接失败: {}", e))?;

    println!("MySQL 连接成功");

    // 创建表记录
    let table_id: i64 = sqlx::query_scalar(
        "INSERT INTO db_tables (project_id, name, comment, engine, table_type, row_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         RETURNING id",
    )
    .bind(project_id)
    .bind(table_name)
    .bind(table_comment)
    .bind(_engine)
    .bind(if table_type == "BASE TABLE" || table_type == "table" {
        "table"
    } else {
        "view"
    })
    .bind(0)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("创建表记录失败: {}", e))?;

    println!(
        "表记录创建成功，table_id = {}, 项目ID = {}",
        table_id, project_id
    );

    // 查询列信息
    println!(
        "开始查询列信息，数据库: {}, 表: {}",
        database_name, table_name
    );

    let columns_result: Result<Vec<MySQLColumnInfo>, _> = sqlx::query_as(
        "SELECT
            COLUMN_NAME as name,
            DATA_TYPE as data_type,
            CAST(CHARACTER_MAXIMUM_LENGTH AS SIGNED) as length,
            IS_NULLABLE as is_nullable,
            COLUMN_KEY as column_key,
            COLUMN_DEFAULT as default_value,
            COLUMN_COMMENT as comment,
            ORDINAL_POSITION as ordinal_position
        FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?
        ORDER BY ORDINAL_POSITION",
    )
    .bind(database_name)
    .bind(table_name)
    .fetch_all(&mysql_pool)
    .await;

    let columns = match columns_result {
        Ok(cols) => {
            println!("查询成功，返回 {} 行", cols.len());
            cols
        }
        Err(e) => {
            let error_msg = format!("查询列信息失败: {}", e);
            eprintln!("{}", error_msg);
            return Err(error_msg);
        }
    };

    let column_count = columns.len();
    println!("查询到 {} 列", column_count);

    // 插入列记录
    for col in columns {
        let is_primary_key = col.column_key.as_deref() == Some("PRI");
        let is_unique =
            col.column_key.as_deref() == Some("UNI") || col.column_key.as_deref() == Some("PRI");
        let is_nullable = col.is_nullable.as_deref() == Some("YES");

        println!(
            "插入列: {} (类型: {}, 可空: {}, 主键: {})",
            col.name, col.data_type, is_nullable, is_primary_key
        );

        sqlx::query(
            "INSERT INTO db_columns (table_id, name, data_type, length, is_nullable, is_primary_key, is_unique, default_value, comment, ordinal_position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
        )
        .bind(table_id)
        .bind(&col.name)
        .bind(&col.data_type)
        .bind(col.length)
        .bind(is_nullable as i32)
        .bind(is_primary_key as i32)
        .bind(is_unique as i32)
        .bind(&col.default_value)
        .bind(&col.comment)
        .bind(col.ordinal_position.map(|p| p as i32).unwrap_or(0))
        .execute(pool)
        .await
        .map_err(|e| format!("插入列记录失败 [{}]: {}", col.name, e))?;
    }

    println!("成功插入 {} 列", column_count);

    // 更新表的列计数
    let column_count: i32 =
        sqlx::query_scalar("SELECT COUNT(*) FROM db_columns WHERE table_id = ?1")
            .bind(table_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("查询列计数失败: {}", e))?;

    sqlx::query("UPDATE db_tables SET column_count = ?1 WHERE id = ?2")
        .bind(column_count)
        .bind(table_id)
        .execute(pool)
        .await
        .map_err(|e| format!("更新列计数失败: {}", e))?;

    // 更新项目的表数量
    let _ = sqlx::query("UPDATE projects SET table_count = table_count + 1, updated_at = datetime('now') WHERE id = ?1")
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(|e| eprintln!("警告：更新项目表数量失败: {}", e));

    mysql_pool.close().await;

    Ok(())
}

/// 导入单个 PostgreSQL 表
async fn import_postgresql_single_table(
    pool: &SqlitePool,
    project_id: i64,
    datasource: &Datasource,
    database_name: &str,
    table_name: &str,
    table_comment: Option<&str>,
    table_type: &str,
    _engine: Option<&str>,
    _row_count: i64,
) -> Result<(), String> {
    // 连接到 PostgreSQL
    let connection_string = format!(
        "postgresql://{}:{}@{}:{}/{}",
        datasource
            .username
            .as_ref()
            .ok_or("PostgreSQL 用户名未配置")?,
        datasource
            .password
            .as_ref()
            .ok_or("PostgreSQL 密码未配置")?,
        datasource.host.as_ref().ok_or("PostgreSQL 主机未配置")?,
        datasource.port.unwrap_or(5432),
        database_name
    );

    let pg_pool = sqlx::postgres::PgPool::connect(&connection_string)
        .await
        .map_err(|e| format!("PostgreSQL 连接失败: {}", e))?;

    // 创建表记录
    let table_id: i64 = sqlx::query_scalar(
        "INSERT INTO db_tables (project_id, name, comment, engine, table_type, row_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         RETURNING id",
    )
    .bind(project_id)
    .bind(table_name)
    .bind(table_comment)
    .bind(None::<String>)
    .bind(if table_type == "table" {
        "table"
    } else {
        "view"
    })
    .bind(0)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("创建表记录失败: {}", e))?;

    // 查询列信息
    let columns: Vec<PgColumnInfo> = sqlx::query_as(
        "SELECT
            column_name as name,
            data_type,
            character_maximum_length as length,
            is_nullable,
            column_default as default_value,
            col_description((table_schema||'.'||table_name)::regclass::oid, ordinal_position) as comment,
            ordinal_position
        FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = $1
        ORDER BY ordinal_position"
    )
    .bind(table_name)
    .fetch_all(&pg_pool)
    .await
        .map_err(|e| format!("查询列信息失败: {}", e))?;

    println!("查询到 {} 列", columns.len());

    // 插入列记录
    for col in &columns {
        let is_nullable = col.is_nullable.as_deref() == Some("YES");

        sqlx::query(
            "INSERT INTO db_columns (table_id, name, data_type, length, is_nullable, is_primary_key, is_unique, default_value, comment, ordinal_position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
        )
        .bind(table_id)
        .bind(&col.name)
        .bind(&col.data_type)
        .bind(col.length)
        .bind(is_nullable as i32)
        .bind(false as i32) // PostgreSQL 主键需要更复杂的判断
        .bind(false as i32)
        .bind(&col.default_value)
        .bind(&col.comment)
        .bind(col.ordinal_position.unwrap_or(0))
        .execute(pool)
        .await
        .map_err(|e| format!("插入列记录失败 [{}]: {}", col.name, e))?;
    }

    println!("成功插入 {} 列", columns.len());

    // 更新表的列计数
    let column_count: i32 =
        sqlx::query_scalar("SELECT COUNT(*) FROM db_columns WHERE table_id = ?1")
            .bind(table_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("查询列计数失败: {}", e))?;

    sqlx::query("UPDATE db_tables SET column_count = ?1 WHERE id = ?2")
        .bind(column_count)
        .bind(table_id)
        .execute(pool)
        .await
        .map_err(|e| format!("更新列计数失败: {}", e))?;

    // 更新项目的表数量
    let _ = sqlx::query("UPDATE projects SET table_count = table_count + 1, updated_at = datetime('now') WHERE id = ?1")
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(|e| eprintln!("警告：更新项目表数量失败: {}", e));

    pg_pool.close().await;

    Ok(())
}

/// 导入单个 SQLite 表
async fn import_sqlite_single_table(
    pool: &SqlitePool,
    project_id: i64,
    datasource: &Datasource,
    table_name: &str,
    table_comment: Option<&str>,
    table_type: &str,
    _row_count: i64,
) -> Result<(), String> {
    let sqlite_file = datasource
        .sqlite_file
        .as_ref()
        .ok_or_else(|| "SQLite 文件路径未指定".to_string())?;

    if !std::path::Path::new(sqlite_file).exists() {
        return Err("SQLite 文件不存在".to_string());
    }

    let connection_string = format!("sqlite:{}", sqlite_file);

    let sqlite_pool = sqlx::sqlite::SqlitePool::connect(&connection_string)
        .await
        .map_err(|e| format!("SQLite 连接失败: {}", e))?;

    // 创建表记录
    let table_id: i64 = sqlx::query_scalar(
        "INSERT INTO db_tables (project_id, name, comment, engine, table_type, row_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         RETURNING id",
    )
    .bind(project_id)
    .bind(table_name)
    .bind(table_comment)
    .bind(None::<String>)
    .bind(if table_type == "table" {
        "table"
    } else {
        "view"
    })
    .bind(0)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("创建表记录失败: {}", e))?;

    // 查询列信息
    let columns: Vec<SQLiteColumnInfo> = sqlx::query_as("PRAGMA table_info(?)")
        .bind(table_name)
        .fetch_all(&sqlite_pool)
        .await
        .map_err(|e| format!("查询列信息失败: {}", e))?;

    println!("查询到 {} 列", columns.len());

    // 插入列记录
    let mut ordinal_position = 1i32;
    for col in columns {
        let is_primary_key = col.pk > 0;
        let is_nullable = col.not_null == 0;

        sqlx::query(
            "INSERT INTO db_columns (table_id, name, data_type, length, is_nullable, is_primary_key, is_unique, default_value, comment, ordinal_position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
        )
        .bind(table_id)
        .bind(&col.name)
        .bind(&col.data_type)
       .bind(None::<i64>)
        .bind(is_nullable as i32)
        .bind(is_primary_key as i32)
        .bind(is_primary_key as i32) // SQLite 主键即唯一键
        .bind(&col.default_value)
        .bind(None::<String>)
        .bind(ordinal_position)
        .execute(pool)
        .await
        .map_err(|e| format!("插入列记录失败 [{}]: {}", col.name, e))?;

        ordinal_position += 1;
    }

    println!("成功插入 {} 张列", ordinal_position - 1);

    // 更新表的列计数
    let column_count: i32 =
        sqlx::query_scalar("SELECT COUNT(*) FROM db_columns WHERE table_id = ?1")
            .bind(table_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("查询列计数失败: {}", e))?;

    sqlx::query("UPDATE db_tables SET column_count = ?1 WHERE id = ?2")
        .bind(column_count)
        .bind(table_id)
        .execute(pool)
        .await
        .map_err(|e| format!("更新列计数失败: {}", e))?;

    // 更新项目的表数量
    let _ = sqlx::query("UPDATE projects SET table_count = table_count + 1, updated_at = datetime('now') WHERE id = ?1")
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(|e| eprintln!("警告：更新项目表数量失败: {}", e));

    sqlite_pool.close().await;

    Ok(())
}

/// ===== SQL解析功能 =====
/// 只解析SQL，返回表结构信息（不写入数据库）
pub async fn parse_sql_only(
    _pool: &SqlitePool,
    _project_id: i64,
    sql_content: &str,
    sql_dialect: &str,
) -> Result<String, String> {
    use sqlparser::ast::Statement;
    use sqlparser::parser::Parser;

    // 根据方言解析SQL
    let statements = match sql_dialect {
        "mysql" => {
            use sqlparser::dialect::MySqlDialect;
            Parser::parse_sql(&MySqlDialect {}, sql_content)
                .map_err(|e| format!("SQL解析失败: {}", e))?
        }
        "postgresql" => {
            use sqlparser::dialect::PostgreSqlDialect;
            Parser::parse_sql(&PostgreSqlDialect {}, sql_content)
                .map_err(|e| format!("SQL解析失败: {}", e))?
        }
        "sqlite" => {
            use sqlparser::dialect::SQLiteDialect;
            Parser::parse_sql(&SQLiteDialect {}, sql_content)
                .map_err(|e| format!("SQL解析失败: {}", e))?
        }
        _ => {
            use sqlparser::dialect::GenericDialect;
            Parser::parse_sql(&GenericDialect {}, sql_content)
                .map_err(|e| format!("SQL解析失败: {}", e))?
        }
    };

    let mut tables = Vec::new();

    for statement in statements {
        if let Statement::CreateTable(create_table) = statement {
            let table_name = create_table.name.to_string();

            let mut columns = Vec::new();
            for (index, column_def) in create_table.columns.iter().enumerate() {
                let column_name = column_def.name.to_string();
                let data_type = parse_sql_data_type(&column_def.data_type);
                let length = parse_sql_data_type_length(&column_def.data_type);

                let is_primary_key = column_def.options.iter().any(|opt| {
                    matches!(
                        &opt.option,
                        sqlparser::ast::ColumnOption::Unique {
                            is_primary: true,
                            ..
                        }
                    )
                });
                let is_unique = column_def.options.iter().any(|opt| {
                    matches!(
                        &opt.option,
                        sqlparser::ast::ColumnOption::Unique {
                            is_primary: false,
                            ..
                        }
                    )
                });
                let has_not_null = column_def
                    .options
                    .iter()
                    .any(|opt| matches!(&opt.option, sqlparser::ast::ColumnOption::NotNull));
                let has_null = column_def
                    .options
                    .iter()
                    .any(|opt| matches!(&opt.option, sqlparser::ast::ColumnOption::Null));
                let is_nullable = has_null || (!has_not_null && !is_primary_key);

                let default_value = column_def.options.iter().find_map(|opt| {
                    if let sqlparser::ast::ColumnOption::Default(val) = &opt.option {
                        Some(format!("{}", val))
                    } else {
                        None
                    }
                });

                columns.push(serde_json::json!({
                    "name": column_name,
                    "dataType": data_type,
                    "length": length,
                    "isNullable": is_nullable,
                    "isPrimaryKey": is_primary_key,
                    "isUnique": is_unique,
                    "defaultValue": default_value,
                    "ordinalPosition": index as i32 + 1,
                }));
            }

            tables.push(serde_json::json!({
                "name": table_name,
                "columns": columns,
                "columnCount": columns.len() as i32,
            }));
        }
    }

    serde_json::to_string(&serde_json::json!({ "tables": tables }))
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 解析SQL并创建表和字段（使用sqlparser）
pub async fn parse_and_create_from_sql(
    pool: &SqlitePool,
    project_id: i64,
    sql_content: &str,
    sql_dialect: &str,
) -> Result<String, String> {
    use sqlparser::ast::Statement;
    use sqlparser::parser::Parser;

    // 根据方言解析SQL（避免使用trait对象，确保Send）
    let statements = match sql_dialect {
        "mysql" => {
            use sqlparser::dialect::MySqlDialect;
            Parser::parse_sql(&MySqlDialect {}, sql_content)
                .map_err(|e| format!("SQL解析失败: {}", e))?
        }
        "postgresql" => {
            use sqlparser::dialect::PostgreSqlDialect;
            Parser::parse_sql(&PostgreSqlDialect {}, sql_content)
                .map_err(|e| format!("SQL解析失败: {}", e))?
        }
        "sqlite" => {
            use sqlparser::dialect::SQLiteDialect;
            Parser::parse_sql(&SQLiteDialect {}, sql_content)
                .map_err(|e| format!("SQL解析失败: {}", e))?
        }
        _ => {
            use sqlparser::dialect::GenericDialect;
            Parser::parse_sql(&GenericDialect {}, sql_content)
                .map_err(|e| format!("SQL解析失败: {}", e))?
        }
    };

    let mut tables_created = 0;
    let mut columns_created = 0;
    let mut errors = Vec::new();

    for statement in statements {
        if let Statement::CreateTable(create_table) = statement {
            // 提取表名
            let table_name = create_table.name.to_string();

            // 创建表记录
            let table_id = match sqlx::query(
                "INSERT INTO db_tables (project_id, name, table_type, column_count)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .bind(project_id)
            .bind(&table_name)
            .bind("table")
            .bind(create_table.columns.len() as i32)
            .execute(pool)
            .await
            {
                Ok(result) => result.last_insert_rowid(),
                Err(e) => {
                    errors.push(format!("创建表 {} 失败: {}", table_name, e));
                    continue;
                }
            };

            tables_created += 1;

            // 创建字段
            for (index, column_def) in create_table.columns.iter().enumerate() {
                let column_name = column_def.name.to_string();

                // 解析数据类型
                let data_type = parse_sql_data_type(&column_def.data_type);

                // 解析长度
                let length = parse_sql_data_type_length(&column_def.data_type);

                // 判断约束
                let is_primary_key = column_def.options.iter().any(|opt| {
                    matches!(
                        &opt.option,
                        sqlparser::ast::ColumnOption::Unique {
                            is_primary: true,
                            ..
                        }
                    )
                });
                let is_unique = column_def.options.iter().any(|opt| {
                    matches!(
                        &opt.option,
                        sqlparser::ast::ColumnOption::Unique {
                            is_primary: false,
                            ..
                        }
                    )
                });
                let has_not_null = column_def
                    .options
                    .iter()
                    .any(|opt| matches!(&opt.option, sqlparser::ast::ColumnOption::NotNull));
                let has_null = column_def
                    .options
                    .iter()
                    .any(|opt| matches!(&opt.option, sqlparser::ast::ColumnOption::Null));

                let is_nullable = has_null || (!has_not_null && !is_primary_key);

                // 提取默认值
                let default_value = column_def.options.iter().find_map(|opt| {
                    if let sqlparser::ast::ColumnOption::Default(val) = &opt.option {
                        Some(format!("{}", val))
                    } else {
                        None
                    }
                });

                if let Err(e) = sqlx::query(
                    "INSERT INTO db_columns (
                        table_id, name, data_type, length, is_nullable,
                        is_primary_key, is_unique, default_value, ordinal_position
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                )
                .bind(table_id)
                .bind(&column_name)
                .bind(&data_type)
                .bind(length)
                .bind(is_nullable)
                .bind(is_primary_key)
                .bind(is_unique)
                .bind(default_value)
                .bind(index as i32 + 1)
                .execute(pool)
                .await
                {
                    errors.push(format!("创建列 {}.{} 失败: {}", table_name, column_name, e));
                } else {
                    columns_created += 1;
                }
            }
        }
    }

    // 更新项目的表数量
    if tables_created > 0 {
        let _ = sqlx::query("UPDATE projects SET table_count = table_count + ?1, updated_at = datetime('now') WHERE id = ?2")
            .bind(tables_created)
            .bind(project_id)
            .execute(pool)
            .await;
    }

    // 构建结果消息
    let mut result = format!(
        "成功创建 {} 张表，{} 个字段",
        tables_created, columns_created
    );
    if !errors.is_empty() {
        result.push_str(&format!("\n错误: {}", errors.join("; ")));
    }

    Ok(result)
}

/// 从sqlparser的DataType转换为我们的字符串表示
fn parse_sql_data_type(data_type: &sqlparser::ast::DataType) -> String {
    use sqlparser::ast::DataType;

    match data_type {
        DataType::Text => "text".to_string(),
        DataType::Varchar(_) => "varchar".to_string(),
        DataType::Char(_) => "char".to_string(),
        DataType::Int(_) => "int".to_string(),
        DataType::BigInt(_) => "bigint".to_string(),
        DataType::SmallInt(_) => "smallint".to_string(),
        DataType::Float(_) => "float".to_string(),
        DataType::Double => "double".to_string(),
        DataType::Decimal(_) => "decimal".to_string(),
        DataType::Datetime(_) => "datetime".to_string(),
        DataType::Date => "date".to_string(),
        DataType::Timestamp(_, _) => "timestamp".to_string(),
        DataType::Boolean => "boolean".to_string(),
        DataType::JSON => "json".to_string(),
        _ => format!("{:?}", data_type).to_lowercase(),
    }
}

/// 从sqlparser的DataType提取长度
fn parse_sql_data_type_length(data_type: &sqlparser::ast::DataType) -> Option<i64> {
    use sqlparser::ast::{CharacterLength, DataType};

    match data_type {
        DataType::Varchar(Some(len)) | DataType::Char(Some(len)) => match len {
            CharacterLength::IntegerLength { length, .. } => Some(*length as i64),
            _ => None,
        },
        _ => None,
    }
}
