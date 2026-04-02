//! 渲染相关数据模型

use serde::{Deserialize, Serialize};

/// 渲染错误详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderError {
    #[serde(rename = "type")]
    pub error_type: String, // "parse_error", "execute_error", "variable_error"
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub context: Option<String>,
    pub suggestion: Option<String>,
}

/// 渲染后的文件节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedFileInfo {
    pub id: i64,

    /// 渲染后的文件路径（如 "src/main.go"）
    #[serde(rename = "filePath")]
    pub file_path: String,

    /// 渲染后的文件名（如 "main.go"）
    #[serde(rename = "fileName")]
    pub file_name: String,

    /// 渲染后的文件内容（仅文件节点有值）
    #[serde(rename = "fileContent", skip_serializing_if = "Option::is_none")]
    pub file_content: Option<String>,

    /// 是否是目录
    #[serde(rename = "isDirectory")]
    pub is_directory: i32,

    /// 文件大小（字节）
    pub filesize: i32,

    /// 父节点ID
    #[serde(rename = "parentId")]
    pub parent_id: i64,

    /// 子节点列表（仅目录节点有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<RenderedFileInfo>>,

    /// 渲染错误信息（仅当渲染失败时有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render_error: Option<RenderError>,
}

/// 渲染文件树响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderFileTreeResponse {
    #[serde(rename = "templateId")]
    pub template_id: i64,

    /// 渲染后的文件树
    pub tree: Vec<RenderedFileInfo>,

    /// 使用的变量
    pub variables: serde_json::Value,

    /// 统计信息
    #[serde(rename = "totalFiles")]
    pub total_files: i32, // 总文件数

    #[serde(rename = "totalSize")]
    pub total_size: i64, // 总大小（字节）

    /// 渲染失败的文件数
    #[serde(rename = "failedFiles")]
    pub failed_files: i32,
}
