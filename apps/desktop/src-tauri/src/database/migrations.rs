use super::Database;

impl Database {
    /// 运行数据库迁移
    /// 公开包装：供集成测试对指定库执行完整迁移链
    pub async fn run_migrations_for_test(&self) -> Result<(), sqlx::Error> {
        self.run_migrations().await
    }

    pub(crate) async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        println!("运行数据库迁移...");

        // 创建 migrations 表（用于跟踪迁移版本）
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&self.pool)
        .await?;

        // 检查当前版本
        let current_version: i32 =
            sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM schema_migrations")
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

        if current_version < 11 {
            self.migration_011_add_mimo_provider().await?;
        }

        if current_version < 12 {
            self.migration_012_add_cherry_studio_provider().await?;
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
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_projects_name ON projects(name)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_projects_database_type ON projects(database_type)",
        )
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
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_datasources_name ON datasources(name)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_datasources_type ON datasources(type)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_datasources_is_active ON datasources(is_active)",
        )
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
            )",
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
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_columns_table_id ON db_columns(table_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_columns_name ON db_columns(name)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_columns_ordinal ON db_columns(ordinal_position)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("INSERT INTO schema_migrations (version) VALUES (4)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 迁移 005: 更新数据库架构（添加语言支持）
    ///
    /// 历史 BUG 修复：本迁移原实现直接 DROP 旧表重建，v4 及更早版本升级时
    /// 用户的 projects/datasources/db_tables/db_columns 数据全部清空。
    /// 现改为备份-重建-回填：先把带数据的旧表改名暂存，建好新表后按字段映射
    /// 迁回旧数据（无法映射的字段用兜底值），最后清理暂存表。
    async fn migration_005_update_schema(&self) -> Result<(), sqlx::Error> {
        // 幂等保护：新结构已存在（重复运行/半途失败后重试）时直接补版本号
        let already_migrated: bool = sqlx::query_scalar(
            "SELECT COUNT(*) = 1 FROM pragma_table_info('projects') WHERE name = 'datasource_id'",
        )
        .fetch_one(&self.pool)
        .await?;
        if already_migrated {
            println!("迁移 005: 新表结构已存在，跳过重建");
            sqlx::query("INSERT OR IGNORE INTO schema_migrations (version) VALUES (5)")
                .execute(&self.pool)
                .await?;
            return Ok(());
        }

        // ---- 暂存带数据的旧表（无数据/不存在的表直接 DROP，无需暂存）----
        let old_tables = ["projects", "datasources", "db_tables", "db_columns"];
        for t in old_tables {
            let exists: bool = sqlx::query_scalar(&format!(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='{}'",
                t
            ))
            .fetch_one(&self.pool)
            .await?;
            let has_rows: bool = if exists {
                sqlx::query_scalar(&format!("SELECT COUNT(*) > 0 FROM {}", t))
                    .fetch_one(&self.pool)
                    .await?
            } else {
                false
            };
            if has_rows {
                sqlx::query(&format!("ALTER TABLE {} RENAME TO {}__mig005_old", t, t))
                    .execute(&self.pool)
                    .await?;
                let n: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}__mig005_old", t))
                    .fetch_one(&self.pool)
                    .await?;
                println!("迁移 005: 已暂存旧表 {}（{} 行）", t, n);
            } else if exists {
                sqlx::query(&format!("DROP TABLE IF EXISTS {}", t))
                    .execute(&self.pool)
                    .await?;
            }
        }

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
            )",
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
            ('C#', '🔷', 'purple', 'Microsoft 开发语言', 1)",
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
            )",
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
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_projects_name ON projects(name)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_projects_datasource_id ON projects(datasource_id)",
        )
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
            )",
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
            )",
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
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_columns_table_id ON db_columns(table_id)")
            .execute(&self.pool)
            .await?;

        // ---- 回填旧数据 ----
        // datasources：新旧表同名列直迁（database 列 006 才加，无需处理）
        self.mig005_copy_if_exists(
            "datasources",
            &[
                "name",
                "type",
                "host",
                "port",
                "username",
                "password",
                "is_active",
                "created_at",
                "updated_at",
            ],
        )
        .await?;

        // projects：旧表无 datasource_id/database_name/primary_language_id。
        // datasource_id 用第一个可用数据源兜底，database_name 用旧 database_type 占位。
        let fallback_ds: i64 = sqlx::query_scalar("SELECT COALESCE(MIN(id), 0) FROM datasources")
            .fetch_one(&self.pool)
            .await?;
        let has_old_projects: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='projects__mig005_old'",
        )
        .fetch_one(&self.pool)
        .await?;
        if has_old_projects && fallback_ds > 0 {
            sqlx::query(
                r#"INSERT INTO projects (name, description, datasource_id, database_name, table_count, created_at, updated_at)
                   SELECT name, description, ?, database_type, table_count, created_at, updated_at
                   FROM projects__mig005_old"#,
            )
            .bind(fallback_ds)
            .execute(&self.pool)
            .await?;
            println!("迁移 005: 已回填 projects（数据源兜底 id={}）", fallback_ds);
        } else if has_old_projects {
            println!(
                "迁移 005: 无可用数据源，projects 旧数据保留在 projects__mig005_old 供人工恢复"
            );
        }

        // db_tables：按项目名关联迁回（新 project_id 与旧不同）
        self.mig005_backfill_tables().await?;

        // db_columns：按 表名+项目名 双重关联迁回
        self.mig005_backfill_columns().await?;

        // ---- 清理暂存表（回填失败的数据仍留在暂存表可人工恢复，故最后才删）----
        for t in old_tables {
            sqlx::query(&format!("DROP TABLE IF EXISTS {}__mig005_old", t))
                .execute(&self.pool)
                .await?;
        }

        sqlx::query("INSERT OR IGNORE INTO schema_migrations (version) VALUES (5)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// mig005 辅助：旧暂存表存在时按同名列直迁回填
    async fn mig005_copy_if_exists(&self, table: &str, cols: &[&str]) -> Result<(), sqlx::Error> {
        let old = format!("{}__mig005_old", table);
        let exists: bool = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='{}'",
            old
        ))
        .fetch_one(&self.pool)
        .await?;
        if !exists {
            return Ok(());
        }
        let col_list = cols.join(", ");
        let n = sqlx::query(&format!(
            "INSERT INTO {} ({}) SELECT {} FROM {}",
            table, col_list, col_list, old
        ))
        .execute(&self.pool)
        .await?
        .rows_affected();
        println!("迁移 005: 已回填 {}（{} 行，同名列直迁）", table, n);
        Ok(())
    }

    /// mig005 辅助：db_tables 按项目名关联回填
    async fn mig005_backfill_tables(&self) -> Result<(), sqlx::Error> {
        let exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='db_tables__mig005_old'",
        )
        .fetch_one(&self.pool)
        .await?;
        if !exists {
            return Ok(());
        }
        let n = sqlx::query(
            r#"INSERT INTO db_tables (project_id, name, comment, engine, table_type, row_count, column_count, created_at, updated_at)
               SELECT np.id, t.name, t.comment, t.engine, t.table_type, t.row_count, t.column_count, t.created_at, t.updated_at
               FROM db_tables__mig005_old t
               JOIN projects__mig005_old op ON op.id = t.project_id
               JOIN projects np ON np.name = op.name"#,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        println!("迁移 005: 已回填 db_tables（{} 行，按项目名关联）", n);
        Ok(())
    }

    /// mig005 辅助：db_columns 按 表名+项目名 双重关联回填
    async fn mig005_backfill_columns(&self) -> Result<(), sqlx::Error> {
        let exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='db_columns__mig005_old'",
        )
        .fetch_one(&self.pool)
        .await?;
        if !exists {
            return Ok(());
        }
        let n = sqlx::query(
            r#"INSERT INTO db_columns (table_id, name, data_type, length, is_nullable, is_primary_key, is_unique, default_value, comment, created_at)
               SELECT nt.id, c.name, c.data_type, c.length, c.is_nullable, c.is_primary_key, c.is_unique, c.default_value, c.comment, c.created_at
               FROM db_columns__mig005_old c
               JOIN db_tables__mig005_old ot ON ot.id = c.table_id
               JOIN projects__mig005_old op ON op.id = ot.project_id
               JOIN db_tables nt ON nt.name = ot.name
               JOIN projects np ON np.name = op.name AND nt.project_id = np.id"#,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        println!("迁移 005: 已回填 db_columns（{} 行，按表名+项目名关联）", n);
        Ok(())
    }

    /// 迁移 006: 添加 database 列到 datasources 表
    async fn migration_006_add_database_column(&self) -> Result<(), sqlx::Error> {
        println!("执行迁移 006: 添加 database 列");

        // 检查列是否存在
        let column_exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) = 1 FROM pragma_table_info('datasources') WHERE name = 'database'",
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
            )",
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
            )",
        )
        .execute(&self.pool)
        .await?;

        // 创建索引
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_ai_providers_name ON ai_providers(provider_name)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_ai_providers_type ON ai_providers(provider_type)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_ai_models_provider ON ai_models(provider_name)",
        )
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
                ('glm', '智谱 GLM', 'glm', 'https://open.bigmodel.cn/api/paas/v4', 0),
                ('mimo', 'Xiaomi MiMo', 'openai', 'https://api.xiaomimimo.com/v1', 0),
                ('cherry-studio', 'Cherry Studio', 'openai', 'http://127.0.0.1:23333/v1', 0)",
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
                ('glm-4-plus', 'GLM-4 Plus', 'glm', 'chat', '智谱 GLM-4 Plus 增强模型'),
                ('mimo-v2-flash', 'MiMo-V2-Flash', 'mimo', 'chat', '309B 参数，激活 15B，轻量快速'),
                ('mimo-v2-pro', 'MiMo-V2-Pro', 'mimo', 'chat', '旗舰级 Agent 基座模型')"
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
            )",
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
            )",
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
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_system_mappings_language ON system_type_mappings(language_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_system_mappings_db ON system_type_mappings(db_type)",
        )
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
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_project_mappings_project ON project_type_mappings(project_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_project_mappings_scope ON project_type_mappings(scope)",
        )
        .execute(&self.pool)
        .await?;

        // 4. 初始化系统默认映射数据
        self.init_default_system_mappings().await?;

        sqlx::query("INSERT INTO schema_migrations (version) VALUES (10)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 迁移 011: 添加 MiMo 预置提供商和模型
    async fn migration_011_add_mimo_provider(&self) -> Result<(), sqlx::Error> {
        println!("执行迁移 011: 添加 MiMo 预置提供商和模型");

        sqlx::query(
            "INSERT OR IGNORE INTO ai_providers (
                provider_name, display_name, provider_type, api_endpoint, is_enabled
            ) VALUES
                ('mimo', 'Xiaomi MiMo', 'openai', 'https://api.xiaomimimo.com/v1', 0)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "INSERT OR IGNORE INTO ai_models (model_id, model_name, provider_name, group_id, description) VALUES
                ('mimo-v2-flash', 'MiMo-V2-Flash', 'mimo', 'chat', '309B 参数，激活 15B，轻量快速'),
                ('mimo-v2-pro', 'MiMo-V2-Pro', 'mimo', 'chat', '旗舰级 Agent 基座模型')"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("INSERT INTO schema_migrations (version) VALUES (11)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// 迁移 012: 添加 Cherry Studio 预置提供商
    async fn migration_012_add_cherry_studio_provider(&self) -> Result<(), sqlx::Error> {
        println!("执行迁移 012: 添加 Cherry Studio 预置提供商");

        sqlx::query(
            "INSERT OR IGNORE INTO ai_providers (
                provider_name, display_name, provider_type, api_endpoint, is_enabled
            ) VALUES
                ('cherry-studio', 'Cherry Studio', 'openai', 'http://127.0.0.1:23333/v1', 0)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("INSERT INTO schema_migrations (version) VALUES (12)")
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
            (
                "Rust",
                vec![
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
                ],
            ),
            // Go
            (
                "Go",
                vec![
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
                ],
            ),
            // Python
            (
                "Python",
                vec![
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
                ],
            ),
            // TypeScript
            (
                "TypeScript",
                vec![
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
                ],
            ),
            // JavaScript
            (
                "JavaScript",
                vec![
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
                ],
            ),
            // Java
            (
                "Java",
                vec![
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
                ],
            ),
            // C++
            (
                "C++",
                vec![
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
                ],
            ),
            // Kotlin
            (
                "Kotlin",
                vec![
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
                ],
            ),
            // Swift
            (
                "Swift",
                vec![
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
                ],
            ),
            // Dart
            (
                "Dart",
                vec![
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
                ],
            ),
            // PHP
            (
                "PHP",
                vec![
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
                ],
            ),
            // Ruby
            (
                "Ruby",
                vec![
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
                ],
            ),
            // C#
            (
                "C#",
                vec![
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
                ],
            ),
        ];

        // 为每种语言和数据库类型初始化默认映射
        for (lang_name, mappings) in language_mappings {
            // 获取语言 ID
            let lang_id =
                match sqlx::query_scalar::<_, i64>("SELECT id FROM languages WHERE name = ?")
                    .bind(lang_name)
                    .fetch_one(&self.pool)
                    .await
                {
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
}
