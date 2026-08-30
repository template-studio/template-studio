use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};
use template_studio_shared::models::language::{
    LanguageListQuery, LanguageResponse, UpdateLanguageRequest,
};
use template_studio_shared::utils::response::ApiResponse;
use validator::Validate;

pub type AppState = super::super::AppState;

/// 获取语言列表
pub async fn list_languages(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let query = LanguageListQuery::default();
    match state.language_service.list_languages(query).await {
        Ok(languages) => {
            let response: Vec<LanguageResponse> =
                languages.into_iter().map(LanguageResponse::from).collect();
            Ok(Json(
                serde_json::to_value(ApiResponse::success_with_message(
                    json!({
                        "currentPage": 1,
                        "total": response.len(),
                        "languagesList": response
                    }),
                    "OK",
                ))
                .unwrap_or_default(),
            ))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 获取所有语言
pub async fn get_all_languages(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.language_service.get_all_languages().await {
        Ok(languages) => {
            let response: Vec<LanguageResponse> =
                languages.into_iter().map(LanguageResponse::from).collect();
            Ok(Json(
                serde_json::to_value(ApiResponse::success_with_message(
                    json!({
                        "currentPage": 1,
                        "total": response.len(),
                        "languagesList": response
                    }),
                    "OK",
                ))
                .unwrap_or_default(),
            ))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 获取热门语言
pub async fn get_popular_languages(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.language_service.get_popular_languages().await {
        Ok(languages) => {
            let response: Vec<LanguageResponse> =
                languages.into_iter().map(LanguageResponse::from).collect();
            Ok(Json(
                serde_json::to_value(ApiResponse::success_with_message(
                    json!({
                        "currentPage": 1,
                        "total": response.len(),
                        "languagesList": response
                    }),
                    "OK",
                ))
                .unwrap_or_default(),
            ))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 更新编程语言
pub async fn update_language(
    State(state): State<AppState>,
    Json(request): Json<UpdateLanguageRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // 验证请求数据
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.language_service.update_language(request).await {
        Ok(()) => Ok(Json(
            serde_json::to_value(ApiResponse::<()>::success_msg("更新编程语言成功"))
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
