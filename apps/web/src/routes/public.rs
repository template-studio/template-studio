use super::super::AppState;
use crate::handlers::template_files::{
    clear_cache, generate_file_tree, generate_template_file, generate_zip, get_template_variables,
    preview_file_tree, preview_template_file,
};
use crate::handlers::{
    builtin::get_builtin_functions,
    category::get_all_categories,
    engine::{check_engine_update, download_engine, get_engine_info},
    language::{get_all_languages, get_popular_languages},
    system_setting::get_public_settings,
    template::list_public_templates,
    var_preset::{get_enabled_var_presets, get_var_presets_by_category},
};
use axum::{
    routing::{get, post},
    Router,
};

/// 公开路由
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .nest("/categories", category_public_routes())
        .nest("/languages", language_public_routes())
        .nest("/var-presets", var_preset_public_routes())
        .nest("/builtin-functions", builtin_functions_routes())
        // 模板渲染/打包/缓存清理为重型操作，匿名可调，加限速防 DoS
        .nest(
            "/template-files",
            template_files_routes().layer(axum::middleware::from_fn(
                crate::middleware::rate_limit::heavy_rate_limit,
            )),
        )
        .nest("/templates", templates_public_routes())
        .nest("/engine", engine_routes())
        .route("/settings/:group", get(get_public_settings))
}

/// 公开模板路由
fn templates_public_routes() -> Router<AppState> {
    Router::new().route("/list", get(list_public_templates))
}

/// 内置函数公开路由
fn builtin_functions_routes() -> Router<AppState> {
    Router::new().route("/", get(get_builtin_functions))
}

/// 分类公开路由
fn category_public_routes() -> Router<AppState> {
    Router::new().route("/", get(get_all_categories))
}

/// 编程语言公开路由
fn language_public_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_languages))
        .route("/popular", get(get_popular_languages))
}

/// 变量预设公开路由
fn var_preset_public_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_enabled_var_presets))
        .route("/category/:category", get(get_var_presets_by_category))
}

/// 模板文件预览和生成路由
fn template_files_routes() -> Router<AppState> {
    Router::new()
        .route("/preview", post(preview_template_file))
        .route("/preview-tree", post(preview_file_tree))
        .route("/generate", post(generate_template_file))
        .route("/generate-tree", post(generate_file_tree))
        .route("/generate-zip", post(generate_zip))
        .route("/variables", get(get_template_variables))
        .route("/clear-cache", post(clear_cache))
}

/// WASM 引擎管理路由
fn engine_routes() -> Router<AppState> {
    Router::new()
        .route("/info", get(get_engine_info))
        .route("/download", get(download_engine))
        .route("/check-update", get(check_engine_update))
}
