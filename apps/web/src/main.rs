mod handlers;
mod routes;
mod middleware;
mod file_watcher;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use template_studio_infrastructure::{
    config::{settings::load_config, storage::StorageManager},
    database::pool::DatabasePool,
    logging::init_logging,
    file_tree::FileTreeService,
};
use template_studio_repositories::{CategoryRepository, LanguageRepository, TemplateRepository, VarPresetRepository, SystemSettingRepository, UserRepository, RoleRepository, PermissionRepository, PatRepository};
use template_studio_services::{CategoryService, LanguageService, TemplateService, VarPresetService, PresetSubscribeService, TemplateAnalysisService, TemplateVariablesService, TemplateRenderService, FileConditionsService, ReleaseService, BackupService, SystemSettingService, AuthService, UserService, RoleService, PermissionService, PatService};
use template_studio_shared::models::auth::JwtConfig;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 设置Git初始化函数
    handlers::template::set_git_init_fn(git_init_wrapper);

    // 初始化日志
    init_logging();
    info!("Template Studio Web服务启动中...");

    // 加载配置
    let config = load_config()?;
    info!("配置加载完成");

    // 创建数据库连接池
    let db_pool = DatabasePool::new(&config.database).await?;
    info!("数据库连接池创建完成");

    // 运行数据库迁移
    db_pool.run_migrations().await?;
    info!("数据库迁移完成");

    // 执行模板投稿系统迁移（017/018）
    {
        let pool = db_pool.get_pool().clone();
        // 017: 添加 visibility/owner_id 等字段
        let existing: Vec<String> = sqlx::query_scalar(
            "SELECT COLUMN_NAME FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'templates'"
        )
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
        if !existing.contains(&"owner_id".to_string()) {
            sqlx::query("ALTER TABLE templates ADD COLUMN owner_id BIGINT DEFAULT NULL, ADD COLUMN visibility VARCHAR(20) DEFAULT 'public', ADD COLUMN status VARCHAR(20) DEFAULT 'active', ADD COLUMN reviewed_at DATETIME DEFAULT NULL, ADD COLUMN reviewed_by BIGINT DEFAULT NULL, ADD COLUMN download_count INT DEFAULT 0")
                .execute(&pool).await.ok();
            sqlx::query("ALTER TABLE templates ADD INDEX idx_owner_id (owner_id), ADD INDEX idx_visibility (visibility)")
                .execute(&pool).await.ok();
            sqlx::query("UPDATE templates SET visibility='public', status='active' WHERE owner_id IS NULL")
                .execute(&pool).await.ok();
            info!("Migration 017 applied: templates table updated with visibility fields");
        }
        // 018: template_reviews 表
        let table_exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM information_schema.tables WHERE table_schema = DATABASE() AND table_name = 'template_reviews'"
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !table_exists {
            sqlx::query(
                "CREATE TABLE template_reviews (
                    id BIGINT PRIMARY KEY AUTO_INCREMENT,
                    template_id BIGINT NOT NULL,
                    reviewer_id BIGINT NOT NULL,
                    action VARCHAR(20) NOT NULL,
                    reason VARCHAR(500) DEFAULT '',
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    INDEX idx_template_id (template_id)
                )"
            ).execute(&pool).await.ok();
            info!("Migration 018 applied: template_reviews table created");
        }
    }

    // 创建存储管理器
    let storage_manager = Arc::new(StorageManager::new(config.storage.clone()));

    // 创建Git服务
    /*
    let git_config = template_studio_infrastructure::config::settings::GitConfig {
        auto_init: true,
        default_branch: "main".to_string(),
    };
    let git_service = Arc::new(GitService::new(git_config));
    */

    // 创建Repository层
    let category_repository = Arc::new(CategoryRepository::new(db_pool.get_pool().clone()));
    let language_repository = Arc::new(LanguageRepository::new(db_pool.get_pool().clone()));
    let template_repository = Arc::new(TemplateRepository::new(db_pool.get_pool().clone()));
    let var_preset_repository = Arc::new(VarPresetRepository::new(db_pool.get_pool().clone()));
    let system_setting_repository = Arc::new(SystemSettingRepository::new(db_pool.get_pool().clone()));
    let user_repository = Arc::new(UserRepository::new(db_pool.get_pool().clone()));
    let role_repository = Arc::new(RoleRepository::new(db_pool.get_pool().clone()));
    let permission_repository = Arc::new(PermissionRepository::new(db_pool.get_pool().clone()));
    let pat_repository = Arc::new(PatRepository::new(db_pool.get_pool().clone()));

    // 创建Service层
    let category_service = Arc::new(CategoryService::new(category_repository.clone()));
    let language_service = Arc::new(LanguageService::new(language_repository.clone()));
    let template_service = Arc::new(TemplateService::new(
        template_repository.clone(),
        category_repository.clone(),
        language_repository.clone(),
        storage_manager.clone(),
    ));
    let var_preset_service = Arc::new(VarPresetService::new(var_preset_repository.clone()));
    let preset_subscribe_service = Arc::new(PresetSubscribeService::new(var_preset_repository));
    let template_analysis_service = Arc::new(TemplateAnalysisService::new(config.storage.base_path.join("templates")));
    let template_variables_service = Arc::new(TemplateVariablesService::new(storage_manager.clone()));
    let template_render_service = Arc::new(TemplateRenderService::new(config.storage.base_path.join("templates")));
    let file_tree_service = Arc::new(FileTreeService::new(storage_manager.clone()));
    let file_conditions_service = Arc::new(FileConditionsService::new(storage_manager.clone()));
    let release_service = Arc::new(ReleaseService::new(storage_manager.clone(), db_pool.get_pool().clone()));
    let backup_service = Arc::new(BackupService::new(
        storage_manager.clone(),
        template_service.clone(),
        template_variables_service.clone(),
        file_conditions_service.clone(),
    ));
    let system_setting_service = Arc::new(SystemSettingService::new(system_setting_repository));
    let jwt_config = JwtConfig::default();
    let auth_service = Arc::new(AuthService::new(user_repository.clone(), jwt_config));
    let user_service = Arc::new(UserService::new(user_repository));
    let role_service = Arc::new(RoleService::new(role_repository));
    let permission_service = Arc::new(PermissionService::new(permission_repository));
    let pat_service = Arc::new(PatService::new(pat_repository));

    // 启动文件系统监听（监听 templates 目录）
    let templates_cache = template_render_service.get_cache();
    let templates_path = config.storage.base_path.join("templates");
    match file_watcher::start_file_watcher(templates_path, templates_cache) {
        Ok(_) => info!("文件系统监听已启动"),
        Err(e) => warn!("文件系统监听启动失败: {}", e),
    }

    // 创建应用状态
    let app_state = AppState {
        category_service,
        language_service,
        template_service,
        var_preset_service,
        preset_subscribe_service,
        template_analysis_service,
        template_variables_service,
        template_render_service,
        file_tree_service,
        file_conditions_service,
        release_service,
        backup_service,
        system_setting_service,
        auth_service,
        user_service,
        role_service,
        permission_service,
        pat_service,
        storage_manager,
    };

    // 创建路由
    let app = create_app(app_state);

    // 启动服务器
    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("服务器启动成功: http://{}", bind_addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub category_service: Arc<CategoryService>,
    pub language_service: Arc<LanguageService>,
    pub template_service: Arc<TemplateService>,
    pub var_preset_service: Arc<VarPresetService>,
    pub preset_subscribe_service: Arc<PresetSubscribeService>,
    pub template_analysis_service: Arc<TemplateAnalysisService>,
    pub template_variables_service: Arc<TemplateVariablesService>,
    pub template_render_service: Arc<TemplateRenderService>,
    pub file_tree_service: Arc<FileTreeService>,
    pub file_conditions_service: Arc<FileConditionsService>,
    pub release_service: Arc<ReleaseService>,
    pub backup_service: Arc<BackupService>,
    pub system_setting_service: Arc<SystemSettingService>,
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
    pub role_service: Arc<RoleService>,
    pub permission_service: Arc<PermissionService>,
    pub pat_service: Arc<PatService>,
    pub storage_manager: Arc<StorageManager>,
}

/// 创建应用路由
pub fn create_app(state: AppState) -> Router {
    let admin = routes::admin_routes()
        .layer(axum::middleware::from_fn_with_state(state.clone(), middleware::auth::auth_middleware));

    Router::new()
        // 健康检查
        .route("/health", get(health_check))
        // 认证API（公开）
        .nest("/api/v1/auth", routes::auth::auth_routes())
        // 模板API
        .nest("/api/v1/template", template_routes())
        // 管理员API（受认证保护）
        .nest("/api/v1/admin", admin)
        // 编辑器API
        .nest("/api/v1/editor", editor_routes())
        // Studio API
        .nest("/api/v1/studio", studio_routes())
        // 备份API
        .nest("/api/v1/backup", backup_routes())
        // 公开API
        .nest("/api/v1", routes::public_routes())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// 模板路由
fn template_routes() -> Router<AppState> {
    Router::new()
        .route("/templates/types", get(handlers::template::get_template_types))
        .route("/templateList", get(handlers::template::list_templates))
        .route("/templates/add", post(handlers::template::create_template))
        .route("/templates/detail", get(handlers::template::get_template_detail))
        .route("/templates/edit", put(handlers::template::update_template))
        .route("/templates/toggle-featured", put(handlers::template::toggle_featured))
        .route("/templates/del", delete(handlers::template::delete_template))
        .route("/templates/fork", post(handlers::template::fork_template))
        .route("/templates/:id/analyze-variables", post(handlers::template_analysis::analyze_variables))
        .route("/templates/:id/export", get(handlers::template::export_template))
        // 版本发布路由
        .route("/templates/:id/releases", get(handlers::releases::list_releases))
        .route("/templates/:id/releases", post(handlers::releases::create_release))
        .route("/templates/:id/releases/reset-to-latest", post(handlers::releases::reset_to_latest))
        .route("/templates/:id/releases/:version/rollback", post(handlers::releases::rollback_version))
        .route("/templates/:id/releases/:version/deprecate", post(handlers::releases::deprecate_version))
        // 下载版本模板
        .route("/templates/:id/releases/:version/download", get(handlers::template::download_template_version))
}

/// 编辑器路由
fn editor_routes() -> Router<AppState> {
    Router::new()
        .nest("/templateFiles", routes::admin::editor_routes())
        .nest("/templates", templates_editor_routes())
        .route("/templateFiles/render", post(handlers::template_render::render_file))
        .route("/templateFiles/renderFileTree", post(handlers::template_render::render_file_tree))
        // 文件条件管理路由（方案B：使用 templateId + filePath）
        .route(
            "/file-conditions",
            get(handlers::file_conditions::get_file_condition)
                .post(handlers::file_conditions::set_file_condition)
                .delete(handlers::file_conditions::delete_file_condition),
        )
        .route(
            "/file-conditions/evaluate",
            post(handlers::file_conditions::evaluate_file_condition),
        )
        .route(
            "/templates/:template_id/conditions/export",
            get(handlers::file_conditions::export_conditions_yaml),
        )
        .route(
            "/templates/:template_id/conditions/import",
            post(handlers::file_conditions::import_conditions_yaml),
        )
}

/// 模板编辑器路由
fn templates_editor_routes() -> Router<AppState> {
    Router::new()
        .nest("/preset-variables", preset_variables_routes())
        .nest("/:template_id", template_preset_routes())
}

/// 预设变量路由
fn preset_variables_routes() -> Router<AppState> {
    Router::new()
        .route("/available", get(handlers::var_preset::get_available_var_presets))
}

/// 模板预设变量路由
fn template_preset_routes() -> Router<AppState> {
    Router::new()
        .route("/preset-variables", get(handlers::preset_subscribe::get_preset_variables))
        .route("/preset-variables/subscribe", get(handlers::preset_subscribe::get_subscribe_list))
        .route("/preset-variables/subscribe", post(handlers::preset_subscribe::subscribe))
        .route("/preset-variables/subscribe/:preset_id", delete(handlers::preset_subscribe::unsubscribe))
        .nest("/variables", template_variables_routes())
}

/// 模板变量路由
fn template_variables_routes() -> Router<AppState> {
    Router::new()
        .route("/data", get(handlers::template_variables::get_variables))
        .route("/data", post(handlers::template_variables::save_variables))
        .route("/test", get(handlers::template_variables::get_test_data))
        .route("/test", post(handlers::template_variables::save_test_data))
}

/// Studio路由
fn studio_routes() -> Router<AppState> {
    Router::new()
        .route("/index", get(handlers::studio::studio_index))
        .route("/categories", get(handlers::category::get_all_categories))
        .route("/languages", get(handlers::language::get_all_languages))
        .route("/languages/popular", get(handlers::language::get_popular_languages))
        .route("/templates/types", get(handlers::template::get_template_types))
        .route("/template-types", get(handlers::template::get_template_types))  // 兼容前端调用
        .route("/templates/list", get(handlers::template::list_templates))
}

/// 备份路由
fn backup_routes() -> Router<AppState> {
    Router::new()
        .route("/create", get(handlers::backup::create_backup))
        .route("/preview", post(handlers::backup::preview_backup))
        .route("/restore", post(handlers::backup::restore_backup))
}

/// 健康检查
async fn health_check() -> &'static str {
    "OK"
}

/// Git初始化wrapper函数
fn git_init_wrapper(
    repo_path: &std::path::PathBuf,
    template_name: &str,
    author_name: Option<&str>,
    author_email: Option<&str>,
) -> Result<(), anyhow::Error> {
    use template_studio_infrastructure::git::service::GitService;

    let git_config = template_studio_infrastructure::config::settings::GitConfig {
        auto_init: true,
        default_branch: "main".to_string(),
    };

    let git_service = GitService::new(git_config);

    // 由于wrapper是同步的，需要在blocking task中运行async函数
    let repo_path = repo_path.clone();
    let template_name = template_name.to_string();
    let author_name = author_name.map(|s| s.to_string());
    let author_email = author_email.map(|s| s.to_string());

    // 创建runtime来执行async操作
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        git_service.init_repository(
            &repo_path,
            &template_name,
            author_name.as_deref(),
            author_email.as_deref()
        ).await
    })
}