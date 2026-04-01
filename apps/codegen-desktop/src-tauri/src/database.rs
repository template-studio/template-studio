use sqlx::{SqlitePool, Row};
use std::path::PathBuf;
use std::fs;
use dirs::home_dir;

/// 数据库路径
pub fn get_database_path() -> Result<PathBuf, std::io::Error> {
    // C:\Users\{user}\.ciclebyte\template_studio_rust\db\codegen-desktop.db
    let mut db_dir = home_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "无法获取用户主目录"))?;
    db_dir.push(".ciclebyte");
    db_dir.push("template_studio_rust");
    db_dir.push("db");

    // 确保目录存在
    fs::create_dir_all(&db_dir)?;

    db_dir.push("codegen-desktop.db");
    Ok(db_dir)
}

/// 数据库连接池
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// 初始化数据库连接池（如果不存在则创建）
    pub async fn init() -> Result<Self, sqlx::Error> {
        let db_path = get_database_path()
            .map_err(|e| sqlx::Error::Io(e.into()))?;

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

    /// 运行数据库迁移
    async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        println!("运行数据库迁移...");

        // 创建 migrations 表（用于跟踪迁移版本）
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        )
        .execute(&self.pool)
        .await?;

        // 检查当前版本
        let current_version: i32 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations"
        )
        .fetch_one(&self.pool)
        .await?;

        println!("当前数据库版本: {}", current_version);

        // 执行迁移
        if current_version < 1 {
            self.migration_001_create_projects_table().await?;
        }

        if current_version < 2 {
            self.migration_002_create_datasources_table().await?;
        }

        if current_version < 3 {
            self.migration_003_create_tables_table().await?;
        }

        if current_version < 4 {
            self.migration_004_create_columns_table().await?;
        }

        if current_version < 5 {
            self.migration_005_update_schema().await?;
        }

        if current_version < 6 {
            self.migration_006_add_database_column().await?;
        }

        if current_version < 7 {
            self.migration_007_create_ai_tables().await?;
        }

        if current_version < 8 {
            self.migration_008_add_table_preferences().await?;
        }

        if current_version < 9 {
            self.migration_009_create_language_field_types().await?;
        }

        if current_version < 10 {
            self.migration_010_create_project_mappings().await?;
        }

        Ok(())
    }

    /// 迁移 001: 创建 projects 表
    async fn migration_001_create_projects_table(&self) -> Result<(), sqlx::Error> {
        println!("执行迁移 001: 创建 projects 表");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                database_type TEXT NOT NULL DEFAULT 'mysql',
                table_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_projects_name ON projects(name)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_projects_database_type ON projects(database_type)")
            .execute(&self.pool)
            .await?;

        sqlx::query("INSERT INTO schema_migrations (version) VALUES (1)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 迁移 002: 创建 datasources 表
    async fn migration_002_create_datasources_table(&self) -> Result<(), sqlx::Error> {
        println!("执行迁移 002: 创建 datasources 表");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS datasources (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                type TEXT NOT NULL DEFAULT 'mysql',
                host TEXT NOT NULL,
                port INTEGER NOT NULL DEFAULT 3306,
                database TEXT NOT NULL,
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_datasources_name ON datasources(name)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_datasources_type ON datasources(type)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_datasources_is_active ON datasources(is_active)")
            .execute(&self.pool)
            .await?;

        sqlx::query("INSERT INTO schema_migrations (version) VALUES (2)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 迁移 003: 创建 tables 表（项目中的数据库表）
    async fn migration_003_create_tables_table(&self) -> Result<(), sqlx::Error> {
        println!("执行迁移 003: 创建 tables 表");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS db_tables (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                comment TEXT,
                engine TEXT,
                table_type TEXT NOT NULL DEFAULT 'table',
                row_count INTEGER NOT NULL DEFAULT 0,
                column_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                UNIQUE(project_id, name)
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tables_project_id ON db_tables(project_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tables_name ON db_tables(name)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tables_type ON db_tables(table_type)")
            .execute(&self.pool)
            .await?;

        sqlx::query("INSERT INTO schema_migrations (version) VALUES (3)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 迁移 004: 创建 columns 表（表的字段）
    async fn migration_004_create_columns_table(&self) -> Result<(), sqlx::Error> {
        println!("执行迁移 004: 创建 columns 表");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS db_columns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                table_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                data_type TEXT NOT NULL,
                length INTEGER,
                is_nullable INTEGER NOT NULL DEFAULT 1,
                is_primary_key INTEGER NOT NULL DEFAULT 0,
                is_unique INTEGER NOT NULL DEFAULT 0,
                default_value TEXT,
                comment TEXT,
                ordinal_position INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (table_id) REFERENCES db_tables(id) ON DELETE CASCADE,
                UNIQUE(table_id, name)
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_columns_table_id ON db_columns(table_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_columns_name ON db_columns(name)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_columns_ordinal ON db_columns(ordinal_position)")
            .execute(&self.pool)
            .await?;

        sqlx::query("INSERT INTO schema_migrations (version) VALUES (4)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 迁移 005: 更新数据库架构（添加语言支持）
    async fn migration_005_update_schema(&self) -> Result<(), sqlx::Error> {
        println!("执行迁移 005: 更新数据库架构");

        // 1. 创建 languages 表
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS languages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                icon TEXT,
                color TEXT,
                description TEXT,
                is_builtin INTEGER NOT NULL DEFAULT 0,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_languages_name ON languages(name)")
            .execute(&self.pool)
            .await?;

        // 2. 插入预设语言数据
        sqlx::query(
            "INSERT OR IGNORE INTO languages (name, icon, color, description, is_builtin) VALUES
            ('Rust', '🦀', 'orange', '系统级编程语言', 1),
            ('Go', '🐹', 'blue', '简洁高效的编程语言', 1),
            ('Python', '🐍', 'green', '易学易用的编程语言', 1),
            ('TypeScript', '💛', 'blue', 'JavaScript 的类型化超集', 1),
            ('JavaScript', '💛', 'yellow', 'Web 开发语言', 1),
            ('Java', '☕', 'red', '企业级开发语言', 1),
            ('C++', '⚡', 'blue', '高性能系统语言', 1),
            ('Kotlin', '🤖', 'purple', '现代编程语言', 1),
            ('Swift', '🍎', 'orange', 'Apple 平台开发语言', 1),
            ('Dart', '🎯', 'cyan', 'Flutter 开发语言', 1),
            ('PHP', '🐘', 'indigo', 'Web 开发语言', 1),
            ('Ruby', '💎', 'red', '优雅的编程语言', 1),
            ('C#', '🔷', 'purple', 'Microsoft 开发语言', 1)"
        )
        .execute(&self.pool)
        .await?;

        // 3. 重建 datasources 表（删除旧表，创建新表）
        sqlx::query("DROP TABLE IF EXISTS datasources")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE TABLE datasources (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                host TEXT,
                port INTEGER,
                username TEXT,
                password TEXT,
                database TEXT,
                sqlite_file TEXT,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_datasources_name ON datasources(name)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_datasources_type ON datasources(type)")
            .execute(&self.pool)
            .await?;

        // 4. 重建 projects 表（添加主语言字段）
        sqlx::query("DROP TABLE IF EXISTS db_columns")
            .execute(&self.pool)
            .await?;

        sqlx::query("DROP TABLE IF EXISTS db_tables")
            .execute(&self.pool)
            .await?;

        sqlx::query("DROP TABLE IF EXISTS project_languages")
            .execute(&self.pool)
            .await?;

        sqlx::query("DROP TABLE IF EXISTS projects")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE TABLE projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT,
                datasource_id INTEGER NOT NULL,
                database_name TEXT NOT NULL,
                primary_language_id INTEGER,
                table_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (datasource_id) REFERENCES datasources(id) ON DELETE CASCADE,
                FOREIGN KEY (primary_language_id) REFERENCES languages(id)
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_projects_name ON projects(name)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_projects_datasource_id ON projects(datasource_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_projects_primary_language_id ON projects(primary_language_id)")
            .execute(&self.pool)
            .await?;

        // 5. 创建 project_languages 表（多对多关系）
        sqlx::query(
            "CREATE TABLE project_languages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                language_id INTEGER NOT NULL,
                is_primary INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                FOREIGN KEY (language_id) REFERENCES languages(id) ON DELETE CASCADE,
                UNIQUE(project_id, language_id)
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_project_languages_project_id ON project_languages(project_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_project_languages_language_id ON project_languages(language_id)")
            .execute(&self.pool)
            .await?;

        // 6. 重新创建空的 db_tables 和 db_columns 表
        sqlx::query(
            "CREATE TABLE db_tables (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                comment TEXT,
                engine TEXT,
                table_type TEXT NOT NULL DEFAULT 'table',
                row_count INTEGER NOT NULL DEFAULT 0,
                column_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tables_project_id ON db_tables(project_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE TABLE db_columns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                table_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                data_type TEXT NOT NULL,
                length INTEGER,
                is_nullable INTEGER NOT NULL DEFAULT 1,
                is_primary_key INTEGER NOT NULL DEFAULT 0,
                is_unique INTEGER NOT NULL DEFAULT 0,
                default_value TEXT,
                comment TEXT,
                ordinal_position INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (table_id) REFERENCES db_tables(id) ON DELETE CASCADE,
                UNIQUE(table_id, name)
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_columns_table_id ON db_columns(table_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("INSERT INTO schema_migrations (version) VALUES (5)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 迁移 006: 添加 database 列到 datasources 表
    async fn migration_006_add_database_column(&self) -> Result<(), sqlx::Error> {
        println!("执行迁移 006: 添加 database 列");

        // 检查列是否存在
        let column_exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) = 1 FROM pragma_table_info('datasources') WHERE name = 'database'"
        )
        .fetch_one(&self.pool)
        .await?;

        if !column_exists {
            println!("  添加 database 列到 datasources 表");
            sqlx::query("ALTER TABLE datasources ADD COLUMN database TEXT")
                .execute(&self.pool)
                .await?;
        } else {
            println!("  database 列已存在，跳过");
        }

        sqlx::query("INSERT INTO schema_migrations (version) VALUES (6)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 迁移 007: 创建 AI 提供商和模型表
    async fn migration_007_create_ai_tables(&self) -> Result<(), sqlx::Error> {
        println!("执行迁移 007: 创建 AI 提供商和模型表");

        // 创建 AI 提供商表
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS ai_providers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                provider_name TEXT UNIQUE NOT NULL,
                display_name TEXT NOT NULL,
                provider_type TEXT NOT NULL,
                api_key TEXT,
                api_endpoint TEXT,
                is_enabled INTEGER DEFAULT 0,
                is_default INTEGER DEFAULT 0,
                temperature REAL DEFAULT 0.7,
                max_tokens INTEGER DEFAULT 4096,
                timeout_seconds INTEGER DEFAULT 30,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        )
        .execute(&self.pool)
        .await?;

        // 创建 AI 模型表
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS ai_models (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                model_id TEXT NOT NULL,
                model_name TEXT NOT NULL,
                provider_name TEXT NOT NULL,
                group_id TEXT DEFAULT 'chat',
                description TEXT,
                max_tokens INTEGER DEFAULT 4096,
                supports_functions INTEGER DEFAULT 0,
                supports_vision INTEGER DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (provider_name) REFERENCES ai_providers(provider_name)
            )"
        )
        .execute(&self.pool)
        .await?;

        // 创建索引
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_ai_providers_name ON ai_providers(provider_name)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_ai_providers_type ON ai_providers(provider_type)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_ai_models_provider ON ai_models(provider_name)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_ai_models_group ON ai_models(group_id)")
            .execute(&self.pool)
            .await?;

        // 插入默认 AI 提供商
        sqlx::query(
            "INSERT OR IGNORE INTO ai_providers (
                provider_name, display_name, provider_type, api_endpoint, is_enabled
            ) VALUES
                ('deepseek', 'DeepSeek', 'deepseek', 'https://api.deepseek.com/v1', 0),
                ('glm', '智谱 GLM', 'glm', 'https://open.bigmodel.cn/api/paas/v4', 0)"
        )
        .execute(&self.pool)
        .await?;

        // 插入默认模型
        sqlx::query(
            "INSERT OR IGNORE INTO ai_models (model_id, model_name, provider_name, group_id, description) VALUES
                ('deepseek-chat', 'DeepSeek Chat', 'deepseek', 'chat', 'DeepSeek 对话模型'),
                ('deepseek-coder', 'DeepSeek Coder', 'deepseek', 'code', 'DeepSeek 代码模型'),
                ('glm-4', 'GLM-4', 'glm', 'chat', '智谱 GLM-4 模型'),
                ('glm-4-flash', 'GLM-4 Flash', 'glm', 'chat', '智谱 GLM-4 Flash 快速模型'),
                ('glm-4-plus', 'GLM-4 Plus', 'glm', 'chat', '智谱 GLM-4 Plus 增强模型')"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("INSERT INTO schema_migrations (version) VALUES (7)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 迁移 008: 创建 table_preferences 表
    async fn migration_008_add_table_preferences(&self) -> Result<(), sqlx::Error> {
        println!("执行迁移 008: 创建 table_preferences 表");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS table_preferences (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,

                -- 主键规范
                pk_enabled INTEGER DEFAULT 1,
                pk_field_name TEXT DEFAULT 'id',
                pk_field_type TEXT DEFAULT 'BIGINT',
                pk_auto_increment INTEGER DEFAULT 1,
                pk_comment TEXT,

                -- 审计字段配置
                audit_enabled INTEGER DEFAULT 1,
                audit_fields TEXT,

                -- 软删除字段配置
                soft_delete_enabled INTEGER DEFAULT 0,
                soft_delete_field TEXT DEFAULT 'deleted_at',
                soft_delete_field_type TEXT DEFAULT 'TIMESTAMP',
                soft_delete_nullable INTEGER DEFAULT 1,
                soft_delete_default TEXT,
                soft_delete_comment TEXT,

                -- 命名规范
                boolean_prefix TEXT DEFAULT 'is_',
                datetime_suffix TEXT DEFAULT '_at',

                -- 其他配置
                engine_type TEXT DEFAULT 'InnoDB',
                charset TEXT DEFAULT 'utf8mb4',
                collation TEXT DEFAULT 'utf8mb4_unicode_ci',

                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),

                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            )"
        )
        .execute(&self.pool)
        .await?;

        // 创建索引
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_table_preferences_project ON table_preferences(project_id)")
            .execute(&self.pool)
            .await?;

        // 插入默认配置（对于现有项目）
        sqlx::query(
            "INSERT INTO table_preferences (
                project_id,
                pk_enabled,
                pk_field_name,
                pk_field_type,
                pk_auto_increment,
                pk_comment,
                audit_enabled,
                audit_fields,
                soft_delete_enabled
            ) SELECT
                id,
                1,
                'id',
                'BIGINT',
                1,
                '主键',
                1,
                '[{\"field\":\"created_at\",\"type\":\"TIMESTAMP\",\"default\":\"CURRENT_TIMESTAMP\",\"nullable\":false,\"comment\":\"创建时间\"},{\"field\":\"updated_at\",\"type\":\"TIMESTAMP\",\"default\":\"CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP\",\"nullable\":false,\"comment\":\"更新时间\"}]',
                0
            FROM projects"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("INSERT INTO schema_migrations (version) VALUES (8)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 迁移 009: 创建语言类型字段表
    async fn migration_009_create_language_field_types(&self) -> Result<(), sqlx::Error> {
        println!("执行迁移 009: 创建 language_field_types 表");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS language_field_types (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                language_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                is_builtin INTEGER NOT NULL DEFAULT 0,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (language_id) REFERENCES languages(id) ON DELETE CASCADE,
                UNIQUE(language_id, name)
            )"
        )
        .execute(&self.pool)
        .await?;

        // 创建索引
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_language_field_types_language_id ON language_field_types(language_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_language_field_types_name ON language_field_types(name)")
            .execute(&self.pool)
            .await?;

        sqlx::query("INSERT INTO schema_migrations (version) VALUES (9)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 迁移 010: 创建项目级类型映射表
    async fn migration_010_create_project_mappings(&self) -> Result<(), sqlx::Error> {
        println!("执行迁移 010: 创建项目级类型映射表");

        // 1. 给 projects 表添加前后端语言字段
        sqlx::query("ALTER TABLE projects ADD COLUMN frontend_language_id INTEGER")
            .execute(&self.pool)
            .await
            .ok(); // 忽略已存在的错误

        sqlx::query("ALTER TABLE projects ADD COLUMN backend_language_id INTEGER")
            .execute(&self.pool)
            .await
            .ok();

        // 2. 创建系统级类型映射模板表
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS system_type_mappings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                language_id INTEGER NOT NULL,
                db_type TEXT NOT NULL,
                pattern TEXT NOT NULL,
                target_type TEXT,
                priority INTEGER NOT NULL DEFAULT 10,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (language_id) REFERENCES languages(id) ON DELETE CASCADE,
                UNIQUE(language_id, db_type, pattern)
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_system_mappings_language ON system_type_mappings(language_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_system_mappings_db ON system_type_mappings(db_type)")
            .execute(&self.pool)
            .await?;

        // 3. 创建项目级类型映射表
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS project_type_mappings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                scope TEXT NOT NULL CHECK(scope IN ('frontend', 'backend')),
                db_type TEXT NOT NULL,
                pattern TEXT NOT NULL,
                target_type TEXT,
                priority INTEGER NOT NULL DEFAULT 10,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                UNIQUE(project_id, scope, db_type, pattern)
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_project_mappings_project ON project_type_mappings(project_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_project_mappings_scope ON project_type_mappings(scope)")
            .execute(&self.pool)
            .await?;

        // 4. 初始化系统默认映射数据
        self.init_default_system_mappings().await?;

        sqlx::query("INSERT INTO schema_migrations (version) VALUES (10)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 初始化系统默认类型映射数据
    async fn init_default_system_mappings(&self) -> Result<(), sqlx::Error> {
        println!("初始化系统默认类型映射数据");

        // 语言到默认类型映射的配置
        let language_mappings = vec![
            // Rust
            ("Rust", vec![
                ("mysql", "VARCHAR(%)", "String"),
                ("mysql", "CHAR(%)", "String"),
                ("mysql", "TEXT", "String"),
                ("mysql", "LONGTEXT", "String"),
                ("mysql", "INT", "i32"),
                ("mysql", "BIGINT", "i64"),
                ("mysql", "SMALLINT", "i16"),
                ("mysql", "TINYINT(1)", "bool"),
                ("mysql", "TINYINT(%)", "i8"),
                ("mysql", "DECIMAL(%,%)", "Decimal"),
                ("mysql", "FLOAT", "f32"),
                ("mysql", "DOUBLE", "f64"),
                ("mysql", "BOOLEAN", "bool"),
                ("mysql", "DATE", "NaiveDate"),
                ("mysql", "TIMESTAMP", "NaiveDateTime"),
                ("mysql", "DATETIME", "NaiveDateTime"),
                ("mysql", "TIME", "NaiveTime"),
                ("mysql", "BLOB", "Vec<u8>"),
                ("mysql", "JSON", "serde_json::Value"),
            ]),
            // Go
            ("Go", vec![
                ("mysql", "VARCHAR(%)", "string"),
                ("mysql", "CHAR(%)", "string"),
                ("mysql", "TEXT", "string"),
                ("mysql", "LONGTEXT", "string"),
                ("mysql", "INT", "int32"),
                ("mysql", "BIGINT", "int64"),
                ("mysql", "SMALLINT", "int16"),
                ("mysql", "TINYINT(1)", "bool"),
                ("mysql", "TINYINT(%)", "int8"),
                ("mysql", "DECIMAL(%,%)", "float64"),
                ("mysql", "FLOAT", "float32"),
                ("mysql", "DOUBLE", "float64"),
                ("mysql", "BOOLEAN", "bool"),
                ("mysql", "DATE", "time.Time"),
                ("mysql", "TIMESTAMP", "time.Time"),
                ("mysql", "DATETIME", "time.Time"),
                ("mysql", "TIME", "time.Time"),
                ("mysql", "BLOB", "[]byte"),
                ("mysql", "JSON", "interface{}"),
            ]),
            // Python
            ("Python", vec![
                ("mysql", "VARCHAR(%)", "str"),
                ("mysql", "CHAR(%)", "str"),
                ("mysql", "TEXT", "str"),
                ("mysql", "LONGTEXT", "str"),
                ("mysql", "INT", "int"),
                ("mysql", "BIGINT", "int"),
                ("mysql", "SMALLINT", "int"),
                ("mysql", "TINYINT(1)", "bool"),
                ("mysql", "TINYINT(%)", "int"),
                ("mysql", "DECIMAL(%,%)", "Decimal"),
                ("mysql", "FLOAT", "float"),
                ("mysql", "DOUBLE", "float"),
                ("mysql", "BOOLEAN", "bool"),
                ("mysql", "DATE", "datetime.date"),
                ("mysql", "TIMESTAMP", "datetime.datetime"),
                ("mysql", "DATETIME", "datetime.datetime"),
                ("mysql", "TIME", "datetime.time"),
                ("mysql", "BLOB", "bytes"),
                ("mysql", "JSON", "dict"),
            ]),
            // TypeScript
            ("TypeScript", vec![
                ("mysql", "VARCHAR(%)", "string"),
                ("mysql", "CHAR(%)", "string"),
                ("mysql", "TEXT", "string"),
                ("mysql", "LONGTEXT", "string"),
                ("mysql", "INT", "number"),
                ("mysql", "BIGINT", "number"),
                ("mysql", "SMALLINT", "number"),
                ("mysql", "TINYINT(1)", "boolean"),
                ("mysql", "TINYINT(%)", "number"),
                ("mysql", "DECIMAL(%,%)", "number"),
                ("mysql", "FLOAT", "number"),
                ("mysql", "DOUBLE", "number"),
                ("mysql", "BOOLEAN", "boolean"),
                ("mysql", "DATE", "Date"),
                ("mysql", "TIMESTAMP", "Date"),
                ("mysql", "DATETIME", "Date"),
                ("mysql", "TIME", "Date"),
                ("mysql", "BLOB", "Buffer"),
                ("mysql", "JSON", "any"),
            ]),
            // JavaScript
            ("JavaScript", vec![
                ("mysql", "VARCHAR(%)", "string"),
                ("mysql", "CHAR(%)", "string"),
                ("mysql", "TEXT", "string"),
                ("mysql", "LONGTEXT", "string"),
                ("mysql", "INT", "number"),
                ("mysql", "BIGINT", "number"),
                ("mysql", "SMALLINT", "number"),
                ("mysql", "TINYINT(1)", "boolean"),
                ("mysql", "TINYINT(%)", "number"),
                ("mysql", "DECIMAL(%,%)", "number"),
                ("mysql", "FLOAT", "number"),
                ("mysql", "DOUBLE", "number"),
                ("mysql", "BOOLEAN", "boolean"),
                ("mysql", "DATE", "Date"),
                ("mysql", "TIMESTAMP", "Date"),
                ("mysql", "DATETIME", "Date"),
                ("mysql", "TIME", "Date"),
                ("mysql", "BLOB", "Buffer"),
                ("mysql", "JSON", "any"),
            ]),
            // Java
            ("Java", vec![
                ("mysql", "VARCHAR(%)", "String"),
                ("mysql", "CHAR(%)", "String"),
                ("mysql", "TEXT", "String"),
                ("mysql", "LONGTEXT", "String"),
                ("mysql", "INT", "Integer"),
                ("mysql", "BIGINT", "Long"),
                ("mysql", "SMALLINT", "Short"),
                ("mysql", "TINYINT(1)", "Boolean"),
                ("mysql", "TINYINT(%)", "Byte"),
                ("mysql", "DECIMAL(%,%)", "BigDecimal"),
                ("mysql", "FLOAT", "Float"),
                ("mysql", "DOUBLE", "Double"),
                ("mysql", "BOOLEAN", "Boolean"),
                ("mysql", "DATE", "LocalDate"),
                ("mysql", "TIMESTAMP", "LocalDateTime"),
                ("mysql", "DATETIME", "LocalDateTime"),
                ("mysql", "TIME", "LocalTime"),
                ("mysql", "BLOB", "byte[]"),
                ("mysql", "JSON", "String"),
            ]),
            // C++
            ("C++", vec![
                ("mysql", "VARCHAR(%)", "std::string"),
                ("mysql", "CHAR(%)", "std::string"),
                ("mysql", "TEXT", "std::string"),
                ("mysql", "LONGTEXT", "std::string"),
                ("mysql", "INT", "int32_t"),
                ("mysql", "BIGINT", "int64_t"),
                ("mysql", "SMALLINT", "int16_t"),
                ("mysql", "TINYINT(1)", "bool"),
                ("mysql", "TINYINT(%)", "int8_t"),
                ("mysql", "DECIMAL(%,%)", "double"),
                ("mysql", "FLOAT", "float"),
                ("mysql", "DOUBLE", "double"),
                ("mysql", "BOOLEAN", "bool"),
                ("mysql", "DATE", "std::string"),
                ("mysql", "TIMESTAMP", "std::string"),
                ("mysql", "DATETIME", "std::string"),
                ("mysql", "TIME", "std::string"),
                ("mysql", "BLOB", "std::vector<char>"),
                ("mysql", "JSON", "std::string"),
            ]),
            // Kotlin
            ("Kotlin", vec![
                ("mysql", "VARCHAR(%)", "String"),
                ("mysql", "CHAR(%)", "String"),
                ("mysql", "TEXT", "String"),
                ("mysql", "LONGTEXT", "String"),
                ("mysql", "INT", "Int"),
                ("mysql", "BIGINT", "Long"),
                ("mysql", "SMALLINT", "Short"),
                ("mysql", "TINYINT(1)", "Boolean"),
                ("mysql", "TINYINT(%)", "Byte"),
                ("mysql", "DECIMAL(%,%)", "BigDecimal"),
                ("mysql", "FLOAT", "Float"),
                ("mysql", "DOUBLE", "Double"),
                ("mysql", "BOOLEAN", "Boolean"),
                ("mysql", "DATE", "LocalDate"),
                ("mysql", "TIMESTAMP", "LocalDateTime"),
                ("mysql", "DATETIME", "LocalDateTime"),
                ("mysql", "TIME", "LocalTime"),
                ("mysql", "BLOB", "ByteArray"),
                ("mysql", "JSON", "String"),
            ]),
            // Swift
            ("Swift", vec![
                ("mysql", "VARCHAR(%)", "String"),
                ("mysql", "CHAR(%)", "String"),
                ("mysql", "TEXT", "String"),
                ("mysql", "LONGTEXT", "String"),
                ("mysql", "INT", "Int32"),
                ("mysql", "BIGINT", "Int64"),
                ("mysql", "SMALLINT", "Int16"),
                ("mysql", "TINYINT(1)", "Bool"),
                ("mysql", "TINYINT(%)", "Int8"),
                ("mysql", "DECIMAL(%,%)", "Double"),
                ("mysql", "FLOAT", "Float"),
                ("mysql", "DOUBLE", "Double"),
                ("mysql", "BOOLEAN", "Bool"),
                ("mysql", "DATE", "Date"),
                ("mysql", "TIMESTAMP", "Date"),
                ("mysql", "DATETIME", "Date"),
                ("mysql", "TIME", "Date"),
                ("mysql", "BLOB", "Data"),
                ("mysql", "JSON", "Any"),
            ]),
            // Dart
            ("Dart", vec![
                ("mysql", "VARCHAR(%)", "String"),
                ("mysql", "CHAR(%)", "String"),
                ("mysql", "TEXT", "String"),
                ("mysql", "LONGTEXT", "String"),
                ("mysql", "INT", "int"),
                ("mysql", "BIGINT", "int"),
                ("mysql", "SMALLINT", "int"),
                ("mysql", "TINYINT(1)", "bool"),
                ("mysql", "TINYINT(%)", "int"),
                ("mysql", "DECIMAL(%,%)", "double"),
                ("mysql", "FLOAT", "double"),
                ("mysql", "DOUBLE", "double"),
                ("mysql", "BOOLEAN", "bool"),
                ("mysql", "DATE", "DateTime"),
                ("mysql", "TIMESTAMP", "DateTime"),
                ("mysql", "DATETIME", "DateTime"),
                ("mysql", "TIME", "DateTime"),
                ("mysql", "BLOB", "Uint8List"),
                ("mysql", "JSON", "dynamic"),
            ]),
            // PHP
            ("PHP", vec![
                ("mysql", "VARCHAR(%)", "string"),
                ("mysql", "CHAR(%)", "string"),
                ("mysql", "TEXT", "string"),
                ("mysql", "LONGTEXT", "string"),
                ("mysql", "INT", "int"),
                ("mysql", "BIGINT", "int"),
                ("mysql", "SMALLINT", "int"),
                ("mysql", "TINYINT(1)", "bool"),
                ("mysql", "TINYINT(%)", "int"),
                ("mysql", "DECIMAL(%,%)", "float"),
                ("mysql", "FLOAT", "float"),
                ("mysql", "DOUBLE", "float"),
                ("mysql", "BOOLEAN", "bool"),
                ("mysql", "DATE", "DateTime"),
                ("mysql", "TIMESTAMP", "DateTime"),
                ("mysql", "DATETIME", "DateTime"),
                ("mysql", "TIME", "DateTime"),
                ("mysql", "BLOB", "string"),
                ("mysql", "JSON", "array"),
            ]),
            // Ruby
            ("Ruby", vec![
                ("mysql", "VARCHAR(%)", "String"),
                ("mysql", "CHAR(%)", "String"),
                ("mysql", "TEXT", "String"),
                ("mysql", "LONGTEXT", "String"),
                ("mysql", "INT", "Integer"),
                ("mysql", "BIGINT", "Integer"),
                ("mysql", "SMALLINT", "Integer"),
                ("mysql", "TINYINT(1)", "Boolean"),
                ("mysql", "TINYINT(%)", "Integer"),
                ("mysql", "DECIMAL(%,%)", "Float"),
                ("mysql", "FLOAT", "Float"),
                ("mysql", "DOUBLE", "Float"),
                ("mysql", "BOOLEAN", "Boolean"),
                ("mysql", "DATE", "Date"),
                ("mysql", "TIMESTAMP", "Time"),
                ("mysql", "DATETIME", "Time"),
                ("mysql", "TIME", "Time"),
                ("mysql", "BLOB", "String"),
                ("mysql", "JSON", "Hash"),
            ]),
            // C#
            ("C#", vec![
                ("mysql", "VARCHAR(%)", "string"),
                ("mysql", "CHAR(%)", "string"),
                ("mysql", "TEXT", "string"),
                ("mysql", "LONGTEXT", "string"),
                ("mysql", "INT", "int"),
                ("mysql", "BIGINT", "long"),
                ("mysql", "SMALLINT", "short"),
                ("mysql", "TINYINT(1)", "bool"),
                ("mysql", "TINYINT(%)", "byte"),
                ("mysql", "DECIMAL(%,%)", "decimal"),
                ("mysql", "FLOAT", "float"),
                ("mysql", "DOUBLE", "double"),
                ("mysql", "BOOLEAN", "bool"),
                ("mysql", "DATE", "DateTime"),
                ("mysql", "TIMESTAMP", "DateTime"),
                ("mysql", "DATETIME", "DateTime"),
                ("mysql", "TIME", "TimeSpan"),
                ("mysql", "BLOB", "byte[]"),
                ("mysql", "JSON", "string"),
            ]),
        ];

        // 为每种语言和数据库类型初始化默认映射
        for (lang_name, mappings) in language_mappings {
            // 获取语言 ID
            let lang_id = match sqlx::query_scalar::<_, i64>("SELECT id FROM languages WHERE name = ?")
                .bind(lang_name)
                .fetch_one(&self.pool)
                .await {
                Ok(id) => id,
                Err(_) => continue, // 语言不存在，跳过
            };

            // 插入映射
            for (db_type, pattern, target_type) in mappings {
                // 计算优先级
                let priority = match pattern {
                    "TINYINT(1)" => 20, // 精确匹配优先
                    _ if pattern.contains("(%)") => 10,
                    _ => 10,
                };

                sqlx::query(
                    "INSERT OR IGNORE INTO system_type_mappings (language_id, db_type, pattern, target_type, priority)
                     VALUES (?, ?, ?, ?, ?)"
                )
                .bind(lang_id)
                .bind(db_type)
                .bind(pattern)
                .bind(target_type)
                .bind(priority)
                .execute(&self.pool)
                .await?;
            }
        }

        println!("系统默认类型映射数据初始化完成");
        Ok(())
    }

    /// 获取数据库连接池的引用
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

/// 数据库操作 API
impl Database {
    /// ===== 项目操作 =====

    /// 创建项目
    pub async fn create_project(
        &self,
        name: &str,
        description: Option<&str>,
        datasource_id: i64,
        database_name: &str,
        primary_language_id: Option<i64>,
        frontend_language_id: Option<i64>,
        backend_language_id: Option<i64>,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO projects (name, description, datasource_id, database_name, primary_language_id, frontend_language_id, backend_language_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
        )
        .bind(name)
        .bind(description)
        .bind(datasource_id)
        .bind(database_name)
        .bind(primary_language_id)
        .bind(frontend_language_id)
        .bind(backend_language_id)
        .execute(&self.pool)
        .await?;

        let project_id = result.last_insert_rowid();

        // 如果指定了前端语言，复制系统级映射到项目级
        if let Some(lang_id) = frontend_language_id {
            self.copy_system_mappings_to_project(project_id, lang_id, "frontend", "mysql").await?;
        }

        // 如果指定了后端语言，复制系统级映射到项目级
        if let Some(lang_id) = backend_language_id {
            self.copy_system_mappings_to_project(project_id, lang_id, "backend", "mysql").await?;
        }

        Ok(project_id)
    }

    /// 获取所有项目
    pub async fn get_all_projects(&self) -> Result<Vec<Project>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, name, description, datasource_id, database_name, primary_language_id, frontend_language_id, backend_language_id, table_count, created_at, updated_at
             FROM projects
             ORDER BY updated_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let projects = rows.into_iter().map(|row| {
            Project {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                datasource_id: row.get("datasource_id"),
                database_name: row.get("database_name"),
                primary_language_id: row.try_get("primary_language_id").ok(),
                frontend_language_id: row.try_get("frontend_language_id").ok(),
                backend_language_id: row.try_get("backend_language_id").ok(),
                table_count: row.get("table_count"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                datasource: None,
                primary_language: None,
                frontend_language: None,
                backend_language: None,
                languages: None,
            }
        }).collect();

        Ok(projects)
    }

    /// 根据 ID 获取项目
    pub async fn get_project(&self, id: i64) -> Result<Option<Project>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, description, datasource_id, database_name, primary_language_id, frontend_language_id, backend_language_id, table_count, created_at, updated_at
             FROM projects
             WHERE id = ?1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            Project {
                id: r.get("id"),
                name: r.get("name"),
                description: r.get("description"),
                datasource_id: r.get("datasource_id"),
                database_name: r.get("database_name"),
                primary_language_id: r.try_get("primary_language_id").ok(),
                frontend_language_id: r.try_get("frontend_language_id").ok(),
                backend_language_id: r.try_get("backend_language_id").ok(),
                table_count: r.get("table_count"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                datasource: None,
                primary_language: None,
                frontend_language: None,
                backend_language: None,
                languages: None,
            }
        }))
    }

    /// 更新项目
    pub async fn update_project(
        &self,
        id: i64,
        name: Option<&str>,
        description: Option<&str>,
        primary_language_id: Option<i64>,
        frontend_language_id: Option<i64>,
        backend_language_id: Option<i64>,
    ) -> Result<(), sqlx::Error> {
        if let Some(project_name) = name {
            sqlx::query(
                "UPDATE projects SET name = ?1, updated_at = datetime('now') WHERE id = ?2"
            )
            .bind(project_name)
            .bind(id)
            .execute(&self.pool)
            .await?;
        }

        if let Some(desc) = description {
            sqlx::query(
                "UPDATE projects SET description = ?1, updated_at = datetime('now') WHERE id = ?2"
            )
            .bind(desc)
            .bind(id)
            .execute(&self.pool)
            .await?;
        }

        if let Some(lang_id) = primary_language_id {
            // 只更新主语言字段，不添加到 project_languages 表
            sqlx::query("UPDATE projects SET primary_language_id = ?1, updated_at = datetime('now') WHERE id = ?2")
                .bind(lang_id)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        if let Some(lang_id) = frontend_language_id {
            sqlx::query("UPDATE projects SET frontend_language_id = ?1, updated_at = datetime('now') WHERE id = ?2")
                .bind(lang_id)
                .bind(id)
                .execute(&self.pool)
                .await?;

            // 复制系统级映射到项目级
            self.copy_system_mappings_to_project(id, lang_id, "frontend", "mysql").await?;
        }

        if let Some(lang_id) = backend_language_id {
            sqlx::query("UPDATE projects SET backend_language_id = ?1, updated_at = datetime('now') WHERE id = ?2")
                .bind(lang_id)
                .bind(id)
                .execute(&self.pool)
                .await?;

            // 复制系统级映射到项目级
            self.copy_system_mappings_to_project(id, lang_id, "backend", "mysql").await?;
        }

        Ok(())
    }

    /// 删除项目
    pub async fn delete_project(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM projects WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// ===== 系统级类型映射操作 =====

    /// 获取系统级类型映射
    pub async fn get_system_type_mappings(&self) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT
                stm.id,
                stm.language_id,
                l.name as language_name,
                stm.db_type,
                stm.pattern,
                stm.target_type,
                stm.priority,
                stm.created_at,
                stm.updated_at
             FROM system_type_mappings stm
             LEFT JOIN languages l ON stm.language_id = l.id
             ORDER BY l.name, stm.db_type, stm.priority DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mappings = rows.into_iter().map(|row| {
            serde_json::json!({
                "id": row.get::<i64, _>("id"),
                "language_id": row.get::<i64, _>("language_id"),
                "language_name": row.get::<String, _>("language_name"),
                "db_type": row.get::<String, _>("db_type"),
                "pattern": row.get::<String, _>("pattern"),
                "target_type": row.get::<String, _>("target_type"),
                "priority": row.get::<i32, _>("priority"),
                "created_at": row.get::<String, _>("created_at"),
                "updated_at": row.get::<String, _>("updated_at")
            })
        }).collect();

        Ok(mappings)
    }

    /// 根据语言和数据库类型获取系统级类型映射
    pub async fn get_system_type_mappings_by_lang_db(&self, language_id: i64, db_type: &str) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT
                stm.id,
                stm.language_id,
                l.name as language_name,
                stm.db_type,
                stm.pattern,
                stm.target_type,
                stm.priority,
                stm.created_at,
                stm.updated_at
             FROM system_type_mappings stm
             LEFT JOIN languages l ON stm.language_id = l.id
             WHERE stm.language_id = ?1 AND stm.db_type = ?2
             ORDER BY stm.priority DESC"
        )
        .bind(language_id)
        .bind(db_type)
        .fetch_all(&self.pool)
        .await?;

        let mappings = rows.into_iter().map(|row| {
            serde_json::json!({
                "id": row.get::<i64, _>("id"),
                "language_id": row.get::<i64, _>("language_id"),
                "language_name": row.get::<String, _>("language_name"),
                "db_type": row.get::<String, _>("db_type"),
                "pattern": row.get::<String, _>("pattern"),
                "target_type": row.get::<String, _>("target_type"),
                "priority": row.get::<i32, _>("priority"),
                "created_at": row.get::<String, _>("created_at"),
                "updated_at": row.get::<String, _>("updated_at")
            })
        }).collect();

        Ok(mappings)
    }

    /// 创建系统级类型映射
    pub async fn create_system_type_mapping(&self, language_id: i64, db_type: &str, pattern: &str, target_type: &str, priority: i32) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO system_type_mappings (language_id, db_type, pattern, target_type, priority)
             VALUES (?1, ?2, ?3, ?4, ?5)"
        )
        .bind(language_id)
        .bind(db_type)
        .bind(pattern)
        .bind(target_type)
        .bind(priority)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 更新系统级类型映射
    pub async fn update_system_type_mapping(&self, id: i64, target_type: &str, priority: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE system_type_mappings
             SET target_type = ?1, priority = ?2, updated_at = datetime('now')
             WHERE id = ?3"
        )
        .bind(target_type)
        .bind(priority)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除系统级类型映射
    pub async fn delete_system_type_mapping(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM system_type_mappings WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 批量保存系统级类型映射
    pub async fn batch_save_system_type_mappings(&self, mappings: Vec<serde_json::Value>) -> Result<(), sqlx::Error> {
        // 先删除所有现有映射
        sqlx::query("DELETE FROM system_type_mappings")
            .execute(&self.pool)
            .await?;

        // 重新插入新映射
        for mapping in mappings {
            let language_id: i64 = mapping["language_id"].as_i64().unwrap_or(0);
            let db_type: String = mapping["db_type"].as_str().unwrap_or("").to_string();
            let pattern: String = mapping["pattern"].as_str().unwrap_or("").to_string();
            let target_type: String = mapping["target_type"].as_str().unwrap_or("").to_string();
            let priority: i32 = mapping["priority"].as_i64().unwrap_or(10) as i32;

            if language_id > 0 && !db_type.is_empty() && !pattern.is_empty() && !target_type.is_empty() {
                sqlx::query(
                    "INSERT INTO system_type_mappings (language_id, db_type, pattern, target_type, priority)
                     VALUES (?1, ?2, ?3, ?4, ?5)"
                )
                .bind(language_id)
                .bind(db_type)
                .bind(pattern)
                .bind(target_type)
                .bind(priority)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    /// ===== 项目级类型映射操作 =====

    /// 获取项目级类型映射
    pub async fn get_project_type_mappings(&self, project_id: i64) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT
                ptm.id,
                ptm.project_id,
                ptm.scope,
                ptm.db_type,
                ptm.pattern,
                ptm.target_type,
                ptm.priority,
                ptm.created_at,
                ptm.updated_at
             FROM project_type_mappings ptm
             WHERE ptm.project_id = ?1
             ORDER BY ptm.scope, ptm.db_type, ptm.priority DESC"
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        let mappings = rows.into_iter().map(|row| {
            serde_json::json!({
                "id": row.get::<i64, _>("id"),
                "project_id": row.get::<i64, _>("project_id"),
                "scope": row.get::<String, _>("scope"),
                "db_type": row.get::<String, _>("db_type"),
                "pattern": row.get::<String, _>("pattern"),
                "target_type": row.get::<String, _>("target_type"),
                "priority": row.get::<i32, _>("priority"),
                "created_at": row.get::<String, _>("created_at"),
                "updated_at": row.get::<String, _>("updated_at")
            })
        }).collect();

        Ok(mappings)
    }

    /// 根据项目和范围获取项目级类型映射
    pub async fn get_project_type_mappings_by_scope(&self, project_id: i64, scope: &str) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT
                ptm.id,
                ptm.project_id,
                ptm.scope,
                ptm.db_type,
                ptm.pattern,
                ptm.target_type,
                ptm.priority,
                ptm.created_at,
                ptm.updated_at
             FROM project_type_mappings ptm
             WHERE ptm.project_id = ?1 AND ptm.scope = ?2
             ORDER BY ptm.db_type, ptm.priority DESC"
        )
        .bind(project_id)
        .bind(scope)
        .fetch_all(&self.pool)
        .await?;

        let mappings = rows.into_iter().map(|row| {
            serde_json::json!({
                "id": row.get::<i64, _>("id"),
                "project_id": row.get::<i64, _>("project_id"),
                "scope": row.get::<String, _>("scope"),
                "db_type": row.get::<String, _>("db_type"),
                "pattern": row.get::<String, _>("pattern"),
                "target_type": row.get::<String, _>("target_type"),
                "priority": row.get::<i32, _>("priority"),
                "created_at": row.get::<String, _>("created_at"),
                "updated_at": row.get::<String, _>("updated_at")
            })
        }).collect();

        Ok(mappings)
    }

    /// 创建项目级类型映射
    pub async fn create_project_type_mapping(&self, project_id: i64, scope: &str, db_type: &str, pattern: &str, target_type: &str, priority: i32) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO project_type_mappings (project_id, scope, db_type, pattern, target_type, priority)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        )
        .bind(project_id)
        .bind(scope)
        .bind(db_type)
        .bind(pattern)
        .bind(target_type)
        .bind(priority)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 更新项目级类型映射
    pub async fn update_project_type_mapping(&self, id: i64, target_type: &str, priority: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE project_type_mappings
             SET target_type = ?1, priority = ?2, updated_at = datetime('now')
             WHERE id = ?3"
        )
        .bind(target_type)
        .bind(priority)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除项目级类型映射
    pub async fn delete_project_type_mapping(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM project_type_mappings WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 批量保存项目级类型映射
    pub async fn batch_save_project_type_mappings(&self, project_id: i64, scope: &str, mappings: Vec<serde_json::Value>) -> Result<(), sqlx::Error> {
        // 先删除该项目该范围的所有现有映射
        sqlx::query("DELETE FROM project_type_mappings WHERE project_id = ?1 AND scope = ?2")
            .bind(project_id)
            .bind(scope)
            .execute(&self.pool)
            .await?;

        // 重新插入新映射
        for mapping in mappings {
            let db_type: String = mapping["db_type"].as_str().unwrap_or("").to_string();
            let pattern: String = mapping["pattern"].as_str().unwrap_or("").to_string();
            let target_type: String = mapping["target_type"].as_str().unwrap_or("").to_string();
            let priority: i32 = mapping["priority"].as_i64().unwrap_or(10) as i32;

            if !db_type.is_empty() && !pattern.is_empty() && !target_type.is_empty() {
                sqlx::query(
                    "INSERT INTO project_type_mappings (project_id, scope, db_type, pattern, target_type, priority)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
                )
                .bind(project_id)
                .bind(scope)
                .bind(db_type)
                .bind(pattern)
                .bind(target_type)
                .bind(priority)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    /// 复制系统级映射到项目级
    pub async fn copy_system_mappings_to_project(&self, project_id: i64, language_id: i64, scope: &str, db_type: &str) -> Result<(), sqlx::Error> {
        // 先删除该项目该范围的现有映射
        sqlx::query("DELETE FROM project_type_mappings WHERE project_id = ?1 AND scope = ?2 AND db_type = ?3")
            .bind(project_id)
            .bind(scope)
            .bind(db_type)
            .execute(&self.pool)
            .await?;

        // 复制系统级映射到项目级
        sqlx::query(
            "INSERT INTO project_type_mappings (project_id, scope, db_type, pattern, target_type, priority)
             SELECT ?1, ?2, stm.db_type, stm.pattern, stm.target_type, stm.priority
             FROM system_type_mappings stm
             WHERE stm.language_id = ?3 AND stm.db_type = ?4"
        )
        .bind(project_id)
        .bind(scope)
        .bind(language_id)
        .bind(db_type)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

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
        .bind(password)
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

        let datasources = rows.into_iter().map(|row| {
            Datasource {
                id: row.get("id"),
                name: row.get("name"),
                type_: row.get("type"),
                host: row.try_get("host").ok(),
                port: row.try_get("port").ok(),
                username: row.try_get("username").ok(),
                password: row.try_get("password").ok(),
                database: row.try_get("database").ok(),
                sqlite_file: row.try_get("sqlite_file").ok(),
                is_active: row.get::<i32, _>("is_active") == 1,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }).collect();

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
                password: row.try_get("password").ok(),
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
        .bind(password)
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
    pub async fn test_datasource_connection(params: TestConnectionParams) -> Result<String, String> {
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
                    .map(|_| if database.is_empty() {
                        "MySQL 服务器连接成功".to_string()
                    } else {
                        format!("MySQL 数据库 '{}' 连接成功", database)
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
                let sqlite_file = params.sqlite_file.as_deref()
                    .ok_or_else(|| "SQLite 文件路径未指定".to_string())?;

                if !std::path::Path::new(sqlite_file).exists() {
                    return Err(format!("SQLite 文件不存在: {}", sqlite_file));
                }

                sqlx::sqlite::SqlitePool::connect(&format!("sqlite:{}", sqlite_file))
                    .await
                    .map(|_| "SQLite 连接成功".to_string())
                    .map_err(|e| format!("SQLite 连接失败: {}", e))
            }
            _ => Err(format!("不支持的数据库类型: {}", params.type_))
        }
    }

    /// ===== 表操作 =====

    /// 为项目创建表
    pub async fn create_table(
        &self,
        project_id: i64,
        name: &str,
        comment: Option<&str>,
        engine: Option<&str>,
        table_type: &str,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO db_tables (project_id, name, comment, engine, table_type)
             VALUES (?1, ?2, ?3, ?4, ?5)"
        )
        .bind(project_id)
        .bind(name)
        .bind(comment)
        .bind(engine)
        .bind(table_type)
        .execute(&self.pool)
        .await?;

        // 更新项目的表计数
        sqlx::query(
            "UPDATE projects SET table_count = (
                SELECT COUNT(*) FROM db_tables WHERE project_id = ?1
            ), updated_at = datetime('now') WHERE id = ?1"
        )
        .bind(project_id)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 获取项目的所有表
    pub async fn get_project_tables(&self, project_id: i64) -> Result<Vec<DbTable>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, project_id, name, comment, engine, table_type, row_count, column_count, created_at, updated_at
             FROM db_tables
             WHERE project_id = ?1
             ORDER BY name"
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        let tables = rows.into_iter().map(|row| {
            DbTable {
                id: row.get("id"),
                project_id: row.get("project_id"),
                name: row.get("name"),
                comment: row.get("comment"),
                engine: row.get("engine"),
                table_type: row.get("table_type"),
                row_count: row.get("row_count"),
                column_count: row.get("column_count"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }).collect();

        Ok(tables)
    }

    /// 创建列记录
    pub async fn create_column(
        &self,
        table_id: i64,
        name: &str,
        data_type: &str,
        length: Option<i64>,
        is_nullable: bool,
        is_primary_key: bool,
        is_unique: bool,
        default_value: Option<&str>,
        comment: Option<&str>,
        ordinal_position: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO db_columns (table_id, name, data_type, length, is_nullable, is_primary_key, is_unique, default_value, comment, ordinal_position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
        )
        .bind(table_id)
        .bind(name)
        .bind(data_type)
        .bind(length)
        .bind(is_nullable as i32)
        .bind(is_primary_key as i32)
        .bind(is_unique as i32)
        .bind(default_value)
        .bind(comment)
        .bind(ordinal_position)
        .execute(&self.pool)
        .await?;

        // 更新表的列计数
        sqlx::query(
            "UPDATE db_tables SET column_count = (
                SELECT COUNT(*) FROM db_columns WHERE table_id = ?1
            ), updated_at = datetime('now') WHERE id = ?1"
        )
        .bind(table_id)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 获取表的所有列
    pub async fn get_table_columns(&self, table_id: i64) -> Result<Vec<DbColumn>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, table_id, name, data_type, length, is_nullable, is_primary_key, is_unique, default_value, comment, ordinal_position, created_at
             FROM db_columns
             WHERE table_id = ?1
             ORDER BY ordinal_position"
        )
        .bind(table_id)
        .fetch_all(&self.pool)
        .await?;

        let columns = rows.into_iter().map(|row| {
            DbColumn {
                id: row.get("id"),
                table_id: row.get("table_id"),
                name: row.get("name"),
                data_type: row.get("data_type"),
                length: row.get("length"),
                is_nullable: row.get::<i32, _>("is_nullable") == 1,
                is_primary_key: row.get::<i32, _>("is_primary_key") == 1,
                is_unique: row.get::<i32, _>("is_unique") == 1,
                default_value: row.get("default_value"),
                comment: row.get("comment"),
                ordinal_position: row.get("ordinal_position"),
                created_at: row.get("created_at"),
            }
        }).collect();

        Ok(columns)
    }

    /// 删除表
    pub async fn delete_table(&self, table_id: i64) -> Result<(), sqlx::Error> {
        // 删除表（由于有外键约束，列会自动级联删除）
        sqlx::query("DELETE FROM db_tables WHERE id = ?1")
            .bind(table_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 更新表信息
    pub async fn update_table(
        &self,
        table_id: i64,
        name: &str,
        comment: Option<&str>,
        engine: Option<&str>,
        table_type: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE db_tables
             SET name = ?1, comment = ?2, engine = ?3, table_type = ?4, updated_at = datetime('now')
             WHERE id = ?5"
        )
        .bind(name)
        .bind(comment)
        .bind(engine)
        .bind(table_type)
        .bind(table_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 更新列信息
    pub async fn update_column(
        &self,
        column_id: i64,
        name: &str,
        data_type: &str,
        length: Option<i64>,
        is_nullable: bool,
        is_primary_key: bool,
        is_unique: bool,
        default_value: Option<&str>,
        comment: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE db_columns
             SET name = ?1, data_type = ?2, length = ?3, is_nullable = ?4,
                 is_primary_key = ?5, is_unique = ?6, default_value = ?7, comment = ?8
             WHERE id = ?9"
        )
        .bind(name)
        .bind(data_type)
        .bind(length)
        .bind(is_nullable)
        .bind(is_primary_key)
        .bind(is_unique)
        .bind(default_value)
        .bind(comment)
        .bind(column_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除列
    pub async fn delete_column(&self, column_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM db_columns WHERE id = ?1")
            .bind(column_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// ===== 语言操作 =====

    /// 创建语言
    pub async fn create_language(
        &self,
        name: &str,
        icon: Option<&str>,
        color: Option<&str>,
        description: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO languages (name, icon, color, description) VALUES (?1, ?2, ?3, ?4)"
        )
        .bind(name)
        .bind(icon)
        .bind(color)
        .bind(description)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 获取所有语言
    pub async fn get_all_languages(&self) -> Result<Vec<Language>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, name, icon, color, description, is_builtin, is_active, created_at, updated_at
             FROM languages
             WHERE is_active = 1
             ORDER BY is_builtin DESC, name ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        let languages = rows.into_iter().map(|row| {
            Language {
                id: row.get("id"),
                name: row.get("name"),
                icon: row.get("icon"),
                color: row.get("color"),
                description: row.get("description"),
                is_builtin: row.get::<i32, _>("is_builtin") == 1,
                is_active: row.get::<i32, _>("is_active") == 1,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }).collect();

        Ok(languages)
    }

    /// 根据 ID 获取语言
    pub async fn get_language(&self, id: i64) -> Result<Option<Language>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, icon, color, description, is_builtin, is_active, created_at, updated_at
             FROM languages
             WHERE id = ?1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            Language {
                id: r.get("id"),
                name: r.get("name"),
                icon: r.get("icon"),
                color: r.get("color"),
                description: r.get("description"),
                is_builtin: r.get::<i32, _>("is_builtin") == 1,
                is_active: r.get::<i32, _>("is_active") == 1,
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            }
        }))
    }

    /// 更新语言
    pub async fn update_language(
        &self,
        id: i64,
        name: &str,
        icon: Option<&str>,
        color: Option<&str>,
        description: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE languages
             SET name = ?1, icon = ?2, color = ?3, description = ?4, updated_at = datetime('now')
             WHERE id = ?5"
        )
        .bind(name)
        .bind(icon)
        .bind(color)
        .bind(description)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除语言
    pub async fn delete_language(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM languages WHERE id = ?1 AND is_builtin = 0")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// ===== 项目语言关联操作 =====

    /// 设置项目的主语言
    pub async fn set_project_primary_language(
        &self,
        project_id: i64,
        language_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE projects SET primary_language_id = ?1 WHERE id = ?2")
            .bind(language_id)
            .bind(project_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 为项目添加语言
    pub async fn add_project_language(
        &self,
        project_id: i64,
        language_id: i64,
        is_primary: bool,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO project_languages (project_id, language_id, is_primary)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(project_id, language_id) DO UPDATE SET is_primary = ?3"
        )
        .bind(project_id)
        .bind(language_id)
        .bind(is_primary as i32)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 移除项目的语言
    pub async fn remove_project_language(
        &self,
        project_id: i64,
        language_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM project_languages WHERE project_id = ?1 AND language_id = ?2")
            .bind(project_id)
            .bind(language_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 获取项目的所有语言
    pub async fn get_project_languages(&self, project_id: i64) -> Result<Vec<Language>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT l.id, l.name, l.icon, l.color, l.description, l.is_builtin, l.is_active,
                    l.created_at, l.updated_at
             FROM project_languages pl
             JOIN languages l ON pl.language_id = l.id
             WHERE pl.project_id = ?1
             ORDER BY pl.is_primary DESC, l.name ASC"
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        let languages = rows.into_iter().map(|row| {
            Language {
                id: row.get("id"),
                name: row.get("name"),
                icon: row.get("icon"),
                color: row.get("color"),
                description: row.get("description"),
                is_builtin: row.get::<i32, _>("is_builtin") == 1,
                is_active: row.get::<i32, _>("is_active") == 1,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }).collect();

        Ok(languages)
    }

    /// ===== 语言类型字段操作 =====

    /// 获取语言的所有类型字段
    pub async fn get_language_field_types(
        &self,
        language_id: i64,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, language_id, name, description, is_builtin, sort_order, created_at, updated_at
             FROM language_field_types
             WHERE language_id = ?1
             ORDER BY sort_order ASC, name ASC"
        )
        .bind(language_id)
        .fetch_all(&self.pool)
        .await?;

        let field_types: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.get::<i64, _>("id"),
                    "language_id": row.get::<i64, _>("language_id"),
                    "name": row.get::<String, _>("name"),
                    "description": row.get::<Option<String>, _>("description"),
                    "is_builtin": row.get::<i32, _>("is_builtin") == 1,
                    "sort_order": row.get::<i32, _>("sort_order"),
                    "created_at": row.get::<String, _>("created_at"),
                    "updated_at": row.get::<String, _>("updated_at"),
                })
            })
            .collect();

        Ok(field_types)
    }

    /// 创建语言类型字段
    pub async fn create_language_field_type(
        &self,
        language_id: i64,
        name: &str,
        description: Option<&str>,
        sort_order: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO language_field_types (language_id, name, description, is_builtin, sort_order)
             VALUES (?1, ?2, ?3, 0, ?4)"
        )
        .bind(language_id)
        .bind(name)
        .bind(description)
        .bind(sort_order)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 更新语言类型字段
    pub async fn update_language_field_type(
        &self,
        id: i64,
        name: &str,
        description: Option<&str>,
        sort_order: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE language_field_types
             SET name = ?1, description = ?2, sort_order = ?3, updated_at = datetime('now')
             WHERE id = ?4 AND is_builtin = 0"
        )
        .bind(name)
        .bind(description)
        .bind(sort_order)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除语言类型字段
    pub async fn delete_language_field_type(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM language_field_types WHERE id = ?1 AND is_builtin = 0"
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 批量保存语言类型字段（用于初始化默认类型）
    pub async fn batch_save_language_field_types(
        &self,
        language_id: i64,
        field_types: Vec<serde_json::Value>,
    ) -> Result<(), sqlx::Error> {
        // 开始事务
        let mut tx = self.pool.begin().await?;

        // 删除该语言的所有非内置类型字段
        sqlx::query(
            "DELETE FROM language_field_types WHERE language_id = ?1 AND is_builtin = 0"
        )
        .bind(language_id)
        .execute(&mut *tx)
        .await?;

        // 插入新的类型字段
        for (index, field_type) in field_types.iter().enumerate() {
            let name = field_type.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let description = field_type.get("description").and_then(|v| v.as_str());
            let is_builtin = field_type.get("is_builtin").and_then(|v| v.as_bool()).unwrap_or(false);

            sqlx::query(
                "INSERT INTO language_field_types (language_id, name, description, is_builtin, sort_order)
                 VALUES (?1, ?2, ?3, ?4, ?5)"
            )
            .bind(language_id)
            .bind(name)
            .bind(description)
            .bind(if is_builtin { 1 } else { 0 })
            .bind(index as i32)
            .execute(&mut *tx)
            .await?;
        }

        // 提交事务
        tx.commit().await?;

        Ok(())
    }

    /// ===== AI 服务相关操作 =====

    /// 获取所有 AI 提供商
    pub async fn get_all_ai_providers(&self) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, provider_name, display_name, provider_type, api_key, api_endpoint,
                    is_enabled, is_default, temperature, max_tokens, timeout_seconds,
                    created_at, updated_at
             FROM ai_providers
             ORDER BY id ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        let providers = rows.into_iter().map(|row| {
            serde_json::json!({
                "id": row.get::<i64, _>("id"),
                "providerName": row.get::<String, _>("provider_name"),
                "displayName": row.get::<String, _>("display_name"),
                "providerType": row.get::<String, _>("provider_type"),
                "apiKey": row.get::<Option<String>, _>("api_key"),
                "apiEndpoint": row.get::<Option<String>, _>("api_endpoint"),
                "isEnabled": row.get::<i32, _>("is_enabled") == 1,
                "isDefault": row.get::<i32, _>("is_default") == 1,
                "temperature": row.get::<f64, _>("temperature"),
                "maxTokens": row.get::<i32, _>("max_tokens"),
                "timeoutSeconds": row.get::<i32, _>("timeout_seconds"),
                "createdAt": row.get::<String, _>("created_at"),
                "updatedAt": row.get::<String, _>("updated_at"),
            })
        }).collect();

        Ok(providers)
    }

    /// 根据 provider_name 获取 AI 提供商
    pub async fn get_ai_provider(&self, provider_name: &str) -> Result<Option<serde_json::Value>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, provider_name, display_name, provider_type, api_key, api_endpoint,
                    is_enabled, is_default, temperature, max_tokens, timeout_seconds,
                    created_at, updated_at
             FROM ai_providers
             WHERE provider_name = ?1"
        )
        .bind(provider_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            serde_json::json!({
                "id": r.get::<i64, _>("id"),
                "providerName": r.get::<String, _>("provider_name"),
                "displayName": r.get::<String, _>("display_name"),
                "providerType": r.get::<String, _>("provider_type"),
                "apiKey": r.get::<Option<String>, _>("api_key"),
                "apiEndpoint": r.get::<Option<String>, _>("api_endpoint"),
                "isEnabled": r.get::<i32, _>("is_enabled") == 1,
                "isDefault": r.get::<i32, _>("is_default") == 1,
                "temperature": r.get::<f64, _>("temperature"),
                "maxTokens": r.get::<i32, _>("max_tokens"),
                "timeoutSeconds": r.get::<i32, _>("timeout_seconds"),
                "createdAt": r.get::<String, _>("created_at"),
                "updatedAt": r.get::<String, _>("updated_at"),
            })
        }))
    }

    /// 保存或更新 AI 提供商配置
    pub async fn save_ai_provider(
        &self,
        provider_name: &str,
        display_name: &str,
        provider_type: &str,
        api_key: Option<&str>,
        api_endpoint: Option<&str>,
        is_enabled: bool,
        temperature: f64,
        max_tokens: i32,
    ) -> Result<i64, sqlx::Error> {
        let id = sqlx::query(
            "INSERT INTO ai_providers (
                provider_name, display_name, provider_type, api_key, api_endpoint,
                is_enabled, temperature, max_tokens
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(provider_name) DO UPDATE SET
                display_name = excluded.display_name,
                provider_type = excluded.provider_type,
                api_key = excluded.api_key,
                api_endpoint = excluded.api_endpoint,
                is_enabled = excluded.is_enabled,
                temperature = excluded.temperature,
                max_tokens = excluded.max_tokens,
                updated_at = datetime('now')
            RETURNING id"
        )
        .bind(provider_name)
        .bind(display_name)
        .bind(provider_type)
        .bind(api_key)
        .bind(api_endpoint)
        .bind(if is_enabled { 1 } else { 0 })
        .bind(temperature)
        .bind(max_tokens)
        .fetch_one(&self.pool)
        .await?
        .get::<i64, _>("id");

        Ok(id)
    }

    /// 切换 AI 提供商启用状态
    pub async fn toggle_ai_provider(&self, provider_name: &str, is_enabled: bool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE ai_providers
             SET is_enabled = ?1, updated_at = datetime('now')
             WHERE provider_name = ?2"
        )
        .bind(if is_enabled { 1 } else { 0 })
        .bind(provider_name)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 删除 AI 提供商
    pub async fn delete_ai_provider(&self, provider_name: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM ai_providers WHERE provider_name = ?1")
            .bind(provider_name)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 获取提供商的所有模型（分组）
    pub async fn get_ai_provider_models_grouped(&self, provider_name: &str) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT group_id, COUNT(*) as count
             FROM ai_models
             WHERE provider_name = ?1
             GROUP BY group_id
             ORDER BY group_id ASC"
        )
        .bind(provider_name)
        .fetch_all(&self.pool)
        .await?;

        let mut groups = Vec::new();

        for row in rows {
            let group_id: String = row.get("group_id");
            let count: i64 = row.get("count");

            // 获取该分组下的所有模型
            let model_rows = sqlx::query(
                "SELECT id, model_id, model_name, description, max_tokens, supports_functions, supports_vision
                 FROM ai_models
                 WHERE provider_name = ?1 AND group_id = ?2
                 ORDER BY id ASC"
            )
            .bind(provider_name)
            .bind(&group_id)
            .fetch_all(&self.pool)
            .await?;

            let models: Vec<serde_json::Value> = model_rows.into_iter().map(|m| {
                serde_json::json!({
                    "id": m.get::<i64, _>("id"),
                    "modelId": m.get::<String, _>("model_id"),
                    "modelName": m.get::<String, _>("model_name"),
                    "description": m.get::<Option<String>, _>("description"),
                    "maxTokens": m.get::<i32, _>("max_tokens"),
                    "supportsFunctions": m.get::<i32, _>("supports_functions") == 1,
                    "supportsVision": m.get::<i32, _>("supports_vision") == 1,
                })
            }).collect();

            groups.push(serde_json::json!({
                "groupId": group_id,
                "groupName": get_group_display_name(&group_id),
                "count": count,
                "models": models,
            }));
        }

        Ok(groups)
    }

    /// 添加 AI 模型
    pub async fn add_ai_model(
        &self,
        model_id: &str,
        model_name: &str,
        provider_name: &str,
        group_id: &str,
        description: Option<&str>,
        max_tokens: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO ai_models (model_id, model_name, provider_name, group_id, description, max_tokens)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        )
        .bind(model_id)
        .bind(model_name)
        .bind(provider_name)
        .bind(group_id)
        .bind(description)
        .bind(max_tokens)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 删除 AI 模型
    pub async fn delete_ai_model(&self, model_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM ai_models WHERE id = ?1")
            .bind(model_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 更新 AI 模型
    pub async fn update_ai_model(
        &self,
        model_id: i64,
        new_model_id: &str,
        model_name: &str,
        group_id: &str,
        description: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE ai_models
             SET model_id = ?1, model_name = ?2, group_id = ?3, description = ?4
             WHERE id = ?5"
        )
        .bind(new_model_id)
        .bind(model_name)
        .bind(group_id)
        .bind(description)
        .bind(model_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ===== 项目表规范管理 =====

    /// 获取项目的表规范配置
    pub async fn get_table_preferences(&self, project_id: i64) -> Result<Option<serde_json::Value>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT * FROM table_preferences WHERE project_id = ?1"
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| serde_json::json!({
            "id": r.get::<i64, _>("id"),
            "projectId": r.get::<i64, _>("project_id"),
            "pkEnabled": r.get::<i32, _>("pk_enabled") == 1,
            "pkFieldName": r.get::<String, _>("pk_field_name"),
            "pkFieldType": r.get::<String, _>("pk_field_type"),
            "pkAutoIncrement": r.get::<i32, _>("pk_auto_increment") == 1,
            "pkComment": r.get::<Option<String>, _>("pk_comment"),
            "auditEnabled": r.get::<i32, _>("audit_enabled") == 1,
            "auditFields": r.get::<Option<String>, _>("audit_fields"),
            "softDeleteEnabled": r.get::<i32, _>("soft_delete_enabled") == 1,
            "softDeleteField": r.get::<String, _>("soft_delete_field"),
            "softDeleteFieldType": r.get::<String, _>("soft_delete_field_type"),
            "softDeleteNullable": r.get::<i32, _>("soft_delete_nullable") == 1,
            "softDeleteDefault": r.get::<Option<String>, _>("soft_delete_default"),
            "softDeleteComment": r.get::<Option<String>, _>("soft_delete_comment"),
            "booleanPrefix": r.get::<Option<String>, _>("boolean_prefix"),
            "datetimeSuffix": r.get::<Option<String>, _>("datetime_suffix"),
            "engineType": r.get::<Option<String>, _>("engine_type"),
            "charset": r.get::<Option<String>, _>("charset"),
            "collation": r.get::<Option<String>, _>("collation"),
            "createdAt": r.get::<String, _>("created_at"),
            "updatedAt": r.get::<String, _>("updated_at"),
        })))
    }

    /// 保存或更新项目表规范配置
    pub async fn save_table_preferences(
        &self,
        project_id: i64,
        prefs: serde_json::Value,
    ) -> Result<i64, sqlx::Error> {
        // 检查是否已存在
        let existing = sqlx::query(
            "SELECT id FROM table_preferences WHERE project_id = ?1"
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;

        let pk_enabled = prefs["pkEnabled"].as_bool().unwrap_or(true);
        let pk_field_name = prefs["pkFieldName"].as_str().unwrap_or("id");
        let pk_field_type = prefs["pkFieldType"].as_str().unwrap_or("BIGINT");
        let pk_auto_increment = prefs["pkAutoIncrement"].as_bool().unwrap_or(true);
        let pk_comment = prefs["pkComment"].as_str();
        let audit_enabled = prefs["auditEnabled"].as_bool().unwrap_or(true);
        let audit_fields = prefs["auditFields"].as_str();
        let soft_delete_enabled = prefs["softDeleteEnabled"].as_bool().unwrap_or(false);
        let soft_delete_field = prefs["softDeleteField"].as_str().unwrap_or("deleted_at");
        let soft_delete_field_type = prefs["softDeleteFieldType"].as_str().unwrap_or("TIMESTAMP");
        let soft_delete_nullable = prefs["softDeleteNullable"].as_bool().unwrap_or(true);
        let soft_delete_default = prefs["softDeleteDefault"].as_str();
        let soft_delete_comment = prefs["softDeleteComment"].as_str();
        let boolean_prefix = prefs["booleanPrefix"].as_str();
        let datetime_suffix = prefs["datetimeSuffix"].as_str();
        let engine_type = prefs["engineType"].as_str();
        let charset = prefs["charset"].as_str();
        let collation = prefs["collation"].as_str();

        if let Some(row) = existing {
            // 更新
            let id = row.get::<i64, _>("id");

            sqlx::query(
                "UPDATE table_preferences SET
                    pk_enabled = ?1,
                    pk_field_name = ?2,
                    pk_field_type = ?3,
                    pk_auto_increment = ?4,
                    pk_comment = ?5,
                    audit_enabled = ?6,
                    audit_fields = ?7,
                    soft_delete_enabled = ?8,
                    soft_delete_field = ?9,
                    soft_delete_field_type = ?10,
                    soft_delete_nullable = ?11,
                    soft_delete_default = ?12,
                    soft_delete_comment = ?13,
                    boolean_prefix = ?14,
                    datetime_suffix = ?15,
                    engine_type = ?16,
                    charset = ?17,
                    collation = ?18,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = ?19"
            )
            .bind(pk_enabled as i32)
            .bind(pk_field_name)
            .bind(pk_field_type)
            .bind(pk_auto_increment as i32)
            .bind(pk_comment)
            .bind(audit_enabled as i32)
            .bind(audit_fields)
            .bind(soft_delete_enabled as i32)
            .bind(soft_delete_field)
            .bind(soft_delete_field_type)
            .bind(soft_delete_nullable as i32)
            .bind(soft_delete_default)
            .bind(soft_delete_comment)
            .bind(boolean_prefix)
            .bind(datetime_suffix)
            .bind(engine_type)
            .bind(charset)
            .bind(collation)
            .bind(id)
            .execute(&self.pool)
            .await?;

            Ok(id)
        } else {
            // 插入
            let result = sqlx::query(
                "INSERT INTO table_preferences (
                    project_id,
                    pk_enabled,
                    pk_field_name,
                    pk_field_type,
                    pk_auto_increment,
                    pk_comment,
                    audit_enabled,
                    audit_fields,
                    soft_delete_enabled,
                    soft_delete_field,
                    soft_delete_field_type,
                    soft_delete_nullable,
                    soft_delete_default,
                    soft_delete_comment,
                    boolean_prefix,
                    datetime_suffix,
                    engine_type,
                    charset,
                    collation
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)"
            )
            .bind(project_id)
            .bind(pk_enabled as i32)
            .bind(pk_field_name)
            .bind(pk_field_type)
            .bind(pk_auto_increment as i32)
            .bind(pk_comment)
            .bind(audit_enabled as i32)
            .bind(audit_fields)
            .bind(soft_delete_enabled as i32)
            .bind(soft_delete_field)
            .bind(soft_delete_field_type)
            .bind(soft_delete_nullable as i32)
            .bind(soft_delete_default)
            .bind(soft_delete_comment)
            .bind(boolean_prefix)
            .bind(datetime_suffix)
            .bind(engine_type)
            .bind(charset)
            .bind(collation)
            .execute(&self.pool)
            .await?;

            Ok(result.last_insert_rowid())
        }
    }
}

/// 获取分组的显示名称
fn get_group_display_name(group_id: &str) -> &'static str {
    match group_id {
        "chat" => "对话模型",
        "code" => "代码模型",
        "image" => "图像模型",
        "embedding" => "嵌入模型",
        _ => "其他模型",
    }
}

// ===== 数据模型 =====

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
        "postgresql" => import_postgresql_tables(pool, project_id, &datasource, database_name).await,
        "sqlite" => import_sqlite_tables(pool, project_id, &datasource).await,
        _ => Err(format!("不支持的数据库类型: {}", datasource.type_))
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
        ORDER BY TABLE_NAME"
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
             RETURNING id"
        )
        .bind(project_id)
        .bind(&table_info.name)
        .bind(&table_info.comment)
        .bind(&table_info.engine)
        .bind(if table_info.table_type == "BASE TABLE" { "table" } else { "view" })
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
            ORDER BY ORDINAL_POSITION"
        )
        .bind(database_name)
        .bind(&table_info.name)
        .fetch_all(&mysql_pool)
        .await
        .map_err(|e| format!("查询列信息失败: {}", e))?;

        // 插入列记录
        for col in columns {
            let is_primary_key = col.column_key.as_deref() == Some("PRI");
            let is_unique = col.column_key.as_deref() == Some("UNI") || col.column_key.as_deref() == Some("PRI");
            let is_nullable = col.is_nullable.as_deref() == Some("YES");

            sqlx::query(
                "INSERT INTO db_columns (table_id, name, data_type, length, is_nullable, is_primary_key, is_unique, default_value, comment, ordinal_position)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
            )
            .bind(table_id)
            .bind(&col.name)
            .bind(&col.data_type)
            .bind(col.length.map(|l| l as i64))
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
        let column_count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM db_columns WHERE table_id = ?1"
        )
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
        datasource.username.as_ref().ok_or("PostgreSQL 用户名未配置")?,
        datasource.password.as_ref().ok_or("PostgreSQL 密码未配置")?,
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
             RETURNING id"
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
             WHERE i.indrelid = $1::regclass AND i.indisprimary"
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
            .bind(col.length.map(|l| l as i64))
            .bind(is_nullable as i32)
            .bind(is_primary_key as i32)
            .bind(0) // PostgreSQL unique 需要额外查询，这里简化处理
            .bind(&col.default_value)
            .bind(&col.comment)
            .bind(col.ordinal_position.map(|p| p as i32).unwrap_or(0))
            .execute(pool)
            .await
            .map_err(|e| format!("创建列记录失败: {}", e))?;
        }

        // 更新表的列计数
        let column_count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM db_columns WHERE table_id = ?1"
        )
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
    let sqlite_file = datasource.sqlite_file.as_ref().ok_or("SQLite 文件路径未配置")?;

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
         ORDER BY name"
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
             RETURNING id"
        )
        .bind(project_id)
        .bind(&table_info.name)
        .bind(None::<&str>)
        .bind(Some("SQLite"))
        .bind(if table_info.table_type == "table" { "table" } else { "view" })
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
        let column_count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM db_columns WHERE table_id = ?1"
        )
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

// ===== 表结构读取和导入相关函数 =====

/// 读取 MySQL 数据库的表列表
pub async fn fetch_mysql_tables(
    pool: &SqlitePool,
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
        ORDER BY TABLE_NAME"
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
    pool: &SqlitePool,
    datasource: &Datasource,
    database_name: &str,
) -> Result<String, String> {
    // 连接到 PostgreSQL 数据库
    let connection_string = format!(
        "postgresql://{}:{}@{}:{}/{}",
        datasource.username.as_ref().ok_or("PostgreSQL 用户名未配置")?,
        datasource.password.as_ref().ok_or("PostgreSQL 密码未配置")?,
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
    pool: &SqlitePool,
    datasource: &Datasource,
) -> Result<String, String> {
    let sqlite_file = datasource.sqlite_file.as_ref()
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
        ORDER BY name"
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
        "mysql" => import_mysql_single_table(pool, project_id, datasource, database_name, table_name, table_comment, table_type, engine, row_count).await,
        "postgresql" => import_postgresql_single_table(pool, project_id, datasource, database_name, table_name, table_comment, table_type, engine, row_count).await,
        "sqlite" => import_sqlite_single_table(pool, project_id, datasource, table_name, table_comment, table_type, row_count).await,
        _ => Err(format!("不支持的数据源类型: {}", datasource.type_))
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
         RETURNING id"
    )
    .bind(project_id)
    .bind(table_name)
    .bind(table_comment)
    .bind(_engine)
    .bind(if table_type == "BASE TABLE" || table_type == "table" { "table" } else { "view" })
    .bind(0)
    .fetch_one(pool)
    .await
        .map_err(|e| format!("创建表记录失败: {}", e))?;

    println!("表记录创建成功，table_id = {}, 项目ID = {}", table_id, project_id);

    // 查询列信息
    println!("开始查询列信息，数据库: {}, 表: {}", database_name, table_name);

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
        ORDER BY ORDINAL_POSITION"
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
        let is_unique = col.column_key.as_deref() == Some("UNI") || col.column_key.as_deref() == Some("PRI");
        let is_nullable = col.is_nullable.as_deref() == Some("YES");

        println!("插入列: {} (类型: {}, 可空: {}, 主键: {})",
                 col.name, col.data_type, is_nullable, is_primary_key);

        sqlx::query(
            "INSERT INTO db_columns (table_id, name, data_type, length, is_nullable, is_primary_key, is_unique, default_value, comment, ordinal_position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
        )
        .bind(table_id)
        .bind(&col.name)
        .bind(&col.data_type)
        .bind(col.length.map(|l| l as i64))
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
    let column_count: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM db_columns WHERE table_id = ?1"
    )
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
    sqlx::query("UPDATE projects SET table_count = table_count + 1, updated_at = datetime('now') WHERE id = ?1")
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
        datasource.username.as_ref().ok_or("PostgreSQL 用户名未配置")?,
        datasource.password.as_ref().ok_or("PostgreSQL 密码未配置")?,
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
         RETURNING id"
    )
    .bind(project_id)
    .bind(table_name)
    .bind(table_comment)
    .bind(None::<String>)
    .bind(if table_type == "table" { "table" } else { "view" })
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
        .bind(col.length.map(|l| l as i64))
        .bind(is_nullable as i32)
        .bind(false as i32) // PostgreSQL 主键需要更复杂的判断
        .bind(false as i32)
        .bind(&col.default_value)
        .bind(&col.comment)
        .bind(col.ordinal_position.map(|p| p as i32).unwrap_or(0))
        .execute(pool)
        .await
        .map_err(|e| format!("插入列记录失败 [{}]: {}", col.name, e))?;
    }

    println!("成功插入 {} 列", columns.len());

    // 更新表的列计数
    let column_count: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM db_columns WHERE table_id = ?1"
    )
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
    sqlx::query("UPDATE projects SET table_count = table_count + 1, updated_at = datetime('now') WHERE id = ?1")
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
    let sqlite_file = datasource.sqlite_file.as_ref()
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
         RETURNING id"
    )
    .bind(project_id)
    .bind(table_name)
    .bind(table_comment)
    .bind(None::<String>)
    .bind(if table_type == "table" { "table" } else { "view" })
    .bind(0)
    .fetch_one(pool)
    .await
        .map_err(|e| format!("创建表记录失败: {}", e))?;

    // 查询列信息
    let columns: Vec<SQLiteColumnInfo> = sqlx::query_as(
        "PRAGMA table_info(?)"
    )
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
    let column_count: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM db_columns WHERE table_id = ?1"
    )
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
    sqlx::query("UPDATE projects SET table_count = table_count + 1, updated_at = datetime('now') WHERE id = ?1")
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
    pool: &SqlitePool,
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
                    matches!(&opt.option, sqlparser::ast::ColumnOption::Unique { is_primary: true, .. })
                });
                let is_unique = column_def.options.iter().any(|opt| {
                    matches!(&opt.option, sqlparser::ast::ColumnOption::Unique { is_primary: false, .. })
                });
                let has_not_null = column_def.options.iter().any(|opt| {
                    matches!(&opt.option, sqlparser::ast::ColumnOption::NotNull)
                });
                let has_null = column_def.options.iter().any(|opt| {
                    matches!(&opt.option, sqlparser::ast::ColumnOption::Null)
                });
                let is_nullable = if has_null { true }
                                  else if has_not_null { false }
                                  else if is_primary_key { false }
                                  else { true };

                let default_value = column_def.options.iter()
                    .find_map(|opt| {
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
                 VALUES (?1, ?2, ?3, ?4)"
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
                    matches!(&opt.option, sqlparser::ast::ColumnOption::Unique { is_primary: true, .. })
                });
                let is_unique = column_def.options.iter().any(|opt| {
                    matches!(&opt.option, sqlparser::ast::ColumnOption::Unique { is_primary: false, .. })
                });
                let has_not_null = column_def.options.iter().any(|opt| {
                    matches!(&opt.option, sqlparser::ast::ColumnOption::NotNull)
                });
                let has_null = column_def.options.iter().any(|opt| {
                    matches!(&opt.option, sqlparser::ast::ColumnOption::Null)
                });

                let is_nullable = if has_null { true }
                                  else if has_not_null { false }
                                  else if is_primary_key { false }
                                  else { true };

                // 提取默认值
                let default_value = column_def.options.iter()
                    .find_map(|opt| {
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
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
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
    let mut result = format!("成功创建 {} 张表，{} 个字段", tables_created, columns_created);
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
    use sqlparser::ast::{DataType, CharacterLength};

    match data_type {
        DataType::Varchar(Some(len)) | DataType::Char(Some(len)) => {
            match len {
                CharacterLength::IntegerLength { length, .. } => Some(*length as i64),
                _ => None,
            }
        }
        _ => None,
    }
}
