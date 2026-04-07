use crate::database::{Database, DatasourceParams, TestConnectionParams};
use crate::state::DbState;

/// 获取所有数据源
#[tauri::command]
pub async fn db_get_all_datasources(
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let datasources = db.get_all_datasources().await
        .map_err(|e| format!("查询数据源失败: {}", e))?;

    serde_json::to_string(&datasources)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 创建数据源
#[tauri::command]
pub async fn db_create_datasource(
    params: DatasourceParams,
    db_state: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = db_state.as_ref();

    let id = db.create_datasource(
        &params.name,
        &params.type_,
        params.host.as_deref(),
        params.port,
        params.username.as_deref(),
        params.password.as_deref(),
        params.database.as_deref(),
        params.sqlite_file.as_deref(),
    ).await.map_err(|e| format!("创建数据源失败: {}", e))?;

    Ok(id)
}

/// 根据 ID 获取数据源
#[tauri::command]
pub async fn db_get_datasource(
    id: i64,
    db_state: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = db_state.as_ref();

    let datasource = db.get_datasource(id).await
        .map_err(|e| format!("查询数据源失败: {}", e))?
        .ok_or_else(|| "数据源不存在".to_string())?;

    serde_json::to_string(&datasource)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 更新数据源
#[tauri::command]
pub async fn db_update_datasource(
    id: i64,
    params: DatasourceParams,
    db_state: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = db_state.as_ref();

    db.update_datasource(
        id,
        &params.name,
        &params.type_,
        params.host.as_deref(),
        params.port,
        params.username.as_deref(),
        params.password.as_deref(),
        params.database.as_deref(),
        params.sqlite_file.as_deref(),
    ).await.map_err(|e| format!("更新数据源失败: {}", e))?;

    Ok(())
}

/// 删除数据源
#[tauri::command]
pub async fn db_delete_datasource(
    id: i64,
    db_state: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = db_state.as_ref();

    db.delete_datasource(id).await
        .map_err(|e| format!("删除数据源失败: {}", e))?;

    Ok(())
}

/// 测试数据源连接
#[tauri::command]
pub async fn test_datasource_connection(params: TestConnectionParams) -> Result<String, String> {
    Database::test_datasource_connection(params).await
}
