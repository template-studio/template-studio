use tauri::Manager;

/// 问候
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// 写入文本文件
#[tauri::command]
pub fn write_text_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, &content).map_err(|e| format!("写入文件失败: {}", e))
}

/// 最小化窗口
#[tauri::command]
pub fn window_minimize(app: tauri::AppHandle) {
    let window = app.get_webview_window("main").unwrap();
    window.minimize().unwrap();
}

/// 最大化/还原窗口
#[tauri::command]
pub fn window_maximize(app: tauri::AppHandle) {
    let window = app.get_webview_window("main").unwrap();
    if window.is_maximized().unwrap() {
        window.unmaximize().unwrap();
    } else {
        window.maximize().unwrap();
    }
}

/// 切换 DevTools（开发者控制台）
#[tauri::command]
pub fn toggle_devtools(app: tauri::AppHandle) -> Result<bool, String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    if window.is_devtools_open() {
        window.close_devtools();
        Ok(false)
    } else {
        window.open_devtools();
        Ok(true)
    }
}

/// 关闭窗口
#[tauri::command]
pub fn window_close(app: tauri::AppHandle) {
    let window = app.get_webview_window("main").unwrap();
    window.close().unwrap();
}

/// 获取系统用户名
#[tauri::command]
pub fn get_username() -> Result<String, String> {
    use std::env;

    // 尝试获取 USER 环境变量
    if let Ok(username) = env::var("USER") {
        Ok(username)
    } else if let Ok(username) = env::var("USERNAME") {
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
pub fn get_system_theme() -> Result<String, String> {
    // 在 Windows 上检测系统主题需要访问注册表，这里返回默认值
    // 用户可以在设置中手动切换主题
    Ok("light".to_string())
}
