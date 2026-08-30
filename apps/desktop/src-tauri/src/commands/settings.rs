use crate::config::Config;

/// 获取配置
#[tauri::command]
pub fn get_config() -> Result<String, String> {
    let config = Config::load().map_err(|e| format!("加载配置失败: {}", e))?;

    serde_json::to_string(&config).map_err(|e| format!("序列化配置失败: {}", e))
}

/// 更新 Web 服务器配置
#[tauri::command]
pub fn update_web_server_config(
    api_url: Option<String>,
    api_key: Option<String>,
) -> Result<String, String> {
    let mut config = Config::load().map_err(|e| format!("加载配置失败: {}", e))?;

    // 更新配置
    if let Some(url) = api_url {
        config.web_server.api_url = url;
    }
    if let Some(key) = api_key {
        // 如果传入空字符串，设置为 None
        config.web_server.api_key = if key.is_empty() { None } else { Some(key) };
    }

    // 保存配置
    config.save().map_err(|e| format!("保存配置失败: {}", e))?;

    Ok("配置已保存".to_string())
}

/// 更新模板存储路径
#[tauri::command]
pub fn update_template_path(template_path: String) -> Result<String, String> {
    use std::path::PathBuf;

    let mut config = Config::load().map_err(|e| format!("加载配置失败: {}", e))?;

    // 验证路径
    let path = PathBuf::from(&template_path);

    // 创建目录（如果不存在）
    std::fs::create_dir_all(&path).map_err(|e| format!("创建模板目录失败: {}", e))?;

    // 更新配置
    config.storage.template_path = path;

    // 保存配置
    config.save().map_err(|e| format!("保存配置失败: {}", e))?;

    Ok("模板存储路径已更新".to_string())
}
