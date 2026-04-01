use sqlx::MySqlPool;
use template_studio_shared::models::language::*;
use template_studio_shared::utils::response::PagedResponse;
use anyhow::Result;

/// 编程语言数据访问层
pub struct LanguageRepository {
    pool: MySqlPool,
}

impl LanguageRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// 创建编程语言
    pub async fn create(&self, request: &CreateLanguageRequest) -> Result<i64> {
        let is_popular = request.is_popular.unwrap_or(0);

        let result = sqlx::query(
            "INSERT INTO languages (name, display_name, code, icon, color, is_popular) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&request.name)
        .bind(&request.display_name)
        .bind(&request.code)
        .bind(&request.icon)
        .bind(&request.color)
        .bind(is_popular)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_id() as i64)
    }

    /// 根据ID获取编程语言
    pub async fn get_by_id(&self, id: u32) -> Result<Option<Language>> {
        let language = sqlx::query_as::<_, Language>(
            "SELECT id, name, display_name, code, icon, color, is_popular, created_at, updated_at FROM languages WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(language)
    }

    /// 根据代码获取编程语言
    pub async fn get_by_code(&self, code: &str) -> Result<Option<Language>> {
        let language = sqlx::query_as::<_, Language>(
            "SELECT id, name, display_name, code, icon, color, is_popular, created_at, updated_at FROM languages WHERE code = ?"
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;

        Ok(language)
    }

    /// 更新编程语言
    pub async fn update(&self, request: &UpdateLanguageRequest) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE languages SET name = ?, display_name = ?, code = ?, icon = ?, color = ?, is_popular = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
        )
        .bind(&request.name)
        .bind(&request.display_name)
        .bind(&request.code)
        .bind(&request.icon)
        .bind(&request.color)
        .bind(request.is_popular)
        .bind(request.id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 删除编程语言
    pub async fn delete(&self, id: u32) -> Result<bool> {
        let result = sqlx::query("DELETE FROM languages WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 分页获取编程语言列表
    pub async fn list(&self, query: &LanguageListQuery) -> Result<PagedResponse<Language>> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).min(100);
        let offset = (page - 1) * page_size;

        // 简化实现：先获取所有数据，然后返回分页结果
        let mut all_languages = self.get_all().await?;

        // 应用过滤条件
        if let Some(name) = &query.name {
            all_languages.retain(|lang| lang.name.to_lowercase().contains(&name.to_lowercase()));
        }

        if let Some(is_popular) = query.is_popular {
            all_languages.retain(|lang| lang.is_popular == is_popular);
        }

        let total = all_languages.len() as u32;

        // 分页
        let start_index = offset as usize;
        let end_index = (start_index + page_size as usize).min(all_languages.len());
        let languages = if start_index < all_languages.len() {
            all_languages[start_index..end_index].to_vec()
        } else {
            vec![]
        };

        Ok(PagedResponse::new(languages, total, page, page_size))
    }

    /// 获取所有编程语言
    pub async fn get_all(&self) -> Result<Vec<Language>> {
        let languages = sqlx::query_as::<_, Language>(
            "SELECT id, name, display_name, code, icon, color, is_popular, created_at, updated_at FROM languages ORDER BY id DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(languages)
    }

    /// 获取热门编程语言
    pub async fn get_popular(&self) -> Result<Vec<Language>> {
        let languages = sqlx::query_as::<_, Language>(
            "SELECT id, name, display_name, code, icon, color, is_popular, created_at, updated_at FROM languages WHERE is_popular = 1 ORDER BY id DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(languages)
    }

    /// 统计总语言数
    pub async fn count_all(&self) -> Result<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM languages")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
        Ok(count)
    }
}