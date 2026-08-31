use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;
use tracing::Instrument;
use uuid::Uuid;

/// 请求日志与 trace-id 中间件
///
/// - 每个请求生成 trace-id，放入 `request` span（handler 日志自动内联关联）
///   并通过 `x-trace-id` 响应头回传，前端报障时可直接提供该 ID 定位请求
/// - 记录 method/path/status/耗时（毫秒）
/// - `/health` 探活不记日志，避免噪音
pub async fn request_log_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let start = Instant::now();
    let trace_id = Uuid::new_v4().simple().to_string();

    let span = tracing::info_span!("request", trace_id = %trace_id);
    let mut response = next.run(request).instrument(span).await;

    if path != "/health" {
        tracing::info!(
            method = %method,
            path = %path,
            status = response.status().as_u16(),
            elapsed_ms = start.elapsed().as_millis() as u64,
            "request"
        );
    }

    if let Ok(value) = trace_id.parse() {
        response.headers_mut().insert("x-trace-id", value);
    }
    response
}
