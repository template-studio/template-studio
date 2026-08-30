mod file_watcher;
mod handlers;
mod middleware;
mod routes;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use template_studio_infrastructure::{
    config::{settings::load_config, storage::StorageManager},
    database::pool::DatabasePool,
    file_tree::FileTreeService,
    logging::init_logging,
};
use template_studio_repositories::{
    CategoryRepository, LanguageRepository, PatRepository, PermissionRepository, RoleRepository,
    SystemSettingRepository, TemplateRepository, UserRepository, VarPresetRepository,
};
use template_studio_services::{
    AuthService, BackupService, CategoryService, EmailService, FileConditionsService,
    LanguageService, PatService, PermissionService, PresetSubscribeService, ReleaseService,
    RoleService, SystemSettingService, TemplateAnalysisService, TemplateRenderService,
    TemplateService, TemplateVariablesService, UserService, VarPresetService,
};
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
            sqlx::query(
                "UPDATE templates SET visibility='public', status='active' WHERE owner_id IS NULL",
            )
            .execute(&pool)
            .await
            .ok();
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
                )",
            )
            .execute(&pool)
            .await
            .ok();
            info!("Migration 018 applied: template_reviews table created");
        }
        // 019: password_reset_tokens 表
        let reset_table_exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM information_schema.tables WHERE table_schema = DATABASE() AND table_name = 'password_reset_tokens'"
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !reset_table_exists {
            sqlx::query(
                "CREATE TABLE password_reset_tokens (
                    id BIGINT PRIMARY KEY AUTO_INCREMENT,
                    user_id BIGINT NOT NULL,
                    token VARCHAR(64) NOT NULL UNIQUE,
                    email VARCHAR(100) NOT NULL,
                    used TINYINT DEFAULT 0,
                    expires_at DATETIME NOT NULL,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    INDEX idx_token (token),
                    INDEX idx_user_id (user_id)
                )",
            )
            .execute(&pool)
            .await
            .ok();
            info!("Migration 019 applied: password_reset_tokens table created");
        }

        // 020: users 表添加 bio 字段
        let bio_exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM information_schema.columns WHERE table_schema = DATABASE() AND table_name = 'users' AND column_name = 'bio'"
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !bio_exists {
            sqlx::query("ALTER TABLE users ADD COLUMN bio VARCHAR(200) DEFAULT '' AFTER avatar")
                .execute(&pool)
                .await
                .ok();
            info!("Migration 020 applied: users.bio column added");
        }

        // 021: PAT 表添加 scopes 字段
        let scopes_exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM information_schema.columns WHERE table_schema = DATABASE() AND table_name = 'personal_access_tokens' AND column_name = 'scopes'"
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !scopes_exists {
            sqlx::query("ALTER TABLE personal_access_tokens ADD COLUMN scopes TEXT NOT NULL COMMENT '权限范围列表，JSON数组格式'")
                .execute(&pool)
                .await
                .ok();
            info!("Migration 021 applied: personal_access_tokens.scopes column added");
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
    let system_setting_repository =
        Arc::new(SystemSettingRepository::new(db_pool.get_pool().clone()));
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
    let template_analysis_service = Arc::new(TemplateAnalysisService::new(
        config.storage.base_path.join("templates"),
    ));
    let template_variables_service =
        Arc::new(TemplateVariablesService::new(storage_manager.clone()));
    let template_render_service = Arc::new(TemplateRenderService::new(
        config.storage.base_path.join("templates"),
    ));
    let file_tree_service = Arc::new(FileTreeService::new(storage_manager.clone()));
    let file_conditions_service = Arc::new(FileConditionsService::new(storage_manager.clone()));
    let release_service = Arc::new(ReleaseService::new(
        storage_manager.clone(),
        db_pool.get_pool().clone(),
    ));
    let backup_service = Arc::new(BackupService::new(
        storage_manager.clone(),
        template_service.clone(),
        template_variables_service.clone(),
        file_conditions_service.clone(),
    ));
    let system_setting_service =
        Arc::new(SystemSettingService::new(system_setting_repository.clone()));
    let jwt_config = JwtConfig::from_env();
    let auth_service = Arc::new(AuthService::new(user_repository.clone(), jwt_config));
    let user_service = Arc::new(UserService::new(user_repository.clone()));
    let role_service = Arc::new(RoleService::new(role_repository));
    let permission_service = Arc::new(PermissionService::new(permission_repository));
    let pat_service = Arc::new(PatService::new(pat_repository));

    // 创建邮件服务
    let server_url = format!("{}:{}", config.server.host, config.server.port);
    let base_url = if server_url.starts_with("0.0.0.0") || server_url.starts_with("127.0.0.1") {
        format!("http://localhost:{}", config.server.port)
    } else {
        format!("http://{}", server_url)
    };
    let email_service = Arc::new(EmailService::new(
        db_pool.get_pool().clone(),
        system_setting_repository.clone(),
        base_url,
    ));

    // 启动文件系统监听（监听 templates 目录）
    let templates_cache = template_render_service.get_cache();
    let templates_path = config.storage.base_path.join("templates");
    match file_watcher::start_file_watcher(
        templates_path,
        templates_cache,
        tokio::runtime::Handle::current(),
    ) {
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
        email_service,
        user_repository,
        template_repository,
        storage_manager,
    };

    // 创建路由
    let app = create_app(app_state, &config);

    // 启动服务器（带连接信息，供限速中间件获取客户端 IP）
    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("服务器启动成功: http://{}", bind_addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

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
    pub email_service: Arc<EmailService>,
    pub user_repository: Arc<UserRepository>,
    pub template_repository: Arc<TemplateRepository>,
    pub storage_manager: Arc<StorageManager>,
}

/// 创建应用路由
pub fn create_app(
    state: AppState,
    config: &template_studio_infrastructure::config::settings::AppConfig,
) -> Router {
    // /api/v1/admin 下混合两类路由：用户自助（仅登录）与管理功能（super_admin 角色校验）
    let admin_self = routes::admin_user_self_routes().layer(axum::middleware::from_fn_with_state(
        state.clone(),
        middleware::auth::auth_middleware,
    ));
    let admin = routes::admin_admin_only_routes().layer(axum::middleware::from_fn_with_state(
        state.clone(),
        middleware::auth::admin_auth_middleware,
    ));

    // 模板写操作与 admin 一样需通过认证中间件（读操作保持公开）
    let template_protected = template_protected_routes().layer(
        axum::middleware::from_fn_with_state(state.clone(), middleware::auth::auth_middleware),
    );

    // 编辑器（文件增删改/上传/条件管理）与备份（创建/恢复）均为登录后操作，整组认证
    let editor = editor_routes().layer(axum::middleware::from_fn_with_state(
        state.clone(),
        middleware::auth::auth_middleware,
    ));
    let backup = backup_routes().layer(axum::middleware::from_fn_with_state(
        state.clone(),
        middleware::auth::auth_middleware,
    ));

    // 认证API（公开，带限速防暴力破解）
    let auth = routes::auth::auth_routes().layer(axum::middleware::from_fn(
        middleware::rate_limit::auth_rate_limit,
    ));

    Router::new()
        // 健康检查
        .route("/health", get(health_check))
        .nest("/api/v1/auth", auth)
        // 模板API（读公开 + 写认证）
        .nest(
            "/api/v1/template",
            template_routes().merge(template_protected),
        )
        // /api/v1/admin：用户自助路由（仅登录）+ 管理路由（super_admin）
        .nest("/api/v1/admin", admin_self.merge(admin))
        // 编辑器API（受认证保护）
        .nest("/api/v1/editor", editor)
        // Studio API
        .nest("/api/v1/studio", studio_routes())
        // 备份API（受认证保护）
        .nest("/api/v1/backup", backup)
        // 公开API
        .nest("/api/v1", routes::public_routes())
        // 头像静态文件
        .nest_service(
            "/avatars",
            tower_http::services::ServeDir::new("data/avatars"),
        )
        .layer(cors_layer(&config))
        .with_state(state)
}

/// 构建 CORS 层：
/// - 配置了 `server.cors_origins` 时仅放行配置的来源
/// - 未配置时放行 localhost/127.0.0.1 任意端口的开发来源（生产部署必须显式配置）
fn cors_layer(config: &template_studio_infrastructure::config::settings::AppConfig) -> CorsLayer {
    let configured = config.server.cors_origins.clone().unwrap_or_default();
    CorsLayer::new()
        .allow_origin(tower_http::cors::AllowOrigin::predicate(
            move |origin, _| {
                let Ok(text) = origin.to_str() else {
                    return false;
                };
                if configured.iter().any(|allowed| allowed == text) {
                    return true;
                }
                text.starts_with("http://localhost:")
                    || text.starts_with("http://127.0.0.1:")
                    || text.starts_with("http://[::1]:")
            },
        ))
        // 前端使用自定义 token 头认证，预检必须放行对应方法与头
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
}

/// 模板路由
/// 模板公开路由（只读，前台未登录可访问）
fn template_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/templates/types",
            get(handlers::template::get_template_types),
        )
        .route("/templateList", get(handlers::template::list_templates))
        .route(
            "/templates/detail",
            get(handlers::template::get_template_detail),
        )
        .route(
            "/templates/:id/releases",
            get(handlers::releases::list_releases),
        )
    // 导出与版本下载已移入 template_protected_routes（认证组）：
    // 认证中间件支持 ?token= 查询参数（仅 GET），供 <a href> 直链下载
}

/// 模板写操作路由（需认证）
fn template_protected_routes() -> Router<AppState> {
    Router::new()
        .route("/templates/add", post(handlers::template::create_template))
        .route("/templates/edit", put(handlers::template::update_template))
        .route(
            "/templates/toggle-featured",
            put(handlers::template::toggle_featured),
        )
        .route(
            "/templates/del",
            delete(handlers::template::delete_template),
        )
        // fork 的 handler 依赖 AuthUser，必须在认证组内（此前挂在公开路由上导致恒 500）
        .route("/templates/fork", post(handlers::template::fork_template))
        .route(
            "/templates/:id/analyze-variables",
            post(handlers::template_analysis::analyze_variables),
        )
        // 版本发布（写操作）
        .route(
            "/templates/:id/releases",
            post(handlers::releases::create_release),
        )
        .route(
            "/templates/:id/releases/reset-to-latest",
            post(handlers::releases::reset_to_latest),
        )
        .route(
            "/templates/:id/releases/:version/rollback",
            post(handlers::releases::rollback_version),
        )
        .route(
            "/templates/:id/releases/:version/deprecate",
            post(handlers::releases::deprecate_version),
        )
        // 下载类 GET（前端 <a href> 直链 + ?token= 查询参数认证）
        .route(
            "/templates/:id/export",
            get(handlers::template::export_template),
        )
        .route(
            "/templates/:id/releases/:version/download",
            get(handlers::template::download_template_version),
        )
}

/// 编辑器路由
fn editor_routes() -> Router<AppState> {
    Router::new()
        .nest("/templateFiles", routes::admin::editor_routes())
        .nest("/templates", templates_editor_routes())
        .route(
            "/templateFiles/render",
            post(handlers::template_render::render_file),
        )
        .route(
            "/templateFiles/renderFileTree",
            post(handlers::template_render::render_file_tree),
        )
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
    Router::new().route(
        "/available",
        get(handlers::var_preset::get_available_var_presets),
    )
}

/// 模板预设变量路由
fn template_preset_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/preset-variables",
            get(handlers::preset_subscribe::get_preset_variables),
        )
        .route(
            "/preset-variables/subscribe",
            get(handlers::preset_subscribe::get_subscribe_list),
        )
        .route(
            "/preset-variables/subscribe",
            post(handlers::preset_subscribe::subscribe),
        )
        .route(
            "/preset-variables/subscribe/:preset_id",
            delete(handlers::preset_subscribe::unsubscribe),
        )
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
        .route(
            "/languages/popular",
            get(handlers::language::get_popular_languages),
        )
        .route(
            "/templates/types",
            get(handlers::template::get_template_types),
        )
        .route(
            "/template-types",
            get(handlers::template::get_template_types),
        ) // 兼容前端调用
        .route(
            "/templates/list",
            get(handlers::template::list_public_templates_studio),
        )
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
        git_service
            .init_repository(
                &repo_path,
                &template_name,
                author_name.as_deref(),
                author_email.as_deref(),
            )
            .await
    })
}
