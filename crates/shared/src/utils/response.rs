use serde::{Deserialize, Serialize};

/// 统一错误码表
///
/// body 的 code 与 HTTP 状态码语义同步；新增错误码必须在此登记，
/// 禁止在 handler 里手写裸数字（`json!` 手写信封为存量技术债，见 dev-docs）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// 成功
    Ok,
    /// 请求参数/格式错误
    BadRequest,
    /// 未认证（缺令牌/令牌无效）
    Unauthorized,
    /// 已认证但无权操作
    Forbidden,
    /// 资源不存在
    NotFound,
    /// 资源冲突（重名/重复创建）
    Conflict,
    /// 请求过于频繁（限速触发）
    TooManyRequests,
    /// 服务器内部错误
    Internal,
}

impl ErrorCode {
    /// 业务 code（同时也是 HTTP 状态码）
    pub fn code(&self) -> i32 {
        match self {
            ErrorCode::Ok => 0,
            ErrorCode::BadRequest => 400,
            ErrorCode::Unauthorized => 401,
            ErrorCode::Forbidden => 403,
            ErrorCode::NotFound => 404,
            ErrorCode::Conflict => 409,
            ErrorCode::TooManyRequests => 429,
            ErrorCode::Internal => 500,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::Ok => "ok",
            ErrorCode::BadRequest => "bad_request",
            ErrorCode::Unauthorized => "unauthorized",
            ErrorCode::Forbidden => "forbidden",
            ErrorCode::NotFound => "not_found",
            ErrorCode::Conflict => "conflict",
            ErrorCode::TooManyRequests => "too_many_requests",
            ErrorCode::Internal => "internal_error",
        }
    }
}

/// 统一API响应格式（唯一信封：`{code, message, data}`）
///
/// code:0 成功；失败时 HTTP 状态码与语义同步、body 携带同一 code。
/// 新代码应通过本类型的构造方法产生响应，避免 `json!` 手写信封。
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub data: Option<T>,
    pub message: String,
}

impl<T> ApiResponse<T> {
    /// 成功响应
    pub fn success(data: T) -> Self {
        Self {
            code: ErrorCode::Ok.code(),
            data: Some(data),
            message: "success".to_string(),
        }
    }

    /// 成功响应（自定义消息，无数据）
    pub fn success_msg(message: &str) -> Self {
        Self {
            code: ErrorCode::Ok.code(),
            data: None::<T>,
            message: message.to_string(),
        }
    }

    /// 成功响应（数据 + 自定义消息）
    pub fn success_with_message(data: T, message: &str) -> Self {
        Self {
            code: ErrorCode::Ok.code(),
            data: Some(data),
            message: message.to_string(),
        }
    }

    /// 错误响应（按错误码表）
    pub fn error(err: ErrorCode, message: &str) -> Self {
        Self {
            code: err.code(),
            data: None,
            message: message.to_string(),
        }
    }
}

/// 分页响应数据
#[derive(Debug, Serialize, Deserialize)]
pub struct PagedResponse<T> {
    pub items: Vec<T>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

impl<T> PagedResponse<T> {
    pub fn new(items: Vec<T>, total: u32, page: u32, page_size: u32) -> Self {
        let total_pages = if page_size > 0 {
            (total + page_size - 1) / page_size
        } else {
            0
        };

        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }
}

/// 验证错误详情
#[derive(Debug, Serialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

/// 批量操作结果
#[derive(Debug, Serialize)]
pub struct BatchOperationResult {
    pub total: u32,
    pub success: u32,
    pub failed: u32,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_envelope() {
        let r = ApiResponse::success(42);
        assert_eq!(r.code, 0);
        assert_eq!(r.data, Some(42));
        assert_eq!(r.message, "success");
    }

    #[test]
    fn test_error_codes_match_http_semantics() {
        assert_eq!(ErrorCode::Ok.code(), 0);
        assert_eq!(ErrorCode::BadRequest.code(), 400);
        assert_eq!(ErrorCode::Unauthorized.code(), 401);
        assert_eq!(ErrorCode::Forbidden.code(), 403);
        assert_eq!(ErrorCode::NotFound.code(), 404);
        assert_eq!(ErrorCode::Conflict.code(), 409);
        assert_eq!(ErrorCode::TooManyRequests.code(), 429);
        assert_eq!(ErrorCode::Internal.code(), 500);
        // 错误响应不含数据
        let r: ApiResponse<()> = ApiResponse::error(ErrorCode::NotFound, "不存在");
        assert_eq!(r.code, 404);
        assert_eq!(r.data, None);
    }

    #[test]
    fn test_serialized_shape() {
        let r: ApiResponse<i32> = ApiResponse::success_msg("完成");
        let j = serde_json::to_string(&r).unwrap();
        assert!(j.contains("\"code\":0"));
        assert!(j.contains("\"message\":\"完成\""));
        assert!(j.contains("\"data\":null"));
    }
}
