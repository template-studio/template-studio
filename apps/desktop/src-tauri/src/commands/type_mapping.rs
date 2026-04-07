use crate::state::DbState;

// ===== 系统级类型映射命令 =====

/// 获取系统级类型映射
#[tauri::command]
pub async fn db_get_system_type_mappings(
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let mappings = db.get_system_type_mappings().await
        .map_err(|e| format!("查询系统级类型映射失败: {}", e))?;

    serde_json::to_string(&mappings)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 根据语言和数据库类型获取系统级类型映射
#[tauri::command]
pub async fn db_get_system_type_mappings_by_lang_db(
    language_id: i64,
    db_type: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let mappings = db.get_system_type_mappings_by_lang_db(language_id, &db_type).await
        .map_err(|e| format!("查询系统级类型映射失败: {}", e))?;

    serde_json::to_string(&mappings)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 创建系统级类型映射
#[tauri::command]
pub async fn db_create_system_type_mapping(
    language_id: i64,
    db_type: String,
    pattern: String,
    target_type: String,
    priority: i32,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    db.create_system_type_mapping(language_id, &db_type, &pattern, &target_type, priority)
        .await
        .map_err(|e| format!("创建系统级类型映射失败: {}", e))
}

/// 更新系统级类型映射
#[tauri::command]
pub async fn db_update_system_type_mapping(
    id: i64,
    target_type: String,
    priority: i32,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.update_system_type_mapping(id, &target_type, priority)
        .await
        .map_err(|e| format!("更新系统级类型映射失败: {}", e))?;

    Ok(())
}

/// 删除系统级类型映射
#[tauri::command]
pub async fn db_delete_system_type_mapping(
    id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.delete_system_type_mapping(id)
        .await
        .map_err(|e| format!("删除系统级类型映射失败: {}", e))?;

    Ok(())
}

/// 批量保存系统级类型映射
#[tauri::command]
pub async fn db_batch_save_system_type_mappings(
    mappings: String,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    let mappings: Vec<serde_json::Value> = serde_json::from_str(&mappings)
        .map_err(|e| format!("解析类型映射数据失败: {}", e))?;

    db.batch_save_system_type_mappings(mappings)
        .await
        .map_err(|e| format!("批量保存系统级类型映射失败: {}", e))?;

    Ok(())
}

// ===== 项目级类型映射命令 =====

/// 获取项目级类型映射
#[tauri::command]
pub async fn db_get_project_type_mappings(
    project_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let mappings = db.get_project_type_mappings(project_id).await
        .map_err(|e| format!("查询项目级类型映射失败: {}", e))?;

    serde_json::to_string(&mappings)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 根据项目和范围获取项目级类型映射
#[tauri::command]
pub async fn db_get_project_type_mappings_by_scope(
    project_id: i64,
    scope: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let mappings = db.get_project_type_mappings_by_scope(project_id, &scope).await
        .map_err(|e| format!("查询项目级类型映射失败: {}", e))?;

    serde_json::to_string(&mappings)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 创建项目级类型映射
#[tauri::command]
pub async fn db_create_project_type_mapping(
    project_id: i64,
    scope: String,
    db_type: String,
    pattern: String,
    target_type: String,
    priority: i32,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    db.create_project_type_mapping(project_id, &scope, &db_type, &pattern, &target_type, priority)
        .await
        .map_err(|e| format!("创建项目级类型映射失败: {}", e))
}

/// 更新项目级类型映射
#[tauri::command]
pub async fn db_update_project_type_mapping(
    id: i64,
    target_type: String,
    priority: i32,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.update_project_type_mapping(id, &target_type, priority)
        .await
        .map_err(|e| format!("更新项目级类型映射失败: {}", e))?;

    Ok(())
}

/// 删除项目级类型映射
#[tauri::command]
pub async fn db_delete_project_type_mapping(
    id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.delete_project_type_mapping(id)
        .await
        .map_err(|e| format!("删除项目级类型映射失败: {}", e))?;

    Ok(())
}

/// 批量保存项目级类型映射
#[tauri::command]
pub async fn db_batch_save_project_type_mappings(
    project_id: i64,
    scope: String,
    mappings: String,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    let mappings: Vec<serde_json::Value> = serde_json::from_str(&mappings)
        .map_err(|e| format!("解析类型映射数据失败: {}", e))?;

    db.batch_save_project_type_mappings(project_id, &scope, mappings)
        .await
        .map_err(|e| format!("批量保存项目级类型映射失败: {}", e))?;

    Ok(())
}

/// 复制系统级映射到项目级
#[tauri::command]
pub async fn db_copy_system_mappings_to_project(
    project_id: i64,
    language_id: i64,
    scope: String,
    db_type: String,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.copy_system_mappings_to_project(project_id, language_id, &scope, &db_type)
        .await
        .map_err(|e| format!("复制系统级映射到项目级失败: {}", e))?;

    Ok(())
}
