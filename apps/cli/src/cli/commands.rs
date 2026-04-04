use crate::client::ApiClient;
use crate::config::Config;
use crate::tui::run_tui;
use crate::generator::ProjectGenerator;
use crate::renderer::LocalRenderer;
use super::{CreateCommand, TemplateCommands, ConfigCommands};
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
