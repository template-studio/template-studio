use serde::Serialize;
use anyhow::Result;

/// 输出格式
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Json,
    Table,
    Compact,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => OutputFormat::Json,
            "compact" => OutputFormat::Compact,
            _ => OutputFormat::Table,
        }
    }
}

/// 输出格式化器
pub struct OutputFormatter {
    format: OutputFormat,
}

impl OutputFormatter {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// 输出数据
    pub fn print<T: Serialize + PrintTable>(&self, data: &T) -> Result<()> {
        match self.format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(data)?);
            }
            OutputFormat::Table => {
                data.print_table();
            }
            OutputFormat::Compact => {
                data.print_compact();
            }
        }
        Ok(())
    }

    /// 输出简单消息
    pub fn print_message(&self, message: &str) {
        match self.format {
            OutputFormat::Json => {
                println!("{}", serde_json::json!({ "message": message }));
            }
            _ => {
                println!("{}", message);
            }
        }
    }

    /// 输出错误
    pub fn print_error(&self, error: &str) {
        match self.format {
            OutputFormat::Json => {
                eprintln!("{}", serde_json::json!({ "error": error }));
            }
            _ => {
                eprintln!("错误: {}", error);
            }
        }
    }
}

/// 表格输出 trait
pub trait PrintTable {
    fn print_table(&self);
    fn print_compact(&self);
}

/// 变量分析结果的表格输出
impl PrintTable for template_studio_ai_agent::VariableAnalysisResult {
    fn print_table(&self) {
        println!("模板变量分析结果 ({} 个变量):\n", self.total);
        println!("{:<20} {:<10} {:<8} {:<8} {}", "变量名", "类型", "必填", "来源", "描述");
        println!("{}", "-".repeat(80));
        for var in &self.variables {
            println!(
                "{:<20} {:<10} {:<8} {:<8} {}",
                var.name,
                var.var_type,
                if var.required { "是" } else { "否" },
                var.source.as_deref().unwrap_or("-"),
                var.description
            );
        }
        if self.auto_inferred > 0 {
            println!("\n其中 {} 个变量由 AI 推断", self.auto_inferred);
        }
    }

    fn print_compact(&self) {
        println!("{} 变量, {} AI推断", self.total, self.auto_inferred);
    }
}

/// 变量填充结果的表格输出
impl PrintTable for template_studio_ai_agent::VariableFillResult {
    fn print_table(&self) {
        println!("变量填充结果:\n");
        println!("模板: {}", self.template);
        println!("项目: {}", self.project);
        println!("置信度: {:.0}%\n", self.confidence * 100.0);
        println!("填充的变量:");
        if let Some(obj) = self.filled.as_object() {
            for (k, v) in obj {
                println!("  {} = {}", k, v);
            }
        }
        if !self.ai_reasoning.is_empty() {
            println!("\n推理过程: {}", self.ai_reasoning);
        }
    }

    fn print_compact(&self) {
        println!("置信度: {:.0}%", self.confidence * 100.0);
    }
}

/// JSON 值的表格输出
impl PrintTable for serde_json::Value {
    fn print_table(&self) {
        if let Some(arr) = self.as_array() {
            for item in arr {
                if let Some(obj) = item.as_object() {
                    for (k, v) in obj {
                        println!("{}: {}", k, v);
                    }
                    println!();
                }
            }
        } else if let Some(obj) = self.as_object() {
            for (k, v) in obj {
                println!("{}: {}", k, v);
            }
        } else {
            println!("{}", self);
        }
    }

    fn print_compact(&self) {
        if let Some(arr) = self.as_array() {
            println!("{} 项", arr.len());
        } else {
            println!("{}", self);
        }
    }
}
