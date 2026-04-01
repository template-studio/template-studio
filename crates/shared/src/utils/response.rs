use serde::{Deserialize, Serialize};

/// 统一API响应格式
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
            code: 0,
            data: Some(data),
            message: "success".to_string(),
        }
    }

    /// 成功响应（无数据）
    pub fn success_with_message(message: &str) -> Self {
        Self {
            code: 0,
            data: None::<T>,
            message: message.to_string(),
        }
    }

    /// 错误响应
    pub fn error(code: i32, message: &str) -> Self {
        Self {
            code,
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