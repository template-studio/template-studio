use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use template_studio_shared::models::file_tree::FileTreeQuery;
use template_studio_infrastructure::git::service::GitService;
use template_studio_infrastructure::config::settings::GitConfig;

pub type AppState = super::super::AppState;

/// 还原文件请求
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreFileRequest {
    pub template_id: i64,
    pub file_path: String,
}

/// 获取模板文件树（带条件标记）
pub async fn get_file_tree(
    State(state): State<AppState>,
    Query(query): Query<FileTreeQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 获取文件树
    let mut tree_response = match state.file_tree_service.get_template_file_tree(query.template_id).await {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("获取文件树失败: template_id={}, error={}", query.template_id, e);
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
        }
    };

    // 获取条件摘要
    let conditions_summary = match state.file_conditions_service
        .get_conditions_summary(query.template_id)
        .await
    {
        Ok(summary) => summary,
        Err(e) => {
            tracing::warn!("获取条件摘要失败: template_id={}, error={}", query.template_id, e);
            HashMap::new()
        }
    };

    // 为文件树添加条件标记（在结构体层面）
    enrich_tree_with_conditions(&mut tree_response.tree, &conditions_summary);

    Ok(Json(json!({
        "code": 0,
        "message": "OK",
        "data": tree_response
    })))
}

/// 为文件树节点添加条件信息
fn enrich_tree_with_conditions(nodes: &mut [template_studio_shared::models::file_tree::FileTreeNode], conditions: &HashMap<String, String>) {
    for node in nodes.iter_mut() {
        // 检查是否有条件
        if let Some(summary) = conditions.get(&node.file_path) {
            node.has_condition = true;
            node.condition_summary = Some(summary.clone());
        }

        // 递归处理子节点
        if let Some(ref mut children) = node.children {
            enrich_tree_with_conditions(children, conditions);
        }
    }
}

/// 错误响应
fn error_response(status: StatusCode, message: &str) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Err((status, Json(json!({
        "code": status.as_u16() as i32,
        "message": message
    }))))
}

/// 还原文件到上次提交状态（git restore）
/// POST /api/v1/editor/templateFiles/restore
pub async fn restore_file(
    State(state): State<AppState>,
    Json(payload): Json<RestoreFileRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let template_id = payload.template_id;
    let file_path = payload.file_path;

    tracing::info!("还原文件: template_id={}, file_path={}", template_id, file_path);

    // 获取模板路径
    let template_path = state.storage_manager.get_template_path(template_id);

    // 创建 GitService 实例
    let git_config = GitConfig {
        auto_init: true,
        default_branch: "main".to_string(),
    };
    let git_service = GitService::new(git_config);

    // 执行 git restore
    match git_service.restore_file(&template_path, &file_path).await {
        Ok(_) => {
            tracing::info!("文件还原成功: {}", file_path);
            Ok(Json(json!({
                "code": 0,
                "message": "文件已还原到上次提交状态"
            })))
        }
        Err(e) => {
            tracing::error!("文件还原失败: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("还原失败: {}", e))
        }
    }
}
