//! 基于滑动窗口的简易内存限速中间件
//!
//! 目标是堵住两类滥用面：登录/注册暴力破解、公开重型接口（渲染/打包）的匿名 DoS。
//! 进程内单例实现，适合单实例部署；多实例部署时应替换为 tower-governor + 共享存储。

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use serde_json::json;
use std::collections::{HashMap, VecDeque};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

/// 滑动窗口计数器：key -> 窗口内请求时间戳
struct SlidingWindow {
    buckets: Mutex<HashMap<String, VecDeque<Instant>>>,
}

impl SlidingWindow {
    fn new() -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
        }
    }

    /// 记录一次请求并判断是否放行；窗口满时淘汰过期条目防止内存无限增长
    fn allow(&self, key: &str, limit: usize, window: Duration) -> bool {
        let now = Instant::now();
        let mut buckets = self.buckets.lock().unwrap();
        let q = buckets.entry(key.to_string()).or_default();
        while let Some(front) = q.front() {
            if now.duration_since(*front) > window {
                q.pop_front();
            } else {
                break;
            }
        }
        if q.len() >= limit {
            return false;
        }
        q.push_back(now);
        // 顺带清理：桶数量异常膨胀时做一次全量过期清扫
        if buckets.len() > 10_000 {
            buckets.retain(|_, v| {
                v.front()
                    .map(|t| now.duration_since(*t) <= window)
                    .unwrap_or(false)
            });
        }
        true
    }
}

static WINDOW: LazyLock<SlidingWindow> = LazyLock::new(SlidingWindow::new);

/// 认证类接口限速：每 IP 每分钟 20 次（登录/注册/忘记密码，防暴力破解）
pub async fn auth_rate_limit(request: Request, next: Next) -> Response {
    generic_limit(request, next, "auth", 20, Duration::from_secs(60)).await
}

/// 公开重型接口限速：每 IP 每分钟 60 次（模板渲染/打包/缓存清理，防匿名 DoS）
pub async fn heavy_rate_limit(request: Request, next: Next) -> Response {
    generic_limit(request, next, "heavy", 60, Duration::from_secs(60)).await
}

async fn generic_limit(
    request: Request,
    next: Next,
    class: &str,
    limit: usize,
    window: Duration,
) -> Response {
    let ip = client_ip(&request);
    if WINDOW.allow(&format!("{}:{}", class, ip), limit, window) {
        next.run(request).await
    } else {
        too_many_requests()
    }
}

/// 提取客户端 IP：优先代理头（反向代理部署），否则用连接信息，兜底 unknown
fn client_ip(request: &Request) -> String {
    if let Some(xff) = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(first) = xff.split(',').next() {
            let ip = first.trim();
            if !ip.is_empty() {
                return ip.to_string();
            }
        }
    }
    if let Some(rip) = request
        .headers()
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
    {
        if !rip.is_empty() {
            return rip.to_string();
        }
    }
    request
        .extensions()
        .get::<std::net::SocketAddr>()
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn too_many_requests() -> Response {
    let body = json!({
        "code": 429,
        "message": "请求过于频繁，请稍后再试",
        "result": null
    })
    .to_string();

    Response::builder()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .header("content-type", "application/json")
        .body(body.into())
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sliding_window_limit() {
        let w = SlidingWindow::new();
        for i in 0..5 {
            assert!(
                w.allow("k", 5, Duration::from_secs(60)),
                "第 {} 次应放行",
                i + 1
            );
        }
        assert!(!w.allow("k", 5, Duration::from_secs(60)), "第 6 次应拒绝");
        assert!(
            w.allow("other", 5, Duration::from_secs(60)),
            "不同 key 互不影响"
        );
    }
}
