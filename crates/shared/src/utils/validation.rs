use validator::Validate;
use crate::utils::error::AppError;

/// 验证请求数据
pub fn validate_request<T: Validate>(req: &T) -> Result<(), AppError> {
    if let Err(errors) = req.validate() {
        let error_messages: Vec<String> = errors
            .field_errors()
            .into_iter()
            .flat_map(|(field, field_errors)| {
                field_errors.iter().map(move |fe| format!("{}: {}", field, fe.message.as_ref().unwrap_or(&std::borrow::Cow::Borrowed("验证失败"))))
            })
            .collect();

        let error_msg = error_messages.join("; ");
        return Err(AppError::Validation(error_msg));
    }
    Ok(())
}