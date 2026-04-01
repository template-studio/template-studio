use serde::{Deserialize, Serialize};

/// 文件树节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTreeNode {
    pub id: i64,
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "isDirectory")]
    pub is_directory: i32,
    #[serde(rename = "parentId")]
    pub parent_id: i64,
    #[serde(rename = "fileSize")]
    pub file_size: i64,
    pub md5: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileTreeNode>>,
    /// 是否有生成条件
    #[serde(rename = "hasCondition", default)]
    pub has_condition: bool,
    /// 条件摘要描述
    #[serde(rename = "conditionSummary", skip_serializing_if = "Option::is_none")]
    pub condition_summary: Option<String>,
}

/// 文件树响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTreeResponse {
    pub tree: Vec<FileTreeNode>,
}

/// 文件树查询参数
#[derive(Debug, Deserialize)]
pub struct FileTreeQuery {
    #[serde(rename = "templateId")]
    pub template_id: i64,
}
