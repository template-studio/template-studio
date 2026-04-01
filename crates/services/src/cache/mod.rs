//! # 缓存模块
//!
//! 提供模板渲染相关的缓存功能

pub mod dependency_cache;

pub use dependency_cache::{
    DependencyTreeCache,
    DependencyTreeEntry,
    TemplateVersionManager,
    CacheStats,
};
