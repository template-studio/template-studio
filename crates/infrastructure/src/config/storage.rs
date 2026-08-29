use crate::config::settings::StorageConfig;
use std::path::PathBuf;
use template_studio_shared::utils::path::validate_relative_path;

/// 存储管理器
pub struct StorageManager {
    config: StorageConfig,
}

impl StorageManager {
    pub fn new(config: StorageConfig) -> Self {
        Self { config }
    }

    /// 获取模板存储基础路径
    pub fn get_templates_base_path(&self) -> PathBuf {
        self.config.base_path.join("templates")
    }

    /// 获取指定模板的存储路径
    pub fn get_template_path(&self, template_id: i64) -> PathBuf {
        self.get_templates_base_path().join(template_id.to_string())
    }

    /// 获取模板的Git仓库路径
    pub fn get_template_git_path(&self, template_id: i64) -> PathBuf {
        self.get_template_path(template_id).join(".git")
    }

    /// 获取模板的元数据路径
    pub fn get_template_meta_path(&self, template_id: i64) -> PathBuf {
        self.get_template_path(template_id).join(".meta")
    }

    /// 获取模板的源码路径
    pub fn get_template_src_path(&self, template_id: i64) -> PathBuf {
        self.get_template_path(template_id).join("src")
    }

    /// 获取版本发布存储基础路径
    pub fn get_releases_base_path(&self) -> PathBuf {
        self.config.base_path.join("releases")
    }

    /// 获取指定模板指定版本的发布路径
    ///
    /// version 来自客户端请求，必须经过路径校验防止穿越
    /// （../、绝对路径、盘符），这是所有按版本定位存储的统一咽喉点
    pub fn get_release_path(&self, template_id: i64, version: &str) -> anyhow::Result<PathBuf> {
        validate_relative_path(version)
            .map_err(|e| anyhow::anyhow!("非法版本号 {}: {}", version, e))?;
        Ok(self
            .get_releases_base_path()
            .join(template_id.to_string())
            .join(version.replace('\\', "/")))
    }

    /// 确保目录存在
    pub async fn ensure_dir_exists(&self, path: &PathBuf) -> anyhow::Result<()> {
        tokio::fs::create_dir_all(path).await?;
        Ok(())
    }

    /// 初始化模板存储目录结构
    pub async fn initialize_template_structure(&self, template_id: i64) -> anyhow::Result<()> {
        let template_path = self.get_template_path(template_id);
        let meta_path = self.get_template_meta_path(template_id);
        let meta_variables_path = self.get_template_meta_variables_path(template_id);
        let meta_subscribe_path = self.get_template_meta_subscribe_path(template_id);
        let variables_file_path = self.get_variables_file_path(template_id);
        let test_file_path = self.get_test_file_path(template_id);

        // 创建目录结构：模板根目录、.meta目录、.meta/variables、.meta/subscribe
        self.ensure_dir_exists(&template_path).await?;
        self.ensure_dir_exists(&meta_path).await?;
        self.ensure_dir_exists(&meta_variables_path).await?;
        self.ensure_dir_exists(&meta_subscribe_path).await?;

        // 创建空的 variables.json 和 test.json 文件（如果不存在）
        self.ensure_file_exists(&variables_file_path, "{}").await?; // 修复：变量schema应该是对象，不是数组
        self.ensure_file_exists(&test_file_path, "{}").await?;

        Ok(())
    }

    /// 获取模板的元数据变量路径
    pub fn get_template_meta_variables_path(&self, template_id: i64) -> PathBuf {
        self.get_template_meta_path(template_id).join("variables")
    }

    /// 获取模板的元数据订阅路径
    pub fn get_template_meta_subscribe_path(&self, template_id: i64) -> PathBuf {
        self.get_template_meta_path(template_id).join("subscribe")
    }

    /// 获取订阅文件路径
    pub fn get_subscribe_file_path(&self, template_id: i64, preset_id: u64) -> PathBuf {
        self.get_template_meta_subscribe_path(template_id)
            .join(format!("{}.json", preset_id))
    }

    /// 获取变量文件路径 (variables.json)
    pub fn get_variables_file_path(&self, template_id: i64) -> PathBuf {
        self.get_template_meta_variables_path(template_id)
            .join("variables.json")
    }

    /// 获取测试数据文件路径 (test.json)
    pub fn get_test_file_path(&self, template_id: i64) -> PathBuf {
        self.get_template_meta_variables_path(template_id)
            .join("test.json")
    }

    /// 确保文件存在，如果不存在则创建并写入默认内容
    pub async fn ensure_file_exists(
        &self,
        path: &PathBuf,
        default_content: &str,
    ) -> anyhow::Result<()> {
        if !path.exists() {
            tokio::fs::write(path, default_content).await?;
            tracing::debug!("创建文件: {:?}, 内容: {}", path, default_content);
        }
        Ok(())
    }

    /// 读取 JSON 文件内容（错误不携带完整路径，避免经 handler 透传给客户端）
    pub async fn read_json_file(&self, path: &PathBuf) -> anyhow::Result<String> {
        if !path.exists() {
            tracing::warn!("JSON 文件不存在: {:?}", path);
            return Err(anyhow::anyhow!("JSON 文件不存在"));
        }
        let content = tokio::fs::read_to_string(path).await?;
        Ok(content)
    }

    /// 写入 JSON 文件内容（格式化输出）
    pub async fn write_json_file(&self, path: &PathBuf, content: &str) -> anyhow::Result<()> {
        // 确保父目录存在
        if let Some(parent) = path.parent() {
            let parent_buf = parent.to_path_buf();
            self.ensure_dir_exists(&parent_buf).await?;
        }

        // 解析并格式化 JSON，确保有良好的缩进
        let formatted_content =
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
                // 格式化输出：2个空格缩进
                serde_json::to_string_pretty(&value)?
            } else {
                // 如果解析失败，使用原始内容
                content.to_string()
            };

        tokio::fs::write(path, formatted_content).await?;
        tracing::debug!("写入文件: {:?}", path);
        Ok(())
    }
}
