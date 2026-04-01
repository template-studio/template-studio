use sqlx::MySqlPool;
use template_studio_shared::models::category::*;
use template_studio_shared::utils::response::PagedResponse;
use anyhow::Result;

/// 分类数据访问层
pub struct CategoryRepository {
    pool: MySqlPool,
}

impl CategoryRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// 创建分类
    pub async fn create(&self, request: &CreateCategoryRequest) -> Result<i64> {
        let sort = request.sort.unwrap_or(0);

        let result = sqlx::query(
            "INSERT INTO categories (name, description, icon, sort) VALUES (?, ?, ?, ?)"
        )
        .bind(&request.name)
        .bind(&request.description)
        .bind(&request.icon)
        .bind(sort)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_id() as i64)
    }

    /// 根据ID获取分类
    pub async fn get_by_id(&self, id: i64) -> Result<Option<Category>> {
        let category = sqlx::query_as::<_, Category>(
            "SELECT id, name, description, icon, sort, created_at, updated_at FROM categories WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(category)
    }

    /// 根据名称获取分类
    pub async fn get_by_name(&self, name: &str) -> Result<Option<Category>> {
        let category = sqlx::query_as::<_, Category>(
            "SELECT id, name, description, icon, sort, created_at, updated_at FROM categories WHERE name = ?"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(category)
    }

    /// 更新分类
    pub async fn update(&self, request: &UpdateCategoryRequest) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE categories SET name = ?, description = ?, icon = ?, sort = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
        )
        .bind(&request.name)
        .bind(&request.description)
        .bind(&request.icon)
        .bind(request.sort)
        .bind(request.id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 删除分类
    pub async fn delete(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM categories WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 分页获取分类列表
    pub async fn list(&self, query: &CategoryListQuery) -> Result<PagedResponse<Category>> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).min(100);
        let offset = (page - 1) * page_size;

        let mut where_clause = String::new();
        let mut params = Vec::new();

        if let Some(name) = &query.name {
            if !where_clause.is_empty() {
                where_clause.push_str(" AND ");
            } else {
                where_clause.push_str(" WHERE ");
            }
            where_clause.push_str("name LIKE ?");
            params.push(format!("%{}%", name));
        }

        // 获取总数
        let count_sql = format!("SELECT COUNT(*) as count FROM categories{}", where_clause);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        for param in &params {
            count_query = count_query.bind(param);
        }
        let total = count_query.fetch_one(&self.pool).await? as u32;

        // 获取数据
        let data_sql = format!(
            "SELECT id, name, description, icon, sort, created_at, updated_at FROM categories{} ORDER BY sort ASC, id ASC LIMIT ? OFFSET ?",
            where_clause
        );
        let mut data_query = sqlx::query_as::<_, Category>(&data_sql);
        for param in &params {
            data_query = data_query.bind(param);
        }
        data_query = data_query.bind(page_size as i64).bind(offset as i64);

        let categories = data_query.fetch_all(&self.pool).await?;

        Ok(PagedResponse::new(categories, total, page, page_size))
    }

    /// 获取所有分类（用于下拉框等场景）
    pub async fn get_all(&self) -> Result<Vec<Category>> {
        let categories = sqlx::query_as::<_, Category>(
            "SELECT id, name, description, icon, sort, created_at, updated_at FROM categories ORDER BY sort ASC, id ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(categories)
    }

    /// 统计总分类数
    pub async fn count_all(&self) -> Result<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM categories")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
        Ok(count)
    }

    /// 获取分类（限制数量），用于Studio首页
    pub async fn get_with_templates_limit(&self, limit: u32) -> Result<Vec<Category>> {
        let categories = sqlx::query_as::<_, Category>(
            "SELECT id, name, description, icon, sort, created_at, updated_at FROM categories ORDER BY sort ASC, id ASC LIMIT ?"
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(categories)
    }
}