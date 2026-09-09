use crate::state::DbState;

/// 获取所有 AI 提供商
#[tauri::command]
pub async fn ai_get_all_providers(database: tauri::State<'_, DbState>) -> Result<String, String> {
    let db = database.as_ref();

    let providers = db
        .get_all_ai_providers()
        .await
        .map_err(|e| format!("获取 AI 提供商失败: {}", e))?;

    serde_json::to_string(&providers).map_err(|e| format!("序列化失败: {}", e))
}

/// 获取单个 AI 提供商
#[tauri::command]
pub async fn ai_get_provider(
    provider_name: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let provider = db
        .get_ai_provider(&provider_name)
        .await
        .map_err(|e| format!("获取 AI 提供商失败: {}", e))?
        .ok_or_else(|| "AI 提供商不存在".to_string())?;

    serde_json::to_string(&provider).map_err(|e| format!("序列化失败: {}", e))
}

/// 保存 AI 提供商配置
#[tauri::command]
pub async fn ai_save_provider(
    params: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let provider_name = params
        .get("providerName")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少提供商名称".to_string())?;

    let display_name = params
        .get("displayName")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少显示名称".to_string())?;

    let provider_type = params
        .get("providerType")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少提供商类型".to_string())?;

    let api_key = params.get("apiKey").and_then(|v| v.as_str());
    let api_endpoint = params.get("apiEndpoint").and_then(|v| v.as_str());
    let is_enabled = params
        .get("isEnabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let temperature = params
        .get("temperature")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.7);
    let max_tokens = params
        .get("maxTokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(4096) as i32;
    let protocol = params
        .get("protocol")
        .and_then(|v| v.as_str())
        .unwrap_or(crate::ai_runtime::Protocol::OpenAiCompatible.as_str());

    let _id = db
        .save_ai_provider(
            provider_name,
            display_name,
            provider_type,
            api_key,
            api_endpoint,
            is_enabled,
            temperature,
            max_tokens,
            protocol,
        )
        .await
        .map_err(|e| format!("保存 AI 提供商失败: {}", e))?;

    Ok("配置已保存".to_string())
}

/// 切换 AI 提供商启用状态
#[tauri::command]
pub async fn ai_toggle_provider(
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
pub async fn ai_delete_provider(
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
pub async fn ai_get_provider_models_grouped(
    provider_name: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    let groups = db
        .get_ai_provider_models_grouped(&provider_name)
        .await
        .map_err(|e| format!("获取模型列表失败: {}", e))?;

    serde_json::to_string(&groups).map_err(|e| format!("序列化失败: {}", e))
}

/// 添加 AI 模型
#[tauri::command]
pub async fn ai_add_model(
    params: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    let model_id = params
        .get("modelId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少模型 ID".to_string())?;

    let model_name = params
        .get("modelName")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少模型名称".to_string())?;

    let provider_name = params
        .get("providerName")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少提供商名称".to_string())?;

    let group_id = params
        .get("groupId")
        .and_then(|v| v.as_str())
        .unwrap_or("chat");

    let description = params.get("description").and_then(|v| v.as_str());
    let max_tokens = params
        .get("maxTokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(4096) as i32;

    let id = db
        .add_ai_model(
            model_id,
            model_name,
            provider_name,
            group_id,
            description,
            max_tokens,
        )
        .await
        .map_err(|e| format!("添加模型失败: {}", e))?;

    Ok(id)
}

/// 删除 AI 模型
#[tauri::command]
pub async fn ai_delete_model(
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
pub async fn ai_update_model(
    model_id: i64,
    params: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<(), String> {
    let db = database.as_ref();

    let new_model_id = params
        .get("modelId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少模型 ID".to_string())?;

    let model_name = params
        .get("modelName")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少模型名称".to_string())?;

    let group_id = params
        .get("groupId")
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
pub async fn ai_fetch_models(
    provider_name: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    // 获取提供商配置
    let provider_config = db
        .get_ai_provider(&provider_name)
        .await
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

    let response_json: serde_json::Value = response
        .json()
        .await
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

    serde_json::to_string(&models).map_err(|e| format!("序列化失败: {}", e))
}

/// 批量添加 AI 模型
#[tauri::command]
pub async fn ai_batch_add_models(
    provider_name: String,
    models: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<i64, String> {
    let db = database.as_ref();

    let models_array = models
        .as_array()
        .ok_or_else(|| "models 格式错误：应为数组".to_string())?;

    let mut model_tuples: Vec<(&str, &str, &str, &str, Option<&str>, i32)> = Vec::new();
    for m in models_array {
        let model_id = m.get("modelId").and_then(|v| v.as_str()).unwrap_or("");
        let model_name = m
            .get("modelName")
            .and_then(|v| v.as_str())
            .unwrap_or(model_id);
        let group_id = m.get("groupId").and_then(|v| v.as_str()).unwrap_or("chat");
        let description = m.get("description").and_then(|v| v.as_str());
        let max_tokens = m.get("maxTokens").and_then(|v| v.as_i64()).unwrap_or(4096) as i32;

        if !model_id.is_empty() {
            model_tuples.push((
                model_id,
                model_name,
                &provider_name,
                group_id,
                description,
                max_tokens,
            ));
        }
    }

    let count = db
        .batch_add_ai_models(&model_tuples)
        .await
        .map_err(|e| format!("批量添加模型失败: {}", e))?;

    Ok(count)
}

// ===== AI 连接测试命令 =====

/// 测试 AI 提供商连接
#[tauri::command]
pub async fn ai_test_connection(
    provider_name: String,
    provider_type: String,
    api_key: String,
    api_endpoint: String,
    _model: String,
) -> Result<String, String> {
    use reqwest::Client;

    let client = Client::new();

    // 使用 /models 接口快速检测连通性
    let url = match provider_type.as_str() {
        "ollama" => {
            let base = if api_endpoint.is_empty() {
                "http://localhost:11434"
            } else {
                &api_endpoint
            };
            format!("{}/api/tags", base)
        }
        _ => {
            let base = if api_endpoint.is_empty() {
                get_default_endpoint(&provider_name)
            } else {
                api_endpoint.clone()
            };
            format!("{}/models", base.trim_end_matches('/'))
        }
    };

    let mut request = client.get(&url).timeout(std::time::Duration::from_secs(5));

    if !api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", api_key));
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("连接失败: {}", e))?;

    let status = response.status();
    if status.is_success() {
        Ok(format!("连接成功 (HTTP {})", status.as_u16()))
    } else {
        let text = response.text().await.unwrap_or_default();
        Err(format!("连接失败 (HTTP {}): {}", status.as_u16(), text))
    }
}

// ===== AI SQL 生成和修复命令 =====

/// AI 生成 SQL（支持多轮对话）
#[tauri::command]
pub async fn ai_generate_sql(
    provider: String,
    model: String,
    messages: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    // 获取提供商配置
    let provider_config = db
        .get_ai_provider(&provider)
        .await
        .map_err(|e| format!("获取提供商失败: {}", e))?
        .ok_or_else(|| "提供商不存在".to_string())?;

    let protocol = crate::ai_runtime::Protocol::parse(
        provider_config["protocol"].as_str().unwrap_or("openai_compatible"),
    );

    // 验证 messages 格式
    let messages_array = messages
        .as_array()
        .ok_or_else(|| "messages 格式错误：应为数组".to_string())?;

    let mut target = crate::ai_runtime::call_target_from_provider(
        &provider_config, &model, protocol,
    )?;
    // 行为等价迁移:沿用原命令的采样参数
    target.temperature = 0.3;
    target.max_tokens = 2000;

    crate::ai_runtime::chat_openai_style(&target, messages_array).await
}

/// AI 修复 SQL
#[tauri::command]
pub async fn ai_fix_sql(
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
    let provider_config = db
        .get_ai_provider(&provider)
        .await
        .map_err(|e| format!("获取提供商失败: {}", e))?
        .ok_or_else(|| "提供商不存在".to_string())?;

    let protocol = crate::ai_runtime::Protocol::parse(
        provider_config["protocol"].as_str().unwrap_or("openai_compatible"),
    );

    let mut target = crate::ai_runtime::call_target_from_provider(
        &provider_config, &model, protocol,
    )?;
    target.temperature = 0.2;
    target.max_tokens = 2000;

    crate::ai_runtime::chat(&target, None, &prompt, &[]).await
}

/// 解析 AI 生成的 SQL（只返回表结构，不创建）
#[tauri::command]
pub async fn parse_ai_sql(
    project_id: i64,
    sql: String,
    dialect: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    crate::database::import::parse_sql_only(db.pool(), project_id, &sql, &dialect).await
}

/// 执行 AI 生成的 SQL（在数据库中创建表）
#[tauri::command]
pub async fn execute_ai_sql(
    project_id: i64,
    sql: String,
    dialect: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();

    crate::database::import::parse_and_create_from_sql(db.pool(), project_id, &sql, &dialect).await
}

/// 获取默认 API 端点
fn get_default_endpoint(provider: &str) -> String {
    match provider {
        "deepseek" => "https://api.deepseek.com/v1".to_string(),
        "glm" => "https://open.bigmodel.cn/api/paas/v4".to_string(),
        "openai" => "https://api.openai.com/v1".to_string(),
        "longcat" => "https://api.longcat.chat/openai".to_string(),
        "mimo" => "https://api.xiaomimimo.com/v1".to_string(),
        "cherry-studio" => "http://127.0.0.1:23333/v1".to_string(),
        _ => "https://api.openai.com/v1".to_string(),
    }
}

// ===== AI 助手与变量命令（ai_runtime 统一执行层） =====

use crate::ai_runtime::{call_target_from_provider, CallTarget, Protocol};

/// 解析默认调用目标(默认提供商 + 其第一个模型)
async fn default_call_target(db: &crate::database::Database) -> Result<CallTarget, String> {
    let provider = db
        .get_default_ai_provider()
        .await
        .map_err(|e| format!("获取默认提供商失败: {}", e))?
        .ok_or_else(|| "未配置可用的 AI 提供商，请先在设置中配置".to_string())?;

    let provider_name = provider["providerName"].as_str().unwrap_or_default().to_string();
    let model = db
        .get_first_chat_model(&provider_name)
        .await
        .map_err(|e| format!("获取模型失败: {}", e))?
        .ok_or_else(|| "默认提供商下没有模型，请先在设置中添加".to_string())?;

    let protocol = Protocol::parse(provider["protocol"].as_str().unwrap_or("openai_compatible"));
    call_target_from_provider(&provider, &model, protocol)
}

/// 从模板内容提取 {{ 变量 }} 占位符(去过滤器、去重、保留点路径)
fn extract_placeholders(content: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rest = content;
    while let Some(start) = rest.find("{{") {
        rest = &rest[start + 2..];
        let end = match rest.find("}}") {
            Some(e) => e,
            None => break,
        };
        let expr = rest[..end].trim();
        rest = &rest[end + 2..];
        let path = expr.split('|').next().unwrap_or("").trim();
        let valid = !path.is_empty()
            && path.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_')
            && path.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.');
        if valid && !out.iter().any(|v| v == path) {
            out.push(path.to_string());
        }
    }
    out
}

/// 遍历模板目录收集文本文件样本(扩展名白名单 + 大小/数量/深度上限)
fn collect_template_samples(root: &std::path::Path) -> Vec<(String, String)> {
    const EXTS: [&str; 24] = [
        "rs", "ts", "js", "vue", "java", "kt", "go", "py", "cs", "php", "rb", "sql", "yml",
        "yaml", "json", "toml", "xml", "html", "css", "scss", "md", "txt", "sh", "properties",
    ];
    const SKIP_DIRS: [&str; 7] = ["node_modules", ".git", "target", "dist", "build", ".venv", "__pycache__"];
    const MAX_FILE: u64 = 128 * 1024;
    const MAX_TOTAL: usize = 512 * 1024;
    const MAX_FILES: usize = 200;
    const MAX_DEPTH: usize = 6;

    let mut files: Vec<(String, String)> = Vec::new();
    let mut total = 0usize;
    let mut stack: Vec<(std::path::PathBuf, usize)> = vec![(root.to_path_buf(), 0)];

    while let Some((dir, depth)) = stack.pop() {
        if depth > MAX_DEPTH || files.len() >= MAX_FILES || total >= MAX_TOTAL {
            break;
        }
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                if !SKIP_DIRS.contains(&name.as_str()) && !name.starts_with('.') {
                    stack.push((path, depth + 1));
                }
            } else {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if !EXTS.contains(&ext) {
                    continue;
                }
                let Ok(meta) = entry.metadata() else { continue };
                if meta.len() > MAX_FILE || total >= MAX_TOTAL {
                    continue;
                }
                let rel = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
                if let Ok(content) = std::fs::read_to_string(&path) {
                    total += content.len();
                    files.push((rel, content));
                    if files.len() >= MAX_FILES {
                        break;
                    }
                }
            }
        }
    }
    files
}

/// 从 AI 回复中剥离代码围栏提取 JSON 对象
fn parse_json_reply(reply: &str) -> Option<serde_json::Value> {
    let cleaned = reply
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(cleaned) {
        return Some(v);
    }
    let start = cleaned.find('{')?;
    let end = cleaned.rfind('}')?;
    serde_json::from_str::<serde_json::Value>(&cleaned[start..=end]).ok()
}

/// 项目上下文:表与字段概览(用于变量填充/助手)
async fn project_schema_summary(
    db: &crate::database::Database,
    project_id: i64,
) -> Result<String, String> {
    let rows = sqlx::query(
        "SELECT t.name AS table_name, t.comment AS table_comment,
                c.name AS col_name, c.data_type AS col_type, c.comment AS col_comment
         FROM db_tables t LEFT JOIN db_columns c ON c.table_id = t.id
         WHERE t.project_id = ?1 AND t.table_type = 'table'
         ORDER BY t.id, c.ordinal_position",
    )
    .bind(project_id)
    .fetch_all(db.pool())
    .await
    .map_err(|e| format!("读取项目表结构失败: {}", e))?;

    use sqlx::Row;
    let mut out = String::new();
    let mut cur_table = String::new();
    let mut table_count = 0;
    for r in rows {
        let t: String = r.get("table_name");
        if t != cur_table {
            table_count += 1;
            if table_count > 40 {
                break;
            }
            cur_table = t.clone();
            let tc: Option<String> = r.get("table_comment");
            out.push_str(&format!(
                "\n表 {}{}:",
                t,
                tc.map(|c| format!("({})", c)).unwrap_or_default()
            ));
        }
        let cn: Option<String> = r.get("col_name");
        if let Some(cn) = cn {
            let ct: Option<String> = r.get("col_type");
            let cc: Option<String> = r.get("col_comment");
            out.push_str(&format!(
                "\n  {} {}{}",
                cn,
                ct.unwrap_or_default(),
                cc.map(|c| format!("  // {}", c)).unwrap_or_default()
            ));
        }
    }
    Ok(out)
}

/// AI 助手对话(上下文=模板变量/项目表结构/调用方附加上下文;支持多轮历史)
#[tauri::command]
pub async fn ai_chat(
    message: String,
    template_path: Option<String>,
    project_id: Option<i64>,
    extra_context: Option<String>,
    history: Option<serde_json::Value>,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();
    let target = default_call_target(db).await?;

    let mut context = String::from(
        "你是 Template Studio 桌面端的内置助手,熟悉代码模板、变量设计与数据库建模。用简洁的中文回答。",
    );
    if let Some(extra) = extra_context.as_deref().filter(|s| !s.trim().is_empty()) {
        context.push_str("\n\n当前编辑上下文:\n");
        context.push_str(&extra.chars().take(6000).collect::<String>());
    }
    if let Some(tp) = &template_path {
        let root = std::path::Path::new(tp);
        if root.is_dir() {
            let samples = collect_template_samples(root);
            let mut vars: Vec<String> = Vec::new();
            let mut file_list = String::new();
            for (i, (rel, content)) in samples.iter().enumerate() {
                if i < 30 {
                    file_list.push_str(&format!("\n- {}", rel));
                }
                for v in extract_placeholders(content) {
                    if !vars.contains(&v) && vars.len() < 60 {
                        vars.push(v);
                    }
                }
            }
            context.push_str(&format!(
                "\n\n用户当前模板目录({} 个文件):{}",
                samples.len(),
                file_list
            ));
            if !vars.is_empty() {
                context.push_str(&format!("\n模板中的变量占位符: {}", vars.join(", ")));
            }
        }
    }
    if let Some(pid) = project_id {
        if let Ok(schema) = project_schema_summary(db, pid).await {
            context.push_str(&format!("\n\n用户当前项目的表结构:{}", schema));
        }
    }

    // 多轮历史:仅保留最近 20 条,避免上下文膨胀
    let history_arr: Vec<serde_json::Value> = history
        .and_then(|h| h.as_array().cloned())
        .unwrap_or_default();
    let recent: Vec<serde_json::Value> = history_arr
        .into_iter()
        .rev()
        .take(20)
        .rev()
        .collect();

    let reply = crate::ai_runtime::chat(&target, Some(&context), &message, &recent).await?;
    serde_json::to_string(&serde_json::json!({ "response": reply, "tool_calls": [] }))
        .map_err(|e| format!("序列化失败: {}", e))
}

/// AI 分析模板变量:提取占位符并由 AI 推断类型/标题/描述;未配置 AI 时纯提取降级
#[tauri::command]
pub async fn ai_analyze_variables(
    template_path: String,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let root = std::path::PathBuf::from(&template_path);
    if !root.is_dir() {
        return Err("模板路径不存在或不是目录".to_string());
    }

    let samples = collect_template_samples(&root);
    let mut vars: Vec<String> = Vec::new();
    let mut snippet = String::new();
    for (rel, content) in &samples {
        for v in extract_placeholders(content) {
            if !vars.contains(&v) {
                vars.push(v);
            }
        }
        if snippet.len() < 12000 {
            snippet.push_str(&format!("\n--- {} ---\n{}\n", rel, content));
        }
    }

    if vars.is_empty() {
        return Ok(serde_json::json!({ "variables": [] }).to_string());
    }

    // 降级路径:无可用 provider 时返回基础信息
    let target = match default_call_target(database.as_ref()).await {
        Ok(t) => t,
        Err(_) => {
            let list: Vec<serde_json::Value> = vars
                .iter()
                .map(|v| {
                    serde_json::json!({ "name": v, "type": "string", "title": v, "description": "", "required": true })
                })
                .collect();
            return Ok(serde_json::json!({ "variables": list }).to_string());
        }
    };

    let mut target = target;
    target.temperature = 0.2;

    let system = "你是代码模板变量分析师。根据模板代码片段与占位符列表,推断每个变量的类型(string/number/boolean/array/object)、中文标题、一句话描述、是否必填、合理的默认值。只返回 JSON,不要解释。";
    let prompt = format!(
        "占位符列表:\n{}\n\n模板代码片段:\n{}\n\n返回格式:{{\"variables\":[{{\"name\":\"...\",\"type\":\"...\",\"title\":\"...\",\"description\":\"...\",\"required\":true,\"default\":\"...\"}}]}}",
        vars.join(", "),
        snippet
    );

    let reply = crate::ai_runtime::chat(&target, Some(system), &prompt, &[]).await?;
    let parsed = parse_json_reply(&reply);
    let list = parsed
        .and_then(|v| v.get("variables").cloned())
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();

    // 校验:只保留模板中真实存在的变量,缺失字段补默认
    let list: Vec<serde_json::Value> = list
        .into_iter()
        .filter(|v| vars.contains(&v["name"].as_str().unwrap_or("").to_string()))
        .map(|v| {
            serde_json::json!({
                "name": v["name"],
                "type": v.get("type").and_then(|t| t.as_str()).unwrap_or("string"),
                "title": v.get("title").and_then(|t| t.as_str()).unwrap_or(v["name"].as_str().unwrap_or("")),
                "description": v.get("description").and_then(|t| t.as_str()).unwrap_or(""),
                "required": v.get("required").and_then(|t| t.as_bool()).unwrap_or(true),
                "default": v.get("default").cloned().unwrap_or(serde_json::json!("")),
            })
        })
        .collect();

    Ok(serde_json::json!({ "variables": list }).to_string())
}

/// AI 填充变量值:依据项目表结构为模板变量生成合理取值
#[tauri::command]
pub async fn ai_fill_variables(
    template_path: String,
    project_id: i64,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let db = database.as_ref();
    let target = default_call_target(db).await?;

    let root = std::path::PathBuf::from(&template_path);
    if !root.is_dir() {
        return Err("模板路径不存在或不是目录".to_string());
    }

    let samples = collect_template_samples(&root);
    let vars: Vec<String> = {
        let mut v: Vec<String> = Vec::new();
        for (_, content) in &samples {
            for p in extract_placeholders(content) {
                if !v.contains(&p) {
                    v.push(p);
                }
            }
        }
        v
    };
    if vars.is_empty() {
        return Ok(serde_json::json!({ "filled": [] }).to_string());
    }

    let schema = project_schema_summary(db, project_id).await?;
    let mut target = target;
    target.temperature = 0.3;

    let system = "你是代码生成变量填充器。根据项目表结构与模板变量列表,为每个变量给出贴合业务语义的具体取值(不要用占位符文本)。confidence 为 0-1 的置信度小数。只返回 JSON。";
    let prompt = format!(
        "变量列表:\n{}\n\n项目表结构:{}\n\n返回格式:{{\"filled\":[{{\"name\":\"...\",\"value\":\"...\",\"confidence\":0.8}}]}}",
        vars.join(", "),
        schema
    );

    let reply = crate::ai_runtime::chat(&target, Some(system), &prompt, &[]).await?;
    let filled = parse_json_reply(&reply)
        .and_then(|v| v.get("filled").cloned())
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();

    let filled: Vec<serde_json::Value> = filled
        .into_iter()
        .filter(|v| vars.contains(&v["name"].as_str().unwrap_or("").to_string()))
        .collect();

    Ok(serde_json::json!({ "filled": filled }).to_string())
}

/// 将变量值写入模板目录下的 variables.json(纯文件写入,不走 AI)
#[tauri::command]
pub async fn ai_write_variables(
    template_path: String,
    variables: String,
) -> Result<(), String> {
    let root = std::path::PathBuf::from(&template_path);
    if !root.is_dir() {
        return Err("模板路径不存在或不是目录".to_string());
    }

    let map: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&variables)
        .map_err(|e| format!("变量 JSON 解析失败: {}", e))?;

    let out = root.join("variables.json");
    let pretty = serde_json::to_string_pretty(&map).map_err(|e| format!("序列化失败: {}", e))?;
    std::fs::write(&out, pretty).map_err(|e| format!("写入失败: {}", e))?;

    Ok(())
}

/// AI 建议变量:根据模板文件内容为指定变量名推断类型/标题/描述/默认值。
/// 编辑器场景:文件在前端内存中(服务端模板),由前端随调用传入。
#[tauri::command]
pub async fn ai_suggest_variables(
    files: serde_json::Value,
    variable_names: Vec<String>,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    if variable_names.is_empty() {
        return Ok(serde_json::json!({ "suggestions": [] }).to_string());
    }

    // 构建命中目标变量的文件片段(总量截断,给 AI 用法上下文)
    let file_list = files.as_array().cloned().unwrap_or_default();
    let mut snippet = String::new();
    for f in &file_list {
        let path = f["path"].as_str().or_else(|| f["filePath"].as_str()).unwrap_or("");
        let content = f["content"].as_str().or_else(|| f["fileContent"].as_str()).unwrap_or("");
        if content.is_empty() {
            continue;
        }
        let hit = variable_names
            .iter()
            .any(|v| content.contains(&format!("{{{{{}}}", v)) || content.contains(v.as_str()));
        if hit {
            snippet.push_str(&format!("\n--- {} ---\n{}\n", path, content));
            if snippet.len() >= 12000 {
                break;
            }
        }
    }

    // 降级路径:无可用 provider 时返回基础建议
    let target = match default_call_target(database.as_ref()).await {
        Ok(t) => t,
        Err(_) => {
            let list: Vec<serde_json::Value> = variable_names
                .iter()
                .map(|v| {
                    serde_json::json!({
                        "name": v, "type": "string", "title": v,
                        "description": "", "default": ""
                    })
                })
                .collect();
            return Ok(serde_json::json!({ "suggestions": list }).to_string());
        }
    };

    let mut target = target;
    target.temperature = 0.2;

    let system = "你是代码模板变量设计师。根据变量在模板代码中的用法,推断每个变量的类型(string/number/boolean/array/object)、中文标题、一句话描述、合理的默认值。只返回 JSON,不要解释。";
    let prompt = format!(
        "变量列表:\n{}\n\n模板代码片段:\n{}\n\n返回格式:{{\"suggestions\":[{{\"name\":\"...\",\"type\":\"...\",\"title\":\"...\",\"description\":\"...\",\"default\":\"...\"}}]}}",
        variable_names.join(", "),
        snippet
    );

    let reply = crate::ai_runtime::chat(&target, Some(system), &prompt, &[]).await?;
    let list = parse_json_reply(&reply)
        .and_then(|v| v.get("suggestions").cloned())
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();

    // 校验:只保留请求的变量;类型归一到已知集合
    let known = ["string", "number", "boolean", "array", "object"];
    let list: Vec<serde_json::Value> = list
        .into_iter()
        .filter(|v| {
            variable_names
                .contains(&v["name"].as_str().unwrap_or("").to_string())
        })
        .map(|v| {
            let t = v.get("type").and_then(|t| t.as_str()).unwrap_or("string");
            let t = if known.contains(&t) { t } else { "string" };
            serde_json::json!({
                "name": v["name"],
                "type": t,
                "title": v.get("title").and_then(|t| t.as_str()).unwrap_or(v["name"].as_str().unwrap_or("")),
                "description": v.get("description").and_then(|t| t.as_str()).unwrap_or(""),
                "default": v.get("default").and_then(|t| t.as_str()).unwrap_or(""),
            })
        })
        .collect();

    Ok(serde_json::json!({ "suggestions": list }).to_string())
}

// ===== 模板提取向导命令 =====

/// 向导用目录遍历:返回白名单内候选文件(相对路径 + 字节大小),不读内容
fn scan_extract_files(root: &std::path::Path) -> Vec<(String, u64)> {
    const EXTS: [&str; 26] = [
        "rs", "ts", "js", "vue", "java", "kt", "go", "py", "cs", "php", "rb", "sql", "yml",
        "yaml", "json", "toml", "xml", "html", "css", "scss", "md", "txt", "sh", "properties",
        "gradle", "mod",
    ];
    const SKIP_DIRS: [&str; 8] = [
        "node_modules", ".git", "target", "dist", "build", ".venv", "__pycache__", ".idea",
    ];
    const MAX_FILE: u64 = 256 * 1024;
    const MAX_FILES: usize = 500;
    const MAX_DEPTH: usize = 8;

    let mut files: Vec<(String, u64)> = Vec::new();
    let mut stack: Vec<(std::path::PathBuf, usize)> = vec![(root.to_path_buf(), 0)];

    while let Some((dir, depth)) = stack.pop() {
        if depth > MAX_DEPTH || files.len() >= MAX_FILES {
            break;
        }
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                if !SKIP_DIRS.contains(&name.as_str()) && !name.starts_with('.') {
                    stack.push((path, depth + 1));
                }
            } else {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if !EXTS.contains(&ext) {
                    continue;
                }
                let Ok(meta) = entry.metadata() else { continue };
                if meta.len() > MAX_FILE {
                    continue;
                }
                let rel = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                files.push((rel, meta.len()));
                if files.len() >= MAX_FILES {
                    break;
                }
            }
        }
    }
    files.sort();
    files
}

/// 提取向导步骤1:扫描目录,返回候选文件清单
#[tauri::command]
pub async fn extract_scan_dir(path: String) -> Result<String, String> {
    let root = std::path::PathBuf::from(&path);
    if !root.is_dir() {
        return Err("路径不存在或不是目录".to_string());
    }

    let files = scan_extract_files(&root);
    let list: Vec<serde_json::Value> = files
        .iter()
        .map(|(rel, size)| serde_json::json!({ "path": rel, "size": size }))
        .collect();

    Ok(serde_json::json!({
        "root": path,
        "files": list,
        "totalSize": files.iter().map(|(_, s)| s).sum::<u64>(),
    })
    .to_string())
}

/// 提取向导步骤2后:批量读取所选文件内容(总量/单文件上限)
#[tauri::command]
pub async fn extract_read_files(
    path: String,
    selected_files: Vec<String>,
) -> Result<String, String> {
    let root = std::path::PathBuf::from(&path);
    if !root.is_dir() {
        return Err("路径不存在或不是目录".to_string());
    }

    const MAX_FILE: usize = 256 * 1024;
    const MAX_TOTAL: usize = 2 * 1024 * 1024;
    const MAX_COUNT: usize = 120;

    let mut files: Vec<serde_json::Value> = Vec::new();
    let mut total = 0usize;
    for rel in selected_files.iter().take(MAX_COUNT) {
        // 防路径穿越:只接受相对路径且不含 ..
        if rel.contains("..") || std::path::Path::new(rel).is_absolute() {
            continue;
        }
        let full = root.join(rel);
        match std::fs::read_to_string(&full) {
            Ok(content) => {
                let content = if content.len() > MAX_FILE {
                    content[..MAX_FILE].to_string()
                } else {
                    content
                };
                total += content.len();
                files.push(serde_json::json!({ "path": rel, "content": content }));
                if total >= MAX_TOTAL {
                    break;
                }
            }
            Err(_) => continue,
        }
    }

    Ok(serde_json::json!({ "files": files }).to_string())
}

/// 启发式候选:引号字符串与目录名,出现次数达标者
fn heuristic_candidates(root_name: &str, files: &[(String, String)]) -> Vec<(String, usize)> {
    use std::collections::HashMap;
    let mut counts: HashMap<String, usize> = HashMap::new();

    let mut bump = |s: &str| {
        let t = s.trim();
        if t.len() < 2 || t.len() > 48 || t.chars().all(|c| c.is_ascii_digit()) {
            return;
        }
        *counts.entry(t.to_string()).or_insert(0) += 1;
    };

    for (_, content) in files {
        let bytes: Vec<char> = content.chars().collect();
        let mut i = 0usize;
        while i < bytes.len() {
            let q = bytes[i];
            if q == '"' || q == '\'' {
                let mut j = i + 1;
                let mut buf = String::new();
                while j < bytes.len() && bytes[j] != q && buf.len() < 64 {
                    let c = bytes[j];
                    if c == '\n' || c == '\r' {
                        break;
                    }
                    buf.push(c);
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == q && !buf.is_empty() {
                    bump(&buf);
                }
                i = j + 1;
            } else {
                i += 1;
            }
        }
    }

    // 目录名始终作为候选(闭包借用已结束,直接写入)
    if root_name.len() >= 2 {
        counts.insert(root_name.to_string(), usize::MAX / 2);
    }

    let mut out: Vec<(String, usize)> = counts
        .into_iter()
        .filter(|(v, c)| v == root_name || *c >= 3)
        .collect();
    out.sort_by(|a, b| b.1.cmp(&a.1));
    out.truncate(30);
    out
}

/// 提取向导步骤3:AI 参数化分析所选文件,产出变量建议表
#[tauri::command]
pub async fn extract_analyze(
    path: String,
    selected_files: Vec<String>,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let root = std::path::PathBuf::from(&path);
    if !root.is_dir() {
        return Err("路径不存在或不是目录".to_string());
    }
    let root_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    // 读取所选文件(与 extract_read_files 同上限,这里更紧:总量 512KB)
    const MAX_FILE: usize = 128 * 1024;
    const MAX_TOTAL: usize = 512 * 1024;
    const MAX_COUNT: usize = 80;
    let mut files: Vec<(String, String)> = Vec::new();
    let mut total = 0usize;
    for rel in selected_files.iter().take(MAX_COUNT) {
        if rel.contains("..") || std::path::Path::new(rel).is_absolute() {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(root.join(rel)) {
            let content = if content.len() > MAX_FILE {
                content[..MAX_FILE].to_string()
            } else {
                content
            };
            total += content.len();
            files.push((rel.clone(), content));
            if total >= MAX_TOTAL {
                break;
            }
        }
    }

    let candidates = heuristic_candidates(&root_name, &files);
    if candidates.is_empty() {
        return Ok(serde_json::json!({ "suggestions": [] }).to_string());
    }

    // 降级路径:无 provider 时直接返回启发式建议
    let target = match default_call_target(database.as_ref()).await {
        Ok(t) => t,
        Err(_) => {
            let list: Vec<serde_json::Value> = candidates
                .iter()
                .map(|(v, c)| {
                    serde_json::json!({
                        "value": v,
                        "varName": sanitize_var_name(v, &root_name),
                        "type": "string",
                        "defaultValue": v,
                        "title": "",
                        "count": c,
                    })
                })
                .collect();
            return Ok(serde_json::json!({ "suggestions": list }).to_string());
        }
    };

    // 内容摘要:优先包含候选出现位置的文件,截 12KB
    let mut digest = String::new();
    for (rel, content) in &files {
        if candidates.iter().any(|(v, _)| content.contains(v.as_str())) {
            digest.push_str(&format!("\n--- {} ---\n{}\n", rel, content));
            if digest.len() >= 12000 {
                break;
            }
        }
    }

    let mut target = target;
    target.temperature = 0.2;

    let cand_list = candidates
        .iter()
        .map(|(v, c)| format!("{}(出现{}次)", v, c))
        .collect::<Vec<_>>()
        .join(", ");

    let system = "你是代码模板参数化专家。给定一个项目文件片段与一组候选字面量,挑选适合做成模板变量的项(项目名/包名/端口/连接串/版本号/环境相关配置等),为每项给出 snake_case 变量名、中文标题、类型(string/number/boolean)、默认值(原值)。排除通用词(如 main/test/http)与不宜参数化的内容。只返回 JSON。";
    let prompt = format!(
        "项目目录名: {}\n候选字面量: {}\n\n文件片段:\n{}\n\n返回格式:{{\"suggestions\":[{{\"value\":\"原字面量\",\"varName\":\"snake_case名\",\"title\":\"中文标题\",\"type\":\"string\",\"defaultValue\":\"原值\"}}]}}",
        root_name, cand_list, digest
    );

    let reply = crate::ai_runtime::chat(&target, Some(system), &prompt, &[]).await?;
    let list = parse_json_reply(&reply)
        .and_then(|v| v.get("suggestions").cloned())
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();

    // 校验:只保留真实存在于候选/文件中的值,补计数
    let known: Vec<String> = candidates.iter().map(|(v, _)| v.clone()).collect();
    let list: Vec<serde_json::Value> = list
        .into_iter()
        .filter_map(|v| {
            let value = v["value"].as_str()?.to_string();
            if !known.contains(&value) {
                return None;
            }
            let count = candidates
                .iter()
                .find(|(c, _)| *c == value)
                .map(|(_, n)| *n)
                .unwrap_or(0);
            let var_name = v
                .get("varName")
                .and_then(|s| s.as_str())
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| sanitize_var_name(&value, &root_name));
            Some(serde_json::json!({
                "value": value,
                "varName": var_name,
                "title": v.get("title").and_then(|t| t.as_str()).unwrap_or(""),
                "type": v.get("type").and_then(|t| t.as_str()).unwrap_or("string"),
                "defaultValue": v.get("defaultValue").and_then(|t| t.as_str()).unwrap_or(&value),
                "count": count,
            }))
        })
        .collect();

    Ok(serde_json::json!({ "suggestions": list }).to_string())
}

/// 值 → 合法 snake_case 变量名(仅 ASCII 字母数字,其它边界转下划线)
fn sanitize_var_name(value: &str, fallback: &str) -> String {
    let mut name = String::new();
    let mut prev_us = false;
    for c in value.chars() {
        if c.is_ascii_alphanumeric() {
            name.push(c.to_ascii_lowercase());
            prev_us = false;
        } else if !prev_us && !name.is_empty() {
            name.push('_');
            prev_us = true;
        }
    }
    let name = name.trim_matches('_').to_string();
    if name.is_empty() || name.chars().next().map_or(true, |c| c.is_ascii_digit()) {
        if !fallback.is_empty() {
            return sanitize_var_name(fallback, "project_name");
        }
        return "project_name".to_string();
    }
    name
}

// ===== agent 编辑轮次(工具协议核心,前端执行工具) =====

/// edit_agent 系统提示词(资产化,git 版本化)
pub const EDIT_AGENT_PROMPT: &str = include_str!("../../prompts/edit_agent.md");

/// 前端会话消息 → rig Message。
/// 线协议(前端 JSON):
///   {role:"system", content}
///   {role:"user", content}
///   {role:"assistant", content?, tool_calls?:[{id,name,arguments}]}
///   {role:"tool_result", tool_call_id, content}
use rig_core::message::Message;

fn agent_messages_to_rig(messages: &serde_json::Value) -> Result<Vec<Message>, String> {
    use rig_core::message::{
        AssistantContent, Text, ToolCall, ToolCallId, ToolFunction, ToolResult,
        ToolResultContent, UserContent,
    };

    let arr = messages
        .as_array()
        .ok_or_else(|| "messages 格式错误:应为数组".to_string())?;
    let mut out = Vec::new();
    for m in arr {
        let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("user");
        match role {
            "system" => {
                let c = m.get("content").and_then(|c| c.as_str()).unwrap_or("");
                out.push(Message::System { content: c.to_string() });
            }
            "assistant" => {
                let mut content: Vec<AssistantContent> = Vec::new();
                if let Some(t) = m.get("content").and_then(|c| c.as_str()).filter(|s| !s.is_empty()) {
                    content.push(AssistantContent::Text(Text { text: t.to_string(), ..Default::default() }));
                }
                if let Some(calls) = m.get("tool_calls").and_then(|c| c.as_array()) {
                    for c in calls {
                        content.push(AssistantContent::ToolCall(ToolCall {
                            id: ToolCallId::new_or_mint(c.get("id").and_then(|v| v.as_str()).unwrap_or("")),
                            provider: None,
                            function: ToolFunction {
                                name: c.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                arguments: c.get("arguments").cloned().unwrap_or(serde_json::json!({})),
                            },
                            signature: None,
                            additional_params: None,
                        }));
                    }
                }
                if !content.is_empty() {
                    out.push(Message::Assistant { id: None, content });
                }
            }
            "tool_result" => {
                out.push(Message::User {
                    content: vec![UserContent::ToolResult(ToolResult {
                        call: ToolCallId::new_or_mint(
                            m.get("tool_call_id").and_then(|v| v.as_str()).unwrap_or(""),
                        ),
                        provider: None,
                        name: m.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        content: vec![ToolResultContent::Text(Text {
                            text: m.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string(),
                            ..Default::default()
                        })],
                    })],
                });
            }
            _ => {
                out.push(Message::User {
                    content: vec![UserContent::Text(Text {
                        text: m.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string(),
                        ..Default::default()
                    })],
                });
            }
        }
    }
    Ok(out)
}

/// agent 单轮:消息 + 工具 schema 进,返回最终文本或待执行工具调用。
/// 工具由前端执行(工作副本在 JS 侧),结果以 role:"tool_result" 消息回传下一轮。
/// P0 仅支持 OpenAI 兼容协议(当前全部预置 provider 均属此类)。
#[tauri::command]
pub async fn ai_agent_turn(
    messages: serde_json::Value,
    tools: serde_json::Value,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    use rig_core::client::CompletionClient;
    use rig_core::completion::{AssistantContent, ToolDefinition};
    use rig_core::completion::CompletionModel as _;

    let target = default_call_target(database.as_ref()).await?;
    let mut target = target;
    target.temperature = 0.2;

    let mut defs = Vec::new();
    for t in tools.as_array().cloned().unwrap_or_default() {
        let name = t.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if name.is_empty() {
            continue;
        }
        defs.push(ToolDefinition {
            name,
            description: t.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            parameters: t.get("parameters").cloned().unwrap_or(serde_json::json!({})),
        });
    }

    if target.protocol != Protocol::OpenAiCompatible {
        return Err("agent 模式暂仅支持 OpenAI 兼容协议的 provider".to_string());
    }

    let client = crate::ai_runtime::openai_client(&target)?;
    let model = client.completion_model(target.model.clone());
    let rig_messages = agent_messages_to_rig(&messages)?;

    // 会话全在 messages,空 prompt 仅占位(prompt 与 messages 一并序列化)
    let mut req = model
        .completion_request(String::new())
        .temperature(target.temperature)
        .max_tokens(target.max_tokens)
        .messages(rig_messages);
    if !defs.is_empty() {
        req = req.tools(defs);
    }

    let resp = model
        .completion(req.build())
        .await
        .map_err(|e| format!("AI 调用失败: {}", e))?;

    let mut calls = Vec::new();
    let mut text = String::new();
    for c in resp.choice {
        match c {
            AssistantContent::Text(t) => text.push_str(&t.text),
            AssistantContent::ToolCall(tc) => {
                calls.push(serde_json::json!({
                    "id": tc.id.to_string(),
                    "name": tc.function.name,
                    "arguments": tc.function.arguments,
                }));
            }
            _ => {}
        }
    }

    let usage = serde_json::json!({
        "input": resp.usage.input_tokens,
        "output": resp.usage.output_tokens,
    });

    if calls.is_empty() {
        Ok(serde_json::json!({ "type": "final", "text": text, "usage": usage }).to_string())
    } else {
        Ok(serde_json::json!({ "type": "tool_calls", "calls": calls, "usage": usage }).to_string())
    }
}

/// 返回 edit_agent 系统提示词(前端组装 agent 会话用)
#[tauri::command]
pub fn ai_get_agent_prompt() -> String {
    EDIT_AGENT_PROMPT.to_string()
}
