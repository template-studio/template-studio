use axum::{extract::State, http::StatusCode, response::Json};
use serde::Deserialize;
use serde_json::{json, Value};
use validator::Validate;

pub type AppState = super::super::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(length(min = 1, message = "请输入邮箱"))]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(length(min = 1, message = "令牌不能为空"))]
    pub token: String,
    #[validate(length(min = 6, message = "密码至少6位"))]
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct TestEmailQuery {
    pub email: String,
}

/// 忘记密码 - 发送重置邮件
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(request): Json<ForgotPasswordRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.email_service.send_reset_email(&request.email).await {
        Ok(_) => Ok(Json(json!({
            "code": 0,
            "message": "如果该邮箱已注册，重置邮件已发送"
        }))),
        Err(e) => {
            tracing::error!("发送重置邮件失败: {}", e);
            // 不暴露具体错误，防止邮箱枚举
            Ok(Json(json!({
                "code": 0,
                "message": "如果该邮箱已注册，重置邮件已发送"
            })))
        }
    }
}

/// 重置密码
pub async fn reset_password(
    State(state): State<AppState>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state
        .email_service
        .reset_password(&request.token, &request.password)
        .await
    {
        Ok(_) => Ok(Json(json!({
            "code": 0,
            "message": "密码重置成功"
        }))),
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

/// 发送测试邮件（管理员）
pub async fn test_email(
    State(state): State<AppState>,
    Json(query): Json<TestEmailQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if query.email.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "请输入收件邮箱");
    }
    match state.email_service.send_test_email(&query.email).await {
        Ok(_) => Ok(Json(json!({
            "code": 0,
            "message": "测试邮件发送成功"
        }))),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("发送失败: {}", e),
        ),
    }
}

fn error_response(
    status: StatusCode,
    message: &str,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Err((
        status,
        Json(json!({
            "code": status.as_u16(),
            "message": message
        })),
    ))
}
