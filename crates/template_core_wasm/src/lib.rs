//! # Template Studio Template Core WASM Bindings
//!
//! 提供浏览器端的模板渲染功能，支持：
//! - 模板字符串渲染
//! - 文件树批量渲染
//! - 离线模板处理
//!
//! ## 使用示例（JavaScript）
//!
//! ```javascript
//! import init, { render_string, render_tree, get_engine_info } from './pkg/template_studio_template_core_wasm.js';
//!
//! // 初始化 WASM 模块
//! await init();
//!
//! // 渲染单个模板
//! const result = render_string("Hello {{ name }}!", { name: "World" });
//! console.log(result.content); // "Hello World!"
//!
//! // 获取引擎信息
//! const info = get_engine_info();
//! console.log(info.version, info.filters);
//! ```

use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

// 导入核心模板引擎
use template_studio_template_core::{
    filter_files_by_conditions, render_string as core_render_string,
    render_tree as core_render_tree, Condition as CoreCondition, RenderError as CoreRenderError,
    RenderResult as CoreRenderResult, TemplateFile as CoreTemplateFile, Variables,
};

// ============================================================================
// 初始化
// ============================================================================

/// 初始化 WASM 模块
#[wasm_bindgen(start)]
pub fn wasm_init() {
    // 设置 panic hook，在 WASM 中提供更好的错误信息
    #[cfg(feature = "console_log")]
    console_log::init_with_level(log::Level::Info).ok();

    #[cfg(all(debug_assertions, feature = "console_log"))]
    log::info!("[WASM] Template Core initialized");

    // 初始化核心引擎（注册过滤器等）
    template_studio_template_core::initialize();
}

// ============================================================================
// 核心渲染函数
// ============================================================================

/// 渲染模板字符串
///
/// # 参数
/// * `template` - 模板内容（支持 MiniJinja 语法）
/// * `variables` - 变量对象（JavaScript 对象）
///
/// # 返回
/// 渲染结果对象（JSON 格式）
#[wasm_bindgen]
pub fn render_string(template: String, variables: JsValue) -> Result<JsValue, JsValue> {
    // 解析变量
    let vars = parse_variables(&variables)?;

    // 调用核心渲染
    let result = core_render_string(&template, &vars, None)
        .map_err(|e| JsValue::from_str(&format!("Render error: {}", e)))?;

    // 转换为 JS 对象
    let js_result = WasmRenderResult::from_core(result);
    to_value(&js_result).map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// 批量渲染文件树
///
/// # 参数
/// * `files` - 文件列表（JavaScript 数组）
/// * `variables` - 变量对象（JavaScript 对象）
///
/// # 返回
/// 渲染后的文件列表（JavaScript 数组）
#[wasm_bindgen]
pub fn render_tree(files: JsValue, variables: JsValue) -> Result<JsValue, JsValue> {
    // 解析变量
    let vars = parse_variables(&variables)?;

    // 解析文件列表
    let wasm_files: Vec<WasmTemplateFile> = from_value(files)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse files: {}", e)))?;

    // 转换为核心类型
    let core_files: Vec<CoreTemplateFile> = wasm_files
        .into_iter()
        .map(|f| CoreTemplateFile {
            id: f.id,
            file_path: f.file_path,
            file_name: f.file_name,
            file_content: f.file_content,
            is_directory: f.is_directory,
            parent_id: f.parent_id,
            filesize: f.filesize,
            extends: None,
            includes: None,
            imports: None,
            condition: f.condition,
            is_dependency: false,
            required_by: None,
        })
        .collect();

    // 与服务端渲染保持一致：先按文件条件过滤（条件不满足的文件及其子树不参与渲染）
    let filtered_files = filter_files_by_conditions(core_files, &vars);

    // 调用核心渲染
    let results = core_render_tree(filtered_files, &vars)
        .map_err(|e| JsValue::from_str(&format!("Render tree error: {}", e)))?;

    // 转换为 WASM 类型
    let wasm_results: Vec<WasmRenderedFile> = results
        .into_iter()
        .map(|r| WasmRenderedFile::from_core(r))
        .collect();

    // 返回 JavaScript 值
    to_value(&wasm_results)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize results: {}", e)))
}

/// 获取 WASM 引擎信息
#[wasm_bindgen]
pub fn get_engine_info() -> Result<JsValue, JsValue> {
    // 获取过滤器信息
    let filter_infos = template_studio_template_core::get_available_filters();
    let filters: Vec<FilterInfoJs> = filter_infos
        .iter()
        .map(|f| FilterInfoJs {
            name: f.name.clone(),
            description: f.description.clone(),
            example: f.example.clone(),
        })
        .collect();

    // 获取内置函数信息
    let function_categories = template_studio_template_core::get_builtin_function_categories();
    let functions: Vec<String> = function_categories
        .iter()
        .flat_map(|cat| cat.functions.iter().map(|f| f.name.clone()))
        .collect();

    let info = WasmEngineInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_time: build_time(),
        filters,
        functions,
    };

    to_value(&info).map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
}

/// 获取可用的过滤器列表（带详细信息）
#[wasm_bindgen]
pub fn get_filters() -> Result<JsValue, JsValue> {
    let filter_infos = template_studio_template_core::get_available_filters();

    let filters: Vec<FilterInfoJs> = filter_infos
        .iter()
        .map(|f| FilterInfoJs {
            name: f.name.clone(),
            description: f.description.clone(),
            example: f.example.clone(),
        })
        .collect();

    to_value(&filters).map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
}

/// 检查模板语法是否有效
#[wasm_bindgen]
pub fn validate_template(template: String) -> Result<JsValue, JsValue> {
    let variables = Variables::from_value(serde_json::Value::Object(serde_json::Map::new()));

    match core_render_string(&template, &variables, None) {
        Ok(result) => {
            if result.success {
                to_value(&serde_json::json!({ "valid": true }))
                    .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
            } else {
                to_value(&serde_json::json!({
                    "valid": false,
                    "error": WasmRenderError::from_core(result.error.unwrap())
                }))
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e)))
            }
        }
        Err(e) => to_value(&serde_json::json!({
            "valid": false,
            "error": {
                "type": "render_error",
                "message": e.to_string()
            }
        }))
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize: {}", e))),
    }
}

/// 清除模板缓存
#[wasm_bindgen]
pub fn clear_cache() {
    template_studio_template_core::clear_template_cache();
}

/// 获取模板缓存大小
#[wasm_bindgen]
pub fn get_cache_size() -> usize {
    template_studio_template_core::get_cache_size()
}

// ============================================================================
// 类型定义（用于序列化）
// ============================================================================

/// WASM 渲染错误
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WasmRenderError {
    /// 错误类型
    #[serde(rename = "type")]
    pub error_type: String,
    /// 错误消息
    pub message: String,
    /// 错误所在行号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// 错误所在列号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
    /// 错误上下文
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// 修复建议
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

impl WasmRenderError {
    fn from_core(err: CoreRenderError) -> Self {
        Self {
            error_type: err.error_type,
            message: err.message,
            line: err.line.map(|l| l as u32),
            column: err.column.map(|c| c as u32),
            context: err.context,
            suggestion: err.suggestion,
        }
    }
}

/// WASM 渲染结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WasmRenderResult {
    /// 渲染后的内容
    pub content: String,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果失败）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<WasmRenderError>,
}

impl WasmRenderResult {
    fn from_core(result: CoreRenderResult) -> Self {
        Self {
            content: result.content,
            success: result.success,
            error: result.error.map(WasmRenderError::from_core),
        }
    }
}

/// WASM 渲染后的文件
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WasmRenderedFile {
    /// 节点 ID
    pub id: i64,
    /// 渲染后的文件路径
    pub file_path: String,
    /// 渲染后的文件名
    pub file_name: String,
    /// 渲染后的内容（目录为 null）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_content: Option<String>,
    /// 是否为目录
    pub is_directory: i32,
    /// 文件大小
    pub filesize: i32,
    /// 父节点 ID
    pub parent_id: i64,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<WasmRenderError>,
}

impl WasmRenderedFile {
    fn from_core(r: template_studio_template_core::RenderedFile) -> Self {
        Self {
            id: r.id,
            file_path: r.file_path,
            file_name: r.file_name,
            file_content: r.file_content,
            is_directory: r.is_directory,
            filesize: r.filesize,
            parent_id: r.parent_id,
            error: r.error.map(WasmRenderError::from_core),
        }
    }
}

/// WASM 模板文件（输入）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WasmTemplateFile {
    /// 节点 ID
    pub id: i64,
    /// 相对路径
    pub file_path: String,
    /// 文件名
    pub file_name: String,
    /// 文件内容
    pub file_content: String,
    /// 是否为目录
    pub is_directory: i32,
    /// 父节点 ID
    pub parent_id: i64,
    /// 文件大小
    pub filesize: i32,
    /// 文件生成条件（可选；条件不满足时该文件/目录及其子树不参与渲染）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<CoreCondition>,
}

/// 引擎信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WasmEngineInfo {
    /// 版本号
    pub version: String,
    /// 构建时间
    pub build_time: String,
    /// 支持的过滤器列表
    pub filters: Vec<FilterInfoJs>,
    /// 支持的内置函数列表
    pub functions: Vec<String>,
}

/// 过滤器信息（JavaScript 友好）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FilterInfoJs {
    pub name: String,
    pub description: String,
    pub example: String,
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 解析 JavaScript 变量对象
fn parse_variables(variables: &JsValue) -> Result<Variables, JsValue> {
    if variables.is_null() || variables.is_undefined() {
        return Ok(Variables::from_value(serde_json::Value::Object(
            serde_json::Map::new(),
        )));
    }

    // 尝试直接解析为 JSON
    let json_value: serde_json::Value = from_value(variables.clone())
        .map_err(|e| JsValue::from_str(&format!("Failed to parse variables: {}", e)))?;

    Ok(Variables::from_value(json_value))
}

/// 获取构建时间
fn build_time() -> String {
    // 在 release 构建时，通过环境变量注入构建时间
    option_env!("BUILD_TIME").unwrap_or("unknown").to_string()
}

// ============================================================================
// Console 日志辅助
// ============================================================================

/// 在控制台打印日志（调试用）
#[cfg(debug_assertions)]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(debug_assertions)]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}
