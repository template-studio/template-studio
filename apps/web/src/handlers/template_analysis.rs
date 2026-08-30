//! 模板变量分析处理器

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use template_studio_shared::models::auth::AuthUser;

pub type AppState = super::super::AppState;

/// 分析模板变量
pub async fn analyze_variables(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(template_id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(resp) =
        crate::handlers::access::ensure_template_access(&state, &auth_user, template_id).await
    {
        return Err(resp);
    }
    match state
        .template_analysis_service
        .analyze_variables(template_id)
        .await
    {
        Ok(response) => Ok(Json(json!({
            "code": 0,
            "data": response,
            "message": "OK"
        }))),
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
