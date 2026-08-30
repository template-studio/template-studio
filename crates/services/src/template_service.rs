use std::sync::Arc;
use template_studio_infrastructure::config::storage::StorageManager;
use template_studio_repositories::{CategoryRepository, LanguageRepository, TemplateRepository};
use template_studio_shared::{
    constants::api::ApiConstants,
    models::studio::*,
    models::template::*,
    utils::{error::AppError, validation::validate_request},
};

/// 模板业务服务
pub struct TemplateService {
    repository: Arc<TemplateRepository>,
    category_repository: Arc<CategoryRepository>,
    language_repository: Arc<LanguageRepository>,
    storage_manager: Arc<StorageManager>,
}

impl TemplateService {
    pub fn new(
        repository: Arc<TemplateRepository>,
        category_repository: Arc<CategoryRepository>,
        language_repository: Arc<LanguageRepository>,
        storage_manager: Arc<StorageManager>,
    ) -> Self {
        Self {
            repository,
            category_repository,
            language_repository,
            storage_manager,
        }
    }

    /// 创建模板
    pub async fn create_template(&self, request: CreateTemplateRequest) -> Result<i64, AppError> {
        // 验证请求数据
        validate_request(&request)?;

        // 检查分类是否存在
        let _category = self
            .category_repository
            .get_by_id(request.category_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("分类 {} 不存在", request.category_id)))?;

        // 检查语言是否存在
        for lang in &request.languages {
            let _language = self
                .language_repository
                .get_by_id(lang.language_id)
                .await?
                .ok_or_else(|| {
                    AppError::NotFound(format!("编程语言 {} 不存在", lang.language_id))
                })?;
        }

        // 创建模板ID和存储路径
        let template_id = chrono::Utc::now().timestamp_millis() as i64;
        let git_repo_path = self
            .storage_manager
            .get_template_path(template_id)
            .to_string_lossy()
            .to_string();

        // 初始化模板存储结构
        self.storage_manager
            .initialize_template_structure(template_id)
            .await?;

        // 创建模板记录 (使用Service层生成的ID)
        let created_id = self
            .repository
            .create(&request, template_id, &git_repo_path)
            .await?;

        // TODO: 初始化Git仓库 - Git服务已经实现,需要集成到模板创建流程中
        // 当前状态: GitService已在 infrastructure/git/service.rs 实现
        // 集成步骤:
        // 1. 在TemplateService中添加GitService依赖
        // 2. 调用git_service.init_repository()初始化Git仓库
        // 3. 创建.gitignore和README.md文件
        // 4. 进行初始提交
        // 示例代码:
        // let repo_path = std::path::PathBuf::from(&git_repo_path);
        // self.git_service.init_repository(&repo_path, &request.name, Some("Template Studio"), Some("template@studio.local")).await?;

        tracing::info!("创建模板成功: id={}, name={}", created_id, request.name);
        Ok(created_id)
    }

    /// 获取模板详情
    pub async fn get_template(&self, id: i64) -> Result<Option<TemplateDetailResponse>, AppError> {
        let template = self.repository.get_by_id(id).await?;

        match template {
            Some(tmpl) => {
                // 获取关联的语言信息
                let languages = self.get_template_languages(id).await?;

                let response = TemplateDetailResponse {
                    id: tmpl.id,
                    name: tmpl.name,
                    description: tmpl.description,
                    introduction: tmpl.introduction,
                    category_id: tmpl.category_id,
                    is_featured: tmpl.is_featured,
                    logo: tmpl.logo,
                    icon: tmpl.icon,
                    template_type: tmpl.template_type,
                    type_config: tmpl.type_config,
                    git_repo_path: tmpl.git_repo_path.unwrap_or_default(),
                    current_version: tmpl.current_version.unwrap_or_default(),
                    created_at: tmpl.created_at,
                    updated_at: tmpl.updated_at,
                    languages,
                };

                Ok(Some(response))
            }
            None => Ok(None),
        }
    }

    /// 更新模板
    pub async fn update_template(&self, request: UpdateTemplateRequest) -> Result<(), AppError> {
        // 验证请求数据
        validate_request(&request)?;

        // 检查模板是否存在
        let _template = self
            .repository
            .get_by_id(request.id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("模板 {} 不存在", request.id)))?;

        // 检查分类是否存在
        let _category = self
            .category_repository
            .get_by_id(request.category_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("分类 {} 不存在", request.category_id)))?;

        // 检查语言是否存在
        for lang in &request.languages {
            let _language = self
                .language_repository
                .get_by_id(lang.language_id)
                .await?
                .ok_or_else(|| {
                    AppError::NotFound(format!("编程语言 {} 不存在", lang.language_id))
                })?;
        }

        // 更新模板
        let updated = self.repository.update(&request).await?;
        if !updated {
            return Err(AppError::NotFound(format!("模板 {} 不存在", request.id)));
        }

        tracing::info!("更新模板成功: id={}, name={}", request.id, request.name);
        Ok(())
    }

    /// 删除模板
    pub async fn delete_template(&self, id: i64) -> Result<(), AppError> {
        // 先获取模板信息用于日志记录（可选）
        let template = self.repository.get_by_id(id).await?;

        // 获取模板目录路径
        let template_path = self.storage_manager.get_template_path(id);

        // 执行删除操作
        let _deleted = self.repository.delete(id).await?;

        // 删除文件系统中的模板目录（包括 Git 仓库）
        if template_path.exists() {
            match tokio::fs::remove_dir_all(&template_path).await {
                Ok(_) => {
                    tracing::info!("删除模板目录成功: id={}, path={:?}", id, template_path);
                }
                Err(e) => {
                    tracing::error!(
                        "删除模板目录失败: id={}, path={}, error={}",
                        id,
                        template_path.display(),
                        e
                    );
                    // 目录删除失败不影响数据库删除成功的状态
                    // 但记录错误日志供后续排查
                }
            }
        } else {
            tracing::warn!(
                "模板目录不存在，跳过删除: id={}, path={:?}",
                id,
                template_path
            );
        }

        // 同步删除该模板的全部发布快照目录，避免 releases/<id>/ 成为磁盘孤儿
        let releases_path = self
            .storage_manager
            .get_releases_base_path()
            .join(id.to_string());
        if releases_path.exists() {
            match tokio::fs::remove_dir_all(&releases_path).await {
                Ok(_) => tracing::info!("删除发布快照目录成功: id={}", id),
                Err(e) => tracing::error!(
                    "删除发布快照目录失败: id={}, path={}, error={}",
                    id,
                    releases_path.display(),
                    e
                ),
            }
        }

        // 无论模板是否存在，只要删除操作执行了就认为成功
        // 这符合RESTful API的DELETE幂等性原则
        if let Some(tmpl) = template {
            tracing::info!("删除模板成功: id={}, name={}", id, tmpl.name);
        } else {
            tracing::info!("删除模板成功: id={} (模板已不存在)", id);
        }

        Ok(())
    }

    /// 分页获取模板列表
    pub async fn list_templates(
        &self,
        query: TemplateListQuery,
    ) -> Result<template_studio_shared::utils::response::PagedResponse<Template>, AppError> {
        let paged_response = self.repository.list(&query).await?;
        Ok(paged_response)
    }

    /// 获取模板列表（匹配原系统格式）
    pub async fn list_templates_original_format(
        &self,
        query: TemplateListQuery,
    ) -> Result<template_studio_shared::models::template::TemplateListResponse, AppError> {
        tracing::info!(
            "TemplateService::list_templates_original_format called with query: {:?}",
            query
        );
        let paged_response = self.repository.list(&query).await?;

        // 转换为原系统格式
        let mut templates_list: Vec<template_studio_shared::models::template::TemplateItem> =
            Vec::new();

        for tmpl in paged_response.items {
            // 获取模板的关联语言
            let languages = self
                .repository
                .get_template_languages(tmpl.id)
                .await
                .unwrap_or_default();

            // 格式化时间
            let created_at = tmpl.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
            let updated_at = tmpl.updated_at.format("%Y-%m-%d %H:%M:%S").to_string();

            templates_list.push(template_studio_shared::models::template::TemplateItem {
                id: tmpl.id,
                name: tmpl.name,
                description: tmpl.description,
                introduction: tmpl.introduction,
                category_id: tmpl.category_id,
                is_featured: tmpl.is_featured,
                template_type: tmpl.template_type,
                type_config: tmpl.type_config.or_else(|| Some(String::new())),
                visibility: tmpl.visibility,
                status: tmpl.status,
                owner_id: tmpl.owner_id,
                owner_name: tmpl.owner_name,
                owner_avatar: tmpl.owner_avatar,
                download_count: tmpl.download_count,
                created_at,
                updated_at,
                languages,
            });
        }

        Ok(
            template_studio_shared::models::template::TemplateListResponse {
                current_page: paged_response.page,
                total: paged_response.total,
                templates_list,
            },
        )
    }

    /// 切换推荐状态
    pub async fn toggle_featured(&self, id: i64, is_featured: i32) -> Result<(), AppError> {
        // 检查模板是否存在
        let _template = self
            .repository
            .get_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("模板 {} 不存在", id)))?;

        // 验证推荐状态值
        if is_featured != ApiConstants::FEATURED_NO && is_featured != ApiConstants::FEATURED_YES {
            return Err(AppError::Validation("推荐状态值必须为0或1".to_string()));
        }

        // 切换推荐状态
        let updated = self.repository.toggle_featured(id, is_featured).await?;
        if !updated {
            return Err(AppError::NotFound(format!("模板 {} 不存在", id)));
        }

        tracing::info!(
            "切换模板推荐状态成功: id={}, is_featured={}",
            id,
            is_featured
        );
        Ok(())
    }

    /// Fork模板
    pub async fn fork_template(&self, request: ForkTemplateRequest) -> Result<i64, AppError> {
        // 提前克隆需要的字段
        let source_id = request.source_id;
        let name = request.name.clone();
        let category_id = request.category_id;

        // 检查源模板是否存在
        let _source_template = self
            .repository
            .get_by_id(source_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("源模板 {} 不存在", source_id)))?;

        // 如果指定了新分类，检查分类是否存在
        if let Some(cat_id) = category_id {
            let _category = self
                .category_repository
                .get_by_id(cat_id)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("分类 {} 不存在", cat_id)))?;
        }

        // Fork模板（先创建数据库记录，获取新模板ID）
        let new_template_id = self.repository.fork(&request).await?;

        // 初始化模板存储结构
        self.storage_manager
            .initialize_template_structure(new_template_id)
            .await?;

        // TODO: Git 服务克隆并清理仓库（暂时注释）
        // self.clone_git_repository(source_id, new_template_id, &name).await?;

        tracing::info!(
            "Fork模板成功: source_id={}, new_id={}, new_name={}",
            source_id,
            new_template_id,
            name
        );
        Ok(new_template_id)
    }

    /// 获取模板类型列表
    pub async fn get_template_types(&self) -> Result<Vec<TemplateTypeInfo>, AppError> {
        // 返回原系统定义的模板类型列表
        let template_types = vec![
            TemplateTypeInfo {
                value: "basic".to_string(),
                label: "基础模板".to_string(),
                description: "简单的基础项目模板，适合快速开始新项目".to_string(),
            },
            TemplateTypeInfo {
                value: "scaffold".to_string(),
                label: "脚手架模板".to_string(),
                description: "完整的项目脚手架，包含完整的项目结构和最佳实践".to_string(),
            },
            TemplateTypeInfo {
                value: "data_driven".to_string(),
                label: "数据驱动模板".to_string(),
                description: "基于数据驱动的动态模板，支持复杂的变量替换和条件逻辑".to_string(),
            },
        ];

        Ok(template_types)
    }

    /// 获取Studio首页数据
    pub async fn get_studio_index(
        &self,
        request: StudioIndexRequest,
    ) -> Result<StudioIndexResponse, AppError> {
        let category_limit = request.category_limit.unwrap_or(6);
        let featured_limit = request.featured_limit.unwrap_or(8);

        // 获取统计数据
        let statistics = self.get_index_statistics().await?;

        // 获取分类及其模板
        let categories = self.get_categories_with_templates(category_limit).await?;

        // 获取推荐模板
        let featured_templates = self.get_featured_templates(featured_limit).await?;

        Ok(StudioIndexResponse {
            statistics,
            categories,
            featured_templates,
        })
    }

    /// 获取统计数据
    async fn get_index_statistics(&self) -> Result<IndexStatistics, AppError> {
        // 获取总模板数
        let total_templates = self.repository.count_all().await.unwrap_or(0);

        // 获取总分类数
        let total_categories = self.category_repository.count_all().await.unwrap_or(0);

        // 获取总语言数
        let total_languages = self.language_repository.count_all().await.unwrap_or(0);

        // 获取推荐模板数
        let featured_templates = self.repository.count_featured().await.unwrap_or(0);

        Ok(IndexStatistics {
            total_templates,
            total_categories,
            total_languages,
            featured_templates,
        })
    }

    /// 获取分类及其模板
    async fn get_categories_with_templates(
        &self,
        limit: u32,
    ) -> Result<Vec<CategoryWithTemplates>, AppError> {
        let categories = self
            .category_repository
            .get_with_templates_limit(limit)
            .await?;

        let mut result = Vec::new();
        for category in categories {
            let templates = self
                .repository
                .get_by_category_limit(category.id, 3)
                .await
                .unwrap_or_default();

            let category_templates: Vec<CategoryTemplate> = templates
                .into_iter()
                .map(|tmpl| CategoryTemplate {
                    id: tmpl.id,
                    name: tmpl.name,
                    description: tmpl.description,
                    template_type: tmpl.template_type,
                    created_at: tmpl.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                })
                .collect();

            result.push(CategoryWithTemplates {
                id: category.id,
                name: category.name,
                description: category.description,
                icon: category.icon,
                templates: category_templates,
            });
        }

        Ok(result)
    }

    /// 获取推荐模板
    async fn get_featured_templates(&self, limit: u32) -> Result<Vec<FeaturedTemplate>, AppError> {
        let templates = self
            .repository
            .get_featured_limit(limit)
            .await
            .unwrap_or_default();

        let mut result = Vec::new();
        for template in templates {
            // 获取分类信息
            let category = self
                .category_repository
                .get_by_id(template.category_id)
                .await?
                .ok_or_else(|| {
                    AppError::NotFound(format!("分类 {} 不存在", template.category_id))
                })?;

            // 获取语言信息
            let languages = self
                .repository
                .get_template_languages(template.id)
                .await
                .unwrap_or_default();

            let template_languages: Vec<StudioTemplateLanguage> = languages
                .into_iter()
                .map(|lang| StudioTemplateLanguage {
                    language_id: lang.language_id,
                    name: format!("语言{}", lang.language_id), // 临时实现，应该关联查询languages表
                    is_primary: lang.is_primary,
                })
                .collect();

            result.push(FeaturedTemplate {
                id: template.id,
                name: template.name,
                description: template.description,
                introduction: template.introduction,
                template_type: template.template_type,
                category_id: template.category_id,
                category_name: category.name,
                logo: template.logo,
                icon: template.icon,
                created_at: template.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                owner_name: template.owner_name,
                owner_avatar: template.owner_avatar,
                languages: template_languages,
            });
        }

        Ok(result)
    }

    /// 获取模板的语言信息
    async fn get_template_languages(
        &self,
        template_id: i64,
    ) -> Result<Vec<TemplateLanguageInfo>, AppError> {
        let languages = self
            .repository
            .get_template_language_details(template_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(languages)
    }

    /// 获取模板总数
    pub async fn get_template_count(&self) -> Result<i64, AppError> {
        let count = self.repository.count_all().await?;
        Ok(count)
    }

    // ===== 用户模板投稿 =====

    /// 用户创建模板
    pub async fn create_user_template(
        &self,
        user_id: i64,
        mut request: CreateTemplateRequest,
    ) -> Result<i64, AppError> {
        validate_request(&request)?;
        request.owner_id = Some(user_id);
        request.visibility = request.visibility.or_else(|| Some("private".to_string()));
        self.create_template(request).await
    }

    /// 用户更新模板
    pub async fn update_user_template(
        &self,
        user_id: i64,
        request: UpdateTemplateRequest,
    ) -> Result<(), AppError> {
        validate_request(&request)?;
        let updated = self
            .repository
            .update_user_template(&request, user_id)
            .await
            .map_err(|e| AppError::Forbidden(e.to_string()))?;
        if !updated {
            return Err(AppError::NotFound(format!("模板 {} 不存在", request.id)));
        }
        tracing::info!("用户 {} 更新模板 {}", user_id, request.id);
        Ok(())
    }

    /// 用户删除模板
    pub async fn delete_user_template(
        &self,
        user_id: i64,
        template_id: i64,
    ) -> Result<(), AppError> {
        let deleted = self
            .repository
            .delete_user_template(template_id, user_id)
            .await
            .map_err(|e| AppError::Forbidden(e.to_string()))?;
        if !deleted {
            return Err(AppError::NotFound(format!("模板 {} 不存在", template_id)));
        }
        let template_path = self.storage_manager.get_template_path(template_id);
        if template_path.exists() {
            let _ = tokio::fs::remove_dir_all(&template_path).await;
        }
        tracing::info!("用户 {} 删除模板 {}", user_id, template_id);
        Ok(())
    }

    /// 判断用户是否为模板属主（供 handler 层做属主校验）
    pub async fn is_template_owner(
        &self,
        template_id: i64,
        user_id: i64,
    ) -> Result<bool, AppError> {
        self.repository
            .is_owner(template_id, user_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))
    }

    /// 提交审核 (private → pending)
    pub async fn submit_for_review(&self, user_id: i64, template_id: i64) -> Result<(), AppError> {
        let is_owner = self
            .repository
            .is_owner(template_id, user_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        if !is_owner {
            return Err(AppError::Forbidden("无权操作此模板".to_string()));
        }
        self.repository
            .update_visibility(template_id, "pending")
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        tracing::info!("用户 {} 提交模板 {} 审核", user_id, template_id);
        Ok(())
    }

    /// 列出用户的模板
    pub async fn list_user_templates(
        &self,
        user_id: i64,
        query: UserTemplateListQuery,
    ) -> Result<TemplateListResponse, AppError> {
        let paged = self
            .repository
            .list_user_templates(user_id, &query)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let mut list = Vec::new();
        for tmpl in paged.items {
            let languages = self
                .repository
                .get_template_languages(tmpl.id)
                .await
                .unwrap_or_default();
            list.push(TemplateItem {
                id: tmpl.id,
                name: tmpl.name,
                description: tmpl.description,
                introduction: tmpl.introduction,
                category_id: tmpl.category_id,
                is_featured: tmpl.is_featured,
                template_type: tmpl.template_type,
                type_config: tmpl.type_config.or_else(|| Some(String::new())),
                visibility: tmpl.visibility,
                status: tmpl.status,
                owner_id: tmpl.owner_id,
                owner_name: tmpl.owner_name,
                owner_avatar: tmpl.owner_avatar,
                download_count: tmpl.download_count,
                created_at: tmpl.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: tmpl.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                languages,
            });
        }
        Ok(TemplateListResponse {
            current_page: paged.page,
            total: paged.total,
            templates_list: list,
        })
    }

    /// 获取公开模板列表
    pub async fn list_public_templates(
        &self,
        query: UserTemplateListQuery,
    ) -> Result<TemplateListResponse, AppError> {
        let paged = self
            .repository
            .list_public_templates(&query)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let mut list = Vec::new();
        for tmpl in paged.items {
            let languages = self
                .repository
                .get_template_languages(tmpl.id)
                .await
                .unwrap_or_default();
            list.push(TemplateItem {
                id: tmpl.id,
                name: tmpl.name,
                description: tmpl.description,
                introduction: tmpl.introduction,
                category_id: tmpl.category_id,
                is_featured: tmpl.is_featured,
                template_type: tmpl.template_type,
                type_config: tmpl.type_config.or_else(|| Some(String::new())),
                visibility: tmpl.visibility,
                status: tmpl.status,
                owner_id: tmpl.owner_id,
                owner_name: tmpl.owner_name,
                owner_avatar: tmpl.owner_avatar,
                download_count: tmpl.download_count,
                created_at: tmpl.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: tmpl.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                languages,
            });
        }
        Ok(TemplateListResponse {
            current_page: paged.page,
            total: paged.total,
            templates_list: list,
        })
    }

    /// 获取待审核模板列表（管理员）
    pub async fn list_pending_templates(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<TemplateListResponse, AppError> {
        let paged = self
            .repository
            .list_pending_templates(page, page_size)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let mut list = Vec::new();
        for tmpl in paged.items {
            let languages = self
                .repository
                .get_template_languages(tmpl.id)
                .await
                .unwrap_or_default();
            list.push(TemplateItem {
                id: tmpl.id,
                name: tmpl.name,
                description: tmpl.description,
                introduction: tmpl.introduction,
                category_id: tmpl.category_id,
                is_featured: tmpl.is_featured,
                template_type: tmpl.template_type,
                type_config: tmpl.type_config.or_else(|| Some(String::new())),
                visibility: tmpl.visibility,
                status: tmpl.status,
                owner_id: tmpl.owner_id,
                owner_name: tmpl.owner_name,
                owner_avatar: tmpl.owner_avatar,
                download_count: tmpl.download_count,
                created_at: tmpl.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: tmpl.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                languages,
            });
        }
        Ok(TemplateListResponse {
            current_page: paged.page,
            total: paged.total,
            templates_list: list,
        })
    }

    /// 审核模板（管理员）
    pub async fn review_template(
        &self,
        reviewer_id: i64,
        req: ReviewTemplateRequest,
    ) -> Result<(), AppError> {
        let _template = self
            .repository
            .get_by_id(req.template_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("模板 {} 不存在", req.template_id)))?;
        let reason = req.reason.unwrap_or_default();
        self.repository
            .review_template(req.template_id, reviewer_id, &req.action, &reason)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        tracing::info!(
            "审核人 {} 审核模板 {} action={}",
            reviewer_id,
            req.template_id,
            req.action
        );
        Ok(())
    }
}
