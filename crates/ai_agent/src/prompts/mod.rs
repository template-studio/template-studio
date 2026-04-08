pub mod variable;

/// 系统 prompt
pub const SYSTEM_PROMPT: &str = r#"你是一个模板分析助手。你的任务是帮助用户分析和操作 Template Studio 中的模板。

你的能力：
1. 分析模板变量 - 从模板文件中提取变量并推断类型
2. 自动填充变量 - 根据项目上下文推断变量值
3. 验证模板 - 检查语法和变量完整性
4. 推荐模板 - 根据项目特征推荐合适的模板

请用中文回答，输出结构化的 JSON 格式。"#;

/// 变量分析 prompt
pub const VARIABLE_ANALYSIS_PROMPT: &str = r#"分析以下模板文件，提取所有变量。

模板文件内容:
```
{template_content}
```

请输出 JSON 格式:
```json
{{
  "variables": [
    {{
      "name": "变量名",
      "type": "string|number|boolean|array|object",
      "title": "中文标题",
      "description": "变量描述",
      "required": true/false,
      "default": "默认值（可选）",
      "options": ["选项1", "选项2"]（可选）,
      "group": "分组名（可选）",
      "source": "regex|inferred"
    }}
  ]
}}
```

规则：
- 从 {{ variable }} 语法中提取变量名
- 分析过滤器（| filter）推断变量类型
- 分析条件（{% if %}）推断是否必填
- 根据变量名和上下文推断描述
- source 为 "regex" 表示直接从模板语法提取，"inferred" 表示 AI 推断"#;

/// 变量填充 prompt
pub const VARIABLE_FILL_PROMPT: &str = r#"根据项目上下文，为模板变量推断合适的值。

模板变量:
```json
{variables}
```

项目上下文:
```
{project_context}
```

请输出 JSON 格式:
```json
{{
  "filled": {{
    "变量名": "推断的值"
  }},
  "confidence": 0.95,
  "reasoning": "推断理由"
}}
```

规则：
- 根据表名推断类名（snake_case → PascalCase）
- 根据表结构推断包名、作者等
- 置信度表示推断的可信程度（0-1）
- reasoning 解释推断过程"#;
