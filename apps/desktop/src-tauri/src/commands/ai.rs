use crate::state::DbState;

/// 获取所有 AI 提供商
#[tauri::command]
pub async fn ai_get_all_providers(
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
pub async fn ai_get_provider(
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
pub async fn ai_save_provider(
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

    let groups = db.get_ai_provider_models_grouped(&provider_name).await
        .map_err(|e| format!("获取模型列表失败: {}", e))?;

    serde_json::to_string(&groups)
        .map_err(|e| format!("序列化失败: {}", e))
}

/// 添加 AI 模型
#[tauri::command]
pub async fn ai_add_model(
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
pub async fn ai_fetch_models(
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
pub async fn ai_batch_add_models(
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
            let base = if api_endpoint.is_empty() { "http://localhost:11434" } else { &api_endpoint };
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

    let mut request = client.get(&url)
        .timeout(std::time::Duration::from_secs(5));

    if !api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", api_key));
    }

    let response = request.send().await
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
