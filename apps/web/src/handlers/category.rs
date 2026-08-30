use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use template_studio_shared::models::category::*;
use template_studio_shared::utils::response::ApiResponse;
use validator::Validate;

pub type AppState = super::super::AppState;

/// 创建分类
pub async fn create_category(
    State(state): State<AppState>,
    Json(request): Json<CreateCategoryRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证请求数据
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.category_service.create_category(request).await {
        Ok(id) => Ok(Json(
            serde_json::to_value(ApiResponse::success_with_message(
                json!({ "id": id }),
                "创建分类成功",
            ))
            .unwrap_or_default(),
        )),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 获取分类详情
pub async fn get_category(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.category_service.get_category(id).await {
        Ok(Some(category)) => Ok(Json(
            serde_json::to_value(ApiResponse::success_with_message(category, "获取分类成功"))
                .unwrap_or_default(),
        )),
        Ok(None) => error_response(StatusCode::NOT_FOUND, "分类不存在"),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 更新分类
pub async fn update_category(
    State(state): State<AppState>,
    Json(request): Json<UpdateCategoryRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证请求数据
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.category_service.update_category(request).await {
        Ok(()) => Ok(Json(
            serde_json::to_value(ApiResponse::<()>::success_msg("更新分类成功"))
                .unwrap_or_default(),
        )),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 删除分类
pub async fn delete_category(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.category_service.delete_category(id).await {
        Ok(()) => Ok(Json(
            serde_json::to_value(ApiResponse::<()>::success_msg("删除分类成功"))
                .unwrap_or_default(),
        )),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 分类列表
pub async fn list_categories(
    State(state): State<AppState>,
    Query(query): Query<CategoryListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证请求数据
    if let Err(e) = query.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    let page = query.page.unwrap_or(1);

    match state.category_service.list_categories(query).await {
        Ok(categories) => Ok(Json(
            serde_json::to_value(ApiResponse::success_with_message(
                json!({
                    "currentPage": page,
                    "total": categories.len(),
                    "categoriesList": categories
                }),
                "OK",
            ))
            .unwrap_or_default(),
        )),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 获取所有分类
pub async fn get_all_categories(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.category_service.get_all_categories().await {
        Ok(categories) => Ok(Json(
            serde_json::to_value(ApiResponse::success_with_message(
                json!({
                    "currentPage": 1,
                    "total": categories.len(),
                    "categoriesList": categories
                }),
                "OK",
            ))
            .unwrap_or_default(),
        )),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 删除分类查询参数
#[derive(serde::Deserialize)]
pub struct DeleteCategoryQuery {
    id: i64,
}

/// 删除分类（通过查询参数）
pub async fn delete_category_by_query(
    State(state): State<AppState>,
    Query(params): Query<DeleteCategoryQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.category_service.delete_category(params.id).await {
        Ok(()) => Ok(Json(
            serde_json::to_value(ApiResponse::<()>::success_msg("删除分类成功"))
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
