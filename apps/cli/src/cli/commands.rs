use crate::client::ApiClient;
use crate::config::Config;
use crate::tui::run_tui;
use crate::generator::ProjectGenerator;
use crate::renderer::LocalRenderer;
use super::{CreateCommand, TemplateCommands, ConfigCommands, AiCommands, AiConfigCommands};
use anyhow::{Context, Result};
use tracing::{info, warn};

pub async fn handle_create(
    cmd: CreateCommand,
    config_path: Option<String>,
    server_url: Option<String>,
    api_key: Option<String>,
) -> Result<()> {
    // 加载配置
    let mut config = Config::load(config_path)?;

    // 命令行参数覆盖配置
    if let Some(url) = server_url {
        config.server.url = url;
    }
    if let Some(key) = api_key {
        config.server.api_key = key;
    }

    // 创建API客户端
    let client = ApiClient::new(&config.server.url, &config.server.api_key);

    // 自动进入TUI模式的条件：没有项目名称或没有模板
    let need_tui = cmd.project_name.is_none() || cmd.template.is_none();

    if need_tui || cmd.tui {
        // TUI模式
        info!("启动TUI模式...");
        run_tui(
            client,
            cmd.project_name,
            cmd.template,
            &cmd.output,
            cmd.force,
        ).await?;
    } else {
        // CLI模式
        let project_name = cmd.project_name.unwrap();
        let template_name = cmd.template.unwrap();

        info!("CLI模式创建项目: {} from {}", project_name, template_name);

        // 查找模板
        let template = client.find_template(&template_name).await
            .context("查找模板失败")?;

        // 获取模板版本列表
        let versions = client.get_template_versions(&template.id).await
            .context("获取模板版本失败")?;

        if versions.is_empty() {
            anyhow::bail!("模板没有可用版本");
        }

        // 使用第一个版本（或者可以指定版本）
        let version = &versions[0];

        info!("使用模板版本: {}", version.version);

        // 下载模板到本地缓存
        let cache_dir = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".cicbyte")
            .join("template_studio")
            .join("data")
            .join("templates")
            .join(template.id.clone())
            .join(&version.version);

        // 检查缓存是否已存在
        if !cache_dir.exists() {
            info!("下载模板中...");

            // 下载 ZIP
            let zip_bytes = client.download_template_version(&template.id, &version.version).await
                .context("下载模板失败")?;

            info!("ZIP文件大小: {} bytes", zip_bytes.len());

            // 创建缓存目录
            std::fs::create_dir_all(&cache_dir)
                .context("创建缓存目录失败")?;

            // 解压 ZIP
            use std::io::Cursor;
            let cursor = Cursor::new(zip_bytes);
            let mut archive = zip::ZipArchive::new(cursor)
                .context("解析ZIP文件失败")?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i)
                    .context("读取ZIP条目失败")?;
                let file_path = cache_dir.join(file.enclosed_name()
                    .context("ZIP文件路径非法")?);

                if file.is_dir() {
                    std::fs::create_dir_all(&file_path)
                        .context("创建目录失败")?;
                } else {
                    if let Some(parent) = file_path.parent() {
                        if !parent.exists() {
                            std::fs::create_dir_all(parent)
                                .context("创建父目录失败")?;
                        }
                    }

                    let mut outfile = std::fs::File::create(&file_path)
                        .context("创建文件失败")?;
                    std::io::copy(&mut file, &mut outfile)
                        .context("写入文件失败")?;
                }
            }

            info!("✅ 模板下载完成");
        } else {
            info!("使用已缓存的模板: {:?}", cache_dir);
        }

        // 收集变量
        let variables = serde_json::json!({
            "ProjectName": project_name,
            "project_name": project_name.to_lowercase(),
            "PROJECT_NAME": project_name.to_uppercase(),
        });

        // 转换为 HashMap
        let variables_map: std::collections::HashMap<String, serde_json::Value> = variables
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // 本地渲染
        use crate::renderer::LocalRenderer;
        let renderer = LocalRenderer::new(cache_dir.clone());
        let rendered_files = renderer.render(&variables_map)
            .context("本地渲染失败")?;

        // 生成项目
        let generator = ProjectGenerator::new(&cmd.output, cmd.force)
            .with_template_path(cache_dir);
        generator.generate(&project_name, &rendered_files)?;

        // 自动执行 Git 初始化（如果 git 可用）
        generator.git_init(&project_name)?;

        info!("✅ 项目创建成功: {}", project_name);
    }

    Ok(())
}

pub async fn handle_template(
    cmd: TemplateCommands,
    config_path: Option<String>,
    server_url: Option<String>,
    api_key: Option<String>,
) -> Result<()> {
    // 加载配置
    let mut config = Config::load(config_path)?;

    // 命令行参数覆盖配置
    if let Some(url) = server_url {
        config.server.url = url;
    }
    if let Some(key) = api_key {
        config.server.api_key = key;
    }

    let client = ApiClient::new(&config.server.url, &config.server.api_key);

    match cmd {
        TemplateCommands::List { category } => {
            let templates = client.list_templates(category.as_deref()).await?;

            if templates.is_empty() {
                println!("没有找到模板");
                return Ok(());
            }

            println!("找到 {} 个模板:\n", templates.len());
            for tmpl in templates {
                println!("• {} (ID: {})", tmpl.name, tmpl.id);
                if let Some(desc) = &tmpl.description {
                    println!("  {}", desc);
                }
                println!();
            }
        }

        TemplateCommands::Info { template_name, variables, files } => {
            let template = client.get_template_info(&template_name).await?;

            println!("模板名称: {}", template.name);
            println!("模板ID: {}", template.id);
            println!("描述: {}", template.description.as_deref().unwrap_or("无描述"));

            if variables {
                if let Some(vars) = template.variables {
                    println!("\n变量列表:");
                    for var in vars {
                        println!("• {} ({})", var.name, var.variable_type);
                        if let Some(desc) = &var.description {
                            println!("  {}", desc);
                        }
                        if let Some(default) = &var.default_value {
                            println!("  默认值: {}", default);
                        }
                        println!();
                    }
                }
            }

            if files {
                if let Some(files) = template.files {
                    println!("\n文件结构:");
                    for file in files {
                        if file.is_directory {
                            println!("📁 {}/", file.path);
                        } else {
                            println!("📄 {}", file.path);
                        }
                        if let Some(condition) = &file.condition {
                            println!("   条件: {}", condition);
                        }
                    }
                }
            }
        }

        TemplateCommands::Search { keyword, category } => {
            let templates = client.list_templates(category.as_deref()).await?;

            let keyword_lower = keyword.to_lowercase();
            let filtered: Vec<_> = templates.into_iter()
                .filter(|t| {
                    t.name.to_lowercase().contains(&keyword_lower)
                        || t.description.as_ref()
                            .map(|d| d.to_lowercase().contains(&keyword_lower))
                            .unwrap_or(false)
                })
                .collect();

            if filtered.is_empty() {
                println!("没有找到包含 '{}' 的模板", keyword);
                return Ok(());
            }

            println!("找到 {} 个包含 '{}' 的模板:\n", filtered.len(), keyword);
            for tmpl in filtered {
                println!("• {} (ID: {})", tmpl.name, tmpl.id);
                if let Some(desc) = &tmpl.description {
                    println!("  {}", desc);
                }
                println!();
            }
        }
    }

    Ok(())
}

pub async fn handle_config(
    cmd: ConfigCommands,
    config_path: Option<String>,
) -> Result<()> {
    let config = Config::load(config_path)?;

    match cmd {
        ConfigCommands::Show => {
            println!("当前配置:");
            println!("  服务器 URL: {}", config.server.url);
            println!("  API 密钥: {}", if config.server.api_key.is_empty() { "(未设置)" } else { "****" });
            println!("  默认作者: {}", config.user.author.unwrap_or_else(|| "(未设置)".to_string()));
            println!("  默认邮箱: {}", config.user.email.unwrap_or_else(|| "(未设置)".to_string()));
            println!("  模板存储路径: {}", config.storage.template_path.display());
        }

        ConfigCommands::Set { key, value } => {
            // TODO: 实现配置设置
            println!("设置配置: {} = {}", key, value);
            warn!("配置设置功能尚未实现");
        }
    }

    Ok(())
}

pub async fn handle_ai(
    cmd: AiCommands,
    config_path: Option<String>,
    _server_url: Option<String>,
    _api_key: Option<String>,
) -> Result<()> {
    match cmd {
        AiCommands::AnalyzeVariables { path, format } => {
            handle_analyze_variables(&path, &format).await
        }
        AiCommands::FillVariables { path, project, provider, model, dry_run, write, format } => {
            handle_fill_variables(&path, project, provider, model, dry_run, write, &format).await
        }
        AiCommands::ConvertToTemplate { path, output, name, category, strategy } => {
            handle_convert_to_template(&path, &output, name, category, &strategy).await
        }
        AiCommands::RenderPreview { path, vars_file, vars, full } => {
            handle_render_preview(&path, vars_file, vars, full).await
        }
        AiCommands::Validate { path, vars_file, check_output } => {
            handle_validate(&path, vars_file, check_output).await
        }
        AiCommands::EditFile { path, insert, replace, delete, append, content } => {
            handle_edit_file(&path, insert, replace, delete, append, content).await
        }
        AiCommands::Recommend { project, language, category, explain } => {
            handle_recommend(project, language, category, explain).await
        }
        AiCommands::Config { config_subcommand } => {
            handle_ai_config(config_subcommand, config_path).await
        }
    }
}

async fn handle_analyze_variables(path: &str, format: &str) -> Result<()> {
    use crate::ai::{OutputFormat, OutputFormatter};

    let result = template_studio_ai_agent::analyze_variables(path).await
        .map_err(|e| anyhow::anyhow!("变量分析失败: {}", e))?;

    let formatter = OutputFormatter::new(OutputFormat::from_str(format));
    formatter.print(&result)?;

    Ok(())
}

async fn handle_fill_variables(
    path: &str,
    project: i64,
    provider: Option<String>,
    model: Option<String>,
    dry_run: bool,
    write: bool,
    format: &str,
) -> Result<()> {
    use crate::ai::{OutputFormat, OutputFormatter};
    use template_studio_ai_agent::client::OpenAiClient;
    use template_studio_ai_agent::config::AiConfig;
    use template_studio_ai_agent::context::ProjectContext;

    // 加载 AI 配置
    let config = AiConfig {
        provider: provider.unwrap_or_else(|| "deepseek".to_string()),
        model: model.unwrap_or_else(|| "deepseek-chat".to_string()),
        api_key: std::env::var("AI_API_KEY").unwrap_or_default(),
        base_url: std::env::var("AI_BASE_URL").ok(),
    };

    if config.api_key.is_empty() {
        eprintln!("错误: 未设置 AI_API_KEY 环境变量");
        std::process::exit(3);
    }

    let client = OpenAiClient::new(config);

    // 构建项目上下文（简化版，实际应从数据库读取）
    let context = ProjectContext {
        project_id: project,
        project_name: format!("Project {}", project),
        tables: vec![],
        type_mappings: vec![],
        naming_convention: None,
    };

    let result = template_studio_ai_agent::fill_variables(&client, path, &context).await
        .map_err(|e| anyhow::anyhow!("变量填充失败: {}", e))?;

    let formatter = OutputFormatter::new(OutputFormat::from_str(format));
    formatter.print(&result)?;

    if write && !dry_run {
        // 写入 variables.json
        let vars_path = std::path::Path::new(path)
            .join(".meta")
            .join("variables")
            .join("variables.json");
        if let Some(parent) = vars_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&vars_path, serde_json::to_string_pretty(&result.filled)?)?;
        formatter.print_message(&format!("已写入: {}", vars_path.display()));
    }

    Ok(())
}

async fn handle_convert_to_template(
    _path: &str,
    _output: &str,
    _name: Option<String>,
    _category: Option<String>,
    _strategy: &str,
) -> Result<()> {
    // TODO: 实现项目转换
    println!("项目转换功能正在开发中...");
    Ok(())
}

async fn handle_render_preview(
    _path: &str,
    _vars_file: Option<String>,
    _vars: Option<String>,
    _full: bool,
) -> Result<()> {
    // TODO: 实现渲染预览
    println!("渲染预览功能正在开发中...");
    Ok(())
}

async fn handle_validate(
    _path: &str,
    _vars_file: Option<String>,
    _check_output: bool,
) -> Result<()> {
    // TODO: 实现验证
    println!("验证功能正在开发中...");
    Ok(())
}

async fn handle_edit_file(
    _path: &str,
    _insert: Option<usize>,
    _replace: Option<String>,
    _delete: Option<String>,
    _append: Option<String>,
    _content: Option<String>,
) -> Result<()> {
    // TODO: 实现文件编辑
    println!("文件编辑功能正在开发中...");
    Ok(())
}

async fn handle_recommend(
    _project: Option<i64>,
    _language: Option<String>,
    _category: Option<String>,
    _explain: bool,
) -> Result<()> {
    // TODO: 实现推荐
    println!("推荐功能正在开发中...");
    Ok(())
}

async fn handle_ai_config(
    cmd: AiConfigCommands,
    config_path: Option<String>,
) -> Result<()> {
    let _config = Config::load(config_path)?;

    match cmd {
        AiConfigCommands::Show => {
            println!("AI 配置:");
            println!("  提供商: (未配置)");
            println!("  模型: (未配置)");
            println!("  API Key: (未配置)");
        }
        AiConfigCommands::Set { provider, model, api_key, base_url } => {
            if let Some(p) = provider {
                println!("设置提供商: {}", p);
            }
            if let Some(m) = model {
                println!("设置模型: {}", m);
            }
            if let Some(k) = api_key {
                println!("设置 API Key: {}****", &k[..4.min(k.len())]);
            }
            if let Some(u) = base_url {
                println!("设置 API URL: {}", u);
            }
            println!("AI 配置已更新");
        }
        AiConfigCommands::Test => {
            println!("测试 AI 连接...");
            // TODO: 实现连接测试
            println!("连接测试功能正在开发中...");
        }
    }

    Ok(())
}
