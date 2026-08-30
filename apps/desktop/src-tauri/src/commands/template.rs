use std::path::Path;
use template_studio_template_core::{
    filter_files_by_conditions, render_string, render_tree, ConditionsYaml, TemplateFile, Variables,
};

use crate::config::Config;

// ---------------------------------------------------------------------------
// 数据结构
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Template {
    id: String,
    name: String,
    description: Option<String>,
    template_type: String,
    language: Option<String>,
    is_featured: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct RenderedFile {
    path: String,
    content: String,
}

// ---------------------------------------------------------------------------
// Tauri Commands
// ---------------------------------------------------------------------------

/// 获取模板列表：优先读本地已下载模板，为空时回退 Web 服务端公开模板列表
///
/// 桌面端定位是离线优先：用户通过模板广场下载的模板落盘在本地，
/// 模板选择器应展示这些可离线渲染的模板；本地一个都没有时（首次使用）
/// 引导性地展示服务端公开模板（此时渲染需在线下载）。
#[tauri::command]
pub async fn list_templates() -> Result<Vec<Template>, String> {
    // 1) 本地已下载模板（data/templates/<id>/.meta/variables/variables.json 提供元数据）
    let mut templates: Vec<Template> = Vec::new();
    if let Ok(config) = Config::load() {
        let base = config.storage.template_path.clone();
        if let Ok(entries) = std::fs::read_dir(&base) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let id = entry.file_name().to_string_lossy().to_string();
                let meta_path = path.join(".meta").join("variables").join("variables.json");
                let (name, description) = if meta_path.exists() {
                    match std::fs::read_to_string(&meta_path) {
                        Ok(content) => {
                            let v: serde_json::Value =
                                serde_json::from_str(&content).unwrap_or(serde_json::json!({}));
                            (
                                v.get("_templateName")
                                    .and_then(|x| x.as_str())
                                    .unwrap_or(&id)
                                    .to_string(),
                                v.get("_templateDescription")
                                    .and_then(|x| x.as_str())
                                    .map(|x| x.to_string()),
                            )
                        }
                        Err(_) => (id.clone(), None),
                    }
                } else {
                    (id.clone(), None)
                };
                templates.push(Template {
                    id,
                    name,
                    description,
                    template_type: "web".to_string(),
                    language: None,
                    is_featured: false,
                });
            }
        }
    }
    if !templates.is_empty() {
        templates.sort_by(|a, b| a.name.cmp(&b.name));
        return Ok(templates);
    }

    // 2) 回退：Web 服务端公开模板列表（离线时返回空并提示由前端处理）
    if let Ok(config) = Config::load() {
        let url = format!("{}/api/v1/studio/templates/list", config.web_server.api_url);
        if let Ok(client) = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
        {
            if let Ok(resp) = client
                .get(&url)
                .query(&[("page", "1"), ("pageSize", "50")])
                .send()
                .await
            {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    if let Some(list) = body
                        .pointer("/data/templatesList")
                        .and_then(|x| x.as_array())
                    {
                        for item in list {
                            templates.push(Template {
                                id: item
                                    .get("id")
                                    .and_then(|x| x.as_i64())
                                    .map(|x| x.to_string())
                                    .unwrap_or_default(),
                                name: item
                                    .get("name")
                                    .and_then(|x| x.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                description: item
                                    .get("description")
                                    .and_then(|x| x.as_str())
                                    .map(|x| x.to_string()),
                                template_type: item
                                    .get("templateType")
                                    .and_then(|x| x.as_str())
                                    .unwrap_or("web")
                                    .to_string(),
                                language: None,
                                is_featured: item
                                    .get("isFeatured")
                                    .and_then(|x| x.as_i64())
                                    .map(|x| x != 0)
                                    .unwrap_or(false),
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(templates)
}

/// 获取模板变量定义（从本地 variables.json 读取）
#[tauri::command]
pub async fn get_template_variables(
    template_id: &str,
    version: Option<String>,
) -> Result<String, String> {
    let config = Config::load().map_err(|e| format!("加载配置失败: {}", e))?;
    let version_str = version.unwrap_or_else(|| "latest".to_string());
    let template_path = config.get_template_path(template_id, &version_str);

    if !template_path.exists() {
        return Ok(r#"{"fields": []}"#.to_string());
    }

    let variables_file = template_path.join(".meta/variables/variables.json");
    if variables_file.exists() {
        let content = std::fs::read_to_string(&variables_file)
            .map_err(|e| format!("读取变量文件失败: {}", e))?;
        Ok(content)
    } else {
        Ok(r#"{"fields": []}"#.to_string())
    }
}

/// 渲染模板（旧入口，保留兼容）：按 templateId 定位本地模板并整树渲染
///
/// 历史上这里是硬编码模拟内容；现复用 render_template_preview 的本地渲染
/// 链路（扫描 + 条件过滤 + render_tree），返回前端期望的扁平文件列表。
#[tauri::command]
pub async fn render_template(
    template_id: String,
    variables: serde_json::Value,
) -> Result<Vec<RenderedFile>, String> {
    let result_json = render_template_preview(template_id, variables, None).await?;
    // preview 返回 core::RenderedFile 树（file_path/file_content/is_directory）
    let rendered: Vec<serde_json::Value> =
        serde_json::from_str(&result_json).map_err(|e| format!("解析渲染结果失败: {}", e))?;
    Ok(rendered
        .into_iter()
        .filter(|f| f.get("isDirectory").and_then(|v| v.as_i64()) == Some(0))
        .map(|f| RenderedFile {
            path: f
                .get("filePath")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            content: f
                .get("fileContent")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
        .collect())
}

/// 渲染模板预览（返回文件树）
#[tauri::command]
pub async fn render_template_preview(
    template_id: String,
    variables: serde_json::Value,
    version: Option<String>,
) -> Result<String, String> {
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
    let template_files =
        scan_template_files(&template_path).map_err(|e| format!("扫描模板文件失败: {}", e))?;

    println!("扫描到 {} 个文件/目录", template_files.len());

    // 3. 准备变量
    let variables_json =
        serde_json::to_string(&variables).map_err(|e| format!("序列化变量失败: {}", e))?;

    let render_vars =
        Variables::from_json(&variables_json).map_err(|e| format!("解析变量失败: {}", e))?;

    // 4. 渲染模板树（先应用文件生成条件过滤）
    let filtered_files = apply_file_conditions(&template_path, template_files, &render_vars);
    let rendered_tree =
        render_tree(filtered_files, &render_vars).map_err(|e| format!("渲染模板失败: {}", e))?;

    println!("渲染完成，生成 {} 个文件节点", rendered_tree.len());

    // 5. 序列化为 JSON 返回
    let result_json =
        serde_json::to_string(&rendered_tree).map_err(|e| format!("序列化结果失败: {}", e))?;

    Ok(result_json)
}

/// 渲染并导出模板到指定目录
#[tauri::command]
pub async fn cmd_render_and_export(
    template_id: String,
    version: Option<String>,
    variables_json: serde_json::Value,
    output_dir: String,
) -> Result<String, String> {
    let config = Config::load().map_err(|e| format!("加载配置失败: {}", e))?;
    let version_str = version.unwrap_or_else(|| "latest".to_string());
    let template_path = config.get_template_path(&template_id, &version_str);

    if !template_path.exists() {
        return Err(format!("模板未下载，路径: {:?}", template_path));
    }

    // 扫描并渲染
    let template_files =
        scan_template_files(&template_path).map_err(|e| format!("扫描模板文件失败: {}", e))?;

    let vars_str =
        serde_json::to_string(&variables_json).map_err(|e| format!("序列化变量失败: {}", e))?;
    let render_vars =
        Variables::from_json(&vars_str).map_err(|e| format!("解析变量失败: {}", e))?;

    let rendered_tree = render_tree(
        apply_file_conditions(&template_path, template_files, &render_vars),
        &render_vars,
    )
    .map_err(|e| format!("渲染模板失败: {}", e))?;

    // 写入文件
    let output = Path::new(&output_dir);
    let mut exported = 0u32;
    let mut errors: Vec<String> = Vec::new();

    for file_node in rendered_tree {
        if let Some(err) = &file_node.error {
            errors.push(format!("{}: {}", file_node.file_path, err.message));
            continue;
        }
        if file_node.is_directory == 1 {
            let dir_path = output.join(&file_node.file_path);
            if let Err(e) = std::fs::create_dir_all(&dir_path) {
                errors.push(format!("创建目录失败 {}: {}", file_node.file_path, e));
            }
            continue;
        }

        let file_path = output.join(&file_node.file_path);
        if let Some(parent) = file_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        match file_node.file_content {
            Some(content) => {
                if let Err(e) = std::fs::write(&file_path, &content) {
                    errors.push(format!("写入失败 {}: {}", file_node.file_path, e));
                } else {
                    exported += 1;
                }
            }
            None => {
                // 二进制文件，从模板目录复制
                let source = template_path.join(&file_node.file_path);
                if let Err(e) = std::fs::copy(&source, &file_path) {
                    errors.push(format!("复制失败 {}: {}", file_node.file_path, e));
                } else {
                    exported += 1;
                }
            }
        }
    }

    let result = serde_json::json!({
        "exported": exported,
        "errors": errors,
    });
    serde_json::to_string(&result).map_err(|e| format!("序列化结果失败: {}", e))
}

/// 生成项目
#[tauri::command]
pub async fn generate_project(
    template_id: String,
    variables: serde_json::Value,
    output_path: String,
    version: Option<String>,
) -> Result<String, String> {
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
    let template_files =
        scan_template_files(&template_path).map_err(|e| format!("扫描模板文件失败: {}", e))?;

    println!("扫描到 {} 个文件/目录", template_files.len());

    // 3. 准备变量
    let variables_json =
        serde_json::to_string(&variables).map_err(|e| format!("序列化变量失败: {}", e))?;

    let render_vars =
        Variables::from_json(&variables_json).map_err(|e| format!("解析变量失败: {}", e))?;

    // 4. 渲染模板树（先应用文件生成条件过滤）
    let rendered_tree = render_tree(
        apply_file_conditions(&template_path, template_files, &render_vars),
        &render_vars,
    )
    .map_err(|e| format!("渲染模板失败: {}", e))?;

    println!("渲染完成，生成 {} 个文件节点", rendered_tree.len());

    // 5. 写入文件到输出目录
    let output_dir = Path::new(&output_path);
    let template_path = config.get_template_path(&template_id, &version_str);

    // 如果输出目录已存在，先删除
    if output_dir.exists() {
        std::fs::remove_dir_all(output_dir).map_err(|e| format!("删除已存在目录失败: {}", e))?;
    }

    // 创建输出目录
    std::fs::create_dir_all(output_dir).map_err(|e| format!("创建输出目录失败: {}", e))?;

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
            std::fs::create_dir_all(&file_path)
                .map_err(|e| format!("创建目录 {:?} 失败: {}", file_path, e))?;
        } else {
            // 写入文件
            if let Some(parent) = file_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("创建父目录 {:?} 失败: {}", parent, e))?;
                }
            }

            // 检查文件内容
            match &file_node.file_content {
                Some(content) if !content.is_empty() => {
                    // 有内容：文本文件，写入渲染后的内容
                    std::fs::write(&file_path, content)
                        .map_err(|e| format!("写入文件 {:?} 失败: {}", file_path, e))?;

                    success_count += 1;
                    println!("创建文件: {}", file_node.file_path);
                }
                _ => {
                    // None 或空字符串：二进制文件或读取失败，直接复制原文件
                    match std::fs::copy(&source_path, &file_path) {
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
        output_path, success_count
    ))
}

/// 检查模板版本是否已下载
#[tauri::command]
pub fn check_template_downloaded(template_id: String, version: String) -> bool {
    if let Ok(config) = Config::load() {
        let template_path = config.get_template_path(&template_id, &version);
        template_path.exists()
    } else {
        false
    }
}

/// 检查目录是否存在
#[tauri::command]
pub fn check_directory_exists(path: String) -> bool {
    Path::new(&path).exists()
}

/// 删除目录（用于覆盖已存在的项目）
#[tauri::command]
pub fn remove_directory(path: String) -> Result<String, String> {
    let path = Path::new(&path);

    if path.exists() {
        std::fs::remove_dir_all(path).map_err(|e| format!("删除目录失败: {}", e))?;
        Ok("目录删除成功".to_string())
    } else {
        Ok("目录不存在".to_string())
    }
}

/// 下载模板版本到本地
/// 注意：version 参数应该是实际的版本号（如 "1.0.0"）
/// 前端会自动选择 is_latest 的版本，与 CLI 保持完全一致的逻辑
#[tauri::command]
pub async fn download_template(template_id: String, version: String) -> Result<String, String> {
    use reqwest::Client;
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
    std::fs::create_dir_all(&target_dir).map_err(|e| format!("创建目录失败: {}", e))?;

    // 构建 API URL
    let version_str = if version.is_empty() {
        "latest".to_string()
    } else {
        version.clone()
    };
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

    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("下载失败: HTTP {}", response.status()));
    }

    let zip_bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取响应体失败: {}", e))?;

    // 保存为临时 ZIP 文件
    let zip_path = target_dir.join(".temp.zip");
    let mut file =
        std::fs::File::create(&zip_path).map_err(|e| format!("创建ZIP文件失败: {}", e))?;
    file.write_all(&zip_bytes)
        .map_err(|e| format!("写入ZIP文件失败: {}", e))?;

    // 解压 ZIP 文件
    println!("解压ZIP文件...");
    let mut archive = zip::ZipArchive::new(
        std::fs::File::open(&zip_path).map_err(|e| format!("打开ZIP文件失败: {}", e))?,
    )
    .map_err(|e| format!("读取ZIP存档失败: {}", e))?;

    archive
        .extract(&target_dir)
        .map_err(|e| format!("解压失败: {}", e))?;

    // 删除临时 ZIP 文件
    std::fs::remove_file(&zip_path).map_err(|e| format!("删除临时文件失败: {}", e))?;

    println!("模板下载成功: {:?}", target_dir);

    Ok(target_dir.to_string_lossy().to_string())
}

// ---------------------------------------------------------------------------
// 私有辅助函数
// ---------------------------------------------------------------------------

/// 扫描模板目录，构建文件树
fn scan_template_files(template_path: &Path) -> Result<Vec<TemplateFile>, String> {
    let mut files = Vec::new();
    let mut id_counter = 1i64;

    // 递归扫描模板目录（排除 .meta 目录）
    scan_directory_recursive(template_path, template_path, &mut files, &mut id_counter, 0)?;

    Ok(files)
}

/// 读取模板目录下 .meta/variables/conditions.yml 并应用文件条件过滤
///
/// 与 Web 服务端渲染语义保持一致：无条件配置文件时全部生成；
/// 条件不满足的文件/目录及其子树被排除，不参与渲染
fn apply_file_conditions(
    template_path: &Path,
    files: Vec<TemplateFile>,
    variables: &Variables,
) -> Vec<TemplateFile> {
    let conditions_path = template_path
        .join(".meta")
        .join("variables")
        .join("conditions.yml");

    let Ok(content) = std::fs::read_to_string(&conditions_path) else {
        return files; // 无条件配置，全部生成
    };

    let yaml = match ConditionsYaml::from_yaml(&content) {
        Ok(y) => y,
        Err(e) => {
            eprintln!(
                "[警告] 解析 conditions.yml 失败 ({:?})：{}，将忽略文件条件",
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
    if matched > 0 || !yaml.conditions.is_empty() {
        println!(
            "文件条件：{} 条配置，命中 {} 个文件",
            yaml.conditions.len(),
            matched
        );
    }

    filter_files_by_conditions(files, variables)
}

/// 递归扫描目录
fn scan_directory_recursive(
    base_path: &Path,
    dir: &Path,
    files: &mut Vec<TemplateFile>,
    id_counter: &mut i64,
    parent_id: i64,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("读取目录失败 {:?}: {}", dir, e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();

        // 跳过 .meta 目录和 .git 目录
        if path.is_dir() {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

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

            let metadata = std::fs::metadata(&path)
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

            let is_binary = matches!(
                extension.as_str(),
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

            let file_content = if is_binary {
                // 二进制文件：使用空字符串标记（渲染时会跳过，直接复制原文件）
                println!("  [二进制文件] {} - 跳过内容读取", relative_path.display());
                String::new()
            } else {
                // 文本文件：尝试读取内容
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        println!(
                            "  [文本文件] {} - {} 字节",
                            relative_path.display(),
                            content.len()
                        );
                        content
                    }
                    Err(e) => {
                        // 读取失败，可能是编码问题，记录但继续
                        eprintln!("  [警告] 读取文件失败 {:?}: {}, 将跳过该文件", path, e);
                        String::new() // 使用空字符串，渲染时会跳过
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
