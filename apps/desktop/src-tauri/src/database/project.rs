use sqlx::Row;

use super::{Database, Language, Project, RecentProject, Statistics};

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

    /// 获取统计数据
    pub async fn get_statistics(&self) -> Result<Statistics, sqlx::Error> {
        let pool = &self.pool;

        let project_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM projects")
            .fetch_one(pool)
            .await?;

        let datasource_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM datasources")
            .fetch_one(pool)
            .await?;

        let language_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM languages")
            .fetch_one(pool)
            .await?;

        let table_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM db_tables")
            .fetch_one(pool)
            .await?;

        Ok(Statistics {
            total_projects: project_count,
            total_datasources: datasource_count,
            total_languages: language_count,
            total_tables: table_count,
        })
    }

    /// 获取最近的项目列表
    pub async fn get_recent_projects(&self, limit: i64) -> Result<Vec<RecentProject>, sqlx::Error> {
        let pool = &self.pool;

        let rows = sqlx::query(
            "SELECT id, name, description, database_name, table_count, created_at
             FROM projects
             ORDER BY created_at DESC
             LIMIT ?"
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;

        let projects = rows.iter().map(|row| RecentProject {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            database_name: row.get("database_name"),
            table_count: row.get("table_count"),
            created_at: row.get("created_at"),
        }).collect();

        Ok(projects)
    }
}
