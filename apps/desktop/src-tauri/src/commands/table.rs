use crate::database;
use crate::database::import::import_tables_from_datasource;
use crate::state::DbState;

/// 获取项目的所有表
#[tauri::command]
pub async fn db_get_project_tables(
    project_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let tables = db.get_project_tables(project_id).await
        .map_err(|e| format!("查询表失败: {}", e))?;

    serde_json::to_string(&tables)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 为项目创建表
#[tauri::command]
pub async fn db_create_table(
    project_id: i64,
    name: String,
    comment: Option<String>,
    engine: Option<String>,
    table_type: String,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    let id = db.create_table(
        project_id,
        &name,
        comment.as_deref(),
        engine.as_deref(),
        &table_type,
    ).await.map_err(|e| format!("创建表失败: {}", e))?;

    Ok(id)
}

/// 从数据源导入表结构
#[tauri::command]
pub async fn cmd_import_tables_from_datasource(
    project_id: i64,
    datasource_id: i64,
    database_name: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    import_tables_from_datasource(
        db.pool(),
        project_id,
        datasource_id,
        &database_name,
    ).await
}

/// 获取表的所有列
#[tauri::command]
pub async fn db_get_table_columns(
    table_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let columns = db.get_table_columns(table_id).await
        .map_err(|e| format!("查询列失败: {}", e))?;

    serde_json::to_string(&columns)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 删除表
#[tauri::command]
pub async fn db_delete_table(
    table_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.delete_table(table_id).await
        .map_err(|e| format!("删除表失败: {}", e))?;

    Ok(())
}

/// 更新表信息
#[tauri::command]
pub async fn db_update_table(
    table_id: i64,
    name: String,
    comment: Option<String>,
    engine: Option<String>,
    table_type: String,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.update_table(
        table_id,
        &name,
        comment.as_deref(),
        engine.as_deref(),
        &table_type,
    ).await
    .map_err(|e| format!("更新表失败: {}", e))?;

    Ok(())
}

/// 创建列
#[tauri::command]
pub async fn db_create_column(
    table_id: i64,
    name: String,
    data_type: String,
    length: Option<i64>,
    is_nullable: bool,
    is_primary_key: bool,
    is_unique: bool,
    default_value: Option<String>,
    comment: Option<String>,
    ordinal_position: i32,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    let column_id = db.create_column(
        table_id,
        &name,
        &data_type,
        length,
        is_nullable,
        is_primary_key,
        is_unique,
        default_value.as_deref(),
        comment.as_deref(),
        ordinal_position,
    ).await
    .map_err(|e| format!("创建列失败: {}", e))?;

    Ok(column_id)
}

/// 更新列信息
#[tauri::command]
pub async fn db_update_column(
    column_id: i64,
    name: String,
    data_type: String,
    length: Option<i64>,
    is_nullable: bool,
    is_primary_key: bool,
    is_unique: bool,
    default_value: Option<String>,
    comment: Option<String>,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.update_column(
        column_id,
        &name,
        &data_type,
        length,
        is_nullable,
        is_primary_key,
        is_unique,
        default_value.as_deref(),
        comment.as_deref(),
    ).await
    .map_err(|e| format!("更新列失败: {}", e))?;

    Ok(())
}

/// 删除列
#[tauri::command]
pub async fn db_delete_column(
    column_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.delete_column(column_id).await
        .map_err(|e| format!("删除列失败: {}", e))?;

    Ok(())
}

/// 重新排序列
#[tauri::command]
pub async fn db_reorder_columns(
    _table_id: i64,
    column_ids: Vec<i64>,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    for (index, column_id) in column_ids.iter().enumerate() {
        db.update_column_position(*column_id, (index + 1) as i32).await
            .map_err(|e| format!("更新列位置失败: {}", e))?;
    }

    Ok(())
}

/// 解析SQL并创建表和字段
#[tauri::command]
pub async fn cmd_parse_sql_and_create(
    project_id: i64,
    sql_content: String,
    sql_dialect: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    database::import::parse_and_create_from_sql(
        db.pool(),
        project_id,
        &sql_content,
        &sql_dialect,
    ).await
}

/// 获取项目表规范配置
#[tauri::command]
pub async fn db_get_table_preferences(
    project_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    match db.get_table_preferences(project_id).await {
        Ok(Some(value)) => {
            // 直接返回 JSON 字符串
            serde_json::to_string(&value).map_err(|e| format!("序列化失败: {}", e))
        }
        Ok(None) => {
            // 返回空配置
            Ok(serde_json::to_string(&serde_json::json!({
                "pkEnabled": true,
                "pkFieldName": "id",
                "pkFieldType": "BIGINT",
                "pkAutoIncrement": true,
                "pkComment": "主键",
                "auditEnabled": true,
                "auditFields": "[]",
                "softDeleteEnabled": false,
                "booleanPrefix": "is_",
                "datetimeSuffix": "_at"
            })).unwrap())
        }
        Err(e) => Err(format!("获取表规范失败: {}", e))
    }
}

/// 保存项目表规范配置
#[tauri::command]
pub async fn db_save_table_preferences(
    project_id: i64,
    preferences: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    db.save_table_preferences(project_id, preferences).await
        .map_err(|e| format!("保存表规范失败: {}", e))
}
