//! # 依赖树缓存模块
//!
//! 缓存模板的完整依赖树，避免重复解析 include/extends/import 关系
//!
//! ## 性能优化
//!
//! - 缓存已解析的完整依赖树（包含所有 include/extends/import）
//! - 缓存条件配置（ConditionsYaml）
//! - 版本号管理，快速检测缓存失效

use std::collections::HashMap;
use std::time::SystemTime;
use std::path::PathBuf;
use template_studio_template_core::{
    TemplateFile,
    conditions::ConditionsYaml,
};
use tracing::{debug, info, warn};

/// 模板版本管理器
///
/// 用于快速检测模板是否修改，避免每次都比较文件内容
#[derive(Debug, Clone)]
pub struct TemplateVersionManager {
    /// 模板版本号 (template_id -> version)
    versions: HashMap<i64, u64>,

    /// 文件修改时间戳 (template_id -> (file_path -> mtime))
    file_mtimes: HashMap<i64, HashMap<String, SystemTime>>,

    /// 条件文件修改时间 (template_id -> mtime)
    condition_mtimes: HashMap<i64, SystemTime>,

    /// 模板根目录
    templates_base_path: PathBuf,
}

impl TemplateVersionManager {
    /// 创建新的版本管理器
    pub fn new(templates_base_path: PathBuf) -> Self {
        Self {
            versions: HashMap::new(),
            file_mtimes: HashMap::new(),
            condition_mtimes: HashMap::new(),
            templates_base_path,
        }
    }

    /// 检查模板是否已修改
    ///
    /// # 返回
    ///
    /// - `Ok(true)`: 模板已修改，缓存失效
    /// - `Ok(false)`: 模板未修改，缓存有效
    /// - `Err(_)`: 检查失败（文件不存在等），应重新构建
    pub fn is_template_modified(&mut self, template_id: i64) -> Result<bool, String> {
        // 获取当前版本号
        let current_version = self.calculate_version(template_id)?;

        // 检查版本号是否变化
        let last_version = self.versions.get(&template_id).copied().unwrap_or(0);

        if current_version != last_version {
            debug!(
                "模板 {} 版本变化: {} -> {}",
                template_id, last_version, current_version
            );
            return Ok(true);
        }

        Ok(false)
    }

    /// 更新模板版本号
    pub fn update_version(&mut self, template_id: i64) {
        let version = self.calculate_version(template_id).unwrap_or(0);
        self.versions.insert(template_id, version);
        debug!("更新模板 {} 版本号为 {}", template_id, version);
    }

    /// 计算模板版本号
    ///
    /// 基于以下因素计算哈希：
    /// - 文件数量
    /// - 最新文件修改时间
    /// - 条件文件修改时间
    fn calculate_version(&self, template_id: i64) -> Result<u64, String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // 1. 文件数量和修改时间
        if let Some(mtimes) = self.file_mtimes.get(&template_id) {
            mtimes.len().hash(&mut hasher);

            for (_, mtime) in mtimes {
                if let Ok(duration) = mtime.duration_since(SystemTime::UNIX_EPOCH) {
                    duration.as_secs().hash(&mut hasher);
                }
            }
        }

        // 2. 条件文件修改时间
        if let Some(mtime) = self.condition_mtimes.get(&template_id) {
            if let Ok(duration) = mtime.duration_since(SystemTime::UNIX_EPOCH) {
                (duration.as_secs() + 1).hash(&mut hasher);
            }
        }

        Ok(hasher.finish())
    }

    /// 记录文件的修改时间
    pub fn record_file_mtime(&mut self, template_id: i64, file_path: &str, mtime: SystemTime) {
        self.file_mtimes
            .entry(template_id)
            .or_insert_with(HashMap::new)
            .insert(file_path.to_string(), mtime);
    }

    /// 记录条件文件的修改时间
    pub fn record_condition_mtime(&mut self, template_id: i64, mtime: SystemTime) {
        self.condition_mtimes.insert(template_id, mtime);
    }

    /// 使模板缓存失效
    pub fn invalidate(&mut self, template_id: i64) {
        self.versions.remove(&template_id);
        self.file_mtimes.remove(&template_id);
        self.condition_mtimes.remove(&template_id);
        debug!("模板 {} 缓存已失效", template_id);
    }

    /// 获取模板版本号
    pub fn get_version(&self, template_id: i64) -> u64 {
        self.versions.get(&template_id).copied().unwrap_or(0)
    }
}

/// 依赖树缓存条目
#[derive(Debug, Clone)]
pub struct DependencyTreeEntry {
    /// 完整的依赖树（已解析所有 include/extends/import）
    pub complete_tree: Vec<TemplateFile>,

    /// 条件配置
    pub conditions: ConditionsYaml,

    /// 缓存时间
    pub cached_at: std::time::Instant,

    /// 模板版本（用于快速失效检测）
    pub template_version: u64,

    /// 文件数量
    pub file_count: usize,

    /// ✅ 新增：基础节点数量（不包含依赖，用于检测节点删除/新增）
    pub base_file_count: usize,
}

/// 依赖树缓存
///
/// 缓存模板的完整依赖树，避免重复解析依赖关系
pub struct DependencyTreeCache {
    /// 缓存条目 (template_id -> entry)
    cache: HashMap<i64, DependencyTreeEntry>,

    /// 版本管理器
    version_manager: TemplateVersionManager,

    /// 缓存统计
    stats: CacheStats,
}

/// 缓存统计信息
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    /// 缓存命中次数
    pub hits: u64,

    /// 缓存未命中次数
    pub misses: u64,

    /// 缓存失效次数
    pub invalidations: u64,
}

impl DependencyTreeCache {
    /// 创建新的缓存
    pub fn new(templates_base_path: PathBuf) -> Self {
        Self {
            cache: HashMap::new(),
            version_manager: TemplateVersionManager::new(templates_base_path),
            stats: CacheStats::default(),
        }
    }

    /// 查询缓存
    ///
    /// # 参数
    ///
    /// * `template_id` - 模板 ID
    ///
    /// # 返回
    ///
    /// - `Some(entry)`: 缓存命中且未过期
    /// - `None`: 缓存未命中或已过期
    pub fn get(&mut self, template_id: i64) -> Option<&DependencyTreeEntry> {
        // 先检查模板是否修改
        let should_invalidate = match self.version_manager.is_template_modified(template_id) {
            Ok(true) => {
                // 模板已修改，缓存失效
                warn!("模板 {} 已修改，缓存失效", template_id);
                true
            }
            Ok(false) => {
                // 模板未修改，继续检查缓存
                false
            }
            Err(e) => {
                // 检查失败，缓存失效
                warn!(
                    "模板 {} 版本检查失败: {}, 缓存失效",
                    template_id, e
                );
                true
            }
        };

        if should_invalidate {
            self.invalidate(template_id);
            self.stats.misses += 1;
            return None;
        }

        // 检查缓存是否存在
        let entry = self.cache.get(&template_id)?;

        // 缓存有效
        debug!(
            "L2 缓存命中: template_id={}, file_count={}",
            template_id,
            entry.file_count
        );
        self.stats.hits += 1;
        Some(entry)
    }

    /// 插入缓存
    ///
    /// # 参数
    ///
    /// * `template_id` - 模板 ID
    /// * `complete_tree` - 完整的依赖树
    /// * `conditions` - 条件配置
    pub fn insert(
        &mut self,
        template_id: i64,
        complete_tree: Vec<TemplateFile>,
        conditions: ConditionsYaml,
        base_file_count: usize,
    ) {
        let file_count = complete_tree.len();
        let template_version = self.version_manager.get_version(template_id);

        let entry = DependencyTreeEntry {
            complete_tree,
            conditions,
            cached_at: std::time::Instant::now(),
            template_version,
            file_count,
            base_file_count,  // ✅ 保存基础节点数量
        };

        self.cache.insert(template_id, entry);

        info!(
            "L2 缓存已更新: template_id={}, file_count={}, base_file_count={}, version={}, 缓存数={}",
            template_id,
            file_count,
            base_file_count,
            template_version,
            self.cache.len()
        );
    }

    /// 使缓存失效
    ///
    /// 当模板文件变化时调用
    pub fn invalidate(&mut self, template_id: i64) {
        self.cache.remove(&template_id);
        self.version_manager.invalidate(template_id);
        self.stats.invalidations += 1;
    }

    /// 更新版本号（在缓存新数据后调用）
    pub fn update_version(&mut self, template_id: i64) {
        self.version_manager.update_version(template_id);
    }

    /// 记录文件的修改时间
    pub fn record_file_mtime(&mut self, template_id: i64, file_path: &str, mtime: SystemTime) {
        self.version_manager.record_file_mtime(template_id, file_path, mtime);
    }

    /// 记录条件文件的修改时间
    pub fn record_condition_mtime(&mut self, template_id: i64, mtime: SystemTime) {
        self.version_manager.record_condition_mtime(template_id, mtime);
    }

    /// 获取缓存统计信息
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// 获取缓存命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.stats.hits + self.stats.misses;
        if total == 0 {
            return 0.0;
        }
        self.stats.hits as f64 / total as f64
    }

    /// 清空所有缓存
    pub fn clear(&mut self) {
        self.cache.clear();
        self.version_manager = TemplateVersionManager::new(
            self.version_manager.templates_base_path.clone()
        );
        self.stats = CacheStats::default();
        info!("L2 缓存已清空");
    }

    /// 获取当前缓存数量
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// 检查缓存是否为空
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_manager() {
        let mut manager = TemplateVersionManager::new(PathBuf::from("/tmp"));

        // 首次获取版本号（无文件时也应生成哈希值）
        let version1 = manager.calculate_version(123).unwrap();
        assert!(version1 > 0 || version1 == 0); // 哈希值可以是0或任何值

        // 更新版本号
        manager.update_version(123);
        assert_eq!(manager.get_version(123), version1); // 应该存储相同的版本号

        // 再次计算应该得到相同的版本号
        let version2 = manager.calculate_version(123).unwrap();
        assert_eq!(version1, version2); // 相同输入应得到相同哈希
    }

    #[test]
    fn test_cache_basic() {
        let mut cache = DependencyTreeCache::new(PathBuf::from("/tmp"));

        // 空缓存
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);

        // 插入缓存
        cache.insert(
            123,
            vec![],
            ConditionsYaml::new(),
            0,  // base_file_count
        );

        // 检查缓存
        assert!(!cache.is_empty());
        assert_eq!(cache.len(), 1);

        // 命中率计算
        assert_eq!(cache.hit_rate(), 0.0); // 0 hits / 0 misses
    }
}
