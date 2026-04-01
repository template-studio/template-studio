//! # 文件系统监听模块
//!
//! 监听 templates 目录的文件变化，自动失效相关缓存

use std::path::PathBuf;
use std::sync::Arc;
use notify::{Watcher, RecursiveMode, Event, EventKind, Result as NotifyResult};
use tokio::sync::Mutex;
use tracing::{debug, error, info};
use template_studio_services::cache::DependencyTreeCache;

/// 启动文件系统监听
///
/// 监听 templates 目录的文件变化，自动失效相关缓存
///
/// # 参数
///
/// * `templates_path` - templates 目录路径
/// * `cache` - 依赖树缓存实例
pub fn start_file_watcher(
    templates_path: PathBuf,
    cache: Arc<Mutex<DependencyTreeCache>>,
) -> NotifyResult<()> {
    use notify::recommended_watcher;

    info!("启动文件系统监听: {:?}", templates_path);

    // 创建通道
    let (tx, rx) = std::sync::mpsc::channel();

    // 创建 watcher
    let mut watcher = recommended_watcher(tx)?;

    // 监听 templates 目录
    watcher.watch(&templates_path, RecursiveMode::Recursive)?;

    // 在独立线程中处理事件
    std::thread::spawn(move || {
        info!("文件系统监听线程已启动");

        for event in rx {
            match event {
                Ok(event) => {
                    handle_file_event(event, &templates_path, cache.clone());
                }
                Err(e) => {
                    error!("文件监听错误: {:?}", e);
                }
            }
        }
    });

    info!("文件系统监听已设置");

    Ok(())
}

/// 处理文件变化事件
fn handle_file_event(
    event: Event,
    templates_path: &PathBuf,
    cache: Arc<Mutex<DependencyTreeCache>>,
) {
    // 过滤非目标事件
    let relevant = event.paths.iter().any(|path| {
        // 只处理 templates 目录下的文件
        path.starts_with(templates_path)
    });

    if !relevant {
        return;
    }

    for path in event.paths {
        if !path.starts_with(templates_path) {
            continue;
        }

        // 提取 template_id
        // 路径格式: templates/{template_id}/...
        let relative = path.strip_prefix(templates_path).ok();
        if let Some(rel_path) = relative {
            // 获取第一级目录作为 template_id
            if let Some(template_id_str) = rel_path.iter().next() {
                if let Ok(template_id) = template_id_str.to_string_lossy().parse::<i64>() {
                    handle_template_file_change(
                        template_id,
                        path.clone(),
                        &event.kind,
                        cache.clone()
                    );
                }
            }
        }
    }
}

/// 处理模板文件变化
fn handle_template_file_change(
    template_id: i64,
    file_path: PathBuf,
    event_kind: &EventKind,
    cache: Arc<Mutex<DependencyTreeCache>>,
) {
    // 判断是否是需要处理的事件
    let should_invalidate = match event_kind {
        EventKind::Create(_) => {
            debug!("文件创建: {:?}, template_id={}", file_path, template_id);
            true
        }
        EventKind::Modify(_) => {
            debug!("文件修改: {:?}, template_id={}", file_path, template_id);
            true
        }
        EventKind::Remove(_) => {
            debug!("文件删除: {:?}, template_id={}", file_path, template_id);
            true
        }
        _ => false,
    };

    if should_invalidate {
        // 使用 tokio 运行时来异步处理缓存失效
        tokio::spawn(async move {
            let mut cache = cache.lock().await;
            cache.invalidate(template_id);
            info!("模板 {} 缓存已失效（文件变化）: {:?}", template_id, file_path);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_parsing() {
        let templates_path = PathBuf::from("/templates");
        let file_path = PathBuf::from("/templates/12345/src/main.rs");

        let relative = file_path.strip_prefix(&templates_path).ok();
        assert!(relative.is_some());

        if let Some(rel_path) = relative {
            let template_id_str = rel_path.iter().next();
            assert!(template_id_str.is_some());

            if let Some(id_str) = template_id_str {
                let template_id: i64 = id_str.to_string_lossy().parse().unwrap();
                assert_eq!(template_id, 12345);
            }
        }
    }
}
