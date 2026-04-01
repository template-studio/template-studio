use template_studio_repositories::LanguageRepository;
use template_studio_shared::{
    models::language::*,
    utils::{validation::validate_request, error::AppError},
};
use std::sync::Arc;

/// 编程语言业务服务
pub struct LanguageService {
    repository: Arc<LanguageRepository>,
}

impl LanguageService {
    pub fn new(repository: Arc<LanguageRepository>) -> Self {
        Self { repository }
    }

    /// 创建编程语言
    pub async fn create_language(&self, request: CreateLanguageRequest) -> Result<i64, AppError> {
        // 验证请求数据
        validate_request(&request)?;

        // 检查代码是否重复
        if let Some(_) = self.repository.get_by_code(&request.code).await? {
            return Err(AppError::Duplicate(format!("语言代码 '{}' 已存在", request.code)));
        }

        // 创建编程语言
        let language_id = self.repository.create(&request).await?;
        tracing::info!("创建编程语言成功: id={}, code={}", language_id, request.code);

        Ok(language_id)
    }

    /// 获取编程语言详情
    pub async fn get_language(&self, id: i64) -> Result<Option<Language>, AppError> {
        let language = self.repository.get_by_id(id as u32).await?;
        Ok(language)
    }

    /// 根据代码获取编程语言
    pub async fn get_language_by_code(&self, code: &str) -> Result<Option<Language>, AppError> {
        let language = self.repository.get_by_code(code).await?;
        Ok(language)
    }

    /// 更新编程语言
    pub async fn update_language(&self, request: UpdateLanguageRequest) -> Result<(), AppError> {
        // 验证请求数据
        validate_request(&request)?;

        // 检查编程语言是否存在
        let existing = self.repository.get_by_id(request.id).await?
            .ok_or_else(|| AppError::NotFound(format!("编程语言 {} 不存在", request.id)))?;

        // 如果代码发生变化，检查新代码是否重复
        if existing.code != request.code {
            if let Some(_) = self.repository.get_by_code(&request.code).await? {
                return Err(AppError::Duplicate(format!("语言代码 '{}' 已存在", request.code)));
            }
        }

        // 更新编程语言
        let updated = self.repository.update(&request).await?;
        if !updated {
            return Err(AppError::NotFound(format!("编程语言 {} 不存在", request.id)));
        }

        tracing::info!("更新编程语言成功: id={}, code={}", request.id, request.code);
        Ok(())
    }

    /// 删除编程语言
    pub async fn delete_language(&self, id: i64) -> Result<(), AppError> {
        // 检查编程语言是否存在
        let _language = self.repository.get_by_id(id as u32).await?
            .ok_or_else(|| AppError::NotFound(format!("编程语言 {} 不存在", id)))?;

        // TODO: 检查是否有关联的模板，如果有则不允许删除
        // let template_count = self.template_repository.count_by_language(id).await?;
        // if template_count > 0 {
        //     return Err(AppError::ValidationError(format!("该编程语言下还有 {} 个模板，无法删除", template_count)));
        // }

        // 删除编程语言
        let deleted = self.repository.delete(id as u32).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("编程语言 {} 不存在", id)));
        }

        tracing::info!("删除编程语言成功: id={}", id);
        Ok(())
    }

    /// 分页获取编程语言列表
    pub async fn list_languages(&self, query: LanguageListQuery) -> Result<Vec<Language>, AppError> {
        let paged_response = self.repository.list(&query).await?;
        Ok(paged_response.items)
    }

    /// 获取所有编程语言
    pub async fn get_all_languages(&self) -> Result<Vec<Language>, AppError> {
        let languages = self.repository.get_all().await?;
        Ok(languages)
    }

    /// 获取热门编程语言
    pub async fn get_popular_languages(&self) -> Result<Vec<Language>, AppError> {
        let languages = self.repository.get_popular().await?;
        Ok(languages)
    }

    /// 获取编程语言总数
    pub async fn get_language_count(&self) -> Result<i64, AppError> {
        let count = self.repository.count_all().await?;
        Ok(count)
    }

    /// 获取编程语言流行度
    pub async fn get_language_popularity(&self) -> Result<Vec<LanguagePopularity>, AppError> {
        let languages = self.repository.get_all().await?;
        let mut popularity = Vec::new();
        
        for language in languages {
            popularity.push(LanguagePopularity {
                name: language.name,
                count: 0,
            });
        }
        
        Ok(popularity)
    }
}

/// 编程语言流行度统计
#[derive(Debug, Clone, serde::Serialize)]
pub struct LanguagePopularity {
    pub name: String,
    pub count: i64,
}