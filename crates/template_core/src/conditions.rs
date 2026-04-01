//! # 文件生成条件管理模块
//!
//! 提供文件生成条件的定义、验证和序列化功能。
//!
//! ## 核心特性
//!
//! - **YAML配置** - 支持从 conditions.yml 读取/写入
//! - **多条件组合** - 支持 AND/OR/NOT 逻辑
//! - **Switch条件** - 支持多分支条件
//! - **类型安全** - Rust 类型系统确保正确性
//! - **WASM兼容** - 无文件系统依赖，可编译为WASM

use serde::{Deserialize, Serialize};
use std::fmt;

/// 条件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConditionType {
    If,     // 单条件
    And,    // 且条件（所有条件都满足）
    Or,     // 或条件（任一条件满足）
    Not,    // 非条件（条件不满足）
    Switch, // Switch条件（值匹配任一分支）
}

impl fmt::Display for ConditionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConditionType::If => write!(f, "if"),
            ConditionType::And => write!(f, "and"),
            ConditionType::Or => write!(f, "or"),
            ConditionType::Not => write!(f, "not"),
            ConditionType::Switch => write!(f, "switch"),
        }
    }
}

/// 操作符
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Operator {
    Eq,       // 等于
    Ne,       // 不等于
    Gt,       // 大于
    Lt,       // 小于
    Gte,      // 大于等于
    Lte,      // 小于等于
    In,       // 包含于（数组）
    NotIn,    // 不包含于（数组）
    Contains, // 包含（字符串）
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operator::Eq => write!(f, "=="),
            Operator::Ne => write!(f, "!="),
            Operator::Gt => write!(f, ">"),
            Operator::Lt => write!(f, "<"),
            Operator::Gte => write!(f, ">="),
            Operator::Lte => write!(f, "<="),
            Operator::In => write!(f, "in"),
            Operator::NotIn => write!(f, "not in"),
            Operator::Contains => write!(f, "contains"),
        }
    }
}

/// Switch 分支
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchCase {
    pub value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 条件定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    #[serde(rename = "type")]
    pub condition_type: ConditionType,
    pub variable: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<Operator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<Vec<Condition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cases: Option<Vec<SwitchCase>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Condition {
    /// 创建简单的 If 条件
    pub fn new_if(variable: String, operator: Operator, value: serde_json::Value) -> Self {
        Condition {
            condition_type: ConditionType::If,
            variable,
            operator: Some(operator),
            value: Some(value),
            conditions: None,
            cases: None,
            description: None,
        }
    }

    /// 创建 AND 条件
    pub fn new_and(conditions: Vec<Condition>) -> Self {
        Condition {
            condition_type: ConditionType::And,
            variable: String::new(),
            operator: None,
            value: None,
            conditions: Some(conditions),
            cases: None,
            description: None,
        }
    }

    /// 创建 OR 条件
    pub fn new_or(conditions: Vec<Condition>) -> Self {
        Condition {
            condition_type: ConditionType::Or,
            variable: String::new(),
            operator: None,
            value: None,
            conditions: Some(conditions),
            cases: None,
            description: None,
        }
    }

    /// 创建 NOT 条件
    pub fn new_not(condition: Condition) -> Self {
        Condition {
            condition_type: ConditionType::Not,
            variable: String::new(),
            operator: None,
            value: None,
            conditions: Some(vec![condition]),
            cases: None,
            description: None,
        }
    }

    /// 创建 Switch 条件
    pub fn new_switch(variable: String, cases: Vec<SwitchCase>) -> Self {
        Condition {
            condition_type: ConditionType::Switch,
            variable,
            operator: None,
            value: None,
            conditions: None,
            cases: Some(cases),
            description: None,
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// 验证条件是否有效
    pub fn validate(&self) -> Result<(), String> {
        match self.condition_type {
            ConditionType::If => {
                if self.variable.is_empty() {
                    return Err("If条件必须指定变量名".to_string());
                }
                if self.operator.is_none() {
                    return Err("If条件必须指定操作符".to_string());
                }
                if self.value.is_none() {
                    return Err("If条件必须指定值".to_string());
                }
            }
            ConditionType::And | ConditionType::Or => {
                if let Some(ref conds) = self.conditions {
                    if conds.is_empty() {
                        return Err(format!("{}条件必须包含至少一个子条件", self.condition_type));
                    }
                    for cond in conds {
                        cond.validate()?;
                    }
                } else {
                    return Err(format!("{}条件必须包含子条件列表", self.condition_type));
                }
            }
            ConditionType::Not => {
                if let Some(ref conds) = self.conditions {
                    if conds.len() != 1 {
                        return Err("Not条件必须包含且仅包含一个子条件".to_string());
                    }
                    conds[0].validate()?;
                } else {
                    return Err("Not条件必须包含子条件".to_string());
                }
            }
            ConditionType::Switch => {
                if self.variable.is_empty() {
                    return Err("Switch条件必须指定变量名".to_string());
                }
                if let Some(ref cases) = self.cases {
                    if cases.is_empty() {
                        return Err("Switch条件必须包含至少一个分支".to_string());
                    }
                } else {
                    return Err("Switch条件必须包含分支列表".to_string());
                }
            }
        }
        Ok(())
    }

    /// 评估条件是否满足
    pub fn evaluate(&self, variables: &serde_json::Value) -> Result<bool, String> {
        self.validate()?;

        // 获取变量值
        let get_var_value = |path: &str| -> Result<serde_json::Value, String> {
            let parts: Vec<&str> = path.split('.').collect();
            let mut current = variables.clone();

            for part in parts {
                if let Some(obj) = current.as_object() {
                    if let Some(val) = obj.get(part) {
                        current = val.clone();
                    } else {
                        return Ok(serde_json::Value::Null);
                    }
                } else {
                    return Ok(serde_json::Value::Null);
                }
            }

            Ok(current)
        };

        match self.condition_type {
            ConditionType::If => {
                let var_value = get_var_value(&self.variable)?;
                let operator = self.operator.as_ref().unwrap();
                let target_value = self.value.as_ref().unwrap();

                Ok(Self::evaluate_operator(operator, &var_value, target_value)?)
            }
            ConditionType::And => {
                if let Some(ref conds) = self.conditions {
                    for cond in conds {
                        if !cond.evaluate(variables)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Err("And条件缺少子条件".to_string())
                }
            }
            ConditionType::Or => {
                if let Some(ref conds) = self.conditions {
                    for cond in conds {
                        if cond.evaluate(variables)? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                } else {
                    Err("Or条件缺少子条件".to_string())
                }
            }
            ConditionType::Not => {
                if let Some(ref conds) = self.conditions {
                    Ok(!conds[0].evaluate(variables)?)
                } else {
                    Err("Not条件缺少子条件".to_string())
                }
            }
            ConditionType::Switch => {
                let var_value = get_var_value(&self.variable)?;
                if let Some(ref cases) = self.cases {
                    for case in cases {
                        if Self::evaluate_operator(&Operator::Eq, &var_value, &case.value)? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                } else {
                    Err("Switch条件缺少分支".to_string())
                }
            }
        }
    }

    /// 评估操作符（支持智能类型转换）
    fn evaluate_operator(op: &Operator, left: &serde_json::Value, right: &serde_json::Value) -> Result<bool, String> {
        match op {
            Operator::Eq => Ok(Self::compare_with_type_coercion(left, right)),
            Operator::Ne => Ok(!Self::compare_with_type_coercion(left, right)),
            Operator::Gt => Self::compare_numbers(left, right, |l, r| l > r),
            Operator::Lt => Self::compare_numbers(left, right, |l, r| l < r),
            Operator::Gte => Self::compare_numbers(left, right, |l, r| l >= r),
            Operator::Lte => Self::compare_numbers(left, right, |l, r| l <= r),
            Operator::In => {
                if let Some(arr) = right.as_array() {
                    Ok(arr.contains(left))
                } else {
                    Err("In操作符右侧必须是数组".to_string())
                }
            }
            Operator::NotIn => {
                if let Some(arr) = right.as_array() {
                    Ok(!arr.contains(left))
                } else {
                    Err("NotIn操作符右侧必须是数组".to_string())
                }
            }
            Operator::Contains => {
                if let (Some(left_str), Some(right_str)) = (left.as_str(), right.as_str()) {
                    Ok(left_str.contains(right_str))
                } else {
                    // 尝试转换为字符串后进行比较
                    Ok(left.to_string().contains(&right.to_string()))
                }
            }
        }
    }

    /// 智能类型比较（支持类型转换）
    ///
    /// 规则：
    /// 1. 如果类型相同，直接比较
    /// 2. 如果一个是字符串，尝试转换后比较
    /// 3. 布尔值：字符串 "true"/"false" 转换为布尔值比较
    /// 4. 数字：字符串数字转换为数字比较
    fn compare_with_type_coercion(left: &serde_json::Value, right: &serde_json::Value) -> bool {
        // 情况1：类型相同，直接比较
        if left == right {
            return true;
        }

        // 情况2：一个是字符串，另一个是布尔值
        if let (Some(left_str), Some(right_bool)) = (left.as_str(), right.as_bool()) {
            if let Ok(parsed_bool) = left_str.parse::<bool>() {
                return parsed_bool == right_bool;
            }
        }
        if let (Some(left_bool), Some(right_str)) = (left.as_bool(), right.as_str()) {
            if let Ok(parsed_bool) = right_str.parse::<bool>() {
                return left_bool == parsed_bool;
            }
        }

        // 情况3：一个是字符串，另一个是数字
        if let (Some(left_str), Some(right_num)) = (left.as_str(), right.as_i64()) {
            if let Ok(parsed_num) = left_str.parse::<i64>() {
                return parsed_num == right_num;
            }
        }
        if let (Some(left_num), Some(right_str)) = (left.as_i64(), right.as_str()) {
            if let Ok(parsed_num) = right_str.parse::<i64>() {
                return left_num == parsed_num;
            }
        }

        // 情况4：浮点数比较
        if let (Some(left_str), Some(right_num)) = (left.as_str(), right.as_f64()) {
            if let Ok(parsed_num) = left_str.parse::<f64>() {
                return (parsed_num - right_num).abs() < f64::EPSILON;
            }
        }
        if let (Some(left_num), Some(right_str)) = (left.as_f64(), right.as_str()) {
            if let Ok(parsed_num) = right_str.parse::<f64>() {
                return (left_num - parsed_num).abs() < f64::EPSILON;
            }
        }

        // 默认：不匹配
        false
    }

    /// 比较数字
    fn compare_numbers<F>(
        left: &serde_json::Value,
        right: &serde_json::Value,
        compare: F,
    ) -> Result<bool, String>
    where
        F: FnOnce(f64, f64) -> bool,
    {
        let left_num = left.as_f64().ok_or("左侧值不是数字".to_string())?;
        let right_num = right.as_f64().ok_or("右侧值不是数字".to_string())?;
        Ok(compare(left_num, right_num))
    }
}

/// 文件条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCondition {
    pub id: i64,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<Condition>,
}

/// Conditions YAML 文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionsYaml {
    pub version: String,
    pub conditions: Vec<FileCondition>,
}

impl ConditionsYaml {
    /// 创建空的 conditions YAML
    pub fn new() -> Self {
        ConditionsYaml {
            version: "1.0".to_string(),
            conditions: Vec::new(),
        }
    }

    /// 添加文件条件（如果路径存在则更新）
    pub fn add_condition(&mut self, condition: FileCondition) {
        // 查找是否已存在相同路径的条件
        if let Some(existing) = self.conditions.iter_mut().find(|c| c.path == condition.path) {
            // 更新现有条件
            existing.condition = condition.condition;
        } else {
            // 添加新条件
            self.conditions.push(condition);
        }
    }

    /// 根据 ID 查找条件
    pub fn find_by_id(&self, id: i64) -> Option<&FileCondition> {
        self.conditions.iter().find(|c| c.id == id)
    }

    /// 根据 ID 查找可变条件
    pub fn find_by_id_mut(&mut self, id: i64) -> Option<&mut FileCondition> {
        self.conditions.iter_mut().find(|c| c.id == id)
    }

    /// 根据 ID 删除条件
    pub fn remove_by_id(&mut self, id: i64) -> bool {
        if let Some(pos) = self.conditions.iter().position(|c| c.id == id) {
            self.conditions.remove(pos);
            true
        } else {
            false
        }
    }

    /// 更新文件路径
    pub fn update_path(&mut self, id: i64, new_path: String) -> bool {
        if let Some(cond) = self.find_by_id_mut(id) {
            cond.path = new_path;
            true
        } else {
            false
        }
    }

    /// 根据文件路径获取条件
    pub fn get_condition_by_path(&self, file_path: &str) -> Option<Condition> {
        self.conditions.iter()
            .find(|c| c.path == file_path)
            .and_then(|fc| fc.condition.clone())
    }

    /// 根据文件路径删除条件
    pub fn remove_condition_by_path(&mut self, file_path: &str) -> bool {
        if let Some(pos) = self.conditions.iter().position(|c| c.path == file_path) {
            self.conditions.remove(pos);
            true
        } else {
            false
        }
    }

    /// 更新文件路径（当文件移动或重命名时）
    pub fn update_file_path(&mut self, old_path: &str, new_path: &str) -> bool {
        let mut updated = false;
        for cond in &mut self.conditions {
            if cond.path == old_path {
                cond.path = new_path.to_string();
                updated = true;
            }
        }
        updated
    }

    /// 序列化为 YAML 字符串
    pub fn to_yaml(&self) -> Result<String, String> {
        serde_yaml::to_string(self)
            .map_err(|e| format!("YAML序列化失败: {}", e))
    }

    /// 从 YAML 字符串反序列化
    pub fn from_yaml(content: &str) -> Result<Self, String> {
        serde_yaml::from_str(content)
            .map_err(|e| format!("YAML解析失败: {}", e))
    }

    /// 验证所有条件
    pub fn validate(&self) -> Result<(), String> {
        for cond in &self.conditions {
            if let Some(ref condition) = cond.condition {
                condition.validate()?;
            }
        }
        Ok(())
    }

    /// 过滤出满足条件的文件ID列表
    pub fn filter_enabled_files(&self, variables: &serde_json::Value) -> Result<Vec<i64>, String> {
        let mut enabled = Vec::new();

        for cond in &self.conditions {
            match &cond.condition {
                Some(condition) => {
                    if condition.evaluate(variables)? {
                        enabled.push(cond.id);
                    }
                }
                None => {
                    // 没有条件，默认生成
                    enabled.push(cond.id);
                }
            }
        }

        Ok(enabled)
    }
}

impl Default for ConditionsYaml {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_if_condition() {
        let condition = Condition::new_if(
            "enableFeature".to_string(),
            Operator::Eq,
            json!(true),
        );

        let variables = json!({
            "enableFeature": true
        });

        assert!(condition.evaluate(&variables).unwrap());
    }

    #[test]
    fn test_and_condition() {
        let condition = Condition::new_and(vec![
            Condition::new_if("env".to_string(), Operator::Eq, json!("production")),
            Condition::new_if("debug".to_string(), Operator::Eq, json!(false)),
        ]);

        let variables = json!({
            "env": "production",
            "debug": false
        });

        assert!(condition.evaluate(&variables).unwrap());
    }

    #[test]
    fn test_or_condition() {
        let condition = Condition::new_or(vec![
            Condition::new_if("database".to_string(), Operator::Eq, json!("mysql")),
            Condition::new_if("database".to_string(), Operator::Eq, json!("postgresql")),
        ]);

        let variables1 = json!({ "database": "mysql" });
        let variables2 = json!({ "database": "postgresql" });
        let variables3 = json!({ "database": "sqlite" });

        assert!(condition.evaluate(&variables1).unwrap());
        assert!(condition.evaluate(&variables2).unwrap());
        assert!(!condition.evaluate(&variables3).unwrap());
    }

    #[test]
    fn test_switch_condition() {
        let cases = vec![
            SwitchCase {
                value: json!("mysql"),
                description: Some("MySQL数据库".to_string()),
            },
            SwitchCase {
                value: json!("mariadb"),
                description: Some("MariaDB数据库".to_string()),
            },
        ];

        let condition = Condition::new_switch("database".to_string(), cases);

        let variables1 = json!({ "database": "mysql" });
        let variables2 = json!({ "database": "sqlite" });

        assert!(condition.evaluate(&variables1).unwrap());
        assert!(!condition.evaluate(&variables2).unwrap());
    }

    #[test]
    fn test_conditions_yaml_serialization() {
        let yaml_content = r#"
version: "1.0"
conditions:
  - id: 1
    path: "src/main.go"
    condition:
      type: if
      variable: "enableMain"
      operator: eq
      value: true
      description: "启用主文件时生成"
"#;

        let conditions = ConditionsYaml::from_yaml(yaml_content).unwrap();
        assert_eq!(conditions.conditions.len(), 1);
        assert_eq!(conditions.conditions[0].id, 1);
        assert_eq!(conditions.conditions[0].path, "src/main.go");
    }

    #[test]
    fn test_nested_path_variable() {
        let condition = Condition::new_if(
            "config.database.enabled".to_string(),
            Operator::Eq,
            json!(true),
        );

        let variables = json!({
            "config": {
                "database": {
                    "enabled": true
                }
            }
        });

        assert!(condition.evaluate(&variables).unwrap());
    }

    #[test]
    fn test_operators() {
        // Test Eq
        assert!(Condition::new_if("num".to_string(), Operator::Eq, json!(5))
            .evaluate(&json!({"num": 5})).unwrap());

        // Test Gt
        assert!(Condition::new_if("num".to_string(), Operator::Gt, json!(3))
            .evaluate(&json!({"num": 5})).unwrap());

        // Test In
        assert!(Condition::new_if("db".to_string(), Operator::In, json!(["mysql", "postgresql"]))
            .evaluate(&json!({"db": "mysql"})).unwrap());

        // Test Contains
        assert!(Condition::new_if("text".to_string(), Operator::Contains, json!("hello"))
            .evaluate(&json!({"text": "hello world"})).unwrap());
    }

    // ========== 类型转换测试 ==========

    #[test]
    fn test_type_coercion_bool_string() {
        // 字符串 "true" 与布尔值 true 应该相等
        assert!(Condition::new_if("flag".to_string(), Operator::Eq, json!("true"))
            .evaluate(&json!({"flag": true})).unwrap());

        // 字符串 "false" 与布尔值 false 应该相等
        assert!(Condition::new_if("flag".to_string(), Operator::Eq, json!("false"))
            .evaluate(&json!({"flag": false})).unwrap());

        // 字符串 "false" 与布尔值 true 应该不相等
        assert!(!Condition::new_if("flag".to_string(), Operator::Eq, json!("false"))
            .evaluate(&json!({"flag": true})).unwrap());
    }

    #[test]
    fn test_type_coercion_number_string() {
        // 字符串 "123" 与数字 123 应该相等
        assert!(Condition::new_if("count".to_string(), Operator::Eq, json!("123"))
            .evaluate(&json!({"count": 123})).unwrap());

        // 字符串 "456" 与数字 456 应该相等
        assert!(Condition::new_if("port".to_string(), Operator::Eq, json!("8080"))
            .evaluate(&json!({"port": 8080})).unwrap());
    }

    #[test]
    fn test_type_coercion_mixed() {
        // 变量是布尔值，条件是字符串
        assert!(Condition::new_if("enable".to_string(), Operator::Eq, json!("true"))
            .evaluate(&json!({"enable": true})).unwrap());

        // 变量是字符串，条件是布尔值
        assert!(Condition::new_if("debug".to_string(), Operator::Eq, json!(true))
            .evaluate(&json!({"debug": "true"})).unwrap());

        // 变量是数字，条件是字符串
        assert!(Condition::new_if("age".to_string(), Operator::Eq, json!("18"))
            .evaluate(&json!({"age": 18})).unwrap());
    }

    #[test]
    fn test_type_coercion_not_operator() {
        // Ne 操作符也应该使用类型转换
        assert!(!Condition::new_if("flag".to_string(), Operator::Ne, json!("true"))
            .evaluate(&json!({"flag": true})).unwrap());

        assert!(Condition::new_if("flag".to_string(), Operator::Ne, json!("false"))
            .evaluate(&json!({"flag": true})).unwrap());
    }

    #[test]
    fn test_type_coercion_real_world_scenario() {
        // 真实场景：用户在前端输入 "true"，变量是布尔值
        let condition = Condition::new_if("flag1".to_string(), Operator::Eq, json!("true"));

        // 变量是布尔值 false
        let result = condition.evaluate(&json!({"flag1": false})).unwrap();
        assert!(!result, "flag1=false 应该不等于条件值 'true'");

        // 变量是布尔值 true
        let result = condition.evaluate(&json!({"flag1": true})).unwrap();
        assert!(result, "flag1=true 应该等于条件值 'true'");
    }
}
