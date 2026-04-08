# Template Studio AI 深度集成方案调研报告

> 调研日期：2026-06-24
> 调研范围：template_core 引擎、AI 命令层、模板数据模型、变量系统、渲染管线

---

## 一、现状分析

### 当前 AI 能力（仅 3 个功能）

| 功能 | 命令 | 用途 |
|------|------|------|
| SQL 生成 | `ai_generate_sql` | 自然语言 → CREATE TABLE SQL |
| SQL 修复 | `ai_fix_sql` | 错误 SQL + 报错信息 → 修复后的 SQL |
| 连接测试 | `ai_test_connection` | 检测 AI 服务连通性 |

**局限**：
- AI 仅服务于"建表"场景，与模板系统完全隔离
- 无流式响应，用户等待完整生成
- temperature/max_tokens 硬编码（0.3/2000），未使用提供商配置
- prompt 完全硬编码在前端，无模板化/复用机制
- 不支持 function calling / tool use

### 模板引擎能力（已实现但未被 AI 利用）

| 能力 | 状态 | AI 可利用点 |
|------|------|------------|
| MiniJinja 渲染 | ✅ | AI 生成模板内容 |
| 变量提取（regex） | ✅ Web 端 | AI 可增强为语义分析 |
| 条件系统 | ✅ | AI 可自动生成条件规则 |
| 依赖分析 | ✅ | AI 可理解模板继承关系 |
| 文件树渲染 | ✅ | AI 可预览生成结果 |
| 并行渲染 | ✅ | 验证性能保障 |
| WASM 渲染 | ✅ | 浏览器端实时预览 |

### 变量系统现状

```
当前流程：人工定义 variables.json → 表单渲染 → 用户手动填写 → 渲染

缺失环节：
  1. 无自动变量发现（Web 端有 regex 分析，桌面端未接入）
  2. 无智能变量填充（每次手动填写）
  3. 无变量验证（required 仅做展示，不拦截）
  4. 无变量预设保存/恢复
```

---

## 二、AI 集成总体架构

### 核心理念：AI as Template Agent

不是简单的"AI 补全"，而是让 AI 成为一个**模板代理**，具备：

```
感知（Perceive）→ 推理（Reason）→ 行动（Act）→ 验证（Verify）
     ↑                                              |
     └──────────────── 反馈循环 ←───────────────────┘
```

### 架构分层

```
┌─────────────────────────────────────────────────┐
│                  用户交互层                       │
│  自然语言输入 / 项目选择 / 模板选择 / 确认/修正    │
├─────────────────────────────────────────────────┤
│                  AI Agent 层                      │
│  ┌───────────┐ ┌───────────┐ ┌───────────┐      │
│  │ 变量分析器 │ │ 模板生成器 │ │ 模板编辑器 │      │
│  └─────┬─────┘ └─────┬─────┘ └─────┬─────┘      │
│        │             │             │              │
│  ┌─────┴─────────────┴─────────────┴─────┐      │
│  │         Tool Calling 协议层            │      │
│  │  scan_files / read_file / edit_file    │      │
│  │  render_preview / validate / extract   │      │
│  └───────────────────────────────────────┘      │
├─────────────────────────────────────────────────┤
│                  能力层                           │
│  template_core / database / file_system / AI API │
└─────────────────────────────────────────────────┘
```

---

## 三、六大 AI 功能模块设计

### 模块 1：AI 变量自动提取与填充

**目标**：从项目上下文自动推断并填充模板变量，消除手动填写。

#### 1.1 变量自动发现

**现有基础**：Web 端 `TemplateAnalysisService::analyze_variables()` 已实现 regex 提取。

**增强方案**：

```rust
// 新增 Tauri 命令
#[tauri::command]
async fn ai_analyze_template_variables(
    template_path: String,
    provider: String,
    model: String,
) -> Result<String, String>
```

**工作流**：
1. 读取模板所有文件内容
2. regex 提取 `{{ variable }}` 和 `{{ variable | filter }}` 模式
3. 读取现有 `variables.json`（如有）
4. 将提取结果 + 文件上下文发送给 AI：
   ```
   以下是模板文件中使用的变量：
   - projectName: 出现在 5 个文件中，上下文: "项目 {{ projectName }} 的入口"
   - database: 出现在 3 个文件中，上下文: "{{ database | upper }}://..."
   - enableAuth: 出现在条件语句中: {% if enableAuth %}

   现有 variables.json 定义了: projectName, database

   请：
   1. 为每个变量推断类型（string/number/boolean/select）
   2. 推断 title（中文显示名）
   3. 推断 description（帮助文本）
   4. 推断 default 值
   5. 标记 missing（模板使用但未定义）和 unused（定义但未使用）
   6. 为 select 类型推断 options
   ```
5. AI 返回结构化 JSON，直接写入 `variables.json`

#### 1.2 变量智能填充

**场景**：用户选择项目后，AI 自动根据项目元数据填充变量。

```rust
#[tauri::command]
async fn ai_fill_template_variables(
    template_id: i64,
    project_id: i64,
    provider: String,
    model: String,
) -> Result<String, String>
```

**工作流**：
1. 获取项目信息：名称、描述、数据源类型、数据库名
2. 获取项目表结构：所有表名、列名、类型、注释
3. 获取项目语言配置：主语言、前端/后端语言
4. 获取变量 schema（`variables.json`）
5. 构建 prompt：
   ```
   项目信息：
   - 名称: user-center
   - 描述: 用户中心微服务
   - 数据源: MySQL, 数据库名: user_center
   - 主语言: Java (Spring Boot)
   - 前端语言: TypeScript (Vue)
   - 表结构: users(id, username, email, created_at), roles(id, name)

   模板需要以下变量：
   - projectName (string): 项目名称
   - packageName (string): Java 包名
   - database (select): 数据库类型 [mysql/postgresql/sqlite]
   - enableAuth (boolean): 是否启用认证
   - tables (json): 表配置数组

   请根据项目信息填充所有变量，返回 JSON。
   ```
6. AI 返回填充后的变量 JSON
7. 前端展示 AI 填充结果，用户可逐项确认/修改

#### 1.3 变量验证与修复

```rust
#[tauri::command]
async fn ai_validate_and_fix_variables(
    template_path: String,
    variables_json: String,
) -> Result<String, String>
```

**工作流**：
1. 用 `render_tree()` 尝试渲染
2. 收集所有 `RenderError`（特别是 `undefined_error`、`missing_argument`）
3. 将错误 + 变量 + 模板片段发送给 AI：
   ```
   渲染失败，错误列表：
   1. 文件 src/main.go 第 15 行: 变量 'database_host' 未定义
   2. 文件 src/config.yml 第 3 行: 变量 'port' 缺少必需参数

   当前变量: {"projectName": "user-center", ...}

   请修复变量定义，补充缺失变量并修正错误。
   ```
4. AI 返回修复后的变量 JSON
5. 重新渲染验证

---

### 模块 2：AI 模板智能推荐

**目标**：根据项目特征自动推荐最合适的模板。

#### 2.1 项目特征分析

```rust
#[tauri::command]
async fn ai_recommend_templates(
    project_id: i64,
    provider: String,
    model: String,
) -> Result<String, String>
```

**工作流**：
1. 分析项目元数据：
   - 数据源类型 → 过滤模板的 database 变量
   - 主语言 → 过滤模板的 language 标签
   - 表数量/复杂度 → 推断需要 scaffold 还是 basic
   - 已有映射配置 → 推断技术栈
2. 获取可用模板列表（本地已下载 + 远程可下载）
3. 将项目特征 + 模板列表发送给 AI：
   ```
   项目特征：
   - 技术栈: Java Spring Boot + MyBatis Plus
   - 数据库: MySQL, 12 张表
   - 已配置映射: MySQL → Java (MyBatis)
   - 项目类型: REST API 微服务

   可用模板：
   1. java-springboot-crud (scaffold): Spring Boot CRUD 生成器
   2. mybatis-plus-generator (data_driven): MyBatis Plus 代码生成
   3. java-rest-api (basic): 基础 REST API 模板
   4. vue3-admin (scaffold): Vue 3 管理后台

   请推荐最合适的模板，说明理由，并给出排序。
   ```
4. AI 返回推荐列表 + 理由
5. 前端高亮推荐模板，显示推荐理由

#### 2.2 模板适配度评分

AI 从多个维度评分：
- **技术栈匹配度**：模板语言 vs 项目语言
- **数据库匹配度**：模板支持的 DB vs 项目 DB
- **功能覆盖度**：模板生成的文件类型 vs 项目需求
- **复杂度匹配**：模板类型（basic/scaffold/data_driven）vs 项目规模

---

### 模块 3：AI 模板内容生成

**目标**：AI 直接生成模板文件内容，而非仅填充变量。

#### 3.1 从项目反向生成模板

```rust
#[tauri::command]
async fn ai_generate_template_from_project(
    project_id: i64,
    provider: String,
    model: String,
) -> Result<String, String>
```

**工作流**：
1. 获取项目所有表结构（表名、列、类型、注释、关系）
2. 获取项目类型映射配置（DB 类型 → 目标语言类型）
3. 获取表规范配置（PK 命名、审计字段、软删除等）
4. 构建 prompt：
   ```
   根据以下数据库设计生成 Java MyBatis Plus 模板：

   表结构：
   CREATE TABLE users (
     id BIGINT AUTO_INCREMENT PRIMARY KEY COMMENT '用户ID',
     username VARCHAR(50) NOT NULL COMMENT '用户名',
     email VARCHAR(100) COMMENT '邮箱',
     created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
     deleted_at TIMESTAMP NULL COMMENT '删除时间'
   );

   规范：
   - PK: id, BIGINT, 自增
   - 审计字段: created_at, updated_at
   - 软删除: deleted_at
   - 命名: 驼峰 → 下划线

   请生成以下模板文件：
   1. entity.java.tpl - 实体类
   2. mapper.java.tpl - Mapper 接口
   3. mapper.xml.tpl - MyBatis XML
   4. service.java.tpl - Service 接口
   5. serviceImpl.java.tpl - Service 实现
   6. controller.java.tpl - Controller

   使用 MiniJinja 模板语法，变量名使用 {{ variable }} 格式。
   ```
5. AI 返回多个文件的模板内容
6. 创建新模板目录，写入文件
7. 自动生成 `variables.json`（基于 prompt 中使用的变量）

#### 3.2 AI 辅助模板编辑

**核心能力：Tool Calling**

这是最关键的模块——让 AI 通过工具调用直接操作模板文件。

##### 工具定义

```rust
// 注册给 AI 的工具集
const TOOLS: &[Tool] = &[
    Tool {
        name: "scan_template_files",
        description: "列出模板目录下的所有文件",
        parameters: json!({
            "type": "object",
            "properties": {
                "template_path": { "type": "string" }
            }
        }),
    },
    Tool {
        name: "read_file",
        description: "读取模板文件内容",
        parameters: json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string" },
                "line_start": { "type": "integer" },
                "line_end": { "type": "integer" }
            }
        }),
    },
    Tool {
        name: "edit_file",
        description: "行级编辑模板文件（插入/替换/删除指定行范围）",
        parameters: json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string" },
                "operation": { "type": "string", "enum": ["insert", "replace", "delete"] },
                "line_start": { "type": "integer" },
                "line_end": { "type": "integer" },
                "content": { "type": "string" }
            }
        }),
    },
    Tool {
        name: "create_file",
        description: "创建新模板文件",
        parameters: json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string" },
                "content": { "type": "string" }
            }
        }),
    },
    Tool {
        name: "delete_file",
        description: "删除模板文件",
        parameters: json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string" }
            }
        }),
    },
    Tool {
        name: "render_preview",
        description: "渲染模板预览（使用当前变量）",
        parameters: json!({
            "type": "object",
            "properties": {
                "variables": { "type": "object" }
            }
        }),
    },
    Tool {
        name: "validate_template",
        description: "验证模板语法和变量",
        parameters: json!({
            "type": "object",
            "properties": {
                "check_variables": { "type": "boolean" }
            }
        }),
    },
    Tool {
        name: "extract_variables",
        description: "从模板文件中提取变量定义",
        parameters: json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string" }
            }
        }),
    },
    Tool {
        name: "update_variables_json",
        description: "更新 variables.json 变量定义",
        parameters: json!({
            "type": "object",
            "properties": {
                "variables": { "type": "object" }
            }
        }),
    },
];
```

##### Agent 循环

```rust
async fn ai_template_agent(
    template_path: String,
    user_instruction: String,
    provider: String,
    model: String,
) -> Result<String, String> {
    let mut messages = vec![
        Message::system("你是模板编辑助手。你可以通过工具调用来读取、编辑、验证模板文件。每次修改后请验证结果。"),
        Message::user(&user_instruction),
    ];

    loop {
        // 调用 AI（带 tools）
        let response = call_ai_with_tools(&provider, &model, &messages, &TOOLS).await?;

        // 检查是否有工具调用
        if let Some(tool_calls) = response.tool_calls {
            for call in tool_calls {
                let result = execute_tool(&call.name, &call.arguments).await?;
                messages.push(Message::tool_result(&call.id, &result));
            }
            continue; // 继续循环，让 AI 处理工具结果
        }

        // 无工具调用 → AI 返回最终回答
        return Ok(response.content);
    }
}
```

##### 使用场景示例

**用户指令**：给所有实体类添加 `@Data` 注解和 Lombok 导入

**AI 行动序列**：
1. `scan_template_files` → 获取文件列表
2. `read_file("entity.java.tpl")` → 读取实体模板
3. `edit_file("entity.java.tpl", insert, 1, 1, "import lombok.Data;\n")` → 添加导入
4. `edit_file("entity.java.tpl", insert, 5, 5, "@Data\n")` → 添加注解
5. `validate_template()` → 验证无语法错误
6. 返回："已为 entity.java.tpl 添加 @Data 注解和 Lombok 导入。"

**用户指令**：新增一个 DTO 模板文件

**AI 行动序列**：
1. `read_file("entity.java.tpl")` → 参考实体类结构
2. `create_file("dto.java.tpl", "...")` → 创建 DTO 模板（只保留需要的字段）
3. `extract_variables("dto.java.tpl")` → 提取新变量
4. `update_variables_json(...)` → 更新变量定义
5. `render_preview()` → 预览渲染结果
6. 返回："已创建 dto.java.tpl，新增变量 dtoClassName。"

---

### 模块 4：AI 渲染结果验证

**目标**：AI 自动验证模板渲染输出的正确性。

#### 4.1 语法验证

```rust
#[tauri::command]
async fn ai_validate_rendered_output(
    rendered_files: String,  // JSON: [{path, content}]
    language: String,
    provider: String,
    model: String,
) -> Result<String, String>
```

**验证维度**：
1. **模板语法**：MiniJinja 语法是否正确（`validate_template` 工具）
2. **变量完整性**：所有 `{{ variable }}` 是否都有定义
3. **条件一致性**：`conditions.yml` 中的变量是否都存在于 `variables.json`
4. **文件引用**：`{% include %}` / `{% extends %}` 引用的文件是否存在

#### 4.2 语义验证

将渲染结果发送给 AI 进行语义检查：

```
以下是模板渲染生成的 Java 代码，请检查：

1. 语法是否正确（Java 语法）
2. 类名是否与文件名一致
3. import 是否完整
4. 注解是否正确
5. 字段类型是否与数据库列类型匹配
6. 是否有明显的逻辑错误

文件列表：
- UserEntity.java: {{ content }}
- UserMapper.java: {{ content }}
- UserMapper.xml: {{ content }}
```

AI 返回错误列表和修复建议。

#### 4.3 编译验证（高级）

对于 Java/TypeScript 等有编译器的语言：
1. 渲染输出写入临时目录
2. 调用 `javac` / `tsc` 编译
3. 收集编译错误
4. 将错误 + 源码发送给 AI 修复
5. 重新渲染 → 重新编译（循环直到通过或达到最大重试次数）

---

### 模块 5：AI 模板差异对比

**目标**：智能对比两个模板版本或模板与项目实际代码的差异。

#### 5.1 模板版本对比

```rust
#[tauri::command]
async fn ai_diff_template_versions(
    template_id: i64,
    version_a: String,
    version_b: String,
    provider: String,
    model: String,
) -> Result<String, String>
```

AI 分析两个版本的差异，生成人类可读的变更说明：
- 新增了哪些文件/变量
- 修改了哪些逻辑
- 删除了什么
- 对现有项目的影响

#### 5.2 模板 vs 项目代码对比

对比模板渲染结果与项目中已有的代码文件，识别：
- 哪些文件会被覆盖
- 哪些文件是新增的
- 哪些现有文件不会被模板覆盖（可能需要手动处理）
- 冲突的代码段

---

### 模块 6：AI 对话式模板配置

**目标**：通过自然语言对话完成整个模板配置流程。

#### 6.1 对话流设计

```
用户: 我有一个 Spring Boot 项目，用 MySQL，需要生成 CRUD 代码

AI: 好的，我来帮你配置模板。先了解一下你的项目：
    1. 项目名称是什么？
    2. 包名是什么？
    3. 需要生成哪些表的代码？

用户: 项目名 user-center，包名 com.example.user，需要 users 和 roles 两张表

AI: 我已从数据库获取到表结构。推荐使用 java-springboot-crud 模板。
    已自动填充以下变量：
    - projectName: user-center
    - packageName: com.example.user
    - database: mysql
    - tables: [users, roles]

    预览生成的文件：
    ├── src/main/java/com/example/user/
    │   ├── entity/UserEntity.java
    │   ├── mapper/UserMapper.java
    │   ├── service/UserService.java
    │   └── controller/UserController.java
    │   ├── entity/RoleEntity.java
    │   ├── mapper/RoleMapper.java
    │   ├── service/RoleService.java
    │   └── controller/RoleController.java

    要导出到哪个目录？

用户: 导出到 ./generated

AI: 已导出 8 个文件到 ./generated。验证结果：
    ✅ Java 语法正确
    ✅ 类名与文件名一致
    ✅ 注解完整
    ⚠️ 建议: UserMapper.xml 中缺少 roles 表的关联查询，需要补充吗？

用户: 补充一下

AI: 已更新 UserMapper.xml，添加了 roles 关联查询。重新导出完成。
```

#### 6.2 实现方式

使用多轮对话 + Tool Calling：

```rust
struct TemplateChatSession {
    template_path: Option<String>,
    project_id: Option<i64>,
    variables: serde_json::Value,
    messages: Vec<Message>,
    state: ChatState,
}

enum ChatState {
    GatheringInfo,      // 收集项目信息
    SelectingTemplate,  // 选择模板
    ConfiguringVars,    // 配置变量
    Previewing,         // 预览结果
    Exporting,          // 导出
    FixingIssues,       // 修复问题
}
```

---

## 四、技术实现细节

### 4.1 Tool Calling 协议

使用 OpenAI 兼容的 function calling 格式：

```json
{
  "model": "deepseek-chat",
  "messages": [...],
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "edit_file",
        "description": "行级编辑模板文件",
        "parameters": {
          "type": "object",
          "properties": {
            "file_path": { "type": "string" },
            "operation": { "type": "string" },
            "line_start": { "type": "integer" },
            "content": { "type": "string" }
          },
          "required": ["file_path", "operation"]
        }
      }
    }
  ],
  "tool_choice": "auto"
}
```

**响应解析**：
```rust
struct AiResponse {
    content: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

struct ToolCall {
    id: String,
    name: String,
    arguments: serde_json::Value,
}
```

### 4.2 新增 Tauri 命令清单

| 命令 | 模块 | 说明 |
|------|------|------|
| `ai_analyze_template_variables` | 变量分析 | 从模板提取变量并推断 schema |
| `ai_fill_template_variables` | 变量填充 | 根据项目上下文自动填充变量 |
| `ai_validate_and_fix_variables` | 变量验证 | 渲染失败时 AI 自动修复变量 |
| `ai_recommend_templates` | 推荐 | 根据项目特征推荐模板 |
| `ai_generate_template_from_project` | 生成 | 从项目表结构反向生成模板 |
| `ai_edit_template` | 编辑 | Agent 循环：自然语言 → 工具调用 → 文件修改 |
| `ai_validate_rendered_output` | 验证 | 验证渲染结果的语法和语义 |
| `ai_diff_template_versions` | 对比 | 智能对比模板版本差异 |
| `ai_template_chat` | 对话 | 多轮对话式模板配置 |

### 4.3 前端新增页面/组件

| 组件 | 路径 | 用途 |
|------|------|------|
| `AiTemplateAssistant` | `components/ai/` | 全局 AI 助手浮窗（对话式交互） |
| `AiVariableFill` | `components/ai/` | 变量自动填充面板（显示 AI 填充结果，逐项确认） |
| `AiTemplateGenerator` | `components/ai/` | 模板生成向导（项目 → 模板） |
| `AiValidationResult` | `components/ai/` | 验证结果面板（错误列表 + 修复建议） |
| `AiRecommendCard` | `components/ai/` | 模板推荐卡片（推荐理由 + 匹配度评分） |

### 4.4 共享 Rust 模块

```
crates/ai_agent/
  Cargo.toml
  src/
    lib.rs              → 公开 API
    tools.rs            → 工具定义和执行
    agent.rs            → Agent 循环（感知→推理→行动→验证）
    prompts.rs          → Prompt 模板管理
    validation.rs       → 渲染结果验证逻辑
    context.rs          → 项目上下文构建
```

---

## 五、Prompt 工程策略

### 5.1 系统 Prompt 模板

```rust
const TEMPLATE_AGENT_SYSTEM: &str = r#"
你是 Template Studio 的 AI 模板助手。你的职责是帮助用户管理和生成代码模板。

能力：
1. 读取和分析模板文件结构
2. 提取和推断模板变量
3. 根据项目上下文自动填充变量
4. 行级编辑模板文件
5. 验证渲染结果的正确性
6. 推荐合适的模板

规则：
- 所有模板使用 MiniJinja 语法（{{ variable }}、{% if %}、{% for %}）
- 修改文件前先读取当前内容
- 每次修改后验证结果
- 用中文与用户交流
- 不确定时询问用户确认

可用工具：{tools_description}
"#;
```

### 5.2 变量推断 Prompt

```rust
const VARIABLE_INFERENCE_PROMPT: &str = r#"
分析以下模板文件，为每个变量推断属性。

模板文件：
{file_contents}

已知变量定义（如有）：
{existing_variables}

请为每个变量返回：
- name: 变量名
- type: string | number | boolean | select | multi-select | text | date | json
- title: 中文显示名
- description: 帮助文本
- default: 推荐默认值
- required: 是否必需（根据上下文推断）
- options: select 类型的选项列表
- group: 分组名

返回 JSON 格式。
"#;
```

### 5.3 代码验证 Prompt

```rust
const VALIDATION_PROMPT: &str = r#"
以下是模板渲染生成的 {language} 代码，请验证：

{files}

验证清单：
1. 语法是否正确
2. 命名是否一致（类名/文件名）
3. 引用是否完整（import/include）
4. 类型是否匹配
5. 是否有未渲染的模板变量（{{ ... }} 或 {% ... %}）

返回 JSON：
{{
  "valid": true/false,
  "errors": [{{ "file": "...", "line": N, "message": "...", "fix": "..." }}],
  "warnings": [{{ "file": "...", "message": "...", "suggestion": "..." }}]
}}
"#;
```

---

## 六、实施路线

### Phase 1：基础能力（2 周）

**目标**：AI 能理解模板并自动填充变量

| 任务 | 工作量 | 依赖 |
|------|--------|------|
| 新增 `crates/ai_agent` crate | 1 天 | 无 |
| 实现 tool calling 协议（`ai.rs` 改造） | 2 天 | 无 |
| 实现基础工具集（scan/read/edit/render/validate） | 3 天 | tool calling |
| 实现 `ai_analyze_template_variables` 命令 | 1 天 | 工具集 |
| 实现 `ai_fill_template_variables` 命令 | 2 天 | 工具集 |
| 前端：变量自动填充面板 | 2 天 | 命令 |
| 测试 + 修复 | 2 天 | 全部 |

**交付物**：
- 用户选择项目后，AI 自动填充 80%+ 的模板变量
- 变量填充结果可逐项确认/修改

### Phase 2：模板编辑 Agent（2 周）

**目标**：AI 通过自然语言编辑模板

| 任务 | 工作量 | 依赖 |
|------|--------|------|
| 实现 Agent 循环（多轮 tool calling） | 2 天 | Phase 1 |
| 实现 `ai_edit_template` 命令 | 2 天 | Agent 循环 |
| 实现 `ai_validate_rendered_output` 命令 | 2 天 | 工具集 |
| 前端：AI 助手浮窗（对话式交互） | 3 天 | 命令 |
| 前端：验证结果面板 | 1 天 | 验证命令 |
| 测试 + 修复 | 2 天 | 全部 |

**交付物**：
- 用户可以用自然语言修改模板（如"给所有实体添加 @Data 注解"）
- AI 修改后自动验证，报告问题

### Phase 3：智能推荐与生成（2 周）

**目标**：AI 能推荐模板并从项目反向生成

| 任务 | 工作量 | 依赖 |
|------|--------|------|
| 实现 `ai_recommend_templates` 命令 | 2 天 | Phase 1 |
| 实现 `ai_generate_template_from_project` 命令 | 3 天 | Agent 循环 |
| 前端：模板推荐卡片 | 1 天 | 推荐命令 |
| 前端：模板生成向导 | 3 天 | 生成命令 |
| 测试 + 修复 | 2 天 | 全部 |

**交付物**：
- 选择项目后 AI 推荐最合适的模板
- 可以从项目表结构反向生成模板

### Phase 4：对话式配置 + 闭环验证（2 周）

**目标**：完整的 AI 对话式模板工作流

| 任务 | 工作量 | 依赖 |
|------|--------|------|
| 实现 `ai_template_chat` 命令（多轮对话） | 3 天 | Phase 2 |
| 实现编译验证（Java/TS） | 2 天 | Phase 2 |
| 实现 `ai_diff_template_versions` 命令 | 1 天 | 工具集 |
| 前端：对话式配置界面 | 3 天 | 对话命令 |
| 端到端测试 | 2 天 | 全部 |

**交付物**：
- 完整的对话式模板配置流程
- 渲染 → 编译 → AI 修复 → 重新渲染闭环
- 模板版本智能对比

---

## 七、关键决策点

### 7.1 AI 调用位置：Rust vs 前端

| 方案 | 优点 | 缺点 |
|------|------|------|
| **Rust 后端（推荐）** | 可复用于 CLI/WASM、安全性好、可做 rate limiting | 开发量稍大 |
| 前端 JS | 开发快、直接操作 DOM | 仅桌面端可用、API Key 暴露在前端 |

**决策**：AI 调用统一在 Rust 层（`crates/ai_agent`），前端只负责 UI 展示和用户交互。

### 7.2 Tool Calling 格式

| 方案 | 优点 | 缺点 |
|------|------|------|
| **OpenAI function calling（推荐）** | 标准协议、DeepSeek/GLM/OpenAI 都支持 | 部分模型不支持 |
| 自定义 JSON 模式 | 灵活、所有模型可用 | 需要自己解析、容易出错 |

**决策**：使用 OpenAI function calling 格式。对于不支持的模型，回退到 JSON 模式 + prompt 引导。

### 7.3 文件编辑粒度

| 方案 | 优点 | 缺点 |
|------|------|------|
| **行级编辑（推荐）** | 精确、可审计、支持 undo | 需要维护行号 |
| 全文件替换 | 简单 | 丢失格式、难以 undo |
| Diff/patch | 标准格式 | AI 生成 patch 质量不稳定 |

**决策**：行级编辑。AI 返回 `line_start`/`line_end` + `content`，Rust 端执行实际文件操作。

### 7.4 验证策略

| 层级 | 方式 | 成本 | 准确度 |
|------|------|------|--------|
| L1 模板语法 | MiniJinja 解析 | 低 | 高 |
| L2 变量完整性 | regex + 交叉检查 | 低 | 高 |
| L3 代码语法 | AI 分析 | 中 | 中 |
| L4 编译验证 | javac/tsc | 高 | 高 |
| L5 运行时验证 | 单元测试 | 最高 | 最高 |

**决策**：L1+L2 自动执行，L3 按需执行，L4+L5 用户主动触发。

---

## 八、风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| AI 生成的模板语法错误 | 渲染失败 | L1 语法验证 + 自动修复循环 |
| AI 编辑破坏现有文件 | 数据丢失 | Git 版本控制 + 编辑前备份 |
| AI 推断的变量类型不准确 | 表单体验差 | 用户可手动修正 + 记忆修正 |
| Tool calling 不被所有模型支持 | 兼容性 | JSON 模式回退 |
| AI 响应慢（大文件分析） | 用户体验 | 流式响应 + 进度提示 |
| API Key 安全 | 泄露风险 | 存储在本地 SQLite，仅 Rust 层使用 |

---

## 九、总结

### 核心价值

| 场景 | 当前体验 | AI 集成后 |
|------|---------|----------|
| 填充变量 | 手动填写 20+ 字段 | AI 自动填充，逐项确认 |
| 选择模板 | 浏览列表，凭经验选择 | AI 推荐 + 理由 |
| 编辑模板 | 手动修改文件 | 自然语言描述，AI 执行 |
| 验证结果 | 手动检查 | AI 自动验证 + 修复建议 |
| 新建模板 | 从零编写 | 从项目反向生成 |

### 技术可行性

- **高**：变量自动填充（已有 regex 提取 + 项目元数据丰富）
- **高**：模板推荐（规则 + AI 混合，规则兜底）
- **高**：渲染验证（已有 MiniJinja 错误信息 + AI 语义分析）
- **中**：模板编辑 Agent（依赖 tool calling 稳定性）
- **中**：模板内容生成（质量取决于 prompt 和模型能力）
- **低**：编译验证闭环（需要各语言编译器环境）

### 工作量估算

| 阶段 | 工作量 | 核心交付 |
|------|--------|---------|
| Phase 1 | 2 周 | 变量自动填充 |
| Phase 2 | 2 周 | 模板编辑 Agent |
| Phase 3 | 2 周 | 推荐 + 反向生成 |
| Phase 4 | 2 周 | 对话式配置 + 闭环验证 |
| **总计** | **8 周** | **完整 AI 模板助手** |
