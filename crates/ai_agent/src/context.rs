use serde::{Deserialize, Serialize};

/// 项目上下文 - 用于 AI 变量填充和推荐
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub project_id: i64,
    pub project_name: String,
    pub tables: Vec<TableInfo>,
    pub type_mappings: Vec<TypeMapping>,
    pub naming_convention: Option<NamingConvention>,
}

/// 表信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub comment: Option<String>,
    pub columns: Vec<ColumnInfo>,
}

/// 列信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub length: Option<i32>,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub comment: Option<String>,
}

/// 类型映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMapping {
    pub db_type: String,
    pub language_type: String,
    pub language: String,
}

/// 命名规范
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingConvention {
    pub table_prefix: Option<String>,
    pub table_suffix: Option<String>,
    pub column_prefix: Option<String>,
    pub column_suffix: Option<String>,
}

impl ProjectContext {
    /// 构建上下文摘要（用于 prompt）
    pub fn to_summary(&self) -> String {
        let mut summary = String::new();
        summary.push_str(&format!("项目: {}\n", self.project_name));
        summary.push_str(&format!("表数量: {}\n", self.tables.len()));

        for table in &self.tables {
            summary.push_str(&format!(
                "\n表 {} ({})\n",
                table.name,
                table.comment.as_deref().unwrap_or("")
            ));
            for col in &table.columns {
                summary.push_str(&format!(
                    "  - {} {} {} {}\n",
                    col.name,
                    col.data_type,
                    if col.is_primary_key { "PK" } else { "" },
                    col.comment.as_deref().unwrap_or("")
                ));
            }
        }

        summary
    }
}
