use std::sync::Arc;
use template_studio_repositories::CategoryRepository;
use template_studio_shared::{
    models::category::*,
    utils::{error::AppError, validation::validate_request},
};

/// 分类业务服务
pub struct CategoryService {
    repository: Arc<CategoryRepository>,
}

impl CategoryService {
    pub fn new(repository: Arc<CategoryRepository>) -> Self {
        Self { repository }
    }

    /// 创建分类
    pub async fn create_category(&self, request: CreateCategoryRequest) -> Result<i64, AppError> {
        // 验证请求数据
        validate_request(&request)?;

        // 检查名称是否重复
        if self.repository.get_by_name(&request.name).await?.is_some() {
            return Err(AppError::Duplicate(format!(
                "分类名称 '{}' 已存在",
                request.name
            )));
        }

        // 创建分类
        let category_id = self.repository.create(&request).await?;
        tracing::info!("创建分类成功: id={}, name={}", category_id, request.name);

        Ok(category_id)
    }

    /// 获取分类详情
    pub async fn get_category(&self, id: i64) -> Result<Option<Category>, AppError> {
        let category = self.repository.get_by_id(id).await?;
        Ok(category)
    }

    /// 更新分类
    pub async fn update_category(&self, request: UpdateCategoryRequest) -> Result<(), AppError> {
        // 验证请求数据
        validate_request(&request)?;

        // 检查分类是否存在
        let existing = self
            .repository
            .get_by_id(request.id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("分类 {} 不存在", request.id)))?;

        // 如果名称发生变化，检查新名称是否重复
        if existing.name != request.name {
            if self.repository.get_by_name(&request.name).await?.is_some() {
                return Err(AppError::Duplicate(format!(
                    "分类名称 '{}' 已存在",
                    request.name
                )));
            }
        }

        // 更新分类
        let updated = self.repository.update(&request).await?;
        if !updated {
            return Err(AppError::NotFound(format!("分类 {} 不存在", request.id)));
        }

        tracing::info!("更新分类成功: id={}, name={}", request.id, request.name);
        Ok(())
    }

    /// 删除分类
    pub async fn delete_category(&self, id: i64) -> Result<(), AppError> {
        // 检查分类是否存在
        let _category = self
            .repository
            .get_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("分类 {} 不存在", id)))?;

        // TODO: 检查是否有关联的模板，如果有则不允许删除
        // let template_count = self.template_repository.count_by_category(id).await?;
        // if template_count > 0 {
        //     return Err(AppError::ValidationError(format!("该分类下还有 {} 个模板，无法删除", template_count)));
        // }

        // 删除分类
        let deleted = self.repository.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("分类 {} 不存在", id)));
        }

        tracing::info!("删除分类成功: id={}", id);
        Ok(())
    }

    /// 分页获取分类列表
    pub async fn list_categories(
        &self,
        query: CategoryListQuery,
    ) -> Result<Vec<Category>, AppError> {
        let paged_response = self.repository.list(&query).await?;
        Ok(paged_response.items)
    }

    /// 获取所有分类（用于下拉框等场景）
    pub async fn get_all_categories(&self) -> Result<Vec<Category>, AppError> {
        let categories = self.repository.get_all().await?;
        Ok(categories)
    }

    /// 获取分类总数
    pub async fn get_category_count(&self) -> Result<i64, AppError> {
        let count = self.repository.count_all().await?;
        Ok(count)
    }

    /// 获取分类分布（每个分类的模板数量）
    pub async fn get_category_distribution(&self) -> Result<Vec<CategoryDistribution>, AppError> {
        let categories = self.repository.get_all().await?;
        let mut distribution = Vec::new();

        for category in categories {
            distribution.push(CategoryDistribution {
                name: category.name,
                count: 0,
            });
        }

        Ok(distribution)
    }
}

/// 分类分布统计
#[derive(Debug, Clone, serde::Serialize)]
pub struct CategoryDistribution {
    pub name: String,
    pub count: i64,
}
