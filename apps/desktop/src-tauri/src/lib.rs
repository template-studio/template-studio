// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use template_studio_template_core::{render_string, TemplateFile, Variables};
use tauri::Manager;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

mod config;
mod database;

use config::Config;
use database::{Database, TestConnectionParams, DatasourceParams, import_tables_from_datasource,
                fetch_mysql_tables, fetch_postgresql_tables, fetch_sqlite_tables, import_single_table};

/// 数据库状态包装器，用于线程安全的异步访问
pub struct DbState(Arc<Database>);

impl Clone for DbState {
    fn clone(&self) -> Self {
        DbState(Arc::clone(&self.0))
    }
}

impl AsRef<Database> for DbState {
    fn as_ref(&self) -> &Database {
        &self.0
    }
}

// ===== 浏览器连接池缓存 =====

use sqlx::mysql::MySqlPool;
use sqlx::postgres::PgPool;
use sqlx::sqlite::SqlitePool;

enum BrowserPool {
    MySQL(MySqlPool),
    PostgreSQL(PgPool),
    SQLite(SqlitePool),
}

struct BrowserPoolCache {
    pools: Mutex<HashMap<String, BrowserPool>>,
}

impl BrowserPoolCache {
    fn new() -> Self {
        Self { pools: Mutex::new(HashMap::new()) }
    }

    async fn get_or_create_mysql(&self, url: &str) -> Result<MySqlPool, String> {
        {
            let pools = self.pools.lock().unwrap();
            if let Some(BrowserPool::MySQL(pool)) = pools.get(url) {
                return Ok(pool.clone());
            }
        }
        let pool = MySqlPool::connect(url).await
            .map_err(|e| format!("连接失败: {}", e))?;
        let mut pools = self.pools.lock().unwrap();
        pools.insert(url.to_string(), BrowserPool::MySQL(pool.clone()));
        Ok(pool)
    }

    async fn get_or_create_pg(&self, url: &str) -> Result<PgPool, String> {
        {
            let pools = self.pools.lock().unwrap();
            if let Some(BrowserPool::PostgreSQL(pool)) = pools.get(url) {
                return Ok(pool.clone());
            }
        }
        let pool = PgPool::connect(url).await
            .map_err(|e| format!("连接失败: {}", e))?;
        let mut pools = self.pools.lock().unwrap();
        pools.insert(url.to_string(), BrowserPool::PostgreSQL(pool.clone()));
        Ok(pool)
    }

    async fn get_or_create_sqlite(&self, url: &str) -> Result<SqlitePool, String> {
        {
            let pools = self.pools.lock().unwrap();
            if let Some(BrowserPool::SQLite(pool)) = pools.get(url) {
                return Ok(pool.clone());
            }
        }
        let pool = SqlitePool::connect(url).await
            .map_err(|e| format!("连接失败: {}", e))?;
        let mut pools = self.pools.lock().unwrap();
        pools.insert(url.to_string(), BrowserPool::SQLite(pool.clone()));
        Ok(pool)
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// 写入文本文件
#[tauri::command]
fn write_text_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, &content).map_err(|e| format!("写入文件失败: {}", e))
}

/// 最小化窗口
#[tauri::command]
fn window_minimize(app: tauri::AppHandle) {
    let window = app.get_webview_window("main").unwrap();
    window.minimize().unwrap();
}

/// 最大化/还原窗口
#[tauri::command]
fn window_maximize(app: tauri::AppHandle) {
    let window = app.get_webview_window("main").unwrap();
    if window.is_maximized().unwrap() {
        window.unmaximize().unwrap();
    } else {
        window.maximize().unwrap();
    }
}

/// 关闭窗口
#[tauri::command]
fn window_close(app: tauri::AppHandle) {
    let window = app.get_webview_window("main").unwrap();
    window.close().unwrap();
}

/// 获取模板列表
#[tauri::command]
async fn list_templates() -> Result<Vec<Template>, String> {
    // TODO: 从 API 或本地加载模板列表
    // 这里先返回模拟数据
    Ok(vec![
        Template {
            id: "1".to_string(),
            name: "Go Web Service".to_string(),
            description: Some("一个使用 Gin 框架的 Go Web 服务模板".to_string()),
            template_type: "web".to_string(),
            language: Some("Go".to_string()),
            is_featured: true,
        },
        Template {
            id: "2".to_string(),
            name: "Rust CLI Tool".to_string(),
            description: Some("使用 Clap 的 Rust 命令行工具模板".to_string()),
            template_type: "cli".to_string(),
            language: Some("Rust".to_string()),
            is_featured: false,
        },
    ])
}

/// 获取模板变量定义
#[tauri::command]
async fn get_template_variables(template_id: &str) -> Result<Vec<Variable>, String> {
    // TODO: 从 API 或本地加载变量定义
    // 这里先返回模拟数据
    Ok(match template_id {
        "1" => vec![
            Variable {
                name: "project_name".to_string(),
                title: "项目名称".to_string(),
                description: "请输入项目名称".to_string(),
                type_: "string".to_string(),
                default_value: Some("my-project".to_string()),
                required: true,
            },
            Variable {
                name: "author".to_string(),
                title: "作者".to_string(),
                description: "作者名称".to_string(),
                type_: "string".to_string(),
                default_value: Some("Your Name".to_string()),
                required: true,
            },
        ],
        "2" => vec![
            Variable {
                name: "cli_name".to_string(),
                title: "CLI 名称".to_string(),
                description: "命令行工具名称".to_string(),
                type_: "string".to_string(),
                default_value: Some("my-cli".to_string()),
                required: true,
            },
        ],
        _ => vec![],
    })
}

/// 渲染模板预览
#[tauri::command]
fn render_template(
    template_id: String,
    variables: serde_json::Value,
) -> Result<Vec<RenderedFile>, String> {
    let vars = Variables::from_json(&variables.to_string())
        .map_err(|e| e.to_string())?;

    // TODO: 根据 template_id 加载实际模板内容
    // 这里使用模拟的模板内容
    let template_content = match template_id.as_str() {
        "1" => "package main\n\nimport \"fmt\"\n\nfunc main() {\n    fmt.Println(\"Hello, {{ project_name }}!\")\n    fmt.Println(\"Author: {{ author }}\")\n}",
        "2" => "use std::println;\n\nfn main() {\n    println!(\"Hello, {{ cli_name }}!\");\n}",
        _ => "Hello, {{ name }}!"
    };

    // 简单渲染示例（实际应该使用 render_tree）
    let result = render_string(template_content, &vars, None)
        .map_err(|e| e.to_string())?;

    // 返回模拟的文件列表
    Ok(vec![
        RenderedFile {
            path: match template_id.as_str() {
                "1" => "main.go".to_string(),
                "2" => "main.rs".to_string(),
                _ => "main.txt".to_string()
            },
            content: result.content,
        },
        RenderedFile {
            path: "README.md".to_string(),
            content: format!(
                "# Project\n\nGenerated by: {}\n\nWelcome to your new project!",
                variables.get("project_name")
                    .or_else(|| variables.get("cli_name"))
                    .or_else(|| variables.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
            ),
        },
    ])
}

/// 渲染模板预览（返回文件树）
#[tauri::command]
async fn render_template_preview(
    template_id: String,
    variables: serde_json::Value,
    version: Option<String>,
) -> Result<String, String> {
    use template_studio_template_core::{render_tree, Variables};

    println!("渲染模板预览:");
    println!("  模板ID: {}", template_id);
    println!("  版本: {:?}", version);

    // 1. 加载配置并获取模板路径
    let config = Config::load().map_err(|e| format!("加载配置失败: {}", e))?;

    // 确定版本号
    let version_str = version.unwrap_or_else(|| {
        // 如果没有指定版本，使用 "latest"
        "latest".to_string()
    });

    let template_path = config.get_template_path(&template_id, &version_str);

    // 检查模板是否存在
    if !template_path.exists() {
        return Err(format!(
            "模板未下载，请先下载模板。路径: {:?}",
            template_path
        ));
    }

    println!("模板路径: {:?}", template_path);

    // 2. 扫描模板文件
    let template_files = scan_template_files(&template_path)
        .map_err(|e| format!("扫描模板文件失败: {}", e))?;

    println!("扫描到 {} 个文件/目录", template_files.len());

    // 3. 准备变量
    let variables_json = serde_json::to_string(&variables)
        .map_err(|e| format!("序列化变量失败: {}", e))?;

    let render_vars = Variables::from_json(&variables_json)
        .map_err(|e| format!("解析变量失败: {}", e))?;

    // 4. 渲染模板树
    let rendered_tree = render_tree(template_files, &render_vars)
        .map_err(|e| format!("渲染模板失败: {}", e))?;

    println!("渲染完成，生成 {} 个文件节点", rendered_tree.len());

    // 5. 序列化为 JSON 返回
    let result_json = serde_json::to_string(&rendered_tree)
        .map_err(|e| format!("序列化结果失败: {}", e))?;

    Ok(result_json)
}

/// 生成项目
#[tauri::command]
async fn generate_project(
    template_id: String,
    variables: serde_json::Value,
    output_path: String,
    version: Option<String>,
) -> Result<String, String> {
    use template_studio_template_core::{render_tree, Variables};
    use std::fs;
    use std::path::Path;

    println!("生成项目:");
    println!("  模板ID: {}", template_id);
    println!("  输出路径: {}", output_path);
    println!("  版本: {:?}", version);

    // 1. 加载配置并获取模板路径
    let config = Config::load().map_err(|e| format!("加载配置失败: {}", e))?;

    // 确定版本号
    let version_str = version.unwrap_or_else(|| {
        // 如果没有指定版本，使用 "latest"
        "latest".to_string()
    });

    let template_path = config.get_template_path(&template_id, &version_str);

    // 检查模板是否存在
    if !template_path.exists() {
        return Err(format!(
            "模板未下载，请先下载模板。路径: {:?}",
            template_path
        ));
    }

    println!("模板路径: {:?}", template_path);

    // 2. 扫描模板文件
    let template_files = scan_template_files(&template_path)
        .map_err(|e| format!("扫描模板文件失败: {}", e))?;

    println!("扫描到 {} 个文件/目录", template_files.len());

    // 3. 准备变量
    let variables_json = serde_json::to_string(&variables)
        .map_err(|e| format!("序列化变量失败: {}", e))?;

    let render_vars = Variables::from_json(&variables_json)
        .map_err(|e| format!("解析变量失败: {}", e))?;

    // 4. 渲染模板树
    let rendered_tree = render_tree(template_files, &render_vars)
        .map_err(|e| format!("渲染模板失败: {}", e))?;

    println!("渲染完成，生成 {} 个文件节点", rendered_tree.len());

    // 5. 写入文件到输出目录
    let output_dir = Path::new(&output_path);
    let template_path = config.get_template_path(&template_id, &version_str);

    // 如果输出目录已存在，先删除
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)
            .map_err(|e| format!("删除已存在目录失败: {}", e))?;
    }

    // 创建输出目录
    fs::create_dir_all(output_dir)
        .map_err(|e| format!("创建输出目录失败: {}", e))?;

    let mut success_count = 0;
    let mut error_count = 0;
    let mut _binary_count = 0;
    let mut _skipped_count = 0;

    for file_node in rendered_tree {
        let file_path = output_dir.join(&file_node.file_path);
        let source_path = template_path.join(&file_node.file_path);

        if let Some(render_error) = file_node.error {
            eprintln!("渲染文件失败: {}", file_node.file_path);
            eprintln!("  错误类型: {}", render_error.error_type);
            eprintln!("  错误信息: {}", render_error.message);
            error_count += 1;
            continue;
        }

        if file_node.is_directory == 1 {
            // 创建目录
            fs::create_dir_all(&file_path)
                .map_err(|e| format!("创建目录 {:?} 失败: {}", file_path, e))?;
        } else {
            // 写入文件
            if let Some(parent) = file_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("创建父目录 {:?} 失败: {}", parent, e))?;
                }
            }

            // 检查文件内容
            match &file_node.file_content {
                Some(content) if !content.is_empty() => {
                    // 有内容：文本文件，写入渲染后的内容
                    fs::write(&file_path, content)
                        .map_err(|e| format!("写入文件 {:?} 失败: {}", file_path, e))?;

                    success_count += 1;
                    println!("创建文件: {}", file_node.file_path);
                }
                _ => {
                    // None 或空字符串：二进制文件或读取失败，直接复制原文件
                    match fs::copy(&source_path, &file_path) {
                        Ok(_) => {
                            _binary_count += 1;
                            println!("复制文件: {}", file_node.file_path);
                        }
                        Err(e) => {
                            eprintln!("复制文件失败 {:?} -> {:?}: {}", source_path, file_path, e);
                            error_count += 1;
                        }
                    }
                }
            }
        }
    }

    println!("✅ 项目生成完成!");
    println!("   成功: {} 个文件", success_count);
    if error_count > 0 {
        println!("   失败: {} 个文件", error_count);
    }

    Ok(format!(
        "项目生成成功！路径: {} (成功 {} 个文件)",
        output_path,
        success_count
    ))
}

/// 扫描模板目录，构建文件树
fn scan_template_files(template_path: &Path) -> Result<Vec<TemplateFile>, String> {
    let mut files = Vec::new();
    let mut id_counter = 1i64;

    // 递归扫描模板目录（排除 .meta 目录）
    scan_directory_recursive(template_path, template_path, &mut files, &mut id_counter, 0)?;

    Ok(files)
}

/// 递归扫描目录
fn scan_directory_recursive(
    base_path: &Path,
    dir: &Path,
    files: &mut Vec<TemplateFile>,
    id_counter: &mut i64,
    parent_id: i64,
) -> Result<(), String> {
    use std::fs;

    let entries = fs::read_dir(dir)
        .map_err(|e| format!("读取目录失败 {:?}: {}", dir, e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();

        // 跳过 .meta 目录和 .git 目录
        if path.is_dir() {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if file_name == ".meta" || file_name == ".git" {
                continue;
            }

            // 创建目录节点
            let relative_path = path
                .strip_prefix(base_path)
                .map_err(|e| format!("计算相对路径失败: {}", e))?;

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
            scan_directory_recursive(base_path, &path, files, id_counter, dir_id)?;
        } else {
            // 读取文件
            let relative_path = path
                .strip_prefix(base_path)
                .map_err(|e| format!("计算相对路径失败: {}", e))?;

            let metadata = fs::metadata(&path)
                .map_err(|e| format!("获取文件元数据失败 {:?}: {}", path, e))?;

            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // 检查是否是二进制文件（基于扩展名）
            let extension = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            let is_binary = matches!(extension.as_str(),
                "png" | "jpg" | "jpeg" | "gif" | "ico" | "webp" |
                "woff" | "woff2" | "ttf" | "eot" | "otf" |
                "mp3" | "mp4" | "wav" | "ogg" | "webm" |
                "pdf" | "zip" | "exe" | "dll" | "so" |
                "bin" | "dat" | "db" | "sqlite" | "mdb"
            );

            let file_content = if is_binary {
                // 二进制文件：使用空字符串标记（渲染时会跳过，直接复制原文件）
                println!("  [二进制文件] {} - 跳过内容读取", relative_path.display());
                String::new()
            } else {
                // 文本文件：尝试读取内容
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        println!("  [文本文件] {} - {} 字节", relative_path.display(), content.len());
                        content
                    }
                    Err(e) => {
                        // 读取失败，可能是编码问题，记录但继续
                        eprintln!("  [警告] 读取文件失败 {:?}: {}, 将跳过该文件", path, e);
                        String::new()  // 使用空字符串，渲染时会跳过
                    }
                }
            };

            let file = TemplateFile {
                id: *id_counter,
                file_path: relative_path.to_string_lossy().replace('\\', "/"),
                file_name,
                file_content,
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

/// 检查模板版本是否已下载
#[tauri::command]
fn check_template_downloaded(template_id: String, version: String) -> bool {
    if let Ok(config) = Config::load() {
        let template_path = config.get_template_path(&template_id, &version);
        template_path.exists()
    } else {
        false
    }
}

/// 检查目录是否存在
#[tauri::command]
fn check_directory_exists(path: String) -> bool {
    use std::path::Path;
    Path::new(&path).exists()
}

/// 删除目录（用于覆盖已存在的项目）
#[tauri::command]
fn remove_directory(path: String) -> Result<String, String> {
    use std::path::Path;
    let path = Path::new(&path);

    if path.exists() {
        std::fs::remove_dir_all(path)
            .map_err(|e| format!("删除目录失败: {}", e))?;
        Ok("目录删除成功".to_string())
    } else {
        Ok("目录不存在".to_string())
    }
}

/// 获取配置
#[tauri::command]
fn get_config() -> Result<String, String> {
    let config = Config::load()
        .map_err(|e| format!("加载配置失败: {}", e))?;

    serde_json::to_string(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))
}

/// 更新 Web 服务器配置
#[tauri::command]
fn update_web_server_config(
    api_url: Option<String>,
    api_key: Option<String>,
) -> Result<String, String> {
    let mut config = Config::load()
        .map_err(|e| format!("加载配置失败: {}", e))?;

    // 更新配置
    if let Some(url) = api_url {
        config.web_server.api_url = url;
    }
    if let Some(key) = api_key {
        // 如果传入空字符串，设置为 None
        config.web_server.api_key = if key.is_empty() { None } else { Some(key) };
    }

    // 保存配置
    config.save()
        .map_err(|e| format!("保存配置失败: {}", e))?;

    Ok("配置已保存".to_string())
}

/// 更新模板存储路径
#[tauri::command]
fn update_template_path(template_path: String) -> Result<String, String> {
    use std::path::PathBuf;

    let mut config = Config::load()
        .map_err(|e| format!("加载配置失败: {}", e))?;

    // 验证路径
    let path = PathBuf::from(&template_path);

    // 创建目录（如果不存在）
    std::fs::create_dir_all(&path)
        .map_err(|e| format!("创建模板目录失败: {}", e))?;

    // 更新配置
    config.storage.template_path = path;

    // 保存配置
    config.save()
        .map_err(|e| format!("保存配置失败: {}", e))?;

    Ok("模板存储路径已更新".to_string())
}

// ===== 数据库相关命令 =====

/// 获取统计数据
#[tauri::command]
async fn db_get_statistics(
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let stats = db.get_statistics().await
        .map_err(|e| format!("获取统计数据失败: {}", e))?;

    serde_json::to_string(&stats)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 获取最近项目列表
#[tauri::command]
async fn db_get_recent_projects(
    limit: Option<i64>,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let projects = db.get_recent_projects(limit.unwrap_or(5)).await
        .map_err(|e| format!("获取最近项目失败: {}", e))?;

    serde_json::to_string(&projects)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 获取所有项目
#[tauri::command]
async fn db_get_all_projects(
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let projects = db.get_all_projects().await
        .map_err(|e| format!("查询项目失败: {}", e))?;

    serde_json::to_string(&projects)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 根据 ID 获取项目
#[tauri::command]
async fn db_get_project(
    id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let project = db.get_project(id).await
        .map_err(|e| format!("查询项目失败: {}", e))?;

    match project {
        Some(p) => serde_json::to_string(&p)
            .map_err(|e| format!("序列化失败: {}", e)),
        None => Err("项目不存在".to_string()),
    }
}

/// 创建项目
#[tauri::command]
async fn db_create_project(
    params: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    let name = params.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少项目名称".to_string())?;

    let description = params.get("description")
        .and_then(|v| v.as_str());

    let datasource_id = params.get("datasourceId")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| "缺少数据源ID".to_string())?;

    let database_name = params.get("databaseName")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少数据库名称".to_string())?;

    let primary_language_id = params.get("primaryLanguageId")
        .and_then(|v| v.as_i64());

    let frontend_language_id = params.get("frontendLanguageId")
        .and_then(|v| v.as_i64());

    let backend_language_id = params.get("backendLanguageId")
        .and_then(|v| v.as_i64());

    let id = db.create_project(
        name,
        description,
        datasource_id,
        database_name,
        primary_language_id,
        frontend_language_id,
        backend_language_id,
    ).await.map_err(|e| format!("创建项目失败: {}", e))?;

    Ok(id)
}

/// 更新项目
#[tauri::command]
async fn db_update_project(
    id: i64,
    params: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    let name = params.get("name")
        .and_then(|v| v.as_str());

    let description = params.get("description")
        .and_then(|v| v.as_str());

    let primary_language_id = params.get("primaryLanguageId")
        .and_then(|v| v.as_i64());

    let frontend_language_id = params.get("frontendLanguageId")
        .and_then(|v| v.as_i64());

    let backend_language_id = params.get("backendLanguageId")
        .and_then(|v| v.as_i64());

    db.update_project(
        id,
        name,
        description,
        primary_language_id,
        frontend_language_id,
        backend_language_id,
    ).await.map_err(|e| format!("更新项目失败: {}", e))?;

    Ok(())
}

/// 删除项目
#[tauri::command]
async fn db_delete_project(
    id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.delete_project(id).await
        .map_err(|e| format!("删除项目失败: {}", e))?;

    Ok(())
}

/// 获取所有数据源
#[tauri::command]
async fn db_get_all_datasources(
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let datasources = db.get_all_datasources().await
        .map_err(|e| format!("查询数据源失败: {}", e))?;

    serde_json::to_string(&datasources)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 创建数据源
#[tauri::command]
async fn db_create_datasource(
    params: DatasourceParams,
    db_state: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = db_state.as_ref();

    let id = db.create_datasource(
        &params.name,
        &params.type_,
        params.host.as_deref(),
        params.port,
        params.username.as_deref(),
        params.password.as_deref(),
        params.database.as_deref(),
        params.sqlite_file.as_deref(),
    ).await.map_err(|e| format!("创建数据源失败: {}", e))?;

    Ok(id)
}

/// 根据 ID 获取数据源
#[tauri::command]
async fn db_get_datasource(
    id: i64,
    db_state: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = db_state.as_ref();

    let datasource = db.get_datasource(id).await
        .map_err(|e| format!("查询数据源失败: {}", e))?
        .ok_or_else(|| "数据源不存在".to_string())?;

    serde_json::to_string(&datasource)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 更新数据源
#[tauri::command]
async fn db_update_datasource(
    id: i64,
    params: DatasourceParams,
    db_state: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = db_state.as_ref();

    db.update_datasource(
        id,
        &params.name,
        &params.type_,
        params.host.as_deref(),
        params.port,
        params.username.as_deref(),
        params.password.as_deref(),
        params.database.as_deref(),
        params.sqlite_file.as_deref(),
    ).await.map_err(|e| format!("更新数据源失败: {}", e))?;

    Ok(())
}

/// 删除数据源
#[tauri::command]
async fn db_delete_datasource(
    id: i64,
    db_state: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = db_state.as_ref();

    db.delete_datasource(id).await
        .map_err(|e| format!("删除数据源失败: {}", e))?;

    Ok(())
}

/// 测试数据源连接
#[tauri::command]
async fn test_datasource_connection(params: TestConnectionParams) -> Result<String, String> {
    Database::test_datasource_connection(params).await
}

/// 列出数据库中的表
#[tauri::command]
async fn cmd_list_database_tables(params: TestConnectionParams) -> Result<String, String> {
    use sqlx::mysql::MySqlPool;
    use sqlx::postgres::PgPool;
    use sqlx::sqlite::SqlitePool;
    use sqlx::Row;

    let db_type = params.type_.clone();

    match db_type.as_str() {
        "mysql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(3306);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_default();

            // 如果没有指定数据库，先连接到服务器获取数据库列表
            if database.is_empty() {
                let url = format!("mysql://{}:{}@{}:{}", username, password, host, port);
                let pool = MySqlPool::connect(&url).await
                    .map_err(|e| format!("连接失败: {}", e))?;

                let rows = sqlx::query("SHOW DATABASES")
                    .fetch_all(&pool)
                    .await
                    .map_err(|e| format!("查询失败: {}", e))?;

                let databases: Vec<String> = rows.iter()
                    .map(|row| row.get::<String, _>(0))
                    .collect();

                pool.close().await;
                return serde_json::to_string(&databases).map_err(|e| format!("序列化失败: {}", e));
            }

            let url = format!("mysql://{}:{}@{}:{}/{}", username, password, host, port, database);
            let pool = MySqlPool::connect(&url).await
                .map_err(|e| format!("连接失败: {}", e))?;

            let rows = sqlx::query("SHOW TABLES")
                .fetch_all(&pool)
                .await
                .map_err(|e| format!("查询失败: {}", e))?;

            let tables: Vec<String> = rows.iter()
                .map(|row| row.get::<String, _>(0))
                .collect();

            pool.close().await;
            serde_json::to_string(&tables).map_err(|e| format!("序列化失败: {}", e))
        }
        "postgresql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(5432);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_else(|| "postgres".to_string());

            let url = format!("postgres://{}:{}@{}:{}/{}", username, password, host, port, database);
            let pool = PgPool::connect(&url).await
                .map_err(|e| format!("连接失败: {}", e))?;

            let rows = sqlx::query("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'")
                .fetch_all(&pool)
                .await
                .map_err(|e| format!("查询失败: {}", e))?;

            let tables: Vec<String> = rows.iter()
                .map(|row| row.get::<String, _>(0))
                .collect();

            pool.close().await;
            serde_json::to_string(&tables).map_err(|e| format!("序列化失败: {}", e))
        }
        "sqlite" => {
            let sqlite_file = params.sqlite_file.unwrap_or_default();
            let url = format!("sqlite:{}", sqlite_file);
            let pool = SqlitePool::connect(&url).await
                .map_err(|e| format!("连接失败: {}", e))?;

            let rows = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
                .fetch_all(&pool)
                .await
                .map_err(|e| format!("查询失败: {}", e))?;

            let tables: Vec<String> = rows.iter()
                .map(|row| row.get::<String, _>(0))
                .collect();

            pool.close().await;
            serde_json::to_string(&tables).map_err(|e| format!("序列化失败: {}", e))
        }
        _ => Err(format!("不支持的数据库类型: {}", db_type))
    }
}

/// 获取表的列信息
#[tauri::command]
async fn cmd_get_table_columns(
    params: TestConnectionParams,
    table_name: String,
    pool_cache: tauri::State<'_, BrowserPoolCache>,
) -> Result<String, String> {
    use sqlx::Row;

    let db_type = params.type_.clone();

    match db_type.as_str() {
        "mysql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(3306);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_default();

            let url = format!("mysql://{}:{}@{}:{}/{}", username, password, host, port, database);
            let pool = pool_cache.get_or_create_mysql(&url).await?;

            let rows = sqlx::query(
                "SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE, COLUMN_KEY, COLUMN_DEFAULT, COLUMN_COMMENT \
                 FROM INFORMATION_SCHEMA.COLUMNS \
                 WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? \
                 ORDER BY ORDINAL_POSITION"
            )
            .bind(&database)
            .bind(&table_name)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("查询失败: {}", e))?;

            let columns: Vec<serde_json::Value> = rows.iter().map(|row| {
                serde_json::json!({
                    "name": row.get::<String, _>(0),
                    "type": row.get::<String, _>(1),
                    "nullable": row.get::<String, _>(2) == "YES",
                    "key": row.get::<String, _>(3),
                    "default": row.get::<Option<String>, _>(4),
                    "comment": row.get::<Option<String>, _>(5)
                })
            }).collect();

            serde_json::to_string(&columns).map_err(|e| format!("序列化失败: {}", e))
        }
        "postgresql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(5432);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_else(|| "postgres".to_string());

            let url = format!("postgres://{}:{}@{}:{}/{}", username, password, host, port, database);
            let pool = pool_cache.get_or_create_pg(&url).await?;

            let rows = sqlx::query(
                "SELECT column_name, data_type, is_nullable, \
                 CASE WHEN constraint_type = 'PRIMARY KEY' THEN 'PRI' ELSE '' END as column_key, \
                 column_default, '' as column_comment \
                 FROM information_schema.columns c \
                 LEFT JOIN information_schema.key_column_usage k \
                   ON c.table_name = k.table_name AND c.column_name = k.column_name \
                 LEFT JOIN information_schema.table_constraints t \
                   ON k.constraint_name = t.constraint_name AND t.constraint_type = 'PRIMARY KEY' \
                 WHERE c.table_schema = 'public' AND c.table_name = $1 \
                 ORDER BY c.ordinal_position"
            )
            .bind(&table_name)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("查询失败: {}", e))?;

            let columns: Vec<serde_json::Value> = rows.iter().map(|row| {
                serde_json::json!({
                    "name": row.get::<String, _>(0),
                    "type": row.get::<String, _>(1),
                    "nullable": row.get::<String, _>(2) == "YES",
                    "key": row.get::<String, _>(3),
                    "default": row.get::<Option<String>, _>(4),
                    "comment": row.get::<String, _>(5)
                })
            }).collect();

            serde_json::to_string(&columns).map_err(|e| format!("序列化失败: {}", e))
        }
        "sqlite" => {
            let sqlite_file = params.sqlite_file.unwrap_or_default();
            let url = format!("sqlite:{}", sqlite_file);
            let pool = pool_cache.get_or_create_sqlite(&url).await?;

            let rows = sqlx::query(&format!("PRAGMA table_info('{}')", table_name.replace('\'', "''")))
                .fetch_all(&pool)
                .await
                .map_err(|e| format!("查询失败: {}", e))?;

            let columns: Vec<serde_json::Value> = rows.iter().map(|row| {
                serde_json::json!({
                    "name": row.get::<String, _>(1),
                    "type": row.get::<String, _>(2),
                    "nullable": row.get::<bool, _>(3) == false,
                    "key": if row.get::<bool, _>(5) { "PRI" } else { "" },
                    "default": row.get::<Option<String>, _>(4),
                    "comment": ""
                })
            }).collect();

            serde_json::to_string(&columns).map_err(|e| format!("序列化失败: {}", e))
        }
        _ => Err(format!("不支持的数据库类型: {}", db_type))
    }
}

/// 查询表数据（带分页，使用连接池缓存 + 快速行数估算 + 并行查询）
#[tauri::command]
async fn cmd_query_table_data(
    params: TestConnectionParams,
    table_name: String,
    limit: i64,
    offset: i64,
    pool_cache: tauri::State<'_, BrowserPoolCache>,
) -> Result<String, String> {
    use sqlx::Row;

    let db_type = params.type_.clone();

    match db_type.as_str() {
        "mysql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(3306);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_default();

            let url = format!("mysql://{}:{}@{}:{}/{}", username, password, host, port, database);
            let pool = pool_cache.get_or_create_mysql(&url).await?;

            // 快速行数估算（MySQL: 从 information_schema 获取估计值，避免全表扫描）
            let est_sql = "SELECT TABLE_ROWS FROM INFORMATION_SCHEMA.TABLES \
                           WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?";
            let est_total: i64 = sqlx::query_scalar(est_sql)
                .bind(&database)
                .bind(&table_name)
                .fetch_one(&pool)
                .await
                .unwrap_or(0);

            // 估算值为 0 时回退到 COUNT(*)（小表或统计信息未更新）
            let total = if est_total <= 0 {
                let count_sql = format!("SELECT COUNT(*) FROM `{}`", table_name.replace('`', "``"));
                sqlx::query_scalar(&count_sql).fetch_one(&pool).await.unwrap_or(0)
            } else {
                est_total
            };

            // 获取列名
            let col_rows = sqlx::query(
                "SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS \
                 WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ? ORDER BY ORDINAL_POSITION"
            )
            .bind(&database)
            .bind(&table_name)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("获取列信息失败: {}", e))?;

            let columns: Vec<String> = col_rows.iter()
                .map(|row| row.get::<String, _>(0))
                .collect();

            if columns.is_empty() {
                return serde_json::to_string(&serde_json::json!({
                    "columns": [], "rows": [], "total": total
                })).map_err(|e| format!("序列化失败: {}", e));
            }

            // 用 CAST 将所有列转为字符串，避免类型转换问题
            let cast_cols: Vec<String> = columns.iter().map(|c| {
                format!("CAST(`{}` AS CHAR) AS `{}`", c.replace('`', "``"), c.replace('`', "``"))
            }).collect();
            let data_sql = format!(
                "SELECT {} FROM `{}` LIMIT {} OFFSET {}",
                cast_cols.join(", "),
                table_name.replace('`', "``"),
                limit, offset
            );
            let rows = sqlx::query(&data_sql)
                .fetch_all(&pool)
                .await
                .map_err(|e| format!("查询失败: {}", e))?;

            let data: Vec<Vec<serde_json::Value>> = rows.iter().map(|row| {
                (0..columns.len()).map(|i| {
                    match row.try_get::<Option<String>, _>(i) {
                        Ok(Some(v)) => serde_json::Value::String(v),
                        _ => serde_json::Value::Null
                    }
                }).collect()
            }).collect();

            serde_json::to_string(&serde_json::json!({
                "columns": columns,
                "rows": data,
                "total": total
            })).map_err(|e| format!("序列化失败: {}", e))
        }
        "postgresql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(5432);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_else(|| "postgres".to_string());

            let url = format!("postgres://{}:{}@{}:{}/{}", username, password, host, port, database);
            let pool = pool_cache.get_or_create_pg(&url).await?;

            // 快速行数估算（PostgreSQL: pg_class.reltuples，避免 COUNT(*) 全表扫描）
            let est_total: i64 = sqlx::query_scalar(
                "SELECT COALESCE(reltuples::bigint, 0) FROM pg_class WHERE relname = $1"
            )
            .bind(&table_name)
            .fetch_one(&pool)
            .await
            .unwrap_or(0);

            // 估算值为 0 时回退到 COUNT(*)
            let total = if est_total <= 0 {
                let count_sql = format!("SELECT COUNT(*) FROM \"{}\"", table_name.replace('"', "\"\""));
                sqlx::query_scalar(&count_sql).fetch_one(&pool).await.unwrap_or(0)
            } else {
                est_total
            };

            // 获取列名
            let col_rows = sqlx::query(
                "SELECT column_name FROM information_schema.columns \
                 WHERE table_schema = 'public' AND table_name = $1 ORDER BY ordinal_position"
            )
            .bind(&table_name)
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("获取列信息失败: {}", e))?;

            let columns: Vec<String> = col_rows.iter()
                .map(|row| row.get::<String, _>(0))
                .collect();

            if columns.is_empty() {
                return serde_json::to_string(&serde_json::json!({
                    "columns": [], "rows": [], "total": total
                })).map_err(|e| format!("序列化失败: {}", e));
            }

            // 用 CAST 将所有列转为 TEXT
            let cast_cols: Vec<String> = columns.iter().map(|c| {
                format!("\"{}\"::TEXT AS \"{}\"", c.replace('"', "\"\""), c.replace('"', "\"\""))
            }).collect();
            let data_sql = format!(
                "SELECT {} FROM \"{}\" LIMIT {} OFFSET {}",
                cast_cols.join(", "),
                table_name.replace('"', "\"\""),
                limit, offset
            );
            let rows = sqlx::query(&data_sql)
                .fetch_all(&pool)
                .await
                .map_err(|e| format!("查询失败: {}", e))?;

            let data: Vec<Vec<serde_json::Value>> = rows.iter().map(|row| {
                (0..columns.len()).map(|i| {
                    match row.try_get::<Option<String>, _>(i) {
                        Ok(Some(v)) => serde_json::Value::String(v),
                        _ => serde_json::Value::Null
                    }
                }).collect()
            }).collect();

            serde_json::to_string(&serde_json::json!({
                "columns": columns,
                "rows": data,
                "total": total
            })).map_err(|e| format!("序列化失败: {}", e))
        }
        "sqlite" => {
            let sqlite_file = params.sqlite_file.unwrap_or_default();
            let url = format!("sqlite:{}", sqlite_file);
            let pool = pool_cache.get_or_create_sqlite(&url).await?;

            let count_sql = format!("SELECT COUNT(*) FROM \"{}\"", table_name.replace('"', "\"\""));
            let total: i64 = sqlx::query_scalar(&count_sql)
                .fetch_one(&pool)
                .await
                .map_err(|e| format!("查询总数失败: {}", e))?;

            // 获取列名
            let col_rows = sqlx::query(&format!("PRAGMA table_info('{}')", table_name.replace('\'', "''")))
                .fetch_all(&pool)
                .await
                .map_err(|e| format!("获取列信息失败: {}", e))?;

            let columns: Vec<String> = col_rows.iter()
                .map(|row| row.get::<String, _>(1))
                .collect();

            if columns.is_empty() {
                return serde_json::to_string(&serde_json::json!({
                    "columns": [], "rows": [], "total": total
                })).map_err(|e| format!("序列化失败: {}", e));
            }

            // 用 CAST 将所有列转为 TEXT
            let cast_cols: Vec<String> = columns.iter().map(|c| {
                format!("CAST(\"{}\" AS TEXT) AS \"{}\"", c.replace('"', "\"\""), c.replace('"', "\"\""))
            }).collect();
            let data_sql = format!(
                "SELECT {} FROM \"{}\" LIMIT {} OFFSET {}",
                cast_cols.join(", "),
                table_name.replace('"', "\"\""),
                limit, offset
            );
            let rows = sqlx::query(&data_sql)
                .fetch_all(&pool)
                .await
                .map_err(|e| format!("查询失败: {}", e))?;

            let data: Vec<Vec<serde_json::Value>> = rows.iter().map(|row| {
                (0..columns.len()).map(|i| {
                    match row.try_get::<Option<String>, _>(i) {
                        Ok(Some(v)) => serde_json::Value::String(v),
                        _ => serde_json::Value::Null
                    }
                }).collect()
            }).collect();

            serde_json::to_string(&serde_json::json!({
                "columns": columns,
                "rows": data,
                "total": total
            })).map_err(|e| format!("序列化失败: {}", e))
        }
        _ => Err(format!("不支持的数据库类型: {}", db_type))
    }
}

/// 获取数据库连接状态信息
#[tauri::command]
async fn cmd_get_connection_status(
    params: TestConnectionParams,
    pool_cache: tauri::State<'_, BrowserPoolCache>,
) -> Result<String, String> {
    use sqlx::Row;

    let db_type = params.type_.clone();
    let start = std::time::Instant::now();

    match db_type.as_str() {
        "mysql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(3306);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_default();

            // 构建连接 URL（无数据库时不带路径）
            let url = if database.is_empty() {
                format!("mysql://{}:{}@{}:{}", username, password, host, port)
            } else {
                format!("mysql://{}:{}@{}:{}/{}", username, password, host, port, database)
            };
            let pool = pool_cache.get_or_create_mysql(&url).await?;
            let latency = start.elapsed().as_millis();

            // 获取服务器版本
            let version: String = sqlx::query_scalar("SELECT VERSION()")
                .fetch_one(&pool)
                .await
                .unwrap_or_else(|_| "未知".to_string());

            // 获取活跃连接数
            let active_connections: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM information_schema.processlist"
            )
            .fetch_one(&pool)
            .await
            .unwrap_or(0);

            // 获取最大连接数
            let max_connections: i64 = sqlx::query(
                "SHOW VARIABLES LIKE 'max_connections'"
            )
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten()
            .map(|row| row.get::<String, _>(1).parse::<i64>().unwrap_or(0))
            .unwrap_or(0);

            // 获取运行时间
            let uptime: i64 = sqlx::query(
                "SHOW VARIABLES LIKE 'uptime'"
            )
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten()
            .map(|row| row.get::<String, _>(1).parse::<i64>().unwrap_or(0))
            .unwrap_or(0);

            // 数据库特定信息（仅当指定了数据库时查询）
            let (db_size, table_count) = if !database.is_empty() {
                let size: String = sqlx::query_scalar(
                    "SELECT CONCAT(ROUND(SUM(data_length + index_length) / 1024 / 1024, 2), ' MB') \
                     FROM information_schema.tables WHERE table_schema = ?"
                )
                .bind(&database)
                .fetch_one(&pool)
                .await
                .unwrap_or_else(|_| "未知".to_string());

                let count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = ?"
                )
                .bind(&database)
                .fetch_one(&pool)
                .await
                .unwrap_or(0);

                (size, count)
            } else {
                ("未知".to_string(), 0)
            };

            serde_json::to_string(&serde_json::json!({
                "status": "connected",
                "type": "MySQL",
                "version": version,
                "host": host,
                "port": port,
                "database": if database.is_empty() { None::<String> } else { Some(database) },
                "latency_ms": latency,
                "active_connections": active_connections,
                "max_connections": max_connections,
                "uptime_seconds": uptime,
                "database_size": db_size,
                "table_count": table_count,
                "pool_size": pool.size(),
                "pool_idle": pool.num_idle(),
            })).map_err(|e| format!("序列化失败: {}", e))
        }
        "postgresql" => {
            let host = params.host.unwrap_or_else(|| "localhost".to_string());
            let port = params.port.unwrap_or(5432);
            let username = params.username.unwrap_or_default();
            let password = params.password.unwrap_or_default();
            let database = params.database.unwrap_or_else(|| "postgres".to_string());

            let url = format!("postgres://{}:{}@{}:{}/{}", username, password, host, port, database);
            let pool = pool_cache.get_or_create_pg(&url).await?;
            let latency = start.elapsed().as_millis();

            let version: String = sqlx::query_scalar("SELECT version()")
                .fetch_one(&pool)
                .await
                .unwrap_or_else(|_| "未知".to_string());

            let active_connections: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM pg_stat_activity WHERE state = 'active'"
            )
            .fetch_one(&pool)
            .await
            .unwrap_or(0);

            let max_connections: i64 = sqlx::query_scalar::<_, String>(
                "SHOW max_connections"
            )
            .fetch_one(&pool)
            .await
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0);

            let db_size: String = sqlx::query_scalar(
                "SELECT pg_size_pretty(pg_database_size($1))"
            )
            .bind(&database)
            .fetch_one(&pool)
            .await
            .unwrap_or_else(|_| "未知".to_string());

            let table_count: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM information_schema.tables WHERE table_schema = 'public'"
            )
            .fetch_one(&pool)
            .await
            .unwrap_or(0);

            serde_json::to_string(&serde_json::json!({
                "status": "connected",
                "type": "PostgreSQL",
                "version": version,
                "host": host,
                "port": port,
                "database": database,
                "latency_ms": latency,
                "active_connections": active_connections,
                "max_connections": max_connections,
                "database_size": db_size,
                "table_count": table_count,
                "pool_size": pool.size(),
                "pool_idle": pool.num_idle(),
            })).map_err(|e| format!("序列化失败: {}", e))
        }
        "sqlite" => {
            let sqlite_file = params.sqlite_file.unwrap_or_default();
            let url = format!("sqlite:{}", sqlite_file);
            let pool = pool_cache.get_or_create_sqlite(&url).await?;
            let latency = start.elapsed().as_millis();

            let version: String = sqlx::query_scalar("SELECT sqlite_version()")
                .fetch_one(&pool)
                .await
                .unwrap_or_else(|_| "未知".to_string());

            let table_count: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
            )
            .fetch_one(&pool)
            .await
            .unwrap_or(0);

            // 获取文件大小
            let file_size = std::fs::metadata(&sqlite_file)
                .map(|m| {
                    let bytes = m.len();
                    if bytes > 1024 * 1024 {
                        format!("{:.2} MB", bytes as f64 / 1024.0 / 1024.0)
                    } else if bytes > 1024 {
                        format!("{:.2} KB", bytes as f64 / 1024.0)
                    } else {
                        format!("{} B", bytes)
                    }
                })
                .unwrap_or_else(|_| "未知".to_string());

            serde_json::to_string(&serde_json::json!({
                "status": "connected",
                "type": "SQLite",
                "version": version,
                "file": sqlite_file,
                "latency_ms": latency,
                "database_size": file_size,
                "table_count": table_count,
                "pool_size": pool.size(),
                "pool_idle": pool.num_idle(),
            })).map_err(|e| format!("序列化失败: {}", e))
        }
        _ => Err(format!("不支持的数据库类型: {}", db_type))
    }
}

/// 获取项目的所有表
#[tauri::command]
async fn db_get_project_tables(
    project_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let tables = db.get_project_tables(project_id).await
        .map_err(|e| format!("查询表失败: {}", e))?;

    serde_json::to_string(&tables)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 为项目创建表
#[tauri::command]
async fn db_create_table(
    project_id: i64,
    name: String,
    comment: Option<String>,
    engine: Option<String>,
    table_type: String,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    let id = db.create_table(
        project_id,
        &name,
        comment.as_deref(),
        engine.as_deref(),
        &table_type,
    ).await.map_err(|e| format!("创建表失败: {}", e))?;

    Ok(id)
}

/// 从数据源导入表结构
#[tauri::command]
async fn cmd_import_tables_from_datasource(
    project_id: i64,
    datasource_id: i64,
    database_name: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    import_tables_from_datasource(
        db.pool(),
        project_id,
        datasource_id,
        &database_name,
    ).await
}

/// 获取表的所有列
#[tauri::command]
async fn db_get_table_columns(
    table_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let columns = db.get_table_columns(table_id).await
        .map_err(|e| format!("查询列失败: {}", e))?;

    serde_json::to_string(&columns)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 删除表
#[tauri::command]
async fn db_delete_table(
    table_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.delete_table(table_id).await
        .map_err(|e| format!("删除表失败: {}", e))?;

    Ok(())
}

/// 更新表信息
#[tauri::command]
async fn db_update_table(
    table_id: i64,
    name: String,
    comment: Option<String>,
    engine: Option<String>,
    table_type: String,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.update_table(
        table_id,
        &name,
        comment.as_deref(),
        engine.as_deref(),
        &table_type,
    ).await
    .map_err(|e| format!("更新表失败: {}", e))?;

    Ok(())
}

/// 创建列
#[tauri::command]
async fn db_create_column(
    table_id: i64,
    name: String,
    data_type: String,
    length: Option<i64>,
    is_nullable: bool,
    is_primary_key: bool,
    is_unique: bool,
    default_value: Option<String>,
    comment: Option<String>,
    ordinal_position: i32,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    let column_id = db.create_column(
        table_id,
        &name,
        &data_type,
        length,
        is_nullable,
        is_primary_key,
        is_unique,
        default_value.as_deref(),
        comment.as_deref(),
        ordinal_position,
    ).await
    .map_err(|e| format!("创建列失败: {}", e))?;

    Ok(column_id)
}

/// 更新列信息
#[tauri::command]
async fn db_update_column(
    column_id: i64,
    name: String,
    data_type: String,
    length: Option<i64>,
    is_nullable: bool,
    is_primary_key: bool,
    is_unique: bool,
    default_value: Option<String>,
    comment: Option<String>,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.update_column(
        column_id,
        &name,
        &data_type,
        length,
        is_nullable,
        is_primary_key,
        is_unique,
        default_value.as_deref(),
        comment.as_deref(),
    ).await
    .map_err(|e| format!("更新列失败: {}", e))?;

    Ok(())
}

/// 删除列
#[tauri::command]
async fn db_delete_column(
    column_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.delete_column(column_id).await
        .map_err(|e| format!("删除列失败: {}", e))?;

    Ok(())
}

/// 重新排序列
#[tauri::command]
async fn db_reorder_columns(
    _table_id: i64,
    column_ids: Vec<i64>,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    for (index, column_id) in column_ids.iter().enumerate() {
        db.update_column_position(*column_id, (index + 1) as i32).await
            .map_err(|e| format!("更新列位置失败: {}", e))?;
    }

    Ok(())
}

/// 解析SQL并创建表和字段
#[tauri::command]
async fn cmd_parse_sql_and_create(
    project_id: i64,
    sql_content: String,
    sql_dialect: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    database::parse_and_create_from_sql(
        db.pool(),
        project_id,
        &sql_content,
        &sql_dialect,
    ).await
}

// ===== 语言相关命令 =====

/// 获取所有语言
#[tauri::command]
async fn db_get_all_languages(
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let languages = db.get_all_languages().await
        .map_err(|e| format!("查询语言失败: {}", e))?;

    serde_json::to_string(&languages)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 根据 ID 获取语言
#[tauri::command]
async fn db_get_language(
    id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let language = db.get_language(id).await
        .map_err(|e| format!("查询语言失败: {}", e))?
        .ok_or_else(|| "语言不存在".to_string())?;

    serde_json::to_string(&language)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 创建语言
#[tauri::command]
async fn db_create_language(
    params: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    let name = params.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少语言名称".to_string())?;

    let icon = params.get("icon").and_then(|v| v.as_str());
    let color = params.get("color").and_then(|v| v.as_str());
    let description = params.get("description").and_then(|v| v.as_str());

    let id = db.create_language(name, icon, color, description)
        .await
        .map_err(|e| format!("创建语言失败: {}", e))?;

    Ok(id)
}

/// 更新语言
#[tauri::command]
async fn db_update_language(
    id: i64,
    params: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    let name = params.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少语言名称".to_string())?;

    let icon = params.get("icon").and_then(|v| v.as_str());
    let color = params.get("color").and_then(|v| v.as_str());
    let description = params.get("description").and_then(|v| v.as_str());

    db.update_language(id, name, icon, color, description)
        .await
        .map_err(|e| format!("更新语言失败: {}", e))?;

    Ok(())
}

/// 删除语言
#[tauri::command]
async fn db_delete_language(
    id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.delete_language(id)
        .await
        .map_err(|e| format!("删除语言失败: {}", e))?;

    Ok(())
}

/// 设置项目的主语言
#[tauri::command]
async fn db_set_project_primary_language(
    project_id: i64,
    language_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.set_project_primary_language(project_id, language_id)
        .await
        .map_err(|e| format!("设置主语言失败: {}", e))?;

    Ok(())
}

/// 获取项目的所有语言
#[tauri::command]
async fn db_get_project_languages(
    project_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let languages = db.get_project_languages(project_id).await
        .map_err(|e| format!("查询项目语言失败: {}", e))?;

    serde_json::to_string(&languages)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 为项目添加语言
#[tauri::command]
async fn db_add_project_language(
    project_id: i64,
    language_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.add_project_language(project_id, language_id, false)
        .await
        .map_err(|e| format!("添加语言失败: {}", e))?;

    Ok(())
}

/// 移除项目的语言
#[tauri::command]
async fn db_remove_project_language(
    project_id: i64,
    language_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.remove_project_language(project_id, language_id)
        .await
        .map_err(|e| format!("移除语言失败: {}", e))?;

    Ok(())
}

// ===== 语言类型字段命令 =====

/// 获取语言的所有类型字段
#[tauri::command]
async fn db_get_language_field_types(
    language_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let field_types = db.get_language_field_types(language_id)
        .await
        .map_err(|e| format!("查询类型字段失败: {}", e))?;

    serde_json::to_string(&field_types)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 创建语言类型字段
#[tauri::command]
async fn db_create_language_field_type(
    language_id: i64,
    name: String,
    description: Option<String>,
    sort_order: i32,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    db.create_language_field_type(language_id, &name, description.as_deref(), sort_order)
        .await
        .map_err(|e| format!("创建类型字段失败: {}", e))
}

/// 更新语言类型字段
#[tauri::command]
async fn db_update_language_field_type(
    id: i64,
    name: String,
    description: Option<String>,
    sort_order: i32,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.update_language_field_type(id, &name, description.as_deref(), sort_order)
        .await
        .map_err(|e| format!("更新类型字段失败: {}", e))?;

    Ok(())
}

/// 删除语言类型字段
#[tauri::command]
async fn db_delete_language_field_type(
    id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.delete_language_field_type(id)
        .await
        .map_err(|e| format!("删除类型字段失败: {}", e))?;

    Ok(())
}

/// 批量保存语言类型字段
#[tauri::command]
async fn db_batch_save_language_field_types(
    language_id: i64,
    field_types: String,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    let field_types: Vec<serde_json::Value> = serde_json::from_str(&field_types)
        .map_err(|e| format!("解析类型字段数据失败: {}", e))?;

    db.batch_save_language_field_types(language_id, field_types)
        .await
        .map_err(|e| format!("批量保存类型字段失败: {}", e))?;

    Ok(())
}

// ===== 系统级类型映射命令 =====

/// 获取系统级类型映射
#[tauri::command]
async fn db_get_system_type_mappings(
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let mappings = db.get_system_type_mappings().await
        .map_err(|e| format!("查询系统级类型映射失败: {}", e))?;

    serde_json::to_string(&mappings)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 根据语言和数据库类型获取系统级类型映射
#[tauri::command]
async fn db_get_system_type_mappings_by_lang_db(
    language_id: i64,
    db_type: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let mappings = db.get_system_type_mappings_by_lang_db(language_id, &db_type).await
        .map_err(|e| format!("查询系统级类型映射失败: {}", e))?;

    serde_json::to_string(&mappings)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 创建系统级类型映射
#[tauri::command]
async fn db_create_system_type_mapping(
    language_id: i64,
    db_type: String,
    pattern: String,
    target_type: String,
    priority: i32,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    db.create_system_type_mapping(language_id, &db_type, &pattern, &target_type, priority)
        .await
        .map_err(|e| format!("创建系统级类型映射失败: {}", e))
}

/// 更新系统级类型映射
#[tauri::command]
async fn db_update_system_type_mapping(
    id: i64,
    target_type: String,
    priority: i32,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.update_system_type_mapping(id, &target_type, priority)
        .await
        .map_err(|e| format!("更新系统级类型映射失败: {}", e))?;

    Ok(())
}

/// 删除系统级类型映射
#[tauri::command]
async fn db_delete_system_type_mapping(
    id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.delete_system_type_mapping(id)
        .await
        .map_err(|e| format!("删除系统级类型映射失败: {}", e))?;

    Ok(())
}

/// 批量保存系统级类型映射
#[tauri::command]
async fn db_batch_save_system_type_mappings(
    mappings: String,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    let mappings: Vec<serde_json::Value> = serde_json::from_str(&mappings)
        .map_err(|e| format!("解析类型映射数据失败: {}", e))?;

    db.batch_save_system_type_mappings(mappings)
        .await
        .map_err(|e| format!("批量保存系统级类型映射失败: {}", e))?;

    Ok(())
}

// ===== 项目级类型映射命令 =====

/// 获取项目级类型映射
#[tauri::command]
async fn db_get_project_type_mappings(
    project_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let mappings = db.get_project_type_mappings(project_id).await
        .map_err(|e| format!("查询项目级类型映射失败: {}", e))?;

    serde_json::to_string(&mappings)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 根据项目和范围获取项目级类型映射
#[tauri::command]
async fn db_get_project_type_mappings_by_scope(
    project_id: i64,
    scope: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let mappings = db.get_project_type_mappings_by_scope(project_id, &scope).await
        .map_err(|e| format!("查询项目级类型映射失败: {}", e))?;

    serde_json::to_string(&mappings)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 创建项目级类型映射
#[tauri::command]
async fn db_create_project_type_mapping(
    project_id: i64,
    scope: String,
    db_type: String,
    pattern: String,
    target_type: String,
    priority: i32,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    db.create_project_type_mapping(project_id, &scope, &db_type, &pattern, &target_type, priority)
        .await
        .map_err(|e| format!("创建项目级类型映射失败: {}", e))
}

/// 更新项目级类型映射
#[tauri::command]
async fn db_update_project_type_mapping(
    id: i64,
    target_type: String,
    priority: i32,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.update_project_type_mapping(id, &target_type, priority)
        .await
        .map_err(|e| format!("更新项目级类型映射失败: {}", e))?;

    Ok(())
}

/// 删除项目级类型映射
#[tauri::command]
async fn db_delete_project_type_mapping(
    id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.delete_project_type_mapping(id)
        .await
        .map_err(|e| format!("删除项目级类型映射失败: {}", e))?;

    Ok(())
}

/// 批量保存项目级类型映射
#[tauri::command]
async fn db_batch_save_project_type_mappings(
    project_id: i64,
    scope: String,
    mappings: String,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    let mappings: Vec<serde_json::Value> = serde_json::from_str(&mappings)
        .map_err(|e| format!("解析类型映射数据失败: {}", e))?;

    db.batch_save_project_type_mappings(project_id, &scope, mappings)
        .await
        .map_err(|e| format!("批量保存项目级类型映射失败: {}", e))?;

    Ok(())
}

/// 复制系统级映射到项目级
#[tauri::command]
async fn db_copy_system_mappings_to_project(
    project_id: i64,
    language_id: i64,
    scope: String,
    db_type: String,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.copy_system_mappings_to_project(project_id, language_id, &scope, &db_type)
        .await
        .map_err(|e| format!("复制系统级映射到项目级失败: {}", e))?;

    Ok(())
}

// ===== AI 服务命令 =====

/// 获取所有 AI 提供商
#[tauri::command]
async fn ai_get_all_providers(
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let providers = db.get_all_ai_providers().await
        .map_err(|e| format!("获取 AI 提供商失败: {}", e))?;

    serde_json::to_string(&providers)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 获取单个 AI 提供商
#[tauri::command]
async fn ai_get_provider(
    provider_name: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let provider = db.get_ai_provider(&provider_name).await
        .map_err(|e| format!("获取 AI 提供商失败: {}", e))?
        .ok_or_else(|| "AI 提供商不存在".to_string())?;

    serde_json::to_string(&provider)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 保存 AI 提供商配置
#[tauri::command]
async fn ai_save_provider(
    params: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let provider_name = params.get("providerName")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少提供商名称".to_string())?;

    let display_name = params.get("displayName")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少显示名称".to_string())?;

    let provider_type = params.get("providerType")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少提供商类型".to_string())?;

    let api_key = params.get("apiKey").and_then(|v| v.as_str());
    let api_endpoint = params.get("apiEndpoint").and_then(|v| v.as_str());
    let is_enabled = params.get("isEnabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let temperature = params.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.7);
    let max_tokens = params.get("maxTokens").and_then(|v| v.as_i64()).unwrap_or(4096) as i32;

    let _id = db.save_ai_provider(
        provider_name,
        display_name,
        provider_type,
        api_key,
        api_endpoint,
        is_enabled,
        temperature,
        max_tokens,
    ).await
    .map_err(|e| format!("保存 AI 提供商失败: {}", e))?;

    Ok("配置已保存".to_string())
}

/// 切换 AI 提供商启用状态
#[tauri::command]
async fn ai_toggle_provider(
    provider_name: String,
    enabled: bool,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.toggle_ai_provider(&provider_name, enabled)
        .await
        .map_err(|e| format!("切换状态失败: {}", e))?;

    Ok(())
}

/// 删除 AI 提供商
#[tauri::command]
async fn ai_delete_provider(
    provider_name: String,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.delete_ai_provider(&provider_name)
        .await
        .map_err(|e| format!("删除提供商失败: {}", e))?;

    Ok(())
}

/// 获取提供商的模型分组
#[tauri::command]
async fn ai_get_provider_models_grouped(
    provider_name: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let groups = db.get_ai_provider_models_grouped(&provider_name).await
        .map_err(|e| format!("获取模型列表失败: {}", e))?;

    serde_json::to_string(&groups)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 添加 AI 模型
#[tauri::command]
async fn ai_add_model(
    params: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    let model_id = params.get("modelId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少模型 ID".to_string())?;

    let model_name = params.get("modelName")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少模型名称".to_string())?;

    let provider_name = params.get("providerName")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少提供商名称".to_string())?;

    let group_id = params.get("groupId")
        .and_then(|v| v.as_str())
        .unwrap_or("chat");

    let description = params.get("description").and_then(|v| v.as_str());
    let max_tokens = params.get("maxTokens").and_then(|v| v.as_i64()).unwrap_or(4096) as i32;

    let id = db.add_ai_model(model_id, model_name, provider_name, group_id, description, max_tokens)
        .await
        .map_err(|e| format!("添加模型失败: {}", e))?;

    Ok(id)
}

/// 删除 AI 模型
#[tauri::command]
async fn ai_delete_model(
    model_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    db.delete_ai_model(model_id)
        .await
        .map_err(|e| format!("删除模型失败: {}", e))?;

    Ok(())
}

/// 更新 AI 模型
#[tauri::command]
async fn ai_update_model(
    model_id: i64,
    params: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    let new_model_id = params.get("modelId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少模型 ID".to_string())?;

    let model_name = params.get("modelName")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少模型名称".to_string())?;

    let group_id = params.get("groupId")
        .and_then(|v| v.as_str())
        .unwrap_or("chat");

    let description = params.get("description").and_then(|v| v.as_str());

    db.update_ai_model(model_id, new_model_id, model_name, group_id, description)
        .await
        .map_err(|e| format!("更新模型失败: {}", e))?;

    Ok(())
}

/// 从提供商 API 获取可用模型列表
#[tauri::command]
async fn ai_fetch_models(
    provider_name: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    // 获取提供商配置
    let provider_config = db.get_ai_provider(&provider_name).await
        .map_err(|e| format!("获取提供商失败: {}", e))?
        .ok_or_else(|| "提供商不存在".to_string())?;

    let api_key = provider_config["apiKey"]
        .as_str()
        .ok_or_else(|| "请先配置 API 密钥".to_string())?;

    let base_url = provider_config["apiEndpoint"]
        .as_str()
        .unwrap_or_else(|| get_default_endpoint(&provider_name).leak());

    // 构建 /models 端点
    let models_endpoint = if base_url.ends_with("/models") {
        base_url.to_string()
    } else if base_url.ends_with('/') {
        format!("{}models", base_url)
    } else {
        format!("{}/models", base_url)
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get(&models_endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| format!("请求模型列表失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API 返回错误 ({}): {}", status, body));
    }

    let response_json: serde_json::Value = response.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    // 解析 OpenAI 兼容格式的模型列表
    let models_array = response_json["data"]
        .as_array()
        .ok_or_else(|| "API 返回格式错误：缺少 data 字段".to_string())?;

    let models: Vec<serde_json::Value> = models_array
        .iter()
        .filter_map(|m| {
            let model_id = m["id"].as_str()?;
            Some(serde_json::json!({
                "modelId": model_id,
                "modelName": model_id,
                "ownedBy": m["owned_by"].as_str().unwrap_or("")
            }))
        })
        .collect();

    serde_json::to_string(&models)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 批量添加 AI 模型
#[tauri::command]
async fn ai_batch_add_models(
    provider_name: String,
    models: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    let models_array = models.as_array()
        .ok_or_else(|| "models 格式错误：应为数组".to_string())?;

    let mut model_tuples: Vec<(&str, &str, &str, &str, Option<&str>, i32)> = Vec::new();
    for m in models_array {
        let model_id = m.get("modelId").and_then(|v| v.as_str()).unwrap_or("");
        let model_name = m.get("modelName").and_then(|v| v.as_str()).unwrap_or(model_id);
        let group_id = m.get("groupId").and_then(|v| v.as_str()).unwrap_or("chat");
        let description = m.get("description").and_then(|v| v.as_str());
        let max_tokens = m.get("maxTokens").and_then(|v| v.as_i64()).unwrap_or(4096) as i32;

        if !model_id.is_empty() {
            model_tuples.push((model_id, model_name, &provider_name, group_id, description, max_tokens));
        }
    }

    let count = db.batch_add_ai_models(&model_tuples)
        .await
        .map_err(|e| format!("批量添加模型失败: {}", e))?;

    Ok(count)
}

// ===== AI SQL 生成和修复命令 =====

/// AI 生成 SQL（支持多轮对话）
#[tauri::command]
async fn ai_generate_sql(
    provider: String,
    model: String,
    messages: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    // 获取提供商配置
    let provider_config = db.get_ai_provider(&provider).await
        .map_err(|e| format!("获取提供商失败: {}", e))?
        .ok_or_else(|| "提供商不存在".to_string())?;

    // 构建 AI API 请求
    let api_key = provider_config["apiKey"]
        .as_str()
        .ok_or_else(|| "请先配置 API 密钥".to_string())?;
    let base_url = provider_config["apiEndpoint"]
        .as_str()
        .unwrap_or_else(|| get_default_endpoint(&provider).leak());

    // 构建完整的 API 端点（添加 /chat/completions 路径）
    let api_endpoint = if base_url.ends_with("/chat/completions") {
        base_url.to_string()
    } else if base_url.ends_with('/') {
        format!("{}chat/completions", base_url)
    } else {
        format!("{}/chat/completions", base_url)
    };

    // 验证 messages 格式
    let messages_array = messages.as_array()
        .ok_or_else(|| "messages 格式错误：应为数组".to_string())?;

    if messages_array.is_empty() {
        return Err("messages 不能为空".to_string());
    }

    // 调用 OpenAI 兼容 API（支持多轮对话）
    let client = reqwest::Client::new();
    let response = client
        .post(&api_endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": 0.3,
            "max_tokens": 2000
        }))
        .send()
        .await
        .map_err(|e| format!("AI API 请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("AI API 返回错误: {}", response.status()));
    }

    let response_json: serde_json::Value = response.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    let sql = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| "AI 返回格式错误".to_string())?;

    Ok(sql.to_string())
}

/// AI 修复 SQL
#[tauri::command]
async fn ai_fix_sql(
    provider: String,
    model: String,
    sql: String,
    error: String,
    dialect: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    // 构建 AI 提示词
    let prompt = format!(
        "以下 SQL 执行时出现错误：\n\n{}\n\n错误信息：\n{}\n\n请分析错误原因并修复 SQL 语句。要求：\n\
        1. 保持原有的表结构和字段定义\n\
        2. 只修复导致错误的部分\n\
        3. 确保语法符合 {} 标准\n\
        4. 只返回修复后的完整 SQL，不要其他解释\n\n\
        请直接输出修复后的 SQL：",
        sql, error, dialect.to_uppercase()
    );

    // 获取提供商配置
    let provider_config = db.get_ai_provider(&provider).await
        .map_err(|e| format!("获取提供商失败: {}", e))?
        .ok_or_else(|| "提供商不存在".to_string())?;

    // 构建 AI API 请求
    let api_key = provider_config["apiKey"]
        .as_str()
        .ok_or_else(|| "请先配置 API 密钥".to_string())?;
    let base_url = provider_config["apiEndpoint"]
        .as_str()
        .unwrap_or_else(|| get_default_endpoint(&provider).leak());

    // 构建完整的 API 端点（添加 /chat/completions 路径）
    let api_endpoint = if base_url.ends_with("/chat/completions") {
        base_url.to_string()
    } else if base_url.ends_with('/') {
        format!("{}chat/completions", base_url)
    } else {
        format!("{}/chat/completions", base_url)
    };

    let client = reqwest::Client::new();
    let response = client
        .post(&api_endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.2,
            "max_tokens": 2000
        }))
        .send()
        .await
        .map_err(|e| format!("AI API 请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("AI API 返回错误: {}", response.status()));
    }

    let response_json: serde_json::Value = response.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    let sql = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| "AI 返回格式错误".to_string())?;

    Ok(sql.to_string())
}

/// 解析 AI 生成的 SQL（只返回表结构，不创建）
#[tauri::command]
async fn parse_ai_sql(
    project_id: i64,
    sql: String,
    dialect: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    database::parse_sql_only(db.pool(), project_id, &sql, &dialect).await
}

/// 执行 AI 生成的 SQL（在数据库中创建表）
#[tauri::command]
async fn execute_ai_sql(
    project_id: i64,
    sql: String,
    dialect: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    database::parse_and_create_from_sql(db.pool(), project_id, &sql, &dialect).await
}

/// 获取默认 API 端点
fn get_default_endpoint(provider: &str) -> String {
    match provider {
        "deepseek" => "https://api.deepseek.com/v1".to_string(),
        "glm" => "https://open.bigmodel.cn/api/paas/v4".to_string(),
        "openai" => "https://api.openai.com/v1".to_string(),
        "longcat" => "https://api.longcat.chat/openai".to_string(),
        "mimo" => "https://api.xiaomimimo.com/v1".to_string(),
        _ => "https://api.openai.com/v1".to_string(),
    }
}

// ===== 项目表规范管理命令 =====

/// 获取项目表规范配置
/// 获取项目表规范配置
#[tauri::command]
async fn db_get_table_preferences(
    project_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    match db.get_table_preferences(project_id).await {
        Ok(Some(value)) => {
            // 直接返回 JSON 字符串
            serde_json::to_string(&value).map_err(|e| format!("序列化失败: {}", e))
        }
        Ok(None) => {
            // 返回空配置
            Ok(serde_json::to_string(&serde_json::json!({
                "pkEnabled": true,
                "pkFieldName": "id",
                "pkFieldType": "BIGINT",
                "pkAutoIncrement": true,
                "pkComment": "主键",
                "auditEnabled": true,
                "auditFields": "[]",
                "softDeleteEnabled": false,
                "booleanPrefix": "is_",
                "datetimeSuffix": "_at"
            })).unwrap())
        }
        Err(e) => Err(format!("获取表规范失败: {}", e))
    }
}

/// 保存项目表规范配置
#[tauri::command]
async fn db_save_table_preferences(
    project_id: i64,
    preferences: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    db.save_table_preferences(project_id, preferences).await
        .map_err(|e| format!("保存表规范失败: {}", e))
}

// ===== 表结构读取和导入命令 =====

/// 读取 MySQL 数据库的表列表
#[tauri::command]
async fn cmd_fetch_mysql_tables(
    datasource_id: i64,
    database_name: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    // 获取数据源信息
    let datasource = db.get_datasource(datasource_id).await
        .map_err(|e| format!("获取数据源失败: {}", e))?
        .ok_or_else(|| "数据源不存在".to_string())?;

    fetch_mysql_tables(db.pool(), &datasource, &database_name).await
}

/// 读取 PostgreSQL 数据库的表列表
#[tauri::command]
async fn cmd_fetch_postgresql_tables(
    datasource_id: i64,
    database_name: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    // 获取数据源信息
    let datasource = db.get_datasource(datasource_id).await
        .map_err(|e| format!("获取数据源失败: {}", e))?
        .ok_or_else(|| "数据源不存在".to_string())?;

    fetch_postgresql_tables(db.pool(), &datasource, &database_name).await
}

/// 读取 SQLite 数据库的表列表
#[tauri::command]
async fn cmd_fetch_sqlite_tables(
    datasource_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    // 获取数据源信息
    let datasource = db.get_datasource(datasource_id).await
        .map_err(|e| format!("获取数据源失败: {}", e))?
        .ok_or_else(|| "数据源不存在".to_string())?;

    fetch_sqlite_tables(db.pool(), &datasource).await
}

/// 导入单个表
#[tauri::command]
async fn cmd_import_single_table(
    project_id: i64,
    datasource_id: i64,
    database_name: String,
    table_name: String,
    table_comment: Option<String>,
    table_type: String,
    engine: Option<String>,
    row_count: i64,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    // 获取数据源信息
    let datasource = db.get_datasource(datasource_id).await
        .map_err(|e| format!("获取数据源失败: {}", e))?
        .ok_or_else(|| "数据源不存在".to_string())?;

    import_single_table(
        db.pool(),
        project_id,
        &datasource,
        &database_name,
        &table_name,
        table_comment.as_deref(),
        &table_type,
        engine.as_deref(),
        row_count,
    ).await
}

/// 获取系统用户名
#[tauri::command]
fn get_username() -> Result<String, String> {
    use std::env;

    // 尝试获取 USER 环境变量
    if let Some(username) = env::var("USER").ok() {
        Ok(username)
    } else if let Some(username) = env::var("USERNAME").ok() {
        Ok(username)
    } else {
        // 如果环境变量都获取不到，尝试使用 dirs 库
        if let Some(home_dir) = dirs::home_dir() {
            if let Some(username_osstr) = home_dir.file_name() {
                if let Some(username_str) = username_osstr.to_str() {
                    return Ok(username_str.to_string());
                }
            }
        }
        Err("无法获取用户名".to_string())
    }
}

/// 获取系统主题
#[tauri::command]
fn get_system_theme() -> Result<String, String> {
    // 在 Windows 上检测系统主题需要访问注册表，这里返回默认值
    // 用户可以在设置中手动切换主题
    Ok("light".to_string())
}

/// 下载模板版本到本地
/// 注意：version 参数应该是实际的版本号（如 "1.0.0"）
/// 前端会自动选择 is_latest 的版本，与 CLI 保持完全一致的逻辑
#[tauri::command]
async fn download_template(template_id: String, version: String) -> Result<String, String> {
    use reqwest::Client;
    use std::fs;
    use std::io::Write;

    println!("下载模板: {} 版本: {}", template_id, version);

    // 加载配置
    let config = Config::load().map_err(|e| e.to_string())?;

    // 获取目标路径
    let target_dir = config.get_template_path(&template_id, &version);
    println!("目标路径: {:?}", target_dir);

    // 如果目录已存在，说明已下载
    if target_dir.exists() {
        return Ok(target_dir.to_string_lossy().to_string());
    }

    // 创建目标目录
    fs::create_dir_all(&target_dir).map_err(|e| format!("创建目录失败: {}", e))?;

    // 构建 API URL
    let version_str = if version.is_empty() { "latest".to_string() } else { version.clone() };
    let download_url = format!(
        "{}/api/v1/template/templates/{}/releases/{}/download",
        config.web_server.api_url, template_id, version_str
    );

    println!("下载 URL: {}", download_url);

    // 下载 ZIP 文件
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(300)) // 5分钟超时
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    let response = client.get(&download_url)
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("下载失败: HTTP {}", response.status()));
    }

    let zip_bytes = response.bytes().await
        .map_err(|e| format!("读取响应体失败: {}", e))?;

    // 保存为临时 ZIP 文件
    let zip_path = target_dir.join(".temp.zip");
    let mut file = fs::File::create(&zip_path)
        .map_err(|e| format!("创建ZIP文件失败: {}", e))?;
    file.write_all(&zip_bytes)
        .map_err(|e| format!("写入ZIP文件失败: {}", e))?;

    // 解压 ZIP 文件
    println!("解压ZIP文件...");
    let mut archive = zip::ZipArchive::new(fs::File::open(&zip_path)
        .map_err(|e| format!("打开ZIP文件失败: {}", e))?)
        .map_err(|e| format!("读取ZIP存档失败: {}", e))?;

    archive.extract(&target_dir)
        .map_err(|e| format!("解压失败: {}", e))?;

    // 删除临时 ZIP 文件
    fs::remove_file(&zip_path)
        .map_err(|e| format!("删除临时文件失败: {}", e))?;

    println!("模板下载成功: {:?}", target_dir);

    Ok(target_dir.to_string_lossy().to_string())
}

// 数据结构
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub template_type: String,
    pub language: Option<String>,
    pub is_featured: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Variable {
    pub name: String,
    pub title: String,
    pub description: String,
    pub type_: String,
    pub default_value: Option<String>,
    pub required: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct RenderedFile {
    pub path: String,
    pub content: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 使用 block_in_place 来在 setup 中等待异步数据库初始化
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                println!("初始化数据库...");
                match Database::init().await {
                    Ok(database) => {
                        println!("数据库初始化完成");
                        // 将数据库存储为应用状态
                        let db_state = DbState(Arc::new(database));
                        handle.manage(db_state);
                        handle.manage(BrowserPoolCache::new());
                    }
                    Err(e) => {
                        eprintln!("数据库初始化失败: {}", e);
                        panic!("数据库初始化失败");
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            write_text_file,
            window_minimize,
            window_maximize,
            window_close,
            list_templates,
            get_template_variables,
            render_template,
            render_template_preview,
            generate_project,
            check_template_downloaded,
            download_template,
            check_directory_exists,
            remove_directory,
            get_config,
            update_web_server_config,
            update_template_path,
            get_username,
            get_system_theme,
            // 数据库命令
            db_get_statistics,
            db_get_recent_projects,
            db_get_all_projects,
            db_get_project,
            db_create_project,
            db_update_project,
            db_delete_project,
            db_get_all_datasources,
            db_create_datasource,
            db_get_datasource,
            db_update_datasource,
            db_delete_datasource,
            test_datasource_connection,
            cmd_list_database_tables,
            cmd_get_table_columns,
            cmd_query_table_data,
            cmd_get_connection_status,
            db_get_project_tables,
            db_create_table,
            cmd_import_tables_from_datasource,
            db_get_table_columns,
            db_delete_table,
            db_update_table,
            db_create_column,
            db_update_column,
            db_delete_column,
            db_reorder_columns,
            cmd_parse_sql_and_create,
            // 语言命令
            db_get_all_languages,
            db_get_language,
            db_create_language,
            db_update_language,
            db_delete_language,
            db_set_project_primary_language,
            db_get_project_languages,
            db_add_project_language,
            db_remove_project_language,
            // 语言类型字段命令
            db_get_language_field_types,
            db_create_language_field_type,
            db_update_language_field_type,
            db_delete_language_field_type,
            db_batch_save_language_field_types,
            // 系统级类型映射命令
            db_get_system_type_mappings,
            db_get_system_type_mappings_by_lang_db,
            db_create_system_type_mapping,
            db_update_system_type_mapping,
            db_delete_system_type_mapping,
            db_batch_save_system_type_mappings,
            // 项目级类型映射命令
            db_get_project_type_mappings,
            db_get_project_type_mappings_by_scope,
            db_create_project_type_mapping,
            db_update_project_type_mapping,
            db_delete_project_type_mapping,
            db_batch_save_project_type_mappings,
            db_copy_system_mappings_to_project,
            // 表结构读取和导入命令
            cmd_fetch_mysql_tables,
            cmd_fetch_postgresql_tables,
            cmd_fetch_sqlite_tables,
            cmd_import_single_table,
            // AI 服务命令
            ai_get_all_providers,
            ai_get_provider,
            ai_save_provider,
            ai_toggle_provider,
            ai_delete_provider,
            ai_get_provider_models_grouped,
            ai_add_model,
            ai_delete_model,
            ai_update_model,
            ai_fetch_models,
            ai_batch_add_models,
            ai_generate_sql,
            ai_fix_sql,
            parse_ai_sql,
            execute_ai_sql,
            // 项目规范命令
            db_get_table_preferences,
            db_save_table_preferences,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
