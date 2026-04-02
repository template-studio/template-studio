use axum::{
    routing::{get, post, put, delete},
    Router,
};
use crate::handlers::{
    category::{create_category, get_category, update_category, delete_category, delete_category_by_query, list_categories},
    language::{list_languages, get_all_languages, get_popular_languages, update_language},
    template::{get_template, list_templates, get_template_file_content, add_template_file, delete_template_file, edit_template_file, move_template_file, upload_code, upload_zip},
    var_preset::{create_var_preset, get_var_preset, get_var_preset_by_query, update_var_preset, delete_var_preset, toggle_var_preset, list_var_presets, get_all_var_presets},
    editor::{get_file_tree, restore_file},
    statistics::{get_overview, get_category_distribution, get_language_popularity, get_template_complexity, get_usage_trends},
    system_setting::{get_settings, update_setting, batch_update_settings},
    user_management,
    role_management,
    permission_management,
};
use super::super::AppState;

/// 管理员路由（不含 middleware，由 create_app 添加认证层）
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .nest("/categories", category_admin_routes())
        .nest("/languages", language_admin_routes())
        .nest("/templates", template_admin_routes())
        .nest("/var-preset", var_preset_admin_routes())
        .nest("/statistics", statistics_routes())
        .nest("/settings", settings_admin_routes())
        .nest("/users", users_admin_routes())
        .nest("/roles", roles_admin_routes())
        .nest("/permissions", permissions_admin_routes())
}

/// 分类管理路由
fn category_admin_routes() -> Router<AppState> {
    Router::new()
        .route("/add", post(create_category))
        .route("/:id", get(get_category))
        .route("/edit", put(update_category))
        .route("/del/:id", delete(delete_category))
        .route("/del", delete(delete_category_by_query))
        .route("/list", get(list_categories))
}

/// 编程语言管理路由
fn language_admin_routes() -> Router<AppState> {
    Router::new()
        .route("/list", get(list_languages))
        .route("/all", get(get_all_languages))
        .route("/popular", get(get_popular_languages))
        .route("/edit", put(update_language))
}

/// 模板管理路由
fn template_admin_routes() -> Router<AppState> {
    Router::new()
        .route("/:id", get(get_template))
        .route("/list", get(list_templates))
}

/// 变量预设管理路由
fn var_preset_admin_routes() -> Router<AppState> {
    Router::new()
        .route("/add", post(create_var_preset))
        .route("/:id", get(get_var_preset))
        .route("/detail", get(get_var_preset_by_query))
        .route("/edit", put(update_var_preset))
        .route("/del/:id", delete(delete_var_preset))
        .route("/toggle", put(toggle_var_preset))
        .route("/list", get(list_var_presets))
        .route("/all", get(get_all_var_presets))
}

/// 统计路由
fn statistics_routes() -> Router<AppState> {
    Router::new()
        .route("/overview", get(get_overview))
        .route("/category-distribution", get(get_category_distribution))
        .route("/language-popularity", get(get_language_popularity))
        .route("/template-complexity", get(get_template_complexity))
        .route("/usage-trends", get(get_usage_trends))
}

/// 系统设置管理路由
fn settings_admin_routes() -> Router<AppState> {
    Router::new()
        .route("/list", get(get_settings))
        .route("/edit", put(update_setting))
        .route("/batch-edit", post(batch_update_settings))
}

/// 用户管理路由
fn users_admin_routes() -> Router<AppState> {
    Router::new()
        .route("/list", get(user_management::list_users))
        .route("/add", post(user_management::create_user))
        .route("/edit", put(user_management::update_user))
        .route("/del/:id", delete(user_management::delete_user))
        .route("/:id/roles", put(user_management::assign_roles))
}

/// 角色管理路由
fn roles_admin_routes() -> Router<AppState> {
    Router::new()
        .route("/list", get(role_management::list_roles))
        .route("/add", post(role_management::create_role))
        .route("/edit", put(role_management::update_role))
        .route("/del/:id", delete(role_management::delete_role))
        .route("/:id/permissions", get(role_management::get_role_permissions).put(role_management::assign_permissions))
}

/// 权限管理路由
fn permissions_admin_routes() -> Router<AppState> {
    Router::new()
        .route("/list", get(permission_management::list_permissions))
        .route("/tree", get(permission_management::get_permission_tree))
}

/// 编辑器路由
pub fn editor_routes() -> Router<AppState> {
    Router::new()
        .route("/fileTree", get(get_file_tree))
        .route("/content", get(get_template_file_content))
        .route("/add", post(add_template_file))
        .route("/del", delete(delete_template_file))
        .route("/edit", put(edit_template_file))
        .route("/rename", put(move_template_file))
        .route("/move", put(move_template_file))
        .route("/uploadCode", post(upload_code))
        .route("/uploadZip", post(upload_zip))
        .route("/restore", post(restore_file))
}
