use crate::state::DbState;

/// 获取统计数据
#[tauri::command]
pub async fn db_get_statistics(
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let stats = db.get_statistics().await
        .map_err(|e| format!("获取统计数据失败: {}", e))?;

    serde_json::to_string(&stats)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 获取最近项目列表
#[tauri::command]
pub async fn db_get_recent_projects(
    limit: Option<i64>,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let projects = db.get_recent_projects(limit.unwrap_or(5)).await
        .map_err(|e| format!("获取最近项目失败: {}", e))?;

    serde_json::to_string(&projects)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 获取所有项目
#[tauri::command]
pub async fn db_get_all_projects(
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let projects = db.get_all_projects().await
        .map_err(|e| format!("查询项目失败: {}", e))?;

    serde_json::to_string(&projects)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 根据 ID 获取项目
#[tauri::command]
pub async fn db_get_project(
    id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let project = db.get_project(id).await
        .map_err(|e| format!("查询项目失败: {}", e))?;

    match project {
        Some(p) => serde_json::to_string(&p)
            .map_err(|e| format!("序列化失败: {}", e)),
        None => Err("项目不存在".to_string()),
    }
}

/// 创建项目
#[tauri::command]
pub async fn db_create_project(
    params: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    let name = params.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少项目名称".to_string())?;

    let description = params.get("description")
        .and_then(|v| v.as_str());

    let datasource_id = params.get("datasourceId")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| "缺少数据源ID".to_string())?;

    let database_name = params.get("databaseName")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少数据库名称".to_string())?;

    let primary_language_id = params.get("primaryLanguageId")
        .and_then(|v| v.as_i64());

    let frontend_language_id = params.get("frontendLanguageId")
        .and_then(|v| v.as_i64());

    let backend_language_id = params.get("backendLanguageId")
        .and_then(|v| v.as_i64());

    let id = db.create_project(
        name,
        description,
        datasource_id,
        database_name,
        primary_language_id,
        frontend_language_id,
        backend_language_id,
    ).await.map_err(|e| format!("创建项目失败: {}", e))?;

    Ok(id)
}

/// 更新项目
#[tauri::command]
pub async fn db_update_project(
    id: i64,
    params: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    let name = params.get("name")
        .and_then(|v| v.as_str());

    let description = params.get("description")
        .and_then(|v| v.as_str());

    let primary_language_id = params.get("primaryLanguageId")
        .and_then(|v| v.as_i64());

    let frontend_language_id = params.get("frontendLanguageId")
        .and_then(|v| v.as_i64());

    let backend_language_id = params.get("backendLanguageId")
        .and_then(|v| v.as_i64());

    db.update_project(
        id,
        name,
        description,
        primary_language_id,
        frontend_language_id,
        backend_language_id,
    ).await.map_err(|e| format!("更新项目失败: {}", e))?;

    Ok(())
}

/// 删除项目
#[tauri::command]
pub async fn db_delete_project(
    id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.delete_project(id).await
        .map_err(|e| format!("删除项目失败: {}", e))?;

    Ok(())
}
