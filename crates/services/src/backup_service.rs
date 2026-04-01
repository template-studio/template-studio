//! 模板备份与恢复服务
//!
//! 提供模板的完整备份和恢复功能，使用专有的 .tsbk 格式（基于 ZIP），包含 SHA256 校验防止篡改

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tracing::{debug, info, warn};
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

use template_studio_infrastructure::config::storage::StorageManager;
use template_studio_shared::models::backup::{
    BackupFileCondition, BackupManifest, BackupTemplateInfo,
    BackupPreviewResponse, RestoreBackupResponse, RestoreStats, BACKUP_FORMAT_VERSION,
};
use template_studio_shared::utils::error::AppError;

use crate::file_conditions_service::FileConditionsService;
use crate::template_service::TemplateService;
use crate::template_variables_service::TemplateVariablesService;

/// 模板备份服务
pub struct BackupService {
    storage_manager: Arc<StorageManager>,
    template_service: Arc<TemplateService>,
    template_variables_service: Arc<TemplateVariablesService>,
    file_conditions_service: Arc<FileConditionsService>,
}

impl BackupService {
    pub fn new(
        storage_manager: Arc<StorageManager>,
        template_service: Arc<TemplateService>,
        template_variables_service: Arc<TemplateVariablesService>,
        file_conditions_service: Arc<FileConditionsService>,
    ) -> Self {
        Self {
            storage_manager,
            template_service,
            template_variables_service,
            file_conditions_service,
        }
    }

    /// 创建模板备份，返回 ZIP 文件的字节数据
    pub async fn create_backup(
        &self,
        template_id: i64,
        include_test_data: bool,
        include_conditions: bool,
    ) -> Result<Vec<u8>, AppError> {
        info!("Creating backup for template {}", template_id);

        // 1. 获取模板信息
        let template = self
            .template_service
            .get_template(template_id)
            .await
            .map_err(|e| AppError::Internal(format!("获取模板信息失败: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("模板 {} 不存在", template_id)))?;

        // 语言信息暂时使用空列表
        let backup_template_info = BackupTemplateInfo {
            id: template.id,
            name: template.name.clone(),
            description: template.description.clone(),
            category_id: template.category_id,
            template_type: template.template_type.clone(),
            type_config: template.type_config.clone(),
            introduction: template.introduction.clone(),
            is_featured: template.is_featured,
            logo: template.logo.clone(),
            icon: template.icon.clone(),
            git_repo_path: if template.git_repo_path.is_empty() { None } else { Some(template.git_repo_path.clone()) },
            current_version: if template.current_version.is_empty() { None } else { Some(template.current_version.clone()) },
            languages: Vec::new(), // 暂时不保存语言信息
        };

        // 2. 收集所有文件
        let template_path = self.storage_manager.get_template_path(template_id);
        let mut files: HashMap<String, String> = HashMap::new();

        if template_path.exists() {
            self.collect_files(&template_path, &template_path, &mut files)
                .await?;
        }

        debug!("Collected {} files for backup", files.len());

        // 3. 获取变量定义
        let variables = self
            .template_variables_service
            .get_variables(template_id)
            .await
            .ok();

        // 4. 获取测试数据
        let test_data = if include_test_data {
            self.template_variables_service
                .get_test_data(template_id)
                .await
                .ok()
        } else {
            None
        };

        // 5. 获取文件条件
        let conditions = if include_conditions {
            self.collect_conditions(template_id).await?
        } else {
            Vec::new()
        };

        // 6. 计算校验和
        let (manifest, files_checksum) = self
            .calculate_checksums(
                template_id,
                &template.name,
                &backup_template_info,
                &variables,
                &conditions,
                &test_data,
                &files,
            )
            .await?;

        // 7. 创建 ZIP 文件
        let zip_data = self
            .create_zip_archive(
                &manifest,
                &backup_template_info,
                &variables,
                &conditions,
                &test_data,
                &files,
                &files_checksum,
            )
            .await?;

        info!(
            "Backup created for template {}, size: {} bytes",
            template_id,
            zip_data.len()
        );

        Ok(zip_data)
    }

    /// 收集目录中的所有文件
    async fn collect_files(
        &self,
        base_path: &Path,
        current_path: &Path,
        files: &mut HashMap<String, String>,
    ) -> Result<(), AppError> {
        let mut entries = fs::read_dir(current_path)
            .await
            .map_err(|e| AppError::Internal(format!("读取目录失败: {}", e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| AppError::Internal(format!("读取目录项失败: {}", e)))?
        {
            let path = entry.path();

            // 跳过 .git 目录
            if path.file_name().map(|n| n == ".git").unwrap_or(false) {
                continue;
            }

            if path.is_dir() {
                Box::pin(self.collect_files(base_path, &path, files)).await?;
            } else {
                // 读取文件内容
                let mut content = String::new();
                let mut file = fs::File::open(&path)
                    .await
                    .map_err(|e| AppError::Internal(format!("打开文件失败 {:?}: {}", path, e)))?;

                // 尝试以 UTF-8 读取，如果失败则跳过（二进制文件）
                match file.read_to_string(&mut content).await {
                    Ok(_) => {
                        // 计算相对路径
                        let relative_path = path
                            .strip_prefix(base_path)
                            .map_err(|e| AppError::Internal(format!("计算相对路径失败: {}", e)))?
                            .to_string_lossy()
                            .replace('\\', "/");

                        files.insert(relative_path, content);
                    }
                    Err(_) => {
                        // 跳过二进制文件
                        warn!("Skipping binary file: {:?}", path);
                    }
                }
            }
        }

        Ok(())
    }

    /// 收集文件条件
    async fn collect_conditions(
        &self,
        template_id: i64,
    ) -> Result<Vec<BackupFileCondition>, AppError> {
        let summary = self
            .file_conditions_service
            .get_conditions_summary(template_id)
            .await?;

        let mut conditions = Vec::new();
        for (file_path, condition_str) in summary {
            conditions.push(BackupFileCondition {
                file_path,
                condition: Some(condition_str),
                condition_type: Some("yaml".to_string()),
            });
        }

        Ok(conditions)
    }

    /// 计算校验和
    async fn calculate_checksums(
        &self,
        template_id: i64,
        template_name: &str,
        template_info: &BackupTemplateInfo,
        variables: &Option<String>,
        conditions: &[BackupFileCondition],
        test_data: &Option<String>,
        files: &HashMap<String, String>,
    ) -> Result<(BackupManifest, HashMap<String, String>), AppError> {
        // 计算每个文件的校验和
        let mut files_checksum = HashMap::new();
        for (path, content) in files {
            let checksum = self.calculate_sha256(content);
            files_checksum.insert(path.clone(), checksum);
        }

        // 计算整体校验和
        let combined_content = format!(
            "{}{}{}{}{}",
            serde_json::to_string(template_info).unwrap_or_default(),
            variables.as_deref().unwrap_or(""),
            serde_json::to_string(conditions).unwrap_or_default(),
            test_data.as_deref().unwrap_or(""),
            serde_json::to_string(files).unwrap_or_default(),
        );
        let overall_checksum = self.calculate_sha256(&combined_content);

        let manifest = BackupManifest {
            version: BACKUP_FORMAT_VERSION.to_string(),
            format: "template-studio-backup".to_string(),
            created_at: chrono::Utc::now(),
            template_id,
            template_name: template_name.to_string(),
            checksum: overall_checksum,
            files_checksum: files_checksum.clone(),
        };

        Ok((manifest, files_checksum))
    }

    /// 计算 SHA256 校验和
    fn calculate_sha256(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        format!("sha256:{}", hex::encode(result))
    }

    /// 创建 ZIP 归档
    async fn create_zip_archive(
        &self,
        manifest: &BackupManifest,
        template_info: &BackupTemplateInfo,
        variables: &Option<String>,
        conditions: &[BackupFileCondition],
        test_data: &Option<String>,
        files: &HashMap<String, String>,
        files_checksum: &HashMap<String, String>,
    ) -> Result<Vec<u8>, AppError> {
        let buffer = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(buffer);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // 添加 manifest.json
        let manifest_json = serde_json::to_string_pretty(manifest)
            .map_err(|e| AppError::Internal(format!("序列化 manifest 失败: {}", e)))?;
        zip.start_file("manifest.json", options)
            .map_err(|e| AppError::Internal(format!("写入 ZIP 失败: {}", e)))?;
        zip.write_all(manifest_json.as_bytes())
            .map_err(|e| AppError::Internal(format!("写入 ZIP 失败: {}", e)))?;

        // 添加 template.json
        let template_json = serde_json::to_string_pretty(template_info)
            .map_err(|e| AppError::Internal(format!("序列化 template 失败: {}", e)))?;
        zip.start_file("template.json", options)
            .map_err(|e| AppError::Internal(format!("写入 ZIP 失败: {}", e)))?;
        zip.write_all(template_json.as_bytes())
            .map_err(|e| AppError::Internal(format!("写入 ZIP 失败: {}", e)))?;

        // 添加 variables.json
        let vars_json = serde_json::to_string_pretty(&variables)
            .map_err(|e| AppError::Internal(format!("序列化 variables 失败: {}", e)))?;
        zip.start_file("variables.json", options)
            .map_err(|e| AppError::Internal(format!("写入 ZIP 失败: {}", e)))?;
        zip.write_all(vars_json.as_bytes())
            .map_err(|e| AppError::Internal(format!("写入 ZIP 失败: {}", e)))?;

        // 添加 conditions.json
        let conditions_json = serde_json::to_string_pretty(conditions)
            .map_err(|e| AppError::Internal(format!("序列化 conditions 失败: {}", e)))?;
        zip.start_file("conditions.json", options)
            .map_err(|e| AppError::Internal(format!("写入 ZIP 失败: {}", e)))?;
        zip.write_all(conditions_json.as_bytes())
            .map_err(|e| AppError::Internal(format!("写入 ZIP 失败: {}", e)))?;

        // 添加 testdata.json
        let testdata_json = serde_json::to_string_pretty(&test_data)
            .map_err(|e| AppError::Internal(format!("序列化 testdata 失败: {}", e)))?;
        zip.start_file("testdata.json", options)
            .map_err(|e| AppError::Internal(format!("写入 ZIP 失败: {}", e)))?;
        zip.write_all(testdata_json.as_bytes())
            .map_err(|e| AppError::Internal(format!("写入 ZIP 失败: {}", e)))?;

        // 添加文件
        for (path, content) in files {
            let zip_path = format!("files/{}", path);
            zip.start_file(&zip_path, options)
                .map_err(|e| AppError::Internal(format!("写入 ZIP 失败: {}", e)))?;
            zip.write_all(content.as_bytes())
                .map_err(|e| AppError::Internal(format!("写入 ZIP 失败: {}", e)))?;
        }

        // 添加 .checksum
        let checksum_json = serde_json::to_string_pretty(files_checksum)
            .map_err(|e| AppError::Internal(format!("序列化 checksum 失败: {}", e)))?;
        zip.start_file(".checksum", options)
            .map_err(|e| AppError::Internal(format!("写入 ZIP 失败: {}", e)))?;
        zip.write_all(checksum_json.as_bytes())
            .map_err(|e| AppError::Internal(format!("写入 ZIP 失败: {}", e)))?;

        // finish() 消费 ZipWriter 并返回底层的 writer
        let buffer = zip.finish()
            .map_err(|e| AppError::Internal(format!("完成 ZIP 失败: {}", e)))?;

        Ok(buffer.into_inner())
    }

    /// 预览备份文件
    pub async fn preview_backup(&self, zip_data: &[u8]) -> Result<BackupPreviewResponse, AppError> {
        let zip_data_owned = zip_data.to_vec();

        // 在 spawn_blocking 中读取 ZIP 文件
        let preview_data = tokio::task::spawn_blocking(move || {
            Self::extract_preview_data(&zip_data_owned)
        })
        .await
        .map_err(|e| AppError::Internal(format!("任务执行失败: {}", e)))??;

        Ok(preview_data)
    }

    /// 从 ZIP 中提取预览数据（同步函数）
    fn extract_preview_data(zip_data: &[u8]) -> Result<BackupPreviewResponse, AppError> {
        let reader = Cursor::new(zip_data);
        let mut archive = ZipArchive::new(reader)
            .map_err(|e| AppError::Validation(format!("无效的备份文件: {}", e)))?;

        // 读取 manifest
        let manifest: BackupManifest = {
            let mut file = archive
                .by_name("manifest.json")
                .map_err(|e| AppError::Validation(format!("备份文件缺少 manifest.json: {}", e)))?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| AppError::Validation(format!("读取 manifest 失败: {}", e)))?;
            serde_json::from_str(&content)
                .map_err(|e| AppError::Validation(format!("解析 manifest 失败: {}", e)))?
        };

        // 验证格式
        if manifest.format != "template-studio-backup" {
            return Err(AppError::Validation("不支持的备份格式".to_string()));
        }

        // 读取 template.json
        let template_name = {
            let mut file = archive
                .by_name("template.json")
                .map_err(|_| AppError::Validation("备份文件缺少 template.json".to_string()))?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| AppError::Validation(format!("读取 template 失败: {}", e)))?;
            let info: BackupTemplateInfo = serde_json::from_str(&content)
                .map_err(|e| AppError::Validation(format!("解析 template 失败: {}", e)))?;
            info.name
        };

        // 检查是否有变量
        let has_variables = archive.by_name("variables.json").is_ok();
        let has_test_data = archive.by_name("testdata.json").is_ok();
        let has_conditions = archive.by_name("conditions.json").is_ok();

        // 收集文件路径并验证校验和
        let mut file_paths = Vec::new();
        let mut checksum_valid = true;

        for i in 0..archive.len() {
            let file = archive
                .by_index(i)
                .map_err(|e| AppError::Validation(format!("读取 ZIP 失败: {}", e)))?;
            let name = file.name().to_string();
            if name.starts_with("files/") && !file.is_dir() {
                let relative_path = name[6..].to_string();
                file_paths.push(relative_path);
            }
        }

        // 验证校验和
        for (path, expected_checksum) in &manifest.files_checksum {
            let zip_path = format!("files/{}", path);
            let mut file = match archive.by_name(&zip_path) {
                Ok(f) => f,
                Err(_) => {
                    warn!("File not found in backup: {}", path);
                    checksum_valid = false;
                    continue;
                }
            };

            let mut content = String::new();
            if file.read_to_string(&mut content).is_err() {
                checksum_valid = false;
                continue;
            }

            let actual_checksum = Self::calculate_sha256_static(&content);
            if actual_checksum != *expected_checksum {
                warn!("Checksum mismatch for file: {}", path);
                checksum_valid = false;
            }
        }

        Ok(BackupPreviewResponse {
            template_name,
            file_count: file_paths.len(),
            has_variables,
            has_test_data,
            has_conditions,
            file_paths,
            checksum_valid,
            manifest,
        })
    }

    /// 静态方法计算 SHA256（用于同步上下文）
    fn calculate_sha256_static(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        format!("sha256:{}", hex::encode(result))
    }

    /// 恢复备份
    pub async fn restore_backup(
        &self,
        template_id: i64,
        zip_data: &[u8],
    ) -> Result<RestoreBackupResponse, AppError> {
        info!("Restoring backup for template {}", template_id);

        // 将 ZIP 数据复制到 Vec 以便在 spawn_blocking 中使用
        let zip_data_owned = zip_data.to_vec();

        // 在 spawn_blocking 中读取 ZIP 文件
        let extracted = tokio::task::spawn_blocking(move || {
            Self::extract_backup_data(&zip_data_owned)
        })
        .await
        .map_err(|e| AppError::Internal(format!("任务执行失败: {}", e)))??;

        // 验证格式
        if extracted.manifest.format != "template-studio-backup" {
            return Err(AppError::Validation("不支持的备份格式".to_string()));
        }

        let mut stats = RestoreStats {
            files_restored: 0,
            variables_restored: 0,
            conditions_restored: 0,
            test_data_restored: false,
        };

        // 恢复文件
        stats.files_restored = self.restore_files(template_id, &extracted.files).await?;

        // 恢复变量定义
        if let Some(vars_content) = extracted.variables {
            if self
                .template_variables_service
                .save_variables(template_id, &vars_content)
                .await
                .is_ok()
            {
                stats.variables_restored = 1;
            }
        }

        // 恢复测试数据
        if let Some(test_content) = extracted.test_data {
            if self
                .template_variables_service
                .save_test_data(template_id, &test_content)
                .await
                .is_ok()
            {
                stats.test_data_restored = true;
            }
        }

        info!("Backup restored for template {}: {:?}", template_id, stats);

        Ok(RestoreBackupResponse {
            success: true,
            error: None,
            stats: Some(stats),
        })
    }

    /// 从 ZIP 数据中提取备份数据（同步函数）
    fn extract_backup_data(zip_data: &[u8]) -> Result<ExtractedBackupData, AppError> {
        let reader = Cursor::new(zip_data);
        let mut archive = ZipArchive::new(reader)
            .map_err(|e| AppError::Validation(format!("无效的备份文件: {}", e)))?;

        // 读取 manifest
        let manifest: BackupManifest = {
            let mut file = archive
                .by_name("manifest.json")
                .map_err(|e| AppError::Validation(format!("备份文件缺少 manifest.json: {}", e)))?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| AppError::Validation(format!("读取 manifest 失败: {}", e)))?;
            serde_json::from_str(&content)
                .map_err(|e| AppError::Validation(format!("解析 manifest 失败: {}", e)))?
        };

        // 读取变量定义
        let variables = if let Ok(mut file) = archive.by_name("variables.json") {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                if let Ok(vars) = serde_json::from_str::<Option<String>>(&content) {
                    vars
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // 读取测试数据
        let test_data = if let Ok(mut file) = archive.by_name("testdata.json") {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                if let Ok(data) = serde_json::from_str::<Option<String>>(&content) {
                    data
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // 读取所有文件
        let mut files = HashMap::new();
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| AppError::Internal(format!("读取 ZIP 失败: {}", e)))?;

            let name = file.name().to_string();
            if name.starts_with("files/") && !file.is_dir() {
                let relative_path = name[6..].to_string(); // 去掉 "files/" 前缀
                let mut content = String::new();
                file.read_to_string(&mut content)
                    .map_err(|e| AppError::Internal(format!("读取文件内容失败: {}", e)))?;
                files.insert(relative_path, content);
            }
        }

        Ok(ExtractedBackupData {
            manifest,
            variables,
            test_data,
            files,
        })
    }

    /// 恢复文件
    async fn restore_files(
        &self,
        template_id: i64,
        files: &HashMap<String, String>,
    ) -> Result<usize, AppError> {
        let template_path = self.storage_manager.get_template_path(template_id);
        let mut count = 0;

        for (relative_path, content) in files {
            let file_path = template_path.join(relative_path);

            // 确保父目录存在
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| AppError::Internal(format!("创建目录失败: {}", e)))?;
            }

            // 写入文件
            fs::write(&file_path, content)
                .await
                .map_err(|e| AppError::Internal(format!("写入文件失败: {}", e)))?;

            count += 1;
        }

        Ok(count)
    }
}

/// 从 ZIP 中提取的备份数据
struct ExtractedBackupData {
    manifest: BackupManifest,
    variables: Option<String>,
    test_data: Option<String>,
    files: HashMap<String, String>,
}
