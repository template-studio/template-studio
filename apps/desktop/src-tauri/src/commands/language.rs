use crate::state::DbState;

// ===== 语言相关命令 =====

/// 获取所有语言
#[tauri::command]
pub async fn db_get_all_languages(
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let languages = db.get_all_languages().await
        .map_err(|e| format!("查询语言失败: {}", e))?;

    serde_json::to_string(&languages)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 根据 ID 获取语言
#[tauri::command]
pub async fn db_get_language(
    id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let language = db.get_language(id).await
        .map_err(|e| format!("查询语言失败: {}", e))?
        .ok_or_else(|| "语言不存在".to_string())?;

    serde_json::to_string(&language)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 创建语言
#[tauri::command]
pub async fn db_create_language(
    params: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    let name = params.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少语言名称".to_string())?;

    let icon = params.get("icon").and_then(|v| v.as_str());
    let color = params.get("color").and_then(|v| v.as_str());
    let description = params.get("description").and_then(|v| v.as_str());

    let id = db.create_language(name, icon, color, description)
        .await
        .map_err(|e| format!("创建语言失败: {}", e))?;

    Ok(id)
}

/// 更新语言
#[tauri::command]
pub async fn db_update_language(
    id: i64,
    params: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    let name = params.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少语言名称".to_string())?;

    let icon = params.get("icon").and_then(|v| v.as_str());
    let color = params.get("color").and_then(|v| v.as_str());
    let description = params.get("description").and_then(|v| v.as_str());

    db.update_language(id, name, icon, color, description)
        .await
        .map_err(|e| format!("更新语言失败: {}", e))?;

    Ok(())
}

/// 删除语言
#[tauri::command]
pub async fn db_delete_language(
    id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.delete_language(id)
        .await
        .map_err(|e| format!("删除语言失败: {}", e))?;

    Ok(())
}

/// 设置项目的主语言
#[tauri::command]
pub async fn db_set_project_primary_language(
    project_id: i64,
    language_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.set_project_primary_language(project_id, language_id)
        .await
        .map_err(|e| format!("设置主语言失败: {}", e))?;

    Ok(())
}

/// 获取项目的所有语言
#[tauri::command]
pub async fn db_get_project_languages(
    project_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let languages = db.get_project_languages(project_id).await
        .map_err(|e| format!("查询项目语言失败: {}", e))?;

    serde_json::to_string(&languages)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 为项目添加语言
#[tauri::command]
pub async fn db_add_project_language(
    project_id: i64,
    language_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.add_project_language(project_id, language_id, false)
        .await
        .map_err(|e| format!("添加语言失败: {}", e))?;

    Ok(())
}

/// 移除项目的语言
#[tauri::command]
pub async fn db_remove_project_language(
    project_id: i64,
    language_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.remove_project_language(project_id, language_id)
        .await
        .map_err(|e| format!("移除语言失败: {}", e))?;

    Ok(())
}

// ===== 语言类型字段命令 =====

/// 获取语言的所有类型字段
#[tauri::command]
pub async fn db_get_language_field_types(
    language_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let field_types = db.get_language_field_types(language_id)
        .await
        .map_err(|e| format!("查询类型字段失败: {}", e))?;

    serde_json::to_string(&field_types)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 创建语言类型字段
#[tauri::command]
pub async fn db_create_language_field_type(
    language_id: i64,
    name: String,
    description: Option<String>,
    sort_order: i32,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    db.create_language_field_type(language_id, &name, description.as_deref(), sort_order)
        .await
        .map_err(|e| format!("创建类型字段失败: {}", e))
}

/// 更新语言类型字段
#[tauri::command]
pub async fn db_update_language_field_type(
    id: i64,
    name: String,
    description: Option<String>,
    sort_order: i32,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.update_language_field_type(id, &name, description.as_deref(), sort_order)
        .await
        .map_err(|e| format!("更新类型字段失败: {}", e))?;

    Ok(())
}

/// 删除语言类型字段
#[tauri::command]
pub async fn db_delete_language_field_type(
    id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.delete_language_field_type(id)
        .await
        .map_err(|e| format!("删除类型字段失败: {}", e))?;

    Ok(())
}

/// 批量保存语言类型字段
#[tauri::command]
pub async fn db_batch_save_language_field_types(
    language_id: i64,
    field_types: String,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    let field_types: Vec<serde_json::Value> = serde_json::from_str(&field_types)
        .map_err(|e| format!("解析类型字段数据失败: {}", e))?;

    db.batch_save_language_field_types(language_id, field_types)
        .await
        .map_err(|e| format!("批量保存类型字段失败: {}", e))?;

    Ok(())
}
