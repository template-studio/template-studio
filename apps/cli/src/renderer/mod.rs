use anyhow::{Context, Result};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use template_studio_template_core::{
    filter_files_by_conditions, render_tree, ConditionsYaml, TemplateFile, Variables,
};
use tracing::{error, info, warn};

/// 本地模板渲染器
pub struct LocalRenderer {
    template_path: PathBuf,
}

impl LocalRenderer {
    pub fn new(template_path: PathBuf) -> Self {
        Self { template_path }
    }

    /// 扫描模板目录，构建文件树
    fn scan_template_files(&self) -> Result<Vec<TemplateFile>> {
        let mut files = Vec::new();
        let mut id_counter = 1i64;

        // 递归扫描模板目录（排除 .meta 目录）
        self.scan_directory(&self.template_path, &mut files, &mut id_counter, 0)?;

        info!("✅ 扫描到 {} 个模板文件/目录", files.len());

        // 统计文件和目录数量
        let file_count = files.iter().filter(|f| f.is_directory == 0).count();
        let dir_count = files.iter().filter(|f| f.is_directory == 1).count();
        info!("   - 文件: {}", file_count);
        info!("   - 目录: {}", dir_count);

        Ok(files)
    }

    /// 递归扫描目录
    fn scan_directory(
        &self,
        dir: &Path,
        files: &mut Vec<TemplateFile>,
        id_counter: &mut i64,
        parent_id: i64,
    ) -> Result<()> {
        let entries = fs::read_dir(dir).context("读取目录失败")?;

        for entry in entries {
            let entry = entry.context("读取目录项失败")?;
            let path = entry.path();

            // 跳过 .meta 目录和 .git 目录
            if path.is_dir() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                if file_name == ".meta" || file_name == ".git" {
                    continue;
                }

                // 创建目录节点
                let relative_path = path
                    .strip_prefix(&self.template_path)
                    .context("计算相对路径失败")?;

                let dir_file = TemplateFile {
                    id: *id_counter,
                    file_path: relative_path.to_string_lossy().replace('\\', "/"),
                    file_name: file_name.to_string(),
                    file_content: String::new(),
                    is_directory: 1,
                    parent_id,
                    filesize: 0,
                    extends: None,
                    includes: None,
                    imports: None,
                    condition: None,
                    is_dependency: false,
                    required_by: None,
                };

                *id_counter += 1;
                let dir_id = dir_file.id;
                files.push(dir_file);

                // 递归扫描子目录
                self.scan_directory(&path, files, id_counter, dir_id)?;
            } else {
                // 读取文件
                let relative_path = path
                    .strip_prefix(&self.template_path)
                    .context("计算相对路径失败")?;

                // 检测二进制文件
                let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

                let is_binary = matches!(
                    extension,
                    "png"
                        | "jpg"
                        | "jpeg"
                        | "gif"
                        | "ico"
                        | "webp"
                        | "woff"
                        | "woff2"
                        | "ttf"
                        | "eot"
                        | "otf"
                        | "mp3"
                        | "mp4"
                        | "wav"
                        | "ogg"
                        | "webm"
                        | "pdf"
                        | "zip"
                        | "exe"
                        | "dll"
                        | "so"
                        | "bin"
                        | "dat"
                        | "db"
                        | "sqlite"
                        | "mdb"
                );

                let content = if is_binary {
                    info!("  [二进制文件] {} - 跳过内容读取", relative_path.display());
                    String::new()
                } else {
                    match fs::read_to_string(&path) {
                        Ok(c) => {
                            info!(
                                "  [文本文件] {} - {} 字节",
                                relative_path.display(),
                                c.len()
                            );
                            c
                        }
                        Err(e) => {
                            // 读取失败，可能是编码问题
                            warn!("  [警告] 读取文件失败 {:?}: {}, 将跳过该文件", path, e);
                            String::new()
                        }
                    }
                };

                let metadata = fs::metadata(&path).context("获取文件元数据失败")?;

                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                let file = TemplateFile {
                    id: *id_counter,
                    file_path: relative_path.to_string_lossy().replace('\\', "/"),
                    file_name,
                    file_content: content,
                    is_directory: 0,
                    parent_id,
                    filesize: metadata.len() as i32,
                    extends: None,
                    includes: None,
                    imports: None,
                    condition: None,
                    is_dependency: false,
                    required_by: None,
                };

                *id_counter += 1;
                files.push(file);
            }
        }

        Ok(())
    }

    /// 读取 .meta/variables/conditions.yml 并应用文件条件过滤
    ///
    /// 与 Web 服务端渲染语义保持一致：无条件配置文件时全部生成；
    /// 条件不满足的文件/目录及其子树被排除，不参与渲染
    fn apply_file_conditions(
        &self,
        files: Vec<TemplateFile>,
        variables: &Variables,
    ) -> Vec<TemplateFile> {
        let conditions_path = self
            .template_path
            .join(".meta")
            .join("variables")
            .join("conditions.yml");

        let Ok(content) = fs::read_to_string(&conditions_path) else {
            return files; // 无条件配置，全部生成
        };

        let yaml = match ConditionsYaml::from_yaml(&content) {
            Ok(y) => y,
            Err(e) => {
                warn!(
                    "解析 conditions.yml 失败 ({:?})：{}，将忽略文件条件",
                    conditions_path, e
                );
                return files;
            }
        };

        // conditions.yml 中的路径可能是 Windows 分隔符，统一按 / 规范化后匹配
        let normalize = |p: &str| p.replace('\\', "/");
        let mut files = files;
        let mut matched = 0usize;
        for fc in &yaml.conditions {
            let Some(cond) = &fc.condition else { continue };
            let target = normalize(&fc.path);
            for f in files.iter_mut() {
                if normalize(&f.file_path) == target {
                    f.condition = Some(cond.clone());
                    matched += 1;
                }
            }
        }
        if !yaml.conditions.is_empty() {
            info!(
                "文件条件：{} 条配置，命中 {} 个文件",
                yaml.conditions.len(),
                matched
            );
        }

        filter_files_by_conditions(files, variables)
    }

    /// 渲染模板
    pub fn render(
        &self,
        variables: &HashMap<String, JsonValue>,
    ) -> Result<Vec<crate::client::RenderedFile>> {
        info!("开始本地渲染模板...");

        // 1. 扫描模板文件
        let template_files = self.scan_template_files()?;

        // 2. 转换变量格式
        let variables_json = serde_json::to_string(variables).context("序列化变量失败")?;
        let render_vars = Variables::from_json(&variables_json)
            .map_err(|e| anyhow::anyhow!("创建渲染变量失败: {}", e))?;

        // 3. 应用文件生成条件过滤后渲染文件树
        let filtered_files = self.apply_file_conditions(template_files, &render_vars);
        let rendered_tree = render_tree(filtered_files, &render_vars).context("渲染文件树失败")?;

        info!("渲染完成，生成 {} 个文件节点", rendered_tree.len());

        // 4. 转换为 RenderedFile 格式并检查错误
        let mut rendered_files = Vec::new();
        let mut error_count = 0;

        for f in rendered_tree {
            if f.is_directory == 1 {
                // 目录
                rendered_files.push(crate::client::RenderedFile {
                    path: f.file_path.clone(),
                    content: String::new(),
                    is_directory: true,
                });
            } else if let Some(render_error) = f.error {
                // 渲染失败的文件
                error_count += 1;
                error!("❌ 渲染文件失败: {}", f.file_path);
                error!("   错误类型: {}", render_error.error_type);
                error!("   错误信息: {}", render_error.message);
                if let Some(line) = render_error.line {
                    error!("   错误行号: {}", line);
                }
                if let Some(context) = &render_error.context {
                    error!("   错误上下文: {}", context);
                }

                // 对于非关键文件，可以继续；对于关键文件，可能需要终止
                // 目前记录错误但继续处理
            } else if let Some(content) = f.file_content {
                // 渲染成功的文件
                rendered_files.push(crate::client::RenderedFile {
                    path: f.file_path.clone(),
                    content,
                    is_directory: false,
                });
            } else {
                // file_content 为 None 但没有 error（这种情况不太可能）
                warn!("⚠️  文件内容为空: {}", f.file_path);
            }
        }

        if error_count > 0 {
            warn!(
                "⚠️  共有 {} 个文件渲染失败，请检查日志获取详细信息",
                error_count
            );
            warn!("   日志文件位置: ~/.cicbyte/template_studio/logs/template-cli.log");
        }

        info!("✅ 成功渲染 {} 个文件", rendered_files.len());

        if error_count > 0 {
            // 如果有错误，仍然返回成功渲染的文件，但记录警告
            warn!("⚠️  部分文件渲染失败，项目可能不完整");
        }

        Ok(rendered_files)
    }
}
