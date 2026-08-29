//! 模板渲染服务
//! 使用 template_core 提供的核心渲染能力

use crate::cache::DependencyTreeCache;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use template_studio_shared::models::file_tree::FileTreeNode;
use template_studio_shared::utils::error::AppError;
use template_studio_template_core::conditions::ConditionsYaml;
use template_studio_template_core::{
    render_string, render_tree, TemplateFile, TreeBuilder, Variables,
};
use tokio::fs;
use tracing::{debug, error, info, warn};

/// 模板渲染服务
pub struct TemplateRenderService {
    base_path: PathBuf,
    /// L2 依赖树缓存
    dependency_cache: Arc<tokio::sync::Mutex<DependencyTreeCache>>,
}

impl TemplateRenderService {
    pub fn new(base_path: PathBuf) -> Self {
        let cache = DependencyTreeCache::new(base_path.clone());
        Self {
            base_path,
            dependency_cache: Arc::new(tokio::sync::Mutex::new(cache)),
        }
    }

    /// 获取缓存实例（用于文件系统监听）
    pub fn get_cache(&self) -> Arc<tokio::sync::Mutex<DependencyTreeCache>> {
        self.dependency_cache.clone()
    }

    /// 清除模板缓存
    pub async fn clear_cache(&self, template_id: i64) {
        let mut cache = self.dependency_cache.lock().await;
        cache.invalidate(template_id);
        info!("已清除模板 {} 的缓存", template_id);
    }

    /// 渲染单个模板文件
    pub async fn render_file(
        &self,
        template_id: i64,
        file_path: &str,
        variables: &serde_json::Value,
    ) -> Result<RenderResult, AppError> {
        let template_root_path = self.base_path.join(template_id.to_string());
        let template_path = template_root_path.join(file_path.trim_start_matches('/'));

        tracing::info!("渲染模板文件: {:?}", template_path);

        // 检查文件是否存在
        if !template_path.exists() {
            return Err(AppError::NotFound(format!("文件不存在: {}", file_path)));
        }

        // 读取模板内容（IO 错误含完整路径，仅记日志，不回传客户端）
        let template_content = fs::read_to_string(&template_path).await.map_err(|e| {
            tracing::warn!("读取模板文件失败 {:?}: {}", template_path, e);
            AppError::Internal("读取模板文件失败".to_string())
        })?;

        // 获取文件名
        let file_name = template_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // 使用 template_core 进行渲染
        let render_vars = Variables::from_value(variables.clone());
        let core_result = render_string(&template_content, &render_vars, None)
            .map_err(|e| AppError::Internal(format!("渲染失败: {}", e)))?;

        // 转换结果格式
        Ok(RenderResult {
            file_name,
            file_content: core_result.content,
            variables: variables.clone(),
            success: core_result.success,
            error: core_result.error.map(|e| RenderError {
                error_type: e.error_type,
                message: e.message,
                line: e.line,
                column: e.column,
                context: e.context,
                suggestion: e.suggestion,
            }),
        })
    }

    /// 从指定路径渲染模板文件（用于 preview/generate 功能）
    pub async fn render_file_from_path(
        &self,
        base_path: &PathBuf,
        file_path: &str,
        variables: &serde_json::Value,
    ) -> Result<RenderResult, AppError> {
        // file_path 来自客户端请求，先做穿越校验（../、绝对路径、盘符）
        let template_path = template_studio_shared::utils::path::safe_join(base_path, file_path)
            .map_err(AppError::Validation)?;

        tracing::info!("从指定路径渲染模板文件: {:?}", template_path);

        // 检查文件是否存在
        if !template_path.exists() {
            return Err(AppError::NotFound(format!("文件不存在: {}", file_path)));
        }

        // 读取模板内容（IO 错误含完整路径，仅记日志，不回传客户端）
        let template_content = fs::read_to_string(&template_path).await.map_err(|e| {
            tracing::warn!("读取模板文件失败 {:?}: {}", template_path, e);
            AppError::Internal("读取模板文件失败".to_string())
        })?;

        // 获取文件名
        let file_name = template_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // 收集整棵模板树构建双键映射（与整树渲染共用同一构建函数），
        // 保证单文件预览与最终渲染的继承/include 解析结果一致。
        // 旧实现仅收集同目录 HTML，跨目录 extends 在预览中会失败。
        let all_templates = Some(self.collect_template_map(base_path).await);

        // 使用 template_core 进行渲染
        let render_vars = Variables::from_value(variables.clone());
        let core_result = render_string(&template_content, &render_vars, all_templates.as_ref())
            .map_err(|e| AppError::Internal(format!("渲染失败: {}", e)))?;

        // 转换结果格式
        Ok(RenderResult {
            file_name,
            file_content: core_result.content,
            variables: variables.clone(),
            success: core_result.success,
            error: core_result.error.map(|e| RenderError {
                error_type: e.error_type,
                message: e.message,
                line: e.line,
                column: e.column,
                context: e.context,
                suggestion: e.suggestion,
            }),
        })
    }

    /// 递归收集目录下全部文本文件，构建双键模板映射（与整树渲染共用语义）
    ///
    /// 跳过 .git/.meta 目录；读取失败（二进制/编码问题）的文件静默跳过。
    async fn collect_template_map(
        &self,
        base: &PathBuf,
    ) -> std::collections::HashMap<String, String> {
        let mut entries: Vec<(String, String)> = Vec::new();
        Self::walk_for_templates(base, base, &mut entries).await;
        template_studio_template_core::build_template_map(entries)
    }

    async fn walk_for_templates(
        base: &PathBuf,
        dir: &PathBuf,
        entries: &mut Vec<(String, String)>,
    ) {
        let Ok(mut rd) = tokio::fs::read_dir(dir).await else {
            return;
        };
        while let Ok(Some(entry)) = rd.next_entry().await {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if path.is_dir() {
                if name == ".git" || name == ".meta" {
                    continue;
                }
                Box::pin(Self::walk_for_templates(base, &path, entries)).await;
            } else if let Ok(rel) = path.strip_prefix(base) {
                let rel_path = rel.to_string_lossy().replace('\\', "/");
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    entries.push((rel_path, content));
                }
            }
        }
    }

    /// 渲染整个文件树（带 L2 缓存优化）
    ///
    /// # 流程
    /// 1. 查询 L2 缓存（依赖树 + 条件配置）
    /// 2. 如果缓存命中，跳过依赖树构建
    /// 3. 如果缓存未命中，构建依赖树并更新缓存
    /// 4. 根据条件过滤文件
    /// 5. 调用 render_tree() 统一渲染
    /// 6. 统计信息并构建响应
    pub async fn render_file_tree(
        &self,
        template_id: i64,
        file_tree: Vec<FileTreeNode>,
        variables: &serde_json::Value,
    ) -> Result<RenderFileTreeResponse, AppError> {
        let start_time = std::time::Instant::now();
        info!("开始渲染文件树: template_id={}", template_id);

        // ===== L2 缓存查询 =====
        let mut cache = self.dependency_cache.lock().await;

        if let Some(entry) = cache.get(template_id) {
            // L2 缓存命中！使用缓存的依赖树
            debug!("L2 缓存命中: template_id={}, 跳过依赖树构建", template_id);

            // 使用缓存的依赖树和条件配置
            let complete_tree = entry.complete_tree.clone();
            let conditions = entry.conditions.clone();

            // ✅ 关键修复：检查基础节点是否变化（节点删除/新增）
            let input_base_count = file_tree.len();
            let cached_base_count = entry.base_file_count;

            if input_base_count != cached_base_count {
                info!(
                    "检测到节点变化: input={} 基础节点, cached={} 基础节点, 重建依赖树",
                    input_base_count, cached_base_count
                );

                // 节点数量不匹配，说明有节点被删除或新增，需要重建缓存
                cache.invalidate(template_id);

                // 继续走缓存未命中逻辑
            } else {
                // 基础节点数量匹配，使用缓存
                let builder = TreeBuilder::new()
                    .with_auto_resolve(true)
                    .with_conditions(conditions);

                let original_count = complete_tree.len();
                info!(
                    "使用缓存依赖树: 共 {} 个文件 (基础节点: {})",
                    original_count, cached_base_count
                );

                // 转换变量格式
                let render_vars = Variables::from_value(variables.clone());

                // 根据条件过滤文件
                let filtered_tree = builder.filter_by_conditions(complete_tree, &render_vars);
                debug!(
                    "条件过滤完成: 共 {} 个文件（原始 {} 个）",
                    filtered_tree.len(),
                    original_count
                );

                // 渲染
                let rendered_files = render_tree(filtered_tree, &render_vars)
                    .map_err(|e| AppError::Internal(format!("文件树渲染失败: {}", e)))?;

                // 构建响应
                let response = self.build_render_response(template_id, rendered_files, variables);

                let elapsed = start_time.elapsed();
                info!(
                    "L2 缓存命中渲染完成: template_id={}, 耗时={}ms, 缓存命中率={:.1}%",
                    template_id,
                    elapsed.as_millis(),
                    cache.hit_rate() * 100.0
                );

                return Ok(response);
            }
        }

        // ===== L2 缓存未命中，执行完整流程 =====
        debug!("L2 缓存未命中，执行完整渲染流程");

        // 保存基础节点数量（在 file_tree 被移动之前）
        let base_file_count = file_tree.len();

        // 1. 扁平化文件树并读取文件内容（同时记录 mtime）
        let template_files = self
            .convert_to_template_files_with_mtime(template_id, file_tree, &mut cache)
            .await?;

        info!("文件树转换完成: 共 {} 个文件/目录", template_files.len());

        // 2. 加载条件配置（同时记录 mtime）
        let conditions = self
            .load_conditions_with_mtime(template_id, &mut cache)
            .await?;

        // 3. 构建完整的依赖树（使用 TreeBuilder）
        let builder = TreeBuilder::new()
            .with_auto_resolve(true) // 自动解析所有依赖
            .with_conditions(conditions.clone());

        let complete_tree = builder
            .build_complete_tree(template_files)
            .map_err(|e| AppError::Internal(format!("构建文件树失败: {}", e)))?;

        let original_count = complete_tree.len();
        info!("完整依赖树构建完成: 共 {} 个文件", original_count);

        // 4. 更新 L2 缓存
        cache.insert(
            template_id,
            complete_tree.clone(),
            conditions.clone(),
            base_file_count, // ✅ 传入基础节点数量
        );
        cache.update_version(template_id);

        // 5. 转换变量格式
        let render_vars = Variables::from_value(variables.clone());

        // 6. 根据条件过滤文件
        let filtered_tree = builder.filter_by_conditions(complete_tree, &render_vars);
        info!(
            "条件过滤完成: 共 {} 个文件（原始 {} 个）",
            filtered_tree.len(),
            original_count
        );

        // 7. 调用 render_tree() 渲染
        let rendered_files = render_tree(filtered_tree, &render_vars)
            .map_err(|e| AppError::Internal(format!("文件树渲染失败: {}", e)))?;

        // 8. 构建响应
        let response = self.build_render_response(template_id, rendered_files, variables);

        let elapsed = start_time.elapsed();
        info!(
            "L2 缓存未命中渲染完成: template_id={}, 耗时={}ms, 缓存命中率={:.1}%",
            template_id,
            elapsed.as_millis(),
            cache.hit_rate() * 100.0
        );

        Ok(response)
    }

    /// 构建渲染响应
    fn build_render_response(
        &self,
        template_id: i64,
        rendered_files: Vec<template_studio_template_core::RenderedFile>,
        variables: &serde_json::Value,
    ) -> RenderFileTreeResponse {
        // 统计信息
        let total_files = rendered_files.len() as i32;
        let total_size: i64 = rendered_files.iter().map(|f| f.filesize as i64).sum();
        let failed_files = rendered_files.iter().filter(|f| f.error.is_some()).count() as i32;

        if failed_files > 0 {
            warn!("文件树渲染完成，但有 {} 个文件渲染失败", failed_files);
        } else {
            info!(
                "文件树渲染成功完成: 总文件数={}, 总大小={} bytes",
                total_files, total_size
            );
        }

        // 转换为响应格式（扁平结构）
        let flat_tree: Vec<RenderedFileInfo> = rendered_files
            .into_iter()
            .map(|f| RenderedFileInfo {
                id: f.id,
                file_path: f.file_path,
                file_name: f.file_name,
                file_content: f.file_content,
                is_directory: f.is_directory,
                filesize: f.filesize,
                parent_id: f.parent_id,
                children: None,
                render_error: f.error.map(|e| RenderError {
                    error_type: e.error_type,
                    message: e.message,
                    line: e.line,
                    column: e.column,
                    context: e.context,
                    suggestion: e.suggestion,
                }),
            })
            .collect();

        // 构建树形结构
        let tree = self.build_tree_structure(flat_tree);

        RenderFileTreeResponse {
            template_id,
            tree,
            variables: variables.clone(),
            total_files,
            total_size,
            failed_files,
        }
    }

    /// 将扁平列表转换为嵌套的树形结构
    fn build_tree_structure(&self, flat_list: Vec<RenderedFileInfo>) -> Vec<RenderedFileInfo> {
        use std::collections::HashMap;

        // 创建 id -> node 的映射，初始化所有节点的 children 为空向量
        let mut nodes: HashMap<i64, RenderedFileInfo> = flat_list
            .into_iter()
            .map(|mut node| {
                node.children = Some(Vec::new());
                (node.id, node)
            })
            .collect();

        // 收集根节点ID，并构建父子关系映射
        let mut root_ids = Vec::new();
        let mut child_ids: HashMap<i64, Vec<i64>> = HashMap::new();

        for (id, node) in &nodes {
            if node.parent_id == 0 {
                root_ids.push(*id);
            } else {
                child_ids
                    .entry(node.parent_id)
                    .or_insert_with(Vec::new)
                    .push(*id);
            }
        }

        // 构建根节点列表
        let mut roots: Vec<RenderedFileInfo> = root_ids
            .into_iter()
            .filter_map(|id| nodes.remove(&id))
            .collect();

        // 递归构建树形结构
        self.build_tree_recursive(&mut nodes, &mut roots, &child_ids);

        // 排序
        self.sort_tree_recursive(&mut roots);

        roots
    }

    /// 递归构建树形结构
    fn build_tree_recursive(
        &self,
        nodes: &mut std::collections::HashMap<i64, RenderedFileInfo>,
        parent_nodes: &mut Vec<RenderedFileInfo>,
        child_ids: &std::collections::HashMap<i64, Vec<i64>>,
    ) {
        for parent in parent_nodes.iter_mut() {
            if let Some(children_ids) = child_ids.get(&parent.id) {
                for &child_id in children_ids {
                    if let Some(child) = nodes.remove(&child_id) {
                        if let Some(ref mut children) = parent.children {
                            children.push(child);
                        }
                    }
                }

                // 递归处理子节点
                if let Some(ref mut children) = parent.children {
                    self.build_tree_recursive(nodes, children, child_ids);
                }
            }
        }
    }

    /// 递归排序树形结构
    fn sort_tree_recursive(&self, nodes: &mut [RenderedFileInfo]) {
        nodes.sort_by(|a, b| {
            // 首先按 isDirectory 降序排序（目录在前）
            match b.is_directory.cmp(&a.is_directory) {
                std::cmp::Ordering::Equal => {
                    // 相同类型按文件名升序排序
                    a.file_name.cmp(&b.file_name)
                }
                other => other,
            }
        });

        // 递归排序子节点
        for node in nodes.iter_mut() {
            if let Some(ref mut children) = node.children {
                self.sort_tree_recursive(children);
            }
        }
    }

    #[allow(dead_code)]
    /// 递归扁平化文件树
    async fn flatten_file_tree(
        &self,
        base_path: &PathBuf,
        nodes: &[FileTreeNode],
        result: &mut Vec<TemplateFile>,
        conditions: &ConditionsYaml,
    ) -> Result<(), AppError> {
        for node in nodes {
            // 添加当前节点
            let file_path = base_path.join(&node.file_path);

            // 读取文件内容（仅文件）
            let file_content = if node.is_directory == 0 {
                fs::read_to_string(&file_path).await.unwrap_or_else(|e| {
                    error!("读取文件失败: {:?}, error: {}", file_path, e);
                    String::new()
                })
            } else {
                String::new()
            };

            // 获取文件条件
            let condition = conditions.get_condition_by_path(&node.file_path);

            let template_file = TemplateFile {
                id: node.id,
                file_path: node.file_path.clone(),
                file_name: node.file_name.clone(),
                file_content,
                is_directory: node.is_directory,
                parent_id: node.parent_id,
                filesize: node.file_size as i32,
                // 新增字段
                extends: None,
                includes: None,
                imports: None,
                condition,
                is_dependency: false,
                required_by: None,
            };

            result.push(template_file);

            // 如果有子节点，递归处理
            if let Some(ref children) = node.children {
                Box::pin(self.flatten_file_tree(base_path, children, result, conditions)).await?;
            }
        }

        Ok(())
    }

    /// 加载条件配置文件
    ///
    /// 从 .meta/variables/conditions.yml 加载条件配置
    #[allow(dead_code)]
    async fn load_conditions(&self, template_id: i64) -> Result<ConditionsYaml, AppError> {
        let conditions_path = self
            .base_path
            .join(template_id.to_string())
            .join(".meta/variables/conditions.yml");

        if !conditions_path.exists() {
            // 文件不存在，返回空条件
            return Ok(ConditionsYaml::new());
        }

        let content = fs::read_to_string(&conditions_path)
            .await
            .map_err(|e| AppError::Internal(format!("读取条件文件失败: {}", e)))?;

        ConditionsYaml::from_yaml(&content)
            .map_err(|e| AppError::Internal(format!("解析条件文件失败: {}", e)))
    }

    /// 将 FileTreeNode 树扁平化并转换为 TemplateFile 列表
    ///
    /// 同时读取文件内容和条件信息
    #[allow(dead_code)]
    async fn convert_to_template_files(
        &self,
        template_id: i64,
        file_tree: Vec<FileTreeNode>,
    ) -> Result<Vec<TemplateFile>, AppError> {
        let mut result = Vec::new();
        let template_root_path = self.base_path.join(template_id.to_string());

        // 加载条件配置
        let conditions = self.load_conditions(template_id).await?;

        // 递归扁平化文件树
        self.flatten_file_tree(&template_root_path, &file_tree, &mut result, &conditions)
            .await?;

        Ok(result)
    }

    /// 扁平化文件树并记录文件修改时间（用于缓存版本管理）
    async fn convert_to_template_files_with_mtime(
        &self,
        template_id: i64,
        file_tree: Vec<FileTreeNode>,
        cache: &mut tokio::sync::MutexGuard<'_, crate::cache::DependencyTreeCache>,
    ) -> Result<Vec<TemplateFile>, AppError> {
        let mut result = Vec::new();
        let template_root_path = self.base_path.join(template_id.to_string());

        // 加载条件配置
        let conditions = self.load_conditions_with_mtime(template_id, cache).await?;

        // 递归扁平化文件树（记录 mtime）
        self.flatten_file_tree_with_mtime(
            &template_root_path,
            &file_tree,
            &mut result,
            &conditions,
            template_id,
            cache,
        )
        .await?;

        Ok(result)
    }

    /// 递归扁平化文件树（记录文件修改时间）
    async fn flatten_file_tree_with_mtime(
        &self,
        base_path: &PathBuf,
        nodes: &[FileTreeNode],
        result: &mut Vec<TemplateFile>,
        conditions: &ConditionsYaml,
        template_id: i64,
        cache: &mut tokio::sync::MutexGuard<'_, crate::cache::DependencyTreeCache>,
    ) -> Result<(), AppError> {
        for node in nodes {
            // 添加当前节点
            let file_path = base_path.join(&node.file_path);

            // 记录文件修改时间
            if let Ok(mtime) = fs::metadata(&file_path).await.and_then(|m| m.modified()) {
                cache.record_file_mtime(template_id, &node.file_path, mtime);
            }

            // 读取文件内容（仅文件）
            let file_content = if node.is_directory == 0 {
                fs::read_to_string(&file_path).await.unwrap_or_else(|e| {
                    error!("读取文件失败: {:?}, error: {}", file_path, e);
                    String::new()
                })
            } else {
                String::new()
            };

            // 获取文件条件
            let condition = conditions.get_condition_by_path(&node.file_path);

            let template_file = TemplateFile {
                id: node.id,
                file_path: node.file_path.clone(),
                file_name: node.file_name.clone(),
                file_content,
                is_directory: node.is_directory,
                parent_id: node.parent_id,
                filesize: node.file_size as i32,
                extends: None,
                includes: None,
                imports: None,
                condition,
                is_dependency: false,
                required_by: None,
            };

            result.push(template_file);

            // 如果有子节点，递归处理
            if let Some(ref children) = node.children {
                Box::pin(self.flatten_file_tree_with_mtime(
                    base_path,
                    children,
                    result,
                    conditions,
                    template_id,
                    cache,
                ))
                .await?;
            }
        }

        Ok(())
    }

    /// 加载条件配置并记录修改时间
    async fn load_conditions_with_mtime(
        &self,
        template_id: i64,
        cache: &mut tokio::sync::MutexGuard<'_, crate::cache::DependencyTreeCache>,
    ) -> Result<ConditionsYaml, AppError> {
        let conditions_path = self
            .base_path
            .join(template_id.to_string())
            .join(".meta/variables/conditions.yml");

        if !conditions_path.exists() {
            return Ok(ConditionsYaml::new());
        }

        // 记录条件文件修改时间
        if let Ok(mtime) = fs::metadata(&conditions_path)
            .await
            .and_then(|m| m.modified())
        {
            cache.record_condition_mtime(template_id, mtime);
        }

        let content = fs::read_to_string(&conditions_path)
            .await
            .map_err(|e| AppError::Internal(format!("读取条件文件失败: {}", e)))?;

        ConditionsYaml::from_yaml(&content)
            .map_err(|e| AppError::Internal(format!("解析条件文件失败: {}", e)))
    }
}

/// 渲染错误详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub context: Option<String>,
    pub suggestion: Option<String>,
}

/// 渲染结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderResult {
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "fileContent")]
    pub file_content: String,
    pub variables: serde_json::Value,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RenderError>,
}

/// 渲染后的文件信息（用于文件树响应）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedFileInfo {
    pub id: i64,

    /// 渲染后的文件路径
    #[serde(rename = "filePath")]
    pub file_path: String,

    /// 渲染后的文件名
    #[serde(rename = "fileName")]
    pub file_name: String,

    /// 渲染后的文件内容（仅文件节点有值）
    #[serde(rename = "fileContent", skip_serializing_if = "Option::is_none")]
    pub file_content: Option<String>,

    /// 是否是目录
    #[serde(rename = "isDirectory")]
    pub is_directory: i32,

    /// 文件大小（字节）
    pub filesize: i32,

    /// 父节点ID
    #[serde(rename = "parentId")]
    pub parent_id: i64,

    /// 子节点列表（仅目录节点有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<RenderedFileInfo>>,

    /// 渲染错误信息（仅当渲染失败时有值）
    #[serde(rename = "renderError", skip_serializing_if = "Option::is_none")]
    pub render_error: Option<RenderError>,
}

/// 渲染文件树响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderFileTreeResponse {
    #[serde(rename = "templateId")]
    pub template_id: i64,

    /// 渲染后的文件树（扁平结构）
    pub tree: Vec<RenderedFileInfo>,

    /// 使用的变量
    pub variables: serde_json::Value,

    /// 统计信息
    #[serde(rename = "totalFiles")]
    pub total_files: i32,

    #[serde(rename = "totalSize")]
    pub total_size: i64,

    /// 渲染失败的文件数
    #[serde(rename = "failedFiles")]
    pub failed_files: i32,
}
