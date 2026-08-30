use super::{AiCommands, AiConfigCommands, ConfigCommands, CreateCommand, TemplateCommands};
use crate::client::ApiClient;
use crate::config::Config;
use crate::generator::ProjectGenerator;
use crate::renderer::LocalRenderer;
use crate::tui::run_tui;
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
        )
        .await?;
    } else {
        // CLI模式
        let project_name = cmd.project_name.unwrap();
        let template_name = cmd.template.unwrap();

        info!("CLI模式创建项目: {} from {}", project_name, template_name);

        // 查找模板
        let template = client
            .find_template(&template_name)
            .await
            .context("查找模板失败")?;

        // 获取模板版本列表
        let versions = client
            .get_template_versions(&template.id)
            .await
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
            let zip_bytes = client
                .download_template_version(&template.id, &version.version)
                .await
                .context("下载模板失败")?;

            info!("ZIP文件大小: {} bytes", zip_bytes.len());

            // 创建缓存目录
            std::fs::create_dir_all(&cache_dir).context("创建缓存目录失败")?;

            // 解压 ZIP
            use std::io::Cursor;
            let cursor = Cursor::new(zip_bytes);
            let mut archive = zip::ZipArchive::new(cursor).context("解析ZIP文件失败")?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i).context("读取ZIP条目失败")?;
                let file_path = cache_dir.join(file.enclosed_name().context("ZIP文件路径非法")?);

                if file.is_dir() {
                    std::fs::create_dir_all(&file_path).context("创建目录失败")?;
                } else {
                    if let Some(parent) = file_path.parent() {
                        if !parent.exists() {
                            std::fs::create_dir_all(parent).context("创建父目录失败")?;
                        }
                    }

                    let mut outfile = std::fs::File::create(&file_path).context("创建文件失败")?;
                    std::io::copy(&mut file, &mut outfile).context("写入文件失败")?;
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
        let rendered_files = renderer.render(&variables_map).context("本地渲染失败")?;

        // 生成项目
        let generator = ProjectGenerator::new(&cmd.output, cmd.force).with_template_path(cache_dir);
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

        TemplateCommands::Info {
            template_name,
            variables,
            files,
        } => {
            let template = client.get_template_info(&template_name).await?;

            println!("模板名称: {}", template.name);
            println!("模板ID: {}", template.id);
            println!(
                "描述: {}",
                template.description.as_deref().unwrap_or("无描述")
            );

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
            let filtered: Vec<_> = templates
                .into_iter()
                .filter(|t| {
                    t.name.to_lowercase().contains(&keyword_lower)
                        || t.description
                            .as_ref()
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

pub async fn handle_config(cmd: ConfigCommands, config_path: Option<String>) -> Result<()> {
    let mut config = Config::load(config_path)?;

    match cmd {
        ConfigCommands::Show => {
            println!("当前配置:");
            println!("  服务器 URL: {}", config.server.url);
            println!(
                "  API 密钥: {}",
                if config.server.api_key.is_empty() {
                    "(未设置)"
                } else {
                    "****"
                }
            );
            println!(
                "  默认作者: {}",
                config.user.author.unwrap_or_else(|| "(未设置)".to_string())
            );
            println!(
                "  默认邮箱: {}",
                config.user.email.unwrap_or_else(|| "(未设置)".to_string())
            );
            println!("  模板存储路径: {}", config.storage.template_path.display());
        }

        ConfigCommands::Set { key, value } => {
            match key.as_str() {
                "server.url" => config.server.url = value.clone(),
                "server.api_key" => config.server.api_key = value.clone(),
                "user.author" => config.user.author = Some(value.clone()),
                "user.email" => config.user.email = Some(value.clone()),
                _ => {
                    anyhow::bail!(
                        "未知配置项: {}
支持的配置项: server.url | server.api_key | user.author | user.email",
                        key
                    );
                }
            }
            config.save()?;
            println!("已设置 {} = {}", key, value);
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
        AiCommands::FillVariables {
            path,
            project,
            provider,
            model,
            dry_run,
            write,
            format,
        } => handle_fill_variables(&path, project, provider, model, dry_run, write, &format).await,
        AiCommands::ConvertToTemplate {
            path,
            output,
            name,
            category,
            strategy,
        } => handle_convert_to_template(&path, &output, name, category, &strategy).await,
        AiCommands::RenderPreview {
            path,
            vars_file,
            vars,
            full,
        } => handle_render_preview(&path, vars_file, vars, full).await,
        AiCommands::Validate {
            path,
            vars_file,
            check_output,
        } => handle_validate(&path, vars_file, check_output).await,
        AiCommands::EditFile {
            path,
            insert,
            replace,
            delete,
            append,
            content,
        } => handle_edit_file(&path, insert, replace, delete, append, content).await,
        AiCommands::Recommend {
            project,
            language,
            category,
            explain,
        } => handle_recommend(project, language, category, explain).await,
        AiCommands::Config { config_subcommand } => {
            handle_ai_config(config_subcommand, config_path).await
        }
    }
}

/// 加载变量（从文件或命令行参数）
fn load_variables(
    template_path: &str,
    vars_file: Option<&str>,
    vars_json: Option<&str>,
) -> Result<serde_json::Value> {
    if let Some(file) = vars_file {
        let content = std::fs::read_to_string(file)
            .map_err(|e| anyhow::anyhow!("读取变量文件失败: {}", e))?;
        return serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("解析变量文件失败: {}", e));
    }

    if let Some(json) = vars_json {
        return serde_json::from_str(json)
            .map_err(|e| anyhow::anyhow!("解析变量 JSON 失败: {}", e));
    }

    // 尝试从模板目录的 .meta/variables/variables.json 加载
    let vars_path = std::path::Path::new(template_path)
        .join(".meta")
        .join("variables")
        .join("variables.json");
    if vars_path.exists() {
        let content = std::fs::read_to_string(&vars_path)
            .map_err(|e| anyhow::anyhow!("读取变量文件失败: {}", e))?;
        return serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("解析变量文件失败: {}", e));
    }

    // 返回空对象
    Ok(serde_json::json!({}))
}

async fn handle_analyze_variables(path: &str, format: &str) -> Result<()> {
    use crate::ai::{OutputFormat, OutputFormatter};

    let result = template_studio_ai_agent::analyze_variables(path)
        .await
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

    let result = template_studio_ai_agent::fill_variables(&client, path, &context)
        .await
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
    path: &str,
    output: &str,
    name: Option<String>,
    category: Option<String>,
    strategy: &str,
) -> Result<()> {
    use crate::ai::{OutputFormat, OutputFormatter};

    let result = template_studio_ai_agent::convert_to_template(
        path,
        output,
        name.as_deref(),
        category.as_deref(),
        strategy,
    )
    .await
    .map_err(|e| anyhow::anyhow!("项目转换失败: {}", e))?;

    let formatter = OutputFormatter::new(OutputFormat::Json);
    formatter.print(&result)?;

    Ok(())
}

async fn handle_render_preview(
    path: &str,
    vars_file: Option<String>,
    vars: Option<String>,
    full: bool,
) -> Result<()> {
    use crate::ai::{OutputFormat, OutputFormatter};

    // 加载变量
    let variables = load_variables(path, vars_file.as_deref(), vars.as_deref())?;

    let result = template_studio_ai_agent::render_preview(path, &variables, full)
        .await
        .map_err(|e| anyhow::anyhow!("渲染预览失败: {}", e))?;

    let formatter = OutputFormatter::new(OutputFormat::Json);
    formatter.print(&result)?;

    Ok(())
}

async fn handle_validate(path: &str, vars_file: Option<String>, check_output: bool) -> Result<()> {
    use crate::ai::{OutputFormat, OutputFormatter};

    let formatter = OutputFormatter::new(OutputFormat::Json);

    // 语法验证
    let syntax_result = template_studio_ai_agent::validate_syntax(path)
        .await
        .map_err(|e| anyhow::anyhow!("语法验证失败: {}", e))?;

    println!("=== 语法验证 ===");
    formatter.print(&syntax_result)?;

    // 变量验证（如果有变量文件）
    if let Some(vars_file) = &vars_file {
        let vars_content = std::fs::read_to_string(vars_file)
            .map_err(|e| anyhow::anyhow!("读取变量文件失败: {}", e))?;
        let variables: serde_json::Value = serde_json::from_str(&vars_content)
            .map_err(|e| anyhow::anyhow!("解析变量文件失败: {}", e))?;

        let vars_result = template_studio_ai_agent::validate_variables(path, &variables)
            .await
            .map_err(|e| anyhow::anyhow!("变量验证失败: {}", e))?;

        println!("\n=== 变量验证 ===");
        formatter.print(&vars_result)?;
    }

    if check_output {
        println!("\n=== 输出验证 ===");
        println!("输出验证需要渲染后的结果，请先运行 render-preview");
    }

    Ok(())
}

async fn handle_edit_file(
    path: &str,
    insert: Option<usize>,
    replace: Option<String>,
    delete: Option<String>,
    append: Option<String>,
    content: Option<String>,
) -> Result<()> {
    use crate::ai::{OutputFormat, OutputFormatter};

    let (operation, line, end_line) = if let Some(line_num) = insert {
        ("insert", Some(line_num), None)
    } else if let Some(range) = &replace {
        let parts: Vec<&str> = range.split('-').collect();
        let start = parts[0]
            .parse::<usize>()
            .map_err(|_| anyhow::anyhow!("无效的行号: {}", parts[0]))?;
        let end = if parts.len() > 1 {
            Some(
                parts[1]
                    .parse::<usize>()
                    .map_err(|_| anyhow::anyhow!("无效的行号: {}", parts[1]))?,
            )
        } else {
            None
        };
        ("replace", Some(start), end)
    } else if let Some(range) = &delete {
        let parts: Vec<&str> = range.split('-').collect();
        let start = parts[0]
            .parse::<usize>()
            .map_err(|_| anyhow::anyhow!("无效的行号: {}", parts[0]))?;
        let end = if parts.len() > 1 {
            Some(
                parts[1]
                    .parse::<usize>()
                    .map_err(|_| anyhow::anyhow!("无效的行号: {}", parts[1]))?,
            )
        } else {
            None
        };
        ("delete", Some(start), end)
    } else if append.is_some() {
        ("append", None, None)
    } else {
        anyhow::bail!("请指定操作: --insert, --replace, --delete, 或 --append");
    };

    let result =
        template_studio_ai_agent::edit_file(path, operation, line, end_line, content.as_deref())
            .await
            .map_err(|e| anyhow::anyhow!("文件编辑失败: {}", e))?;

    let formatter = OutputFormatter::new(OutputFormat::Json);
    formatter.print(&result)?;

    Ok(())
}

async fn handle_recommend(
    project: Option<i64>,
    language: Option<String>,
    category: Option<String>,
    explain: bool,
) -> Result<()> {
    use crate::ai::{OutputFormat, OutputFormatter};

    // 项目 ID：CLI 面向本地无数据库场景，ID 无从解析——要求直接给路径，
    // 但给出比「暂未实现」明确得多的指引
    let project_path = if let Some(pid) = project {
        anyhow::bail!(
            "不支持按项目 ID 推荐（CLI 不连接数据库）。
请直接指定项目路径，例如:
  template-studio-cli ai recommend ./{}",
            pid
        );
    } else {
        // 使用当前目录
        std::env::current_dir()
            .map_err(|e| anyhow::anyhow!("获取当前目录失败: {}", e))?
            .to_string_lossy()
            .to_string()
    };

    let result = template_studio_ai_agent::recommend_template(
        &project_path,
        language.as_deref(),
        category.as_deref(),
        explain,
    )
    .await
    .map_err(|e| anyhow::anyhow!("推荐失败: {}", e))?;

    let formatter = OutputFormatter::new(OutputFormat::Json);
    formatter.print(&result)?;

    Ok(())
}

async fn handle_ai_config(cmd: AiConfigCommands, config_path: Option<String>) -> Result<()> {
    let mut config = Config::load(config_path)?;

    match cmd {
        AiConfigCommands::Show => match &config.ai {
            Some(ai) => {
                println!("AI 配置:");
                println!("  提供商: {}", ai.provider);
                println!("  模型: {}", ai.model);
                println!("  API Key: {}****", &ai.api_key[..4.min(ai.api_key.len())]);
                println!(
                    "  API URL: {}",
                    ai.base_url.as_deref().unwrap_or("(提供商默认)")
                );
                println!("  配置文件: ~/.cicbyte/template_studio/config/config.toml");
            }
            None => {
                println!("AI 配置: (未通过 ai config 配置)");
                let env_key = std::env::var("AI_API_KEY").ok();
                println!(
                    "  环境变量 AI_API_KEY: {}",
                    if env_key.is_some() {
                        "已设置"
                    } else {
                        "未设置"
                    }
                );
                println!("  提示: 运行 `template-studio-cli ai config set --provider deepseek --api-key <key>` 持久化配置");
            }
        },
        AiConfigCommands::Set {
            provider,
            model,
            api_key,
            base_url,
        } => {
            // 增量更新：未指定的字段保留现有值
            let mut section = config
                .ai
                .clone()
                .unwrap_or_else(|| crate::config::AiSection {
                    provider: "deepseek".to_string(),
                    model: "deepseek-chat".to_string(),
                    api_key: String::new(),
                    base_url: None,
                });
            if let Some(p) = provider {
                section.provider = p;
            }
            if let Some(m) = model {
                section.model = m;
            }
            if let Some(k) = api_key {
                section.api_key = k;
            }
            if let Some(u) = base_url {
                section.base_url = Some(u);
            }
            if section.api_key.is_empty() {
                anyhow::bail!("API Key 不能为空（--api-key 或先前的配置）");
            }
            config.ai = Some(section);
            config.save()?;
            println!("AI 配置已保存");
        }
        AiConfigCommands::Test => {
            let ai = config.ai.as_ref().or_else(|| {
                // 未持久化配置时尝试环境变量拼一个临时配置
                None::<&crate::config::AiSection>
            });
            let (provider, model, api_key, base_url) = match ai {
                Some(a) => (
                    a.provider.clone(),
                    a.model.clone(),
                    a.api_key.clone(),
                    a.base_url.clone(),
                ),
                None => {
                    let key = std::env::var("AI_API_KEY").unwrap_or_default();
                    if key.is_empty() {
                        anyhow::bail!(
                            "无 AI 配置：请先 `ai config set` 或设置 AI_API_KEY 环境变量"
                        );
                    }
                    (
                        "deepseek".to_string(),
                        "deepseek-chat".to_string(),
                        key,
                        std::env::var("AI_BASE_URL").ok(),
                    )
                }
            };

            use template_studio_ai_agent::config::AiConfig;
            let ai_config = AiConfig {
                provider,
                model,
                api_key,
                base_url,
            };
            println!("测试 AI 连接: {}", ai_config.chat_endpoint());

            let client = reqwest::Client::new();
            let body = serde_json::json!({
                "model": ai_config.model,
                "messages": [{"role": "user", "content": "回复 ok 两个字母即可"}],
                "max_tokens": 8,
            });
            match client
                .post(ai_config.chat_endpoint())
                .header("Authorization", format!("Bearer {}", ai_config.api_key))
                .json(&body)
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    println!("✅ 连接成功（HTTP {}）", resp.status());
                }
                Ok(resp) => {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    anyhow::bail!(
                        "❌ 连接失败（HTTP {}）: {}",
                        status,
                        body.chars().take(200).collect::<String>()
                    );
                }
                Err(e) => anyhow::bail!("❌ 网络错误: {}", e),
            }
        }
    }

    Ok(())
}
