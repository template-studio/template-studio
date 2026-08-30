use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use template_studio_shared::models::var_preset::{
    AvailableVarPresetQuery, CreateVarPresetRequest, ToggleVarPresetRequest,
    UpdateVarPresetRequest, VarPresetDetailQuery, VarPresetDetailResponse, VarPresetListQuery,
    VarPresetResponse,
};
use template_studio_shared::utils::response::ApiResponse;
use validator::Validate;

pub type AppState = super::super::AppState;

/// 创建变量预设
pub async fn create_var_preset(
    State(state): State<AppState>,
    Json(request): Json<CreateVarPresetRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证请求数据
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.var_preset_service.create_var_preset(request).await {
        Ok(id) => Ok(Json(
            serde_json::to_value(ApiResponse::success_with_message(
                json!({ "id": id }),
                "创建变量预设成功",
            ))
            .unwrap_or_default(),
        )),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 获取变量预设详情
pub async fn get_var_preset(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.var_preset_service.get_var_preset(id).await {
        Ok(Some(var_preset)) => {
            let detail_response = VarPresetDetailResponse {
                var_preset: var_preset.into(),
            };
            Ok(Json(
                serde_json::to_value(ApiResponse::success_with_message(detail_response, "OK"))
                    .unwrap_or_default(),
            ))
        }
        Ok(None) => error_response(StatusCode::NOT_FOUND, "变量预设不存在"),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 通过查询参数获取变量预设详情
pub async fn get_var_preset_by_query(
    State(state): State<AppState>,
    Query(query): Query<VarPresetDetailQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证请求数据
    if let Err(e) = query.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.var_preset_service.get_var_preset(query.id).await {
        Ok(Some(var_preset)) => {
            let detail_response = VarPresetDetailResponse {
                var_preset: var_preset.into(),
            };
            Ok(Json(
                serde_json::to_value(ApiResponse::success_with_message(detail_response, "OK"))
                    .unwrap_or_default(),
            ))
        }
        Ok(None) => error_response(StatusCode::NOT_FOUND, "变量预设不存在"),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 更新变量预设
pub async fn update_var_preset(
    State(state): State<AppState>,
    Json(request): Json<UpdateVarPresetRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证请求数据
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.var_preset_service.update_var_preset(request).await {
        Ok(()) => Ok(Json(
            serde_json::to_value(ApiResponse::<()>::success_msg("更新变量预设成功"))
                .unwrap_or_default(),
        )),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 删除变量预设
pub async fn delete_var_preset(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.var_preset_service.delete_var_preset(id).await {
        Ok(()) => Ok(Json(
            serde_json::to_value(ApiResponse::<()>::success_msg("删除变量预设成功"))
                .unwrap_or_default(),
        )),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 切换变量预设启用/禁用状态
pub async fn toggle_var_preset(
    State(state): State<AppState>,
    Json(request): Json<ToggleVarPresetRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证请求数据
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.var_preset_service.toggle_var_preset(request).await {
        Ok(()) => Ok(Json(
            serde_json::to_value(ApiResponse::<()>::success_msg("切换变量预设状态成功"))
                .unwrap_or_default(),
        )),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 变量预设列表
pub async fn list_var_presets(
    State(state): State<AppState>,
    Query(query): Query<VarPresetListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证请求数据
    if let Err(e) = query.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    let _page = query.page.unwrap_or(1);

    match state.var_preset_service.list_var_presets(query).await {
        Ok(var_presets) => {
            let response: Vec<VarPresetResponse> = var_presets
                .into_iter()
                .map(VarPresetResponse::from)
                .collect();
            Ok(Json(
                serde_json::to_value(ApiResponse::success_with_message(
                    json!({
                        "total": response.len(),
                        "varPresetsList": response
                    }),
                    "OK",
                ))
                .unwrap_or_default(),
            ))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 获取所有变量预设
pub async fn get_all_var_presets(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.var_preset_service.get_all_var_presets().await {
        Ok(var_presets) => {
            let response: Vec<VarPresetResponse> = var_presets
                .into_iter()
                .map(VarPresetResponse::from)
                .collect();
            Ok(Json(
                serde_json::to_value(ApiResponse::success_with_message(
                    json!({
                        "total": response.len(),
                        "varPresetsList": response
                    }),
                    "OK",
                ))
                .unwrap_or_default(),
            ))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 获取启用的变量预设
pub async fn get_enabled_var_presets(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.var_preset_service.get_enabled_var_presets().await {
        Ok(var_presets) => {
            let response: Vec<VarPresetResponse> = var_presets
                .into_iter()
                .map(VarPresetResponse::from)
                .collect();
            Ok(Json(
                serde_json::to_value(ApiResponse::success_with_message(
                    json!({
                        "total": response.len(),
                        "varPresetsList": response
                    }),
                    "OK",
                ))
                .unwrap_or_default(),
            ))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 根据分类获取变量预设
pub async fn get_var_presets_by_category(
    State(state): State<AppState>,
    Path(category): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state
        .var_preset_service
        .get_var_presets_by_category(&category)
        .await
    {
        Ok(var_presets) => {
            let response: Vec<VarPresetResponse> = var_presets
                .into_iter()
                .map(VarPresetResponse::from)
                .collect();
            Ok(Json(
                serde_json::to_value(ApiResponse::success_with_message(
                    json!({
                        "total": response.len(),
                        "varPresetsList": response
                    }),
                    "OK",
                ))
                .unwrap_or_default(),
            ))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 获取可用的预设变量列表（用于编辑器）
pub async fn get_available_var_presets(
    State(state): State<AppState>,
    Query(query): Query<AvailableVarPresetQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证请求数据
    if let Err(e) = query.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state
        .var_preset_service
        .get_available_var_presets(query)
        .await
    {
        Ok(response) => Ok(Json(
            serde_json::to_value(ApiResponse::success_with_message(response, "OK"))
                .unwrap_or_default(),
        )),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 错误响应
fn error_response(
    status: StatusCode,
    message: &str,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Err((
        status,
        Json(json!({
            "code": status.as_u16() as i32,
            "message": message
        })),
    ))
}
