use crate::config::storage::StorageManager;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use template_studio_shared::models::file_tree::{FileTreeNode, FileTreeResponse};
use tracing::{info, debug};

/// 文件树服务
pub struct FileTreeService {
    storage_manager: Arc<StorageManager>,
}

impl FileTreeService {
    pub fn new(storage_manager: Arc<StorageManager>) -> Self {
        Self { storage_manager }
    }

    /// 获取模板文件树
    pub async fn get_template_file_tree(&self, template_id: i64) -> Result<FileTreeResponse> {
        let template_path = self.storage_manager.get_template_path(template_id);

        info!("开始扫描模板文件树: template_id={}, path={:?}", template_id, template_path);

        // 检查模板目录是否存在
        if !template_path.exists() {
            return Err(anyhow::anyhow!("模板目录不存在: {:?}", template_path));
        }

        // 用于生成唯一ID的计数器
        let mut id_counter = 1i64;
        let mut tree = Vec::new();

        // 递归构建文件树
        self.build_file_tree(
            &template_path,
            &template_path,
            0, // parentId 0 表示根级
            &mut id_counter,
            &mut tree,
        ).await?;

        info!("文件树扫描完成: template_id={}, 总文件/目录数={}", template_id, id_counter - 1);

        Ok(FileTreeResponse { tree })
    }

    /// 递归构建文件树
    async fn build_file_tree(
        &self,
        base_path: &PathBuf,
        current_path: &PathBuf,
        parent_id: i64,
        id_counter: &mut i64,
        nodes: &mut Vec<FileTreeNode>,
    ) -> Result<()> {
        let mut entries = tokio::fs::read_dir(current_path)
            .await
            .with_context(|| format!("无法读取目录: {:?}", current_path))?;

        let mut entry_nodes = Vec::new();

        // 使用 while let 循环遍历目录项
        while let Some(entry) = entries.next_entry().await
            .with_context(|| format!("无法读取目录项: {:?}", current_path))? {
            let entry_type = entry.file_type().await?;
            let file_name = entry.file_name().into_string()
                .map_err(|_| anyhow::anyhow!("无效的文件名: {:?}", entry.file_name()))?;
            let file_path = entry.path();

            // 跳过 .git 和 .meta 目录
            if file_name == ".git" || file_name == ".meta" {
                debug!("跳过目录: {}", file_name);
                continue;
            }

            let is_directory = entry_type.is_dir();
            let file_size = if !is_directory {
                self.get_file_size(&file_path).await?
            } else {
                0
            };
            let md5 = if !is_directory {
                self.calculate_md5(&file_path).await.unwrap_or_default()
            } else {
                String::new()
            };

            // 生成相对路径
            let relative_path = file_path
                .strip_prefix(base_path)?
                .to_string_lossy()
                .to_string();

            // 创建节点
            let id = *id_counter;
            *id_counter += 1;

            let mut node = FileTreeNode {
                id,
                file_path: relative_path.clone(),
                file_name: file_name.clone(),
                is_directory: if is_directory { 1 } else { 0 },
                parent_id,
                file_size,
                md5,
                children: None,
                has_condition: false,
                condition_summary: None,
            };

            // 如果是目录，递归处理子项
            if is_directory {
                let mut children = Vec::new();
                Box::pin(self.build_file_tree(
                    base_path,
                    &file_path,
                    id,
                    id_counter,
                    &mut children,
                )).await?;
                node.children = Some(children);
            }

            entry_nodes.push(node);
        }

        // 排序：目录在前，文件在后，同类型按名称排序
        entry_nodes.sort_by(|a, b| {
            // 首先按 isDirectory 降序排序（目录在前）
            match b.is_directory.cmp(&a.is_directory) {
                std::cmp::Ordering::Equal => {
                    // 相同类型按文件名升序排序
                    a.file_name.cmp(&b.file_name)
                }
                other => other,
            }
        });

        nodes.extend(entry_nodes);

        Ok(())
    }

    /// 获取文件大小
    async fn get_file_size(&self, path: &PathBuf) -> Result<i64> {
        let metadata = tokio::fs::metadata(path).await?;
        Ok(metadata.len() as i64)
    }

    /// 计算文件MD5
    async fn calculate_md5(&self, path: &PathBuf) -> Result<String> {
        use tokio::io::AsyncReadExt;

        let mut file = tokio::fs::File::open(path).await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;

        Ok(format!("{:x}", md5::compute(contents)))
    }
}
