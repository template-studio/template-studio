//! 模板版本发布服务
//!
//! 提供模板版本管理功能，包括创建发布、版本列表查询、版本回滚等。

use anyhow::Result;
use chrono::Utc;
use sqlx::{MySql, Pool};
use std::path::PathBuf;
use std::sync::Arc;
use template_studio_infrastructure::config::storage::StorageManager;
use template_studio_shared::models::release::*;
use tracing::{info, warn};

/// 发布服务
pub struct ReleaseService {
    storage_manager: Arc<StorageManager>,
    db: Pool<MySql>,
}

impl ReleaseService {
    pub fn new(storage_manager: Arc<StorageManager>, db: Pool<MySql>) -> Self {
        Self {
            storage_manager,
            db,
        }
    }

    /// 创建发布版本
    pub async fn create_release(
        &self,
        template_id: i64,
        req: CreateReleaseRequest,
        creator_id: i64,
        creator_name: String,
    ) -> Result<CreateReleaseResponse> {
        info!("创建发布版本: template_id={:?}", template_id);

        // 1. 确定版本号
        let version = if let Some(v) = &req.version {
            // 手动指定版本号，验证是否不低于当前版本
            let latest_version = self.get_latest_version(template_id).await.ok();
            if let Some(latest) = &latest_version {
                if !self.is_version_valid(v, latest)? {
                    return Err(anyhow::anyhow!("版本号不能低于当前版本 {}", latest));
                }
            }
            v.clone()
        } else {
            // 自动生成版本号
            self.generate_next_version(template_id).await?
        };

        info!("使用版本号: {}", version);

        // 2. Git 提交和打标签
        let template_path = self.storage_manager.get_template_path(template_id);
        let commit_hash = self
            .git_commit_and_tag(&template_path, &version, req.message.as_deref())
            .await?;

        info!("Git commit: {}", commit_hash);

        // 3. 创建发布快照（version 经存储层路径校验）
        let release_path = self
            .storage_manager
            .get_release_path(template_id, &version)?;
        self.clone_to_release(&template_path, &release_path).await?;
        self.remove_git_dir(&release_path).await?;

        info!("发布快照已创建: {:?}", release_path);

        // 4. 收集文件信息
        let (file_count, total_size) = self.collect_file_info(&release_path).await?;

        info!("文件统计: count={}, size={}", file_count, total_size);

        // 5. 写入发布信息文件（使用 JSON 格式）
        let release_info_json = serde_json::to_string_pretty(&serde_json::json!({
            "version": version,
            "templateId": template_id,
            "createdAt": Utc::now().to_rfc3339(),
            "commitHash": commit_hash,
            "commitMessage": req.message,
            "creatorId": creator_id,
            "creatorName": creator_name,
            "isLatest": true,
            "isDeprecated": false,
            "fileCount": file_count,
            "totalSize": total_size,
            "changelog": req.changelog,
        }))?;

        let meta_dir = release_path.join(".meta").join("release");
        tokio::fs::create_dir_all(&meta_dir)
            .await
            .map_err(|e| anyhow::anyhow!("创建元数据目录失败: {}", e))?;
        tokio::fs::write(meta_dir.join("release.json"), release_info_json)
            .await
            .map_err(|e| anyhow::anyhow!("写入发布信息文件失败: {}", e))?;

        // 6+7. 数据库更新放入同一事务：旧版本置 false 与新版本插入必须同成败，
        // 否则任一步失败会让模板处于「无 latest 版本」的损坏状态
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(|e| anyhow::anyhow!("开启事务失败: {}", e))?;

        // 将旧版本的 is_latest 设为 false
        sqlx::query("UPDATE template_versions SET is_latest = false WHERE template_id = ?")
            .bind(template_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow::anyhow!("更新旧版本状态失败: {}", e))?;

        // 插入新版本记录
        let version_id = sqlx::query(
            "INSERT INTO template_versions
            (template_id, version, commit_hash, commit_message, changelog, is_latest, is_deprecated,
             creator_id, creator_name, file_count, total_size, storage_path)
            VALUES (?, ?, ?, ?, ?, true, false, ?, ?, ?, ?, ?)",
        )
        .bind(template_id)
        .bind(&version)
        .bind(&commit_hash)
        .bind(&req.message)
        .bind(&req.changelog)
        .bind(creator_id)
        .bind(&creator_name)
        .bind(file_count)
        .bind(total_size)
        .bind(format!("releases/{}/{}", template_id, version))
        .execute(&mut *tx)
        .await
        .map_err(|e| anyhow::anyhow!("插入版本记录失败: {}", e))?
        .last_insert_id() as i64;

        tx.commit()
            .await
            .map_err(|e| anyhow::anyhow!("提交发布事务失败: {}", e))?;

        info!("发布版本创建成功: id={}, version={}", version_id, version);

        Ok(CreateReleaseResponse {
            id: version_id,
            version: version.clone(),
            commit_hash: Some(commit_hash),
            storage_path: format!("releases/{}/{}", template_id, version),
            is_latest: true,
            created_at: Utc::now(),
            file_count,
            total_size,
        })
    }

    /// 获取所有版本列表
    pub async fn list_versions(&self, template_id: i64) -> Result<Vec<TemplateVersion>> {
        let versions = sqlx::query_as::<_, TemplateVersion>(
            "SELECT * FROM template_versions
             WHERE template_id = ?
             ORDER BY created_at DESC",
        )
        .bind(template_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| anyhow::anyhow!("查询版本列表失败: {}", e))?;

        Ok(versions)
    }

    /// 获取当前版本（is_latest=true）
    pub async fn get_latest_version(&self, template_id: i64) -> Result<String> {
        let version = sqlx::query_as::<_, (String,)>(
            "SELECT version FROM template_versions
             WHERE template_id = ? AND is_latest = true",
        )
        .bind(template_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| anyhow::anyhow!("查询当前版本失败: {}", e))?
        .map(|(v,)| v);

        version.ok_or_else(|| anyhow::anyhow!("模板尚未发布任何版本"))
    }

    /// 回滚到指定版本
    pub async fn rollback_version(
        &self,
        template_id: i64,
        target_version: &str,
    ) -> Result<RollbackResponse> {
        info!(
            "回滚版本: template_id={}, target={}",
            template_id, target_version
        );

        // 1. 检查目标版本是否存在
        let target = sqlx::query_as::<_, (i64, String, bool)>(
            "SELECT id, version, is_latest FROM template_versions
             WHERE template_id = ? AND version = ?",
        )
        .bind(template_id)
        .bind(target_version)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| anyhow::anyhow!("查询目标版本失败: {}", e))?;

        let (target_id, _, is_current) =
            target.ok_or_else(|| anyhow::anyhow!("版本 {} 不存在", target_version))?;

        if is_current {
            return Err(anyhow::anyhow!("版本 {} 已经是当前版本", target_version));
        }

        // 2. 获取之前的当前版本
        let previous_version = sqlx::query_as::<_, (String,)>(
            "SELECT version FROM template_versions
             WHERE template_id = ? AND is_latest = true",
        )
        .bind(template_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| anyhow::anyhow!("查询当前版本失败: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("无法找到当前版本"))?
        .0;

        // 3+4. 两步 UPDATE 放入同一事务：第二步失败会让该模板所有版本
        // is_latest 均为 false（无可用版本的数据损坏状态）
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(|e| anyhow::anyhow!("开启事务失败: {}", e))?;

        // 将当前版本的 is_latest 设为 false
        sqlx::query(
            "UPDATE template_versions SET is_latest = false
             WHERE template_id = ? AND is_latest = true",
        )
        .bind(template_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| anyhow::anyhow!("更新当前版本状态失败: {}", e))?;

        // 将目标版本的 is_latest 设为 true
        sqlx::query(
            "UPDATE template_versions SET is_latest = true, is_deprecated = false
             WHERE id = ?",
        )
        .bind(target_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| anyhow::anyhow!("更新目标版本状态失败: {}", e))?;

        tx.commit()
            .await
            .map_err(|e| anyhow::anyhow!("提交回滚事务失败: {}", e))?;

        info!("回滚成功: {} -> {}", previous_version, target_version);

        Ok(RollbackResponse {
            previous_version,
            current_version: target_version.to_string(),
        })
    }

    /// 标记版本为已弃用
    pub async fn deprecate_version(&self, template_id: i64, version: &str) -> Result<()> {
        info!(
            "标记版本为弃用: template_id={}, version={}",
            template_id, version
        );

        let rows_affected = sqlx::query(
            "UPDATE template_versions SET is_deprecated = true
             WHERE template_id = ? AND version = ?",
        )
        .bind(template_id)
        .bind(version)
        .execute(&self.db)
        .await
        .map_err(|e| anyhow::anyhow!("更新版本状态失败: {}", e))?
        .rows_affected();

        if rows_affected == 0 {
            return Err(anyhow::anyhow!("版本 {} 不存在", version));
        }

        info!("版本已标记为弃用: {}", version);
        Ok(())
    }

    /// 重置到最新版本（类似 git restore .）
    /// 删除工作目录中的所有更改，恢复到最新发布版本的状态
    pub async fn reset_to_latest(&self, template_id: i64) -> Result<ResetToLatestResponse> {
        use std::process::Command;

        info!("重置到最新版本: template_id={}", template_id);

        // 1. 获取最新版本号
        let latest_version = self.get_latest_version(template_id).await?;
        info!("最新版本: {}", latest_version);

        // 2. 获取模板路径
        let template_path = self.storage_manager.get_template_path(template_id);

        // 3. 执行 git reset --hard <tag>
        let status = Command::new("git")
            .args(["reset", "--hard", &latest_version])
            .current_dir(&template_path)
            .status()
            .map_err(|e| anyhow::anyhow!("Git reset 失败: {}", e))?;

        if !status.success() {
            return Err(anyhow::anyhow!(
                "Git reset 失败，请确认版本 {} 的标签存在",
                latest_version
            ));
        }

        // 4. 清理未跟踪的文件（新增的文件）
        let output = Command::new("git")
            .args(["clean", "-fd"])
            .current_dir(&template_path)
            .output()
            .map_err(|e| anyhow::anyhow!("Git clean 失败: {}", e))?;

        let clean_output = String::from_utf8_lossy(&output.stdout);
        let deleted_count = clean_output.lines().filter(|l| !l.is_empty()).count() as i32;

        info!(
            "重置完成: 已恢复到版本 {}, 清理 {} 个未跟踪文件",
            latest_version, deleted_count
        );

        Ok(ResetToLatestResponse {
            version: latest_version,
            deleted_files: deleted_count,
        })
    }

    /// 自动生成下一个版本号
    async fn generate_next_version(&self, template_id: i64) -> Result<String> {
        let latest = self.get_latest_version(template_id).await.ok();

        let (major, minor, patch) = if let Some(v) = latest {
            let parts: Vec<u32> = v
                .trim_start_matches('v')
                .split('.')
                .map(|s| s.parse().unwrap_or(0))
                .collect();

            match parts.as_slice() {
                [major, minor, patch] => {
                    let (new_minor, new_patch) = if *patch >= 9 {
                        (*minor + 1, 0)
                    } else {
                        (*minor, *patch + 1)
                    };
                    (*major, new_minor, new_patch)
                }
                [major, minor] => (*major, *minor, 1),
                [major] => (*major, 0, 1),
                _ => (1, 0, 0),
            }
        } else {
            (1, 0, 0) // 第一个版本
        };

        Ok(format!("v{}.{}.{}", major, minor, patch))
    }

    /// 验证版本号是否有效（不低于当前版本）
    fn is_version_valid(&self, new_version: &str, current_version: &str) -> Result<bool> {
        let v1: Vec<u32> = new_version
            .trim_start_matches('v')
            .split('.')
            .map(|s| s.parse().unwrap_or(0))
            .collect();

        let v2: Vec<u32> = current_version
            .trim_start_matches('v')
            .split('.')
            .map(|s| s.parse().unwrap_or(0))
            .collect();

        // 比较主版本号
        if v1.is_empty() || v2.is_empty() {
            return Ok(false);
        }

        if v1[0] > v2[0] {
            return Ok(true);
        }
        if v1[0] < v2[0] {
            return Ok(false);
        }

        // 主版本相同，比较次版本号
        if v1.len() > 1 && v2.len() > 1 {
            if v1[1] > v2[1] {
                return Ok(true);
            }
            if v1[1] < v2[1] {
                return Ok(false);
            }
        }

        // 次版本相同，比较修订号
        if v1.len() > 2 && v2.len() > 2 {
            if v1[2] >= v2[2] {
                return Ok(true);
            }
            return Ok(false);
        }

        Ok(true)
    }

    /// Git 提交并打标签
    async fn git_commit_and_tag(
        &self,
        repo_path: &PathBuf,
        tag: &str,
        message: Option<&str>,
    ) -> Result<String> {
        use std::process::Command;

        // 1. 提交当前更改
        let default_msg = format!("Release {}", tag);
        let commit_msg = message.unwrap_or(&default_msg);

        let status = Command::new("git")
            .arg("add")
            .arg("-A")
            .current_dir(repo_path)
            .status()
            .map_err(|e| anyhow::anyhow!("Git add 失败: {}", e))?;

        if !status.success() {
            return Err(anyhow::anyhow!("Git add 失败"));
        }

        let status = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(commit_msg)
            .current_dir(repo_path)
            .status()
            .map_err(|e| anyhow::anyhow!("Git commit 失败: {}", e))?;

        if !status.success() {
            warn!("Git commit 没有更改（可能没有新提交）");
        }

        // 2. 获取 commit hash
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("HEAD")
            .current_dir(repo_path)
            .output()
            .map_err(|e| anyhow::anyhow!("获取 commit hash 失败: {}", e))?;

        let commit_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // 3. 创建标签
        let status = Command::new("git")
            .arg("tag")
            .arg("-a")
            .arg(tag)
            .arg("-m")
            .arg(&format!("Release {}", tag))
            .current_dir(repo_path)
            .status()
            .map_err(|e| anyhow::anyhow!("Git tag 创建失败: {}", e))?;

        if !status.success() {
            return Err(anyhow::anyhow!("Git tag 创建失败"));
        }

        Ok(commit_hash)
    }

    /// 克隆到发布目录
    async fn clone_to_release(&self, src: &PathBuf, dest: &PathBuf) -> Result<()> {
        use std::process::Command;

        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| anyhow::anyhow!("创建目录失败: {}", e))?;
        }

        let status = Command::new("git")
            .arg("clone")
            .arg(src)
            .arg(dest)
            .arg("--depth")
            .arg("1")
            .status()
            .map_err(|e| anyhow::anyhow!("Git clone 失败: {}", e))?;

        if !status.success() {
            return Err(anyhow::anyhow!("Git clone 失败"));
        }

        Ok(())
    }

    /// 删除 .git 目录
    async fn remove_git_dir(&self, path: &PathBuf) -> Result<()> {
        let git_dir = path.join(".git");
        if git_dir.exists() {
            tokio::fs::remove_dir_all(&git_dir)
                .await
                .map_err(|e| anyhow::anyhow!("删除 .git 目录失败: {}", e))?;
        }
        Ok(())
    }

    /// 收集文件信息
    async fn collect_file_info(&self, path: &PathBuf) -> Result<(i32, i64)> {
        let mut file_count = 0;
        let mut total_size = 0i64;

        let mut entries = tokio::fs::read_dir(path)
            .await
            .map_err(|e| anyhow::anyhow!("读取目录失败: {}", e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| anyhow::anyhow!("读取目录项失败: {}", e))?
        {
            let name = entry.file_name();
            // 跳过 .meta 目录
            if name == ".meta" {
                continue;
            }

            let entry_path = entry.path();
            if entry_path.is_file() {
                file_count += 1;
                let metadata = tokio::fs::metadata(&entry_path)
                    .await
                    .map_err(|e| anyhow::anyhow!("获取文件信息失败: {}", e))?;
                total_size += metadata.len() as i64;
            } else if entry_path.is_dir() {
                // TODO: 递归统计子目录
                let (sub_count, sub_size) = self.collect_file_info_recursive(&entry_path).await?;
                file_count += sub_count;
                total_size += sub_size;
            }
        }

        Ok((file_count, total_size))
    }

    /// 递归收集文件信息
    async fn collect_file_info_recursive(&self, path: &PathBuf) -> Result<(i32, i64)> {
        let mut file_count = 0;
        let mut total_size = 0i64;

        let mut entries = tokio::fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();

            if entry_path.is_file() {
                file_count += 1;
                let metadata = tokio::fs::metadata(&entry_path).await?;
                total_size += metadata.len() as i64;
            } else if entry_path.is_dir() {
                // 使用 Box::pin 避免递归异步函数的无限大小问题
                let (sub_count, sub_size) =
                    Box::pin(self.collect_file_info_recursive(&entry_path)).await?;
                file_count += sub_count;
                total_size += sub_size;
            }
        }

        Ok((file_count, total_size))
    }
}
