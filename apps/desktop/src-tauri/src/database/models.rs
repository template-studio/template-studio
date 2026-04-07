/// 测试数据库连接参数
#[derive(Debug, serde::Deserialize)]
pub struct TestConnectionParams {
    #[serde(rename = "type")]
    pub type_: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: Option<String>,  // 测试连接时可指定数据库名
    pub sqlite_file: Option<String>,  // SQLite 文件路径
    pub username: Option<String>,
    pub password: Option<String>,
}

/// 创建/更新数据源参数
#[derive(Debug, serde::Deserialize)]
pub struct DatasourceParams {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,  // PostgreSQL 初始数据库（可选）
    pub sqlite_file: Option<String>,  // 仅 SQLite 使用
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub datasource_id: i64,
    pub database_name: String,
    pub primary_language_id: Option<i64>,
    pub frontend_language_id: Option<i64>,
    pub backend_language_id: Option<i64>,
    pub table_count: i32,
    pub created_at: String,
    pub updated_at: String,
    // 附加字段（非数据库字段）
    #[serde(skip)]
    pub datasource: Option<Datasource>,
    #[serde(skip)]
    pub primary_language: Option<Language>,
    #[serde(skip)]
    pub frontend_language: Option<Language>,
    #[serde(skip)]
    pub backend_language: Option<Language>,
    #[serde(skip)]
    pub languages: Option<Vec<Language>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Datasource {
    pub id: i64,
    pub name: String,
    pub type_: String,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,  // PostgreSQL 初始数据库
    pub sqlite_file: Option<String>,  // SQLite 文件路径
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DbTable {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub comment: Option<String>,
    pub engine: Option<String>,
    pub table_type: String,
    pub row_count: i32,
    pub column_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DbColumn {
    pub id: i64,
    pub table_id: i64,
    pub name: String,
    pub data_type: String,
    pub length: Option<i64>,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub is_unique: bool,
    pub default_value: Option<String>,
    pub comment: Option<String>,
    pub ordinal_position: i32,
    pub created_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Language {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
    pub is_builtin: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 统计数据结构
#[derive(Debug, serde::Serialize)]
pub struct Statistics {
    pub total_projects: i64,
    pub total_datasources: i64,
    pub total_languages: i64,
    pub total_tables: i64,
}

/// 最近项目结构
#[derive(Debug, serde::Serialize)]
pub struct RecentProject {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub database_name: Option<String>,
    pub table_count: i64,
    pub created_at: String,
}
