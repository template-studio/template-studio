use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

#[derive(Debug)]
pub enum DirectoryExistsAction {
    Cancel,
    Overwrite,
}

pub struct ProjectGenerator {
    output_dir: String,
    force: bool,
    template_path: Option<PathBuf>,
}

impl ProjectGenerator {
    pub fn new(output_dir: &str, force: bool) -> Self {
        Self {
            output_dir: output_dir.to_string(),
            force,
            template_path: None,
        }
    }

    /// 设置模板路径（用于二进制文件复制）
    pub fn with_template_path(mut self, template_path: PathBuf) -> Self {
        self.template_path = Some(template_path);
        self
    }

    /// 检查目录是否存在，返回存在时的处理建议
    pub fn check_directory_exists(&self, project_name: &str) -> Option<DirectoryExistsAction> {
        let project_path = Path::new(&self.output_dir).join(project_name);

        if project_path.exists() {
            if self.force {
                Some(DirectoryExistsAction::Overwrite)
            } else {
                Some(DirectoryExistsAction::Cancel)
            }
        } else {
            None
        }
    }

    pub fn generate(&self, project_name: &str, files: &[super::client::RenderedFile]) -> Result<()> {
        let project_path = Path::new(&self.output_dir).join(project_name);

        // 检查目录是否已存在
        if project_path.exists() {
            if self.force {
                info!("删除已存在的目录: {:?}", project_path);
                fs::remove_dir_all(&project_path)
                    .context("删除已存在目录失败")?;
            } else {
                anyhow::bail!("目录已存在: {:?} (使用 --force 强制覆盖)", project_path);
            }
        }

        // 创建项目目录
        fs::create_dir_all(&project_path)
            .context("创建项目目录失败")?;

        // 写入文件
        let mut success_count = 0;
        let mut binary_count = 0;
        let mut error_count = 0;

        for file in files {
            let file_path = project_path.join(&file.path);

            if file.is_directory {
                fs::create_dir_all(&file_path)
                    .with_context(|| format!("创建目录失败: {:?}", file_path))?;
            } else {
                // 确保父目录存在
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("创建父目录失败: {:?}", parent))?;
                }

                // 检查文件内容
                if !file.content.is_empty() {
                    // 有内容：文本文件，写入渲染后的内容
                    fs::write(&file_path, &file.content)
                        .with_context(|| format!("写入文件失败: {:?}", file_path))?;

                    success_count += 1;
                    info!("创建文件: {}", file.path);
                } else {
                    // 内容为空：二进制文件，从模板路径复制原始文件
                    if let Some(ref template_path) = self.template_path {
                        let source_path = template_path.join(&file.path);

                        match fs::copy(&source_path, &file_path) {
                            Ok(_) => {
                                binary_count += 1;
                                info!("复制文件: {}", file.path);
                            }
                            Err(e) => {
                                error_count += 1;
                                warn!("复制文件失败 {:?} -> {:?}: {}", source_path, file_path, e);
                            }
                        }
                    } else {
                        // 没有模板路径，创建空文件
                        fs::write(&file_path, &file.content)
                            .with_context(|| format!("写入文件失败: {:?}", file_path))?;
                        success_count += 1;
                        warn!("创建空文件: {} (无模板路径)", file.path);
                    }
                }
            }
        }

        info!("✅ 项目生成成功: {}", project_name);
        info!("   文本文件: {} 个", success_count);
        if binary_count > 0 {
            info!("   二进制文件: {} 个（已复制）", binary_count);
        }
        if error_count > 0 {
            warn!("   失败: {} 个文件", error_count);
        }

        Ok(())
    }

    pub fn git_init(&self, project_name: &str) -> Result<()> {
        let project_path = Path::new(&self.output_dir).join(project_name);

        // 检查 git 是否可用
        let git_check = Command::new("git")
            .arg("--version")
            .output();

        match git_check {
            Ok(output) => {
                if output.status.success() {
                    info!("检测到 Git，正在初始化仓库...");

                    let init_output = Command::new("git")
                        .current_dir(&project_path)
                        .args(["init"])
                        .output()
                        .context("执行 git init 失败")?;

                    if !init_output.status.success() {
                        let error = String::from_utf8_lossy(&init_output.stderr);
                        warn!("Git 初始化失败: {}", error);
                        return Ok(()); // 不中断流程，静默返回
                    }

                    info!("✅ Git 仓库初始化完成");
                } else {
                    info!("Git 命令执行失败，跳过 Git 初始化");
                }
                Ok(())
            }
            Err(_) => {
                info!("Git 未安装或不在 PATH 中，跳过 Git 初始化");
                Ok(())
            }
        }
    }
}
