//! 模板级访问控制：属主或 super_admin 才能操作指定模板

use axum::{http::StatusCode, response::Json};
use serde_json::{json, Value};
use template_studio_shared::models::auth::AuthUser;

use crate::AppState;

/// 校验当前用户对模板的访问权：super_admin 直通，否则必须是模板属主。
///
/// 用于模板写操作（编辑/删除/发布/回滚/文件操作等），调用方需先经过认证中间件
/// （AuthUser 已注入 extensions）。
pub async fn ensure_template_access(
    state: &AppState,
    auth_user: &AuthUser,
    template_id: i64,
) -> Result<(), (StatusCode, Json<Value>)> {
    let roles = state
        .auth_service
        .get_user_role_names(auth_user.user_id)
        .await
        .unwrap_or_default();
    if roles.iter().any(|r| r == "super_admin") {
        return Ok(());
    }

    let is_owner = state
        .template_service
        .is_template_owner(template_id, auth_user.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "code": template_studio_shared::utils::response::ErrorCode::Internal.code(),
                    "message": e.to_string()
                })),
            )
        })?;

    if is_owner {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "code": template_studio_shared::utils::response::ErrorCode::Forbidden.code(),
                "message": "无权操作他人的模板"
            })),
        ))
    }
}
