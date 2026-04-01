use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 变量定义（从 variables.json 加载）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDefinition {
    #[serde(rename = "type")]
    pub variable_type: String,
    pub title: String,
    pub description: String,
    pub required: bool,
    pub default: serde_json::Value,
    #[serde(rename = "insertText")]
    pub insert_text: String,
    pub ui: VariableUI,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableUI {
    pub panel: bool,
    pub order: i32,
    pub group: String,
    pub component: String,
}

/// 变量值（可以是不同类型）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VariableValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<serde_json::Value>),
    Object(HashMap<String, serde_json::Value>),
}

impl VariableValue {
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            VariableValue::String(s) => serde_json::Value::String(s.clone()),
            VariableValue::Number(n) => serde_json::Value::Number(serde_json::Number::from_f64(*n).unwrap_or(serde_json::Number::from(0))),
            VariableValue::Boolean(b) => serde_json::Value::Bool(*b),
            VariableValue::Array(arr) => serde_json::Value::Array(arr.clone()),
            VariableValue::Object(obj) => {
                let map = obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                serde_json::Value::Object(map)
            }
        }
    }
}

/// 变量收集器
pub struct VariableCollector {
    template_path: PathBuf,
}

impl VariableCollector {
    pub fn new(template_path: PathBuf) -> Self {
        Self { template_path }
    }

    /// 从 .meta/variables/variables.json 加载变量定义
    pub fn load_variable_definitions(&self) -> Result<HashMap<String, VariableDefinition>> {
        let variables_path = self.template_path
            .join(".meta")
            .join("variables")
            .join("variables.json");

        if !variables_path.exists() {
            tracing::debug!("模板没有变量定义文件: {:?}", variables_path);
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&variables_path)
            .context("读取变量定义文件失败")?;

        let definitions: HashMap<String, VariableDefinition> = serde_json::from_str(&content)
            .context("解析变量定义失败")?;

        tracing::info!("加载了 {} 个变量定义", definitions.len());
        Ok(definitions)
    }

    /// 收集变量值（交互式）
    pub fn collect_variables(&self, definitions: &HashMap<String, VariableDefinition>) -> Result<HashMap<String, serde_json::Value>> {
        use std::io::{self, Write};

        let mut variables = HashMap::new();

        if definitions.is_empty() {
            println!("该模板没有配置变量，将使用内置变量。");
            return Ok(variables);
        }

        println!("\n配置模板变量:");
        println!("{}", "=".repeat(50));

        // 按分组和顺序排序
        let mut sorted_vars: Vec<_> = definitions.iter().collect();
        sorted_vars.sort_by(|a, b| {
            // 先按分组排序
            let group_cmp = a.1.ui.group.cmp(&b.1.ui.group);
            if group_cmp != std::cmp::Ordering::Equal {
                return group_cmp;
            }
            // 同组内按顺序排序
            a.1.ui.order.cmp(&b.1.ui.order)
        });

        let mut current_group = String::new();

        for (name, def) in sorted_vars {
            // 显示分组标题
            if current_group != def.ui.group {
                current_group = def.ui.group.clone();
                println!("\n[{}]", current_group);
            }

            let value = self.collect_single_variable(name, def)?;
            variables.insert(name.clone(), value);
        }

        Ok(variables)
    }

    /// 收集单个变量
    fn collect_single_variable(&self, _name: &str, def: &VariableDefinition) -> Result<serde_json::Value> {
        use std::io::{self, Write};

        // 构建提示信息
        let label = if def.description.is_empty() {
            format!("{}", def.title)
        } else {
            format!("{} ({})", def.title, def.description)
        };

        let required_mark = if def.required { " *" } else { "" };
        print!("  {}{}: ", label, required_mark);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        // 根据类型解析输入
        let value = match def.variable_type.as_str() {
            "boolean" | "conditional" => {
                let default_bool = def.default.as_bool().unwrap_or(false);
                let value_str = if input.is_empty() {
                    if default_bool { "y" } else { "n" }
                } else {
                    input
                };

                let bool_val = match value_str.to_lowercase().as_str() {
                    "y" | "yes" | "true" | "1" => true,
                    "n" | "no" | "false" | "0" => false,
                    _ => {
                        if def.required {
                            anyhow::bail!("请输入 y/n, yes/no, 或 true/false");
                        }
                        default_bool
                    }
                };
                serde_json::Value::Bool(bool_val)
            }
            "number" | "integer" => {
                if input.is_empty() {
                    if def.required {
                        anyhow::bail!("该字段为必填项");
                    }
                    def.default.clone()
                } else {
                    let num_val: f64 = input.parse()
                        .context("请输入有效的数字")?;
                    if def.variable_type == "integer" {
                        serde_json::Value::Number(serde_json::Number::from(num_val as i64))
                    } else {
                        serde_json::Value::Number(serde_json::Number::from_f64(num_val).unwrap_or(serde_json::Number::from(0)))
                    }
                }
            }
            _ => {
                // string 类型
                let string_val = if input.is_empty() {
                    if def.required {
                        anyhow::bail!("该字段为必填项");
                    }
                    def.default.as_str().unwrap_or("").to_string()
                } else {
                    input.to_string()
                };
                serde_json::Value::String(string_val)
            }
        };

        Ok(value)
    }
}
