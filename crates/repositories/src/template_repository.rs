use sqlx::MySqlPool;
use template_studio_shared::models::template::*;
use template_studio_shared::utils::response::PagedResponse;
use anyhow::Result;

/// 模板数据访问层
pub struct TemplateRepository {
    #[allow(dead_code)]
    pool: MySqlPool,
}

impl TemplateRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// 创建模板
    pub async fn create(&self, request: &CreateTemplateRequest, template_id: i64, git_repo_path: &str) -> Result<i64> {
        let mut tx = self.pool.begin().await?;
        let visibility = request.visibility.as_deref().unwrap_or("public");

        sqlx::query(
            r#"
            INSERT INTO templates (id, name, description, category_id, template_type, type_config, introduction, git_repo_path, is_featured, owner_id, visibility, status)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?, 'active')
            "#
        )
        .bind(template_id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(request.category_id as i64)
        .bind(&request.template_type)
        .bind(&request.type_config)
        .bind(&request.introduction)
        .bind(git_repo_path)
        .bind(request.owner_id)
        .bind(visibility)
        .execute(&mut *tx)
        .await?;

        // 插入模板语言关联
        for lang in &request.languages {
            sqlx::query(
                r#"
                INSERT INTO template_languages (template_id, language_id, is_primary)
                VALUES (?, ?, ?)
                "#
            )
            .bind(template_id)
            .bind(lang.language_id as i64)
            .bind(lang.is_primary)
            .execute(&mut *tx)
            .await?;
        }

        // 提交事务
        tx.commit().await?;

        Ok(template_id)
    }

    /// 根据ID获取模板
    pub async fn get_by_id(&self, id: i64) -> Result<Option<Template>> {
        let template = sqlx::query_as::<_, Template>(
            r#"
            SELECT CAST(id AS SIGNED) as id, name, description,
                   CAST(category_id AS SIGNED) as category_id, is_featured, logo, introduction,
                   icon, template_type, type_config, created_at, updated_at,
                   NULL as git_repo_path, NULL as current_version,
                   owner_id, visibility, status, reviewed_at, reviewed_by, download_count
            FROM templates
            WHERE id = ?
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(template)
    }

    /// 更新模板
    pub async fn update(&self, request: &UpdateTemplateRequest) -> Result<bool> {
        // 开始事务以确保数据一致性
        let mut tx = self.pool.begin().await?;

        // 更新模板基本信息
        sqlx::query(
            r#"
            UPDATE templates
            SET name = ?, description = ?, category_id = ?, introduction = ?, template_type = COALESCE(?, template_type), type_config = ?
            WHERE id = ?
            "#
        )
        .bind(&request.name)
        .bind(&request.description)
        .bind(request.category_id)
        .bind(&request.introduction)
        .bind(&request.template_type)
        .bind(&request.type_config)
        .bind(request.id)
        .execute(&mut *tx)
        .await?;

        // 删除原有的语言关联
        sqlx::query("DELETE FROM template_languages WHERE template_id = ?")
            .bind(request.id)
            .execute(&mut *tx)
            .await?;

        // 插入新的语言关联
        for lang in &request.languages {
            sqlx::query(
                "INSERT INTO template_languages (template_id, language_id, is_primary) VALUES (?, ?, ?)"
            )
            .bind(request.id)
            .bind(lang.language_id as i64)
            .bind(lang.is_primary)
            .execute(&mut *tx)
            .await?;
        }

        // 提交事务
        tx.commit().await?;

        Ok(true)
    }

    /// 删除模板
    pub async fn delete(&self, id: i64) -> Result<bool> {
        // 首先删除模板的语言关联记录
        sqlx::query("DELETE FROM template_languages WHERE template_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        // 然后删除模板记录
        let result = sqlx::query("DELETE FROM templates WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 分页获取模板列表
    pub async fn list(&self, query: &TemplateListQuery) -> Result<PagedResponse<Template>> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).min(100);
        let offset = (page - 1) * page_size;

        // 打印查询参数用于调试
        tracing::info!("TemplateRepository::list called with query: {:?}", query);

        // 构建查询，支持categoryId、languageId、is_featured等条件
        let (templates, total) = if let Some(language_id) = query.language_id {
            // 按语言查询 - 需要JOIN template_languages表
            tracing::info!("Querying templates by language_id: {}", language_id);

            let templates = sqlx::query_as::<_, Template>(
                r#"
                SELECT DISTINCT CAST(t.id AS SIGNED) as id, t.name, t.description,
                       CAST(t.category_id AS SIGNED) as category_id, t.is_featured, t.logo, t.introduction,
                       t.icon, t.template_type, t.type_config, t.created_at, t.updated_at,
                       NULL as git_repo_path, NULL as current_version,
                       t.owner_id, t.visibility, t.status, t.reviewed_at, t.reviewed_by, t.download_count
                FROM templates t
                INNER JOIN template_languages tl ON t.id = tl.template_id
                WHERE tl.language_id = ?
                ORDER BY t.created_at DESC
                LIMIT ? OFFSET ?
                "#
            )
            .bind(language_id as i64)
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?;

            tracing::info!("Found {} templates for language_id: {}", templates.len(), language_id);

            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(DISTINCT t.id) FROM templates t INNER JOIN template_languages tl ON t.id = tl.template_id WHERE tl.language_id = ?"
            )
            .bind(language_id as i64)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

            tracing::info!("Total count for language_id {}: {}", language_id, total);

            (templates, total)
        } else if let Some(category_id) = query.category_id {
            // 按分类查询
            tracing::info!("Querying templates by category_id: {}", category_id);

            let templates = sqlx::query_as::<_, Template>(
                r#"
                SELECT CAST(t.id AS SIGNED) as id, t.name, t.description,
                       CAST(t.category_id AS SIGNED) as category_id, t.is_featured, t.logo, t.introduction,
                       t.icon, t.template_type, t.type_config, t.created_at, t.updated_at,
                       NULL as git_repo_path, NULL as current_version,
                       t.owner_id, t.visibility, t.status, t.reviewed_at, t.reviewed_by, t.download_count
                FROM templates t
                WHERE t.category_id = ?
                ORDER BY t.created_at DESC
                LIMIT ? OFFSET ?
                "#
            )
            .bind(category_id as i64)
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?;

            tracing::info!("Found {} templates for category_id: {}", templates.len(), category_id);

            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM templates WHERE category_id = ?")
                .bind(category_id as i64)
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

            tracing::info!("Total count for category_id {}: {}", category_id, total);

            (templates, total)
        } else if let Some(is_featured) = query.is_featured {
            // 按推荐状态查询
            tracing::info!("Querying templates by is_featured: {}", is_featured);

            let templates = sqlx::query_as::<_, Template>(
                r#"
                SELECT CAST(t.id AS SIGNED) as id, t.name, t.description,
                       CAST(t.category_id AS SIGNED) as category_id, t.is_featured, t.logo, t.introduction,
                       t.icon, t.template_type, t.type_config, t.created_at, t.updated_at,
                       NULL as git_repo_path, NULL as current_version,
                       t.owner_id, t.visibility, t.status, t.reviewed_at, t.reviewed_by, t.download_count
                FROM templates t
                WHERE t.is_featured = ?
                ORDER BY t.created_at DESC
                LIMIT ? OFFSET ?
                "#
            )
            .bind(is_featured)
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?;

            tracing::info!("Found {} templates for is_featured: {}", templates.len(), is_featured);

            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM templates WHERE is_featured = ?")
                .bind(is_featured)
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

            tracing::info!("Total count for is_featured {}: {}", is_featured, total);

            (templates, total)
        } else {
            // 查询所有
            tracing::info!("Querying all templates (no filters)");

            let templates = sqlx::query_as::<_, Template>(
                r#"
                SELECT CAST(t.id AS SIGNED) as id, t.name, t.description,
                       CAST(t.category_id AS SIGNED) as category_id, t.is_featured, t.logo, t.introduction,
                       t.icon, t.template_type, t.type_config, t.created_at, t.updated_at,
                       NULL as git_repo_path, NULL as current_version,
                       t.owner_id, t.visibility, t.status, t.reviewed_at, t.reviewed_by, t.download_count
                FROM templates t
                ORDER BY t.created_at DESC
                LIMIT ? OFFSET ?
                "#
            )
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?;

            tracing::info!("Found {} templates (all)", templates.len());

            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM templates")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

            tracing::info!("Total count (all): {}", total);

            (templates, total)
        };

        Ok(PagedResponse::new(templates, total as u32, page as u32, page_size as u32))
    }

    /// 切换推荐状态
    pub async fn toggle_featured(&self, id: i64, is_featured: i32) -> Result<bool> {
        let result = sqlx::query("UPDATE templates SET is_featured = ? WHERE id = ?")
            .bind(is_featured)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Fork模板
    pub async fn fork(&self, request: &ForkTemplateRequest) -> Result<i64> {
        // 获取源模板信息
        let source = self.get_by_id(request.source_id).await?
            .ok_or_else(|| anyhow::anyhow!("源模板不存在"))?;

        let category_id = request.category_id.unwrap_or(source.category_id);
        let template_type = source.template_type.clone();

        // 生成新模板ID
        let new_template_id = chrono::Utc::now().timestamp_millis() as i64;

        // 开始事务以确保数据一致性
        let mut tx = self.pool.begin().await?;

        // 插入新模板记录
        sqlx::query(
            r#"
            INSERT INTO templates (id, name, description, category_id, template_type, type_config, introduction, is_featured)
            VALUES (?, ?, ?, ?, ?, ?, ?, 0)
            "#
        )
        .bind(new_template_id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(category_id)
        .bind(&template_type)
        .bind(&source.type_config)
        .bind(&request.introduction)
        .execute(&mut *tx)
        .await?;

        // 复制源模板的语言关联
        let languages = sqlx::query_as::<_, (u32, i32)>(
            "SELECT language_id, is_primary FROM template_languages WHERE template_id = ?"
        )
        .bind(request.source_id)
        .fetch_all(&mut *tx)
        .await?;

        // 插入语言关联记录
        for (language_id, is_primary) in languages {
            sqlx::query(
                "INSERT INTO template_languages (template_id, language_id, is_primary) VALUES (?, ?, ?)"
            )
            .bind(new_template_id)
            .bind(language_id)
            .bind(is_primary)
            .execute(&mut *tx)
            .await?;
        }

        // 提交事务
        tx.commit().await?;

        Ok(new_template_id)
    }

    /// 获取模板的关联语言列表
    pub async fn get_template_languages(&self, template_id: i64) -> Result<Vec<template_studio_shared::models::template::TemplateLanguageItem>> {
        let languages = sqlx::query_as::<_, template_studio_shared::models::template::TemplateLanguageItem>(
            r#"
            SELECT CAST(id AS SIGNED) as id, CAST(template_id AS SIGNED) as template_id,
                   CAST(language_id AS SIGNED) as language_id, is_primary
            FROM template_languages
            WHERE template_id = ?
            ORDER BY is_primary DESC, id ASC
            "#
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(languages)
    }

    /// 统计总模板数
    pub async fn count_all(&self) -> Result<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM templates")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
        Ok(count)
    }

    /// 统计推荐模板数
    pub async fn count_featured(&self) -> Result<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM templates WHERE is_featured = 1")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
        Ok(count)
    }

    /// 获取指定分类下的模板（限制数量）
    pub async fn get_by_category_limit(&self, category_id: i64, limit: u32) -> Result<Vec<Template>> {
        let templates = sqlx::query_as::<_, Template>(
            r#"
            SELECT CAST(id AS SIGNED) as id, name, description,
                   CAST(category_id AS SIGNED) as category_id, is_featured, logo, introduction,
                   icon, template_type, type_config, created_at, updated_at,
                   NULL as git_repo_path, NULL as current_version,
                   owner_id, visibility, status, reviewed_at, reviewed_by, download_count
            FROM templates
            WHERE category_id = ?
            ORDER BY created_at DESC
            LIMIT ?
            "#
        )
        .bind(category_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(templates)
    }

    /// 获取推荐模板（限制数量）
    pub async fn get_featured_limit(&self, limit: u32) -> Result<Vec<Template>> {
        let templates = sqlx::query_as::<_, Template>(
            r#"
            SELECT CAST(id AS SIGNED) as id, name, description,
                   CAST(category_id AS SIGNED) as category_id, is_featured, logo, introduction,
                   icon, template_type, type_config, created_at, updated_at,
                   NULL as git_repo_path, NULL as current_version,
                   owner_id, visibility, status, reviewed_at, reviewed_by, download_count
            FROM templates
            WHERE is_featured = 1
            ORDER BY created_at DESC
            LIMIT ?
            "#
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(templates)
    }

    // ===== 用户模板投稿 =====

    /// 列出用户的模板
    pub async fn list_user_templates(&self, owner_id: i64, query: &UserTemplateListQuery) -> Result<PagedResponse<Template>> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).min(100);
        let offset = (page - 1) * page_size;

        let mut where_clauses = vec![format!("owner_id = {}", owner_id)];
        if let Some(ref vis) = query.visibility {
            where_clauses.push(format!("visibility = '{}'", vis.replace('\'', "''")));
        }
        if let Some(ref kw) = query.keyword {
            where_clauses.push(format!("(name LIKE '%{}%' OR description LIKE '%{}%')", kw.replace('\'', "''"), kw.replace('\'', "''")));
        }
        let where_sql = where_clauses.join(" AND ");

        let templates = sqlx::query_as::<_, Template>(
            &format!(r#"
            SELECT CAST(id AS SIGNED) as id, name, description,
                   CAST(category_id AS SIGNED) as category_id, is_featured, logo, introduction,
                   icon, template_type, type_config, created_at, updated_at,
                   NULL as git_repo_path, NULL as current_version,
                   owner_id, visibility, status, reviewed_at, reviewed_by, download_count
            FROM templates WHERE {} ORDER BY created_at DESC LIMIT ? OFFSET ?
            "#, where_sql)
        )
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        let total: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM templates WHERE {}", where_sql))
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        Ok(PagedResponse::new(templates, total as u32, page, page_size))
    }

    /// 获取待审核模板列表
    pub async fn list_pending_templates(&self, page: u32, page_size: u32) -> Result<PagedResponse<Template>> {
        let page = page.max(1);
        let offset = (page - 1) * page_size;

        let templates = sqlx::query_as::<_, Template>(
            r#"
            SELECT CAST(id AS SIGNED) as id, name, description,
                   CAST(category_id AS SIGNED) as category_id, is_featured, logo, introduction,
                   icon, template_type, type_config, created_at, updated_at,
                   NULL as git_repo_path, NULL as current_version,
                   owner_id, visibility, status, reviewed_at, reviewed_by, download_count
            FROM templates WHERE visibility = 'pending' AND status = 'active'
            ORDER BY created_at ASC LIMIT ? OFFSET ?
            "#
        )
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM templates WHERE visibility = 'pending' AND status = 'active'")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        Ok(PagedResponse::new(templates, total as u32, page, page_size))
    }

    /// 获取公开模板列表（客户端，visibility=public 且 status=active）
    pub async fn list_public_templates(&self, query: &UserTemplateListQuery) -> Result<PagedResponse<Template>> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(12).min(100);
        let offset = (page - 1) * page_size;

        let mut where_clauses = vec!["visibility = 'public'".to_string(), "status = 'active'".to_string()];
        if let Some(ref kw) = query.keyword {
            where_clauses.push(format!("(name LIKE '%{}%' OR description LIKE '%{}%')", kw.replace('\'', "''"), kw.replace('\'', "''")));
        }
        if let Some(cat_id) = query.category_id {
            where_clauses.push(format!("category_id = {}", cat_id));
        }
        let where_sql = where_clauses.join(" AND ");

        let templates = sqlx::query_as::<_, Template>(
            &format!(r#"
            SELECT CAST(id AS SIGNED) as id, name, description,
                   CAST(category_id AS SIGNED) as category_id, is_featured, logo, introduction,
                   icon, template_type, type_config, created_at, updated_at,
                   NULL as git_repo_path, NULL as current_version,
                   owner_id, visibility, status, reviewed_at, reviewed_by, download_count
            FROM templates WHERE {} ORDER BY is_featured DESC, created_at DESC LIMIT ? OFFSET ?
            "#, where_sql)
        )
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        let total: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM templates WHERE {}", where_sql))
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);

        Ok(PagedResponse::new(templates, total as u32, page, page_size))
    }

    /// 验证模板所有者
    pub async fn is_owner(&self, template_id: i64, user_id: i64) -> Result<bool> {
        let owner_id: Option<i64> = sqlx::query_scalar("SELECT owner_id FROM templates WHERE id = ?")
            .bind(template_id)
            .fetch_optional(&self.pool)
            .await?
            .flatten();
        Ok(owner_id == Some(user_id))
    }

    /// 更新模板可见性
    pub async fn update_visibility(&self, template_id: i64, visibility: &str) -> Result<()> {
        sqlx::query("UPDATE templates SET visibility = ? WHERE id = ?")
            .bind(visibility)
            .bind(template_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 审核模板
    pub async fn review_template(&self, template_id: i64, reviewer_id: i64, action: &str, reason: &str) -> Result<()> {
        let new_visibility = match action {
            "approve" => "public",
            _ => "private",
        };
        let new_status = if action == "reject" { "rejected" } else { "active" };

        let mut tx = self.pool.begin().await?;

        sqlx::query("UPDATE templates SET visibility = ?, status = ?, reviewed_by = ?, reviewed_at = NOW() WHERE id = ?")
            .bind(new_visibility)
            .bind(new_status)
            .bind(reviewer_id)
            .bind(template_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("INSERT INTO template_reviews (template_id, reviewer_id, action, reason) VALUES (?, ?, ?, ?)")
            .bind(template_id)
            .bind(reviewer_id)
            .bind(action)
            .bind(reason)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    /// 增加下载计数
    pub async fn increment_download_count(&self, template_id: i64) -> Result<()> {
        sqlx::query("UPDATE templates SET download_count = COALESCE(download_count, 0) + 1 WHERE id = ?")
            .bind(template_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 更新模板（带所有权校验）
    pub async fn update_user_template(&self, request: &UpdateTemplateRequest, user_id: i64) -> Result<bool> {
        let is_owner = self.is_owner(request.id, user_id).await?;
        if !is_owner {
            anyhow::bail!("无权编辑此模板");
        }
        self.update(request).await
    }

    /// 删除模板（带所有权校验）
    pub async fn delete_user_template(&self, template_id: i64, user_id: i64) -> Result<bool> {
        let is_owner = self.is_owner(template_id, user_id).await?;
        if !is_owner {
            anyhow::bail!("无权删除此模板");
        }
        self.delete(template_id).await
    }
}