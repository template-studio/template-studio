#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod commands;
mod config;
pub mod database;
mod ddl;
mod state;

use database::Database;
use state::{BrowserPoolCache, DbState};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 使用 block_in_place 来在 setup 中等待异步数据库初始化
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                println!("初始化数据库...");
                match Database::init().await {
                    Ok(database) => {
                        println!("数据库初始化完成");
                        // 将数据库存储为应用状态
                        let db_state = DbState::new(database);
                        handle.manage(db_state);
                        handle.manage(BrowserPoolCache::new());
                    }
                    Err(e) => {
                        eprintln!("数据库初始化失败: {}", e);
                        panic!("数据库初始化失败");
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 窗口命令
            commands::window::greet,
            commands::window::write_text_file,
            commands::window::window_minimize,
            commands::window::window_maximize,
            commands::window::window_close,
            commands::window::get_username,
            commands::window::get_system_theme,
            // 模板命令
            commands::template::list_templates,
            commands::template::get_template_variables,
            commands::template::render_template,
            commands::template::render_template_preview,
            commands::template::render_files,
            commands::template::render_string_content,
            commands::template::get_render_engine_info,
            commands::template::cmd_render_and_export,
            commands::template::generate_project,
            commands::template::check_template_downloaded,
            commands::template::download_template,
            commands::template::check_directory_exists,
            commands::template::remove_directory,
            // 设置命令
            commands::settings::get_config,
            commands::settings::update_web_server_config,
            commands::settings::update_template_path,
            // 项目命令
            commands::project::db_get_statistics,
            commands::project::db_get_recent_projects,
            commands::project::db_get_all_projects,
            commands::project::db_get_project,
            commands::project::db_create_project,
            commands::project::db_update_project,
            commands::project::db_delete_project,
            // 数据源命令
            commands::datasource::db_get_all_datasources,
            commands::datasource::db_create_datasource,
            commands::datasource::db_get_datasource,
            commands::datasource::db_update_datasource,
            commands::datasource::db_delete_datasource,
            commands::datasource::test_datasource_connection,
            // 远程数据库同步命令
            commands::sync::cmd_list_database_tables,
            commands::sync::cmd_get_table_columns,
            commands::sync::cmd_query_table_data,
            commands::sync::cmd_get_connection_status,
            commands::sync::cmd_fetch_mysql_tables,
            commands::sync::cmd_fetch_postgresql_tables,
            commands::sync::cmd_fetch_sqlite_tables,
            commands::sync::cmd_import_single_table,
            commands::sync::cmd_execute_sql_on_remote,
            commands::sync::cmd_push_table_to_remote,
            // 表/列命令
            commands::table::db_get_project_tables,
            commands::table::db_create_table,
            commands::table::cmd_import_tables_from_datasource,
            commands::table::db_get_table_columns,
            commands::table::db_delete_table,
            commands::table::db_update_table,
            commands::table::db_create_column,
            commands::table::db_update_column,
            commands::table::db_delete_column,
            commands::table::db_reorder_columns,
            commands::table::cmd_parse_sql_and_create,
            commands::table::db_get_table_preferences,
            commands::table::db_save_table_preferences,
            // 语言命令
            commands::language::db_get_all_languages,
            commands::language::db_get_language,
            commands::language::db_create_language,
            commands::language::db_update_language,
            commands::language::db_delete_language,
            commands::language::db_set_project_primary_language,
            commands::language::db_get_project_languages,
            commands::language::db_add_project_language,
            commands::language::db_remove_project_language,
            commands::language::db_get_language_field_types,
            commands::language::db_create_language_field_type,
            commands::language::db_update_language_field_type,
            commands::language::db_delete_language_field_type,
            commands::language::db_batch_save_language_field_types,
            // 类型映射命令
            commands::type_mapping::db_get_system_type_mappings,
            commands::type_mapping::db_get_system_type_mappings_by_lang_db,
            commands::type_mapping::db_create_system_type_mapping,
            commands::type_mapping::db_update_system_type_mapping,
            commands::type_mapping::db_delete_system_type_mapping,
            commands::type_mapping::db_batch_save_system_type_mappings,
            commands::type_mapping::db_get_project_type_mappings,
            commands::type_mapping::db_get_project_type_mappings_by_scope,
            commands::type_mapping::db_create_project_type_mapping,
            commands::type_mapping::db_update_project_type_mapping,
            commands::type_mapping::db_delete_project_type_mapping,
            commands::type_mapping::db_batch_save_project_type_mappings,
            commands::type_mapping::db_copy_system_mappings_to_project,
            // AI 服务命令
            commands::ai::ai_get_all_providers,
            commands::ai::ai_get_provider,
            commands::ai::ai_save_provider,
            commands::ai::ai_toggle_provider,
            commands::ai::ai_delete_provider,
            commands::ai::ai_get_provider_models_grouped,
            commands::ai::ai_add_model,
            commands::ai::ai_delete_model,
            commands::ai::ai_update_model,
            commands::ai::ai_fetch_models,
            commands::ai::ai_batch_add_models,
            commands::ai::ai_generate_sql,
            commands::ai::ai_fix_sql,
            commands::ai::ai_test_connection,
            commands::ai::parse_ai_sql,
            commands::ai::execute_ai_sql,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
