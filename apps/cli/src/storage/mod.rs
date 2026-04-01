use crate::client::Template;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::fs;
use tracing::{info, debug};

/// 模板存储管理器
pub struct TemplateStorage {
    storage_dir: PathBuf,
}

impl TemplateStorage {
    /// 创建新的模板存储管理器
    pub fn new(storage_dir: PathBuf) -> Self {
        Self { storage_dir }
    }

    /// 获取模板文件路径
    fn get_template_path(&self, template_id: &str) -> PathBuf {
        self.storage_dir
            .join(format!("{}.json", template_id))
    }

    /// 保存模板到本地
    pub fn save_template(&self, template: &Template) -> Result<()> {
        // 确保存储目录存在
        fs::create_dir_all(&self.storage_dir)
            .context("创建模板存储目录失败")?;

        let template_path = self.get_template_path(&template.id);

        info!("保存模板: {} -> {:?}", template.id, template_path);

        // 序列化模板为JSON
        let template_json = serde_json::to_string_pretty(template)
            .context("序列化模板失败")?;

        // 写入文件
        fs::write(&template_path, template_json)
            .context("写入模板文件失败")?;

        debug!("模板保存成功: {}", template.id);
        Ok(())
    }

    /// 从本地加载模板
    pub fn load_template(&self, template_id: &str) -> Result<Template> {
        let template_path = self.get_template_path(template_id);

        debug!("从本地加载模板: {:?}", template_path);

        if !template_path.exists() {
            anyhow::bail!("模板不存在: {}", template_id);
        }

        let template_json = fs::read_to_string(&template_path)
            .context("读取模板文件失败")?;

        let template: Template = serde_json::from_str(&template_json)
            .context("解析模板失败")?;

        Ok(template)
    }

    /// 检查模板是否已缓存
    pub fn is_cached(&self, template_id: &str) -> bool {
        self.get_template_path(template_id).exists()
    }

    /// 删除缓存的模板
    pub fn delete_template(&self, template_id: &str) -> Result<()> {
        let template_path = self.get_template_path(template_id);

        if template_path.exists() {
            fs::remove_file(&template_path)
                .context("删除模板文件失败")?;
            info!("已删除缓存的模板: {}", template_id);
        }

        Ok(())
    }

    /// 列出所有缓存的模板ID
    pub fn list_cached_templates(&self) -> Result<Vec<String>> {
        if !self.storage_dir.exists() {
            return Ok(Vec::new());
        }

        let mut template_ids = Vec::new();

        for entry in fs::read_dir(&self.storage_dir)
            .context("读取模板目录失败")?
        {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    template_ids.push(file_stem.to_string());
                }
            }
        }

        template_ids.sort();
        Ok(template_ids)
    }
}
