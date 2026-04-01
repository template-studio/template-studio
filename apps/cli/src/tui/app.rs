use crate::client::{ApiClient, Template, TemplateVersion};
use crate::variables::VariableDefinition;
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{backend::Backend, Terminal};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use super::ui;

pub enum AppState {
    Welcome,
    ProjectName,
    OutputDir,
    TemplateSearch,
    TemplateSelect,
    VersionSelect,
    Variables,
    Complete,
}

pub struct App {
    pub state: AppState,
    pub client: ApiClient,
    #[allow(dead_code)]
    pub project_name: Option<String>,
    pub template_id: Option<String>,
    pub output_dir: String,
    pub force: bool,

    // 输入数据
    pub project_input: String,
    pub output_input: String,
    pub search_input: String,

    // 模板数据
    pub templates: Vec<Template>,
    pub filtered_templates: Vec<Template>,
    pub selected_template: Option<usize>,

    // 版本数据
    pub versions: Vec<TemplateVersion>,
    pub selected_version: Option<usize>,

    // 变量数据
    pub variable_definitions: Option<HashMap<String, VariableDefinition>>,
    pub variable_order: Vec<String>,  // 保存稳定的变量顺序
    pub variable_values: HashMap<String, serde_json::Value>,
    pub variable_input_index: usize,
    pub variable_input_buffer: String,

    // 下载的模板路径
    pub template_cache_path: Option<PathBuf>,

    // 错误信息
    pub error_message: Option<String>,

    // 确认对话框
    pub show_confirm_dialog: bool,
    pub confirm_message: String,

    // 退出标志
    pub should_quit: bool,
}

impl App {
    pub fn new(
        client: ApiClient,
        project_name: Option<String>,
        template_id: Option<String>,
        output_dir: &str,
        force: bool,
    ) -> Self {
        let project_input = project_name.clone().unwrap_or_default();
        Self {
            state: AppState::Welcome,
            client,
            project_name,
            template_id,
            output_dir: output_dir.to_string(),
            force,
            project_input,
            output_input: output_dir.to_string(),
            search_input: String::new(),
            templates: Vec::new(),
            filtered_templates: Vec::new(),
            selected_template: None,
            versions: Vec::new(),
            selected_version: None,
            variable_definitions: None,
            variable_order: Vec::new(),
            variable_values: HashMap::new(),
            variable_input_index: 0,
            variable_input_buffer: String::new(),
            template_cache_path: None,
            error_message: None,
            show_confirm_dialog: false,
            confirm_message: String::new(),
            should_quit: false,
        }
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let mut last_tick = std::time::Instant::now();
        let tick_rate = Duration::from_millis(250);

        loop {
            // 绘制UI
            terminal.draw(|f| ui::render_ui(f, self))?;

            // 处理事件
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    // 只处理按键按下事件，忽略释放和重复事件
                    if key.kind == KeyEventKind::Press {
                        self.handle_key_event(key).await?;
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = std::time::Instant::now();
            }

            if self.should_quit {
                return Ok(());
            }
        }
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        debug!("按键事件: {:?}", key);

        match self.state {
            AppState::Welcome => {
                match key.code {
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        self.state = AppState::ProjectName;
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.should_quit = true;
                    }
                    _ => {}
                }
            }

            AppState::ProjectName => {
                match key.code {
                    KeyCode::Enter => {
                        if self.project_input.trim().is_empty() {
                            self.error_message = Some("项目名称不能为空".to_string());
                        } else {
                            self.error_message = None;
                            self.state = AppState::OutputDir;
                        }
                    }
                    KeyCode::Char(c) => {
                        self.project_input.push(c);
                    }
                    KeyCode::Backspace => {
                        self.project_input.pop();
                    }
                    KeyCode::Esc => {
                        self.state = AppState::Welcome;
                    }
                    _ => {}
                }
            }

            AppState::OutputDir => {
                match key.code {
                    KeyCode::Enter => {
                        self.state = AppState::TemplateSearch;
                        self.load_templates().await?;
                    }
                    KeyCode::Char(c) => {
                        self.output_input.push(c);
                    }
                    KeyCode::Backspace => {
                        self.output_input.pop();
                    }
                    KeyCode::Esc => {
                        self.state = AppState::ProjectName;
                    }
                    _ => {}
                }
            }

            AppState::TemplateSearch => {
                match key.code {
                    KeyCode::Enter => {
                        if !self.search_input.trim().is_empty() {
                            self.filter_templates();
                            self.state = AppState::TemplateSelect;
                        }
                    }
                    KeyCode::Char(c) => {
                        self.search_input.push(c);
                    }
                    KeyCode::Backspace => {
                        self.search_input.pop();
                    }
                    KeyCode::Esc => {
                        self.state = AppState::OutputDir;
                    }
                    _ => {}
                }
            }

            AppState::TemplateSelect => {
                match key.code {
                    KeyCode::Enter => {
                        if let Some(idx) = self.selected_template {
                            if let Some(tmpl) = self.filtered_templates.get(idx) {
                                let template_id = tmpl.id.clone();
                                self.template_id = Some(template_id.clone());

                                // 加载版本列表
                                match self.load_versions(&template_id).await {
                                    Ok(_) => {
                                        // 进入版本选择
                                        self.state = AppState::VersionSelect;
                                    }
                                    Err(e) => {
                                        self.error_message = Some(format!("加载版本失败: {}", e));
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if let Some(idx) = self.selected_template {
                            if idx < self.filtered_templates.len().saturating_sub(1) {
                                self.selected_template = Some(idx + 1);
                            }
                        } else if !self.filtered_templates.is_empty() {
                            self.selected_template = Some(0);
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.selected_template = match self.selected_template {
                            Some(0) => None,
                            Some(idx) => Some(idx - 1),
                            None => None,
                        };
                    }
                    KeyCode::Esc => {
                        self.state = AppState::TemplateSearch;
                    }
                    KeyCode::Char('/') => {
                        self.state = AppState::TemplateSearch;
                    }
                    _ => {}
                }
            }

            AppState::VersionSelect => {
                match key.code {
                    KeyCode::Enter => {
                        // 确认选择版本，下载模板
                        if let Some(idx) = self.selected_version {
                            if let Some(version) = self.versions.get(idx).cloned() {
                                match self.download_template(&version).await {
                                    Ok(_) => {
                                        self.state = AppState::Variables;
                                    }
                                    Err(e) => {
                                        self.error_message = Some(format!("下载模板失败: {}", e));
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if let Some(idx) = self.selected_version {
                            if idx < self.versions.len().saturating_sub(1) {
                                self.selected_version = Some(idx + 1);
                            }
                        } else if !self.versions.is_empty() {
                            self.selected_version = Some(0);
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.selected_version = match self.selected_version {
                            Some(0) => None,
                            Some(idx) => Some(idx - 1),
                            None => None,
                        };
                    }
                    KeyCode::Esc => {
                        // 返回模板选择
                        self.state = AppState::TemplateSelect;
                    }
                    _ => {}
                }
            }

            AppState::Variables => {
                match key.code {
                    KeyCode::Enter => {
                        // 确认当前变量输入，移动到下一个
                        self.confirm_variable_input()?;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        // 移动到下一个变量
                        if self.variable_input_index < self.variable_order.len().saturating_sub(1) {
                            self.variable_input_index += 1;
                            self.load_current_variable_value();
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        // 移动到上一个变量
                        if self.variable_input_index > 0 {
                            self.variable_input_index -= 1;
                            self.load_current_variable_value();
                        }
                    }
                    KeyCode::Char(c) => {
                        // 输入字符
                        self.variable_input_buffer.push(c);
                    }
                    KeyCode::Backspace => {
                        // 删除字符
                        self.variable_input_buffer.pop();
                    }
                    KeyCode::Tab => {
                        // 跳过当前变量，移动到下一个
                        if self.variable_input_index < self.variable_order.len().saturating_sub(1) {
                            self.variable_input_index += 1;
                            self.load_current_variable_value();
                        } else {
                            // 最后一个变量，进入完成状态
                            self.state = AppState::Complete;
                        }
                    }
                    KeyCode::Esc => {
                        // 返回版本选择
                        self.state = AppState::VersionSelect;
                    }
                    _ => {}
                }
            }

            AppState::Complete => {
                // 如果显示确认对话框，优先处理确认对话框的按键
                if self.show_confirm_dialog {
                    match key.code {
                        KeyCode::Enter => {
                            // 确认覆盖
                            self.show_confirm_dialog = false;
                            self.do_generate_project().await?;
                            self.should_quit = true;
                        }
                        KeyCode::Esc => {
                            // 取消
                            self.show_confirm_dialog = false;
                            self.confirm_message.clear();
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Enter => {
                            self.generate_project().await?;
                            // 只有在没有显示确认对话框时才退出
                            if !self.show_confirm_dialog {
                                self.should_quit = true;
                            }
                        }
                        KeyCode::Char('q') | KeyCode::Esc => {
                            self.should_quit = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    async fn load_templates(&mut self) -> Result<()> {
        info!("加载模板列表...");

        match self.client.list_templates(None).await {
            Ok(templates) => {
                self.filtered_templates = templates.clone();
                self.templates = templates;
                info!("已加载 {} 个模板", self.templates.len());
                Ok(())
            }
            Err(e) => {
                error!("加载模板失败: {:?}", e);
                self.error_message = Some(format!("加载模板失败: {}", e));
                Err(e)
            }
        }
    }

    fn filter_templates(&mut self) {
        let search_lower = self.search_input.to_lowercase();
        self.filtered_templates = self.templates
            .iter()
            .filter(|t| {
                t.name.to_lowercase().contains(&search_lower)
                    || t.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&search_lower))
                        .unwrap_or(false)
            })
            .cloned()
            .collect();

        if !self.filtered_templates.is_empty() {
            self.selected_template = Some(0);
        } else {
            self.selected_template = None;
        }

        info!("搜索到 {} 个模板", self.filtered_templates.len());
    }

    async fn load_versions(&mut self, template_id: &str) -> Result<()> {
        info!("加载模板版本: {}", template_id);

        match self.client.get_template_versions(template_id).await {
            Ok(versions) => {
                self.versions = versions.clone();

                // 自动选择最新版本
                self.selected_version = versions.iter()
                    .position(|v| v.is_latest)
                    .or_else(|| if !versions.is_empty() { Some(0) } else { None });

                info!("已加载 {} 个版本", versions.len());
                Ok(())
            }
            Err(e) => {
                error!("加载版本失败: {:?}", e);
                Err(e)
            }
        }
    }

    async fn download_template(&mut self, version: &TemplateVersion) -> Result<()> {
        use std::path::PathBuf;
        use std::io::Cursor;

        // 确定本地缓存路径：{template_id}/{version}/
        let cache_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".ciclebyte")
            .join("template_studio_rust")
            .join("data")
            .join("templates")
            .join(version.template_id.to_string())
            .join(&version.version);

        // 检查缓存是否已存在
        if !cache_dir.exists() {
            // 需要下载模板
            info!("请等待模板下载完成");

            // 通过API下载模板zip
            let zip_bytes = self.client.download_template_version(
                &version.template_id.to_string(),
                &version.version
            ).await?;

            info!("ZIP文件大小: {} bytes", zip_bytes.len());

            // 创建缓存目录
            std::fs::create_dir_all(&cache_dir)
                .context("创建缓存目录失败")?;

            // 解压ZIP文件
            let cursor = Cursor::new(zip_bytes);
            let mut archive = zip::ZipArchive::new(cursor)
                .context("解析ZIP文件失败")?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i)
                    .context("读取ZIP条目失败")?;
                let file_path = cache_dir.join(file.enclosed_name()
                    .context("ZIP文件路径非法")?);

                // 创建父目录
                if let Some(parent) = file_path.parent() {
                    std::fs::create_dir_all(parent)
                        .context("创建目标目录失败")?;
                }

                if file.is_dir() {
                    std::fs::create_dir_all(&file_path)
                        .context("创建目录失败")?;
                } else {
                    let mut outfile = std::fs::File::create(&file_path)
                        .context("创建文件失败")?;
                    std::io::copy(&mut file, &mut outfile)
                        .context("写入文件失败")?;
                }
            }
        }

        // 保存模板缓存路径
        self.template_cache_path = Some(cache_dir.clone());

        // 加载变量定义
        use crate::variables::VariableCollector;
        let collector = VariableCollector::new(cache_dir);
        match collector.load_variable_definitions() {
            Ok(definitions) => {
                // 立即建立稳定的变量顺序（只排序一次）
                let mut sorted_vars: Vec<_> = definitions.iter().collect();

                self.variable_definitions = Some(definitions.clone());
                sorted_vars.sort_by(|a, b| {
                    // 先按分组排序
                    let group_cmp = a.1.ui.group.cmp(&b.1.ui.group);
                    if group_cmp != std::cmp::Ordering::Equal {
                        return group_cmp;
                    }
                    // 同组内按顺序排序
                    a.1.ui.order.cmp(&b.1.ui.order)
                });

                // 保存顺序
                self.variable_order = sorted_vars.into_iter().map(|(name, _)| name.clone()).collect();
                self.variable_definitions = Some(definitions);
            }
            Err(e) => {
                warn!("加载变量定义失败: {}", e);
                self.variable_definitions = Some(HashMap::new());
                self.variable_order = Vec::new();
            }
        }

        Ok(())
    }

    async fn generate_project(&mut self) -> Result<()> {
        let project_name = self.project_input.trim();
        let output_dir = if self.output_input.trim().is_empty() {
            &self.output_dir
        } else {
            &self.output_input
        };

        // 检查目录是否已存在
        use crate::generator::ProjectGenerator;
        let generator = ProjectGenerator::new(output_dir, self.force);

        if let Some(action) = generator.check_directory_exists(project_name) {
            match action {
                crate::generator::DirectoryExistsAction::Cancel => {
                    // 需要用户确认是否覆盖
                    self.show_confirm_dialog = true;
                    self.confirm_message = format!(
                        "目录 {} 已存在，是否覆盖？\n\nEnter: 确认覆盖\nEsc: 取消",
                        std::path::Path::new(output_dir).join(project_name).display()
                    );
                    return Ok(());
                }
                crate::generator::DirectoryExistsAction::Overwrite => {
                    // force 模式，静默覆盖
                }
            }
        }

        self.do_generate_project().await
    }

    async fn do_generate_project(&mut self) -> Result<()> {
        info!("生成项目...");

        let cache_path = self.template_cache_path.as_ref()
            .ok_or_else(|| anyhow::anyhow!("模板未下载"))?;
        let project_name = self.project_input.trim();
        let output_dir = if self.output_input.trim().is_empty() {
            &self.output_dir
        } else {
            &self.output_input
        };

        // 添加内置变量
        self.variable_values.insert("ProjectName".to_string(), serde_json::Value::String(project_name.to_string()));
        self.variable_values.insert("project_name".to_string(), serde_json::Value::String(project_name.to_lowercase()));
        self.variable_values.insert("PROJECT_NAME".to_string(), serde_json::Value::String(project_name.to_uppercase()));

        // 本地渲染
        use crate::renderer::LocalRenderer;
        let renderer = LocalRenderer::new(cache_path.clone());
        let rendered_files = renderer.render(&self.variable_values)?;

        // 生成项目（使用 force=true 允许覆盖）
        use crate::generator::ProjectGenerator;
        let generator = ProjectGenerator::new(output_dir, true)
            .with_template_path(cache_path.clone());
        generator.generate(project_name, &rendered_files)?;

        // 自动执行 Git 初始化（如果 git 可用）
        generator.git_init(project_name)?;

        info!("✅ 项目生成成功");
        Ok(())
    }

    /// 确认当前变量输入
    fn confirm_variable_input(&mut self) -> Result<()> {
        if self.variable_input_index < self.variable_order.len() {
            let name = &self.variable_order[self.variable_input_index];

            if let Some(ref defs) = self.variable_definitions {
                if let Some(def) = defs.get(name) {
                    let input = self.variable_input_buffer.trim();

                    // 根据类型解析输入
                    let value = match def.variable_type.as_str() {
                        "boolean" | "conditional" => {
                            let default_bool = def.default.as_bool().unwrap_or(false);
                            let value_str = if input.is_empty() {
                                if default_bool { "y" } else { "n" }
                            } else {
                                input
                            };

                            let bool_val = match value_str.to_lowercase().as_str() {
                                "y" | "yes" | "true" | "1" => true,
                                "n" | "no" | "false" | "0" => false,
                                _ => default_bool,
                            };
                            serde_json::Value::Bool(bool_val)
                        }
                        "number" | "integer" => {
                            if input.is_empty() {
                                def.default.clone()
                            } else {
                                let num_val: f64 = input.parse()
                                    .unwrap_or(0.0);
                                if def.variable_type == "integer" {
                                    serde_json::Value::Number(serde_json::Number::from(num_val as i64))
                                } else {
                                    serde_json::Value::Number(serde_json::Number::from_f64(num_val).unwrap_or(serde_json::Number::from(0)))
                                }
                            }
                        }
                        _ => {
                            // string 类型
                            let string_val = if input.is_empty() {
                                def.default.as_str().unwrap_or("").to_string()
                            } else {
                                input.to_string()
                            };
                            serde_json::Value::String(string_val)
                        }
                    };

                    self.variable_values.insert(name.clone(), value);
                }
            }

            // 移动到下一个变量
            if self.variable_input_index < self.variable_order.len().saturating_sub(1) {
                self.variable_input_index += 1;
                self.load_current_variable_value();
            } else {
                // 最后一个变量，进入完成状态
                self.state = AppState::Complete;
            }
        }
        Ok(())
    }

    /// 加载当前变量的值到输入缓冲区
    fn load_current_variable_value(&mut self) {
        self.variable_input_buffer.clear();

        if self.variable_input_index < self.variable_order.len() {
            let name = &self.variable_order[self.variable_input_index];

            if let Some(ref defs) = self.variable_definitions {
                if let Some(def) = defs.get(name) {
                    // 如果已经输入过，恢复之前的值
                    if let Some(value) = self.variable_values.get(name) {
                        match value {
                            serde_json::Value::String(s) => {
                                self.variable_input_buffer = s.clone();
                            }
                            serde_json::Value::Bool(b) => {
                                self.variable_input_buffer = if *b { "y".to_string() } else { "n".to_string() };
                            }
                            serde_json::Value::Number(n) => {
                                self.variable_input_buffer = n.to_string();
                            }
                            _ => {}
                        }
                    } else {
                        // 否则使用默认值
                        match def.variable_type.as_str() {
                            "boolean" | "conditional" => {
                                let default_bool = def.default.as_bool().unwrap_or(false);
                                self.variable_input_buffer = if default_bool { "y".to_string() } else { "n".to_string() };
                            }
                            _ => {
                                if let Some(default_str) = def.default.as_str() {
                                    self.variable_input_buffer = default_str.to_string();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
