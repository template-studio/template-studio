# Template Studio AI 架构规划

> 日期：2026-06-24（修订 v2）
> 核心问题：AI 能力如何分层？外部 Agent 如何协作？模板创作如何落地？

---

## 一、核心设计原则

### 1.1 AI 必须嵌入

AI 能力要真正有用，必须深入到业务上下文中：

- **变量分析** — 需要读取模板文件内容、解析 MiniJinja 语法、理解过滤器/条件/继承
- **变量填充** — 需要了解项目表结构、类型映射、命名规范、已有变量值
- **模板编辑** — 需要行级定位、语法理解、上下文感知
- **渲染验证** — 需要实际渲染结果、语言语法规则、项目规范

这些场景的共同点：**AI 必须拿到完整上下文才能做好**。MCP 暴露工具描述的方式无法提供足够的上下文。

### 1.2 外部协作是辅助

外部 Agent（Claude Code 等）的定位是**大方向协作**，不是细节操作：

- 触发 AI 分析、渲染、验证等流程
- 读取结果并做高层决策
- 批量操作、流水线编排

细节操作（行级编辑、变量推断、语法修正）由嵌入式 AI 完成。

### 1.3 CLI 是枢纽

CLI 命令是所有外部交互的统一接口：

- **Desktop** → Tauri 命令 → `ai_agent` crate（嵌入式，UI 上下文丰富）
- **CLI 用户** → `ts ai ...` 命令（结构化输出，可脚本化）
- **VS Code 扩展** → 调用 CLI 命令（模板创作体验最优）
- **外部 Agent** → AI Skill 指导调用 CLI 命令（完整上下文，工作流驱动）

---

## 二、架构总览

```
┌─────────────────────────────────────────────────────────────────────┐
│                            应用层                                    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐           │
│  │ Desktop  │  │   CLI    │  │ VS Code  │  │  外部     │           │
│  │ (Tauri)  │  │  (clap)  │  │ Extension│  │  Agent    │           │
│  │          │  │          │  │          │  │ Claude Code│           │
│  │ 嵌入式   │  │ ai 子命令 │  │ 模板创作  │  │ Cursor 等 │           │
│  │ UI 上下文 │  │ 结构化输出│  │ 行内标记  │  │ AI Skill  │           │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘           │
│       │              │              │              │                │
│       ▼              ▼              ▼              ▼                │
│  ┌──────────────────────────────────────────────────────────┐       │
│  │               crates/ai_agent (核心)                      │       │
│  │                                                          │       │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │       │
│  │  │ 变量分析  │ │ 模板编辑  │ │ 渲染验证  │ │ 项目→模板 │   │       │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘   │       │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐                 │       │
│  │  │ 变量填充  │ │ 推荐引擎  │ │ Agent 循环│                 │       │
│  │  └──────────┘ └──────────┘ └──────────┘                 │       │
│  │                                                          │       │
│  │  ┌──────────────────────────────────────┐                │       │
│  │  │ AiClient (OpenAI 兼容)               │                │       │
│  │  │ DeepSeek / GLM / MiMo / OpenAI       │                │       │
│  │  └──────────────────────────────────────┘                │       │
│  └──────────────────────────────────────────────────────────┘       │
│       │              │              │              │                │
│       ▼              ▼              ▼              ▼                │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                          │
│  │template_core│ │ database │  │  shared   │                        │
│  └──────────┘  └──────────┘  └──────────┘                          │
└─────────────────────────────────────────────────────────────────────┘
```

### 关键设计

| 组件 | 职责 | 特点 |
|------|------|------|
| `crates/ai_agent` | 核心 AI 逻辑 | 纯 Rust，不依赖传输协议，所有客户端复用 |
| Desktop Tauri 命令 | 嵌入式 AI 体验 | 直接访问 UI 状态、项目上下文、数据库 |
| CLI `ai` 子命令 | 结构化 CLI 接口 | 可脚本化、JSON 输出、外部 Agent 可调用 |
| VS Code Extension | 模板创作体验 | 行内变量标记、侧边栏预览、一键转换上传 |
| AI Skill | 外部协作层 | 完整工作流描述、CLI 命令指导、边界条件 |

---

## 三、CLI `ai` 子命令设计

### 3.1 命令结构

```
ts ai <subcommand> [options]

子命令：
  analyze-variables    分析模板变量
  fill-variables       自动填充变量
  convert-to-template  项目转换为模板
  validate             验证模板
  render-preview       渲染预览
  render-export        导出渲染结果
  recommend            推荐模板
  edit-file            编辑模板文件
  config               AI 配置管理
```

### 3.2 命令详细设计

#### 变量分析

```bash
# 分析模板变量，输出 JSON schema
ts ai analyze-variables ./my-template

# 指定输出格式
ts ai analyze-variables ./my-template --format json
ts ai analyze-variables ./my-template --format table
```

**输出示例**（JSON 格式）：
```json
{
  "template": "my-template",
  "variables": [
    {
      "name": "tableName",
      "type": "string",
      "title": "表名",
      "description": "数据库表名（snake_case）",
      "required": true,
      "source": "regex"
    },
    {
      "name": "className",
      "type": "string",
      "title": "类名",
      "description": "Java 类名（PascalCase）",
      "required": true,
      "source": "inferred",
      "suggestion": "User"
    }
  ],
  "total": 12,
  "auto_inferred": 5
}
```

#### 变量填充

```bash
# 基于项目上下文自动填充
ts ai fill-variables ./my-template --project 1

# 指定 AI 提供商和模型
ts ai fill-variables ./my-template --project 1 --provider deepseek --model deepseek-chat

# 输出填充结果（不写入文件）
ts ai fill-variables ./my-template --project 1 --dry-run

# 直接写入 variables.json
ts ai fill-variables ./my-template --project 1 --write
```

**输出示例**：
```json
{
  "template": "my-template",
  "project": "user-center",
  "filled": {
    "tableName": "user",
    "className": "User",
    "basePackage": "com.example.user",
    "author": "cicbyte"
  },
  "confidence": 0.92,
  "ai_reasoning": "根据项目表结构和命名规范推断..."
}
```

#### 项目转换为模板

```bash
# 将现有项目转换为模板
ts ai convert-to-template ./my-crud-project -o ./my-template

# 指定模板名称和分类
ts ai convert-to-template ./my-crud-project -o ./my-template --name "Java CRUD" --category crud

# 指定变量替换策略
ts ai convert-to-template ./my-crud-project -o ./my-template --strategy conservative
```

**输出示例**：
```json
{
  "source": "./my-crud-project",
  "output": "./my-template",
  "files_scanned": 24,
  "files_converted": 18,
  "variables_found": [
    { "name": "tableName", "occurrences": 47, "example": "user" },
    { "name": "className", "occurrences": 32, "example": "User" },
    { "name": "basePackage", "occurrences": 18, "example": "com.example.user" }
  ],
  "skipped_files": ["README.md", ".gitignore", "target/"],
  "variables_json": "./my-template/.meta/variables/variables.json"
}
```

**转换流程**：
1. 扫描项目文件，排除 `target/`、`node_modules/`、`.git/` 等
2. AI 分析文件内容，识别重复模式（表名、类名、包名、作者等）
3. 推断变量类型和 schema（snake_case → tableName, PascalCase → className）
4. 替换为 `{{ variableName }}` 语法
5. 生成 `variables.json`（含 type/title/description/required/default）
6. 输出到目标目录

#### 渲染预览

```bash
# 渲染预览（输出文件树 + 内容摘要）
ts ai render-preview ./my-template --vars '{"tableName":"user","className":"User"}'

# 从变量文件读取
ts ai render-preview ./my-template --vars-file ./variables.json

# 输出完整渲染内容
ts ai render-preview ./my-template --vars-file ./variables.json --full
```

**输出示例**：
```
渲染结果 (6 files):
├── src/main/java/com/example/user/UserController.java      [342 bytes]
├── src/main/java/com/example/user/UserService.java         [289 bytes]
├── src/main/java/com/example/user/UserRepository.java      [156 bytes]
├── src/main/java/com/example/user/User.java                [478 bytes]
├── src/main/java/com/example/user/UserMapper.java          [201 bytes]
└── pom.xml                                                  [1,203 bytes]

变量使用: 12/12 已填充, 0 未填充
```

#### 验证

```bash
# 验证模板语法
ts ai validate ./my-template

# 验证变量完整性
ts ai validate ./my-template --vars-file ./variables.json

# 验证渲染输出
ts ai validate ./my-template --vars-file ./variables.json --check-output
```

**输出示例**：
```json
{
  "syntax": { "valid": true, "errors": [] },
  "variables": { "valid": true, "missing": [], "unused": ["debug"] },
  "output": {
    "valid": false,
    "errors": [
      { "file": "User.java", "line": 15, "message": "缺少 import 语句" }
    ]
  }
}
```

#### 编辑文件

```bash
# 行级插入
ts ai edit-file ./my-template/entity.java.tpl --insert 5 --content "@Data"

# 行级替换
ts ai edit-file ./my-template/entity.java.tpl --replace 10-15 --content "新的内容"

# 行级删除
ts ai edit-file ./my-template/entity.java.tpl --delete 20-22

# 追加到文件末尾
ts ai edit-file ./my-template/entity.java.tpl --append "// generated by AI"
```

#### 推荐模板

```bash
# 基于项目推荐模板
ts ai recommend --project 1

# 指定语言和分类
ts ai recommend --language java --category crud

# 输出推荐理由
ts ai recommend --project 1 --explain
```

### 3.3 CLI 输出格式

所有 `ai` 子命令支持 `--format` 参数：

| 格式 | 说明 | 适用场景 |
|------|------|---------|
| `json` | 结构化 JSON | 外部 Agent 解析、脚本集成 |
| `table` | 表格（默认） | 人类阅读 |
| `compact` | 紧凑单行 | 日志、流水线 |

```bash
ts ai analyze-variables ./my-template --format json
ts ai fill-variables ./my-template --project 1 --format compact
```

### 3.4 CLI 错误处理

```bash
# 错误输出到 stderr，JSON 格式
{
  "error": "TEMPLATE_NOT_FOUND",
  "message": "模板不存在: ./nonexistent",
  "suggestion": "运行 'ts template list' 查看可用模板"
}

# 退出码
# 0 = 成功
# 1 = 通用错误
# 2 = 模板/变量错误
# 3 = AI 服务错误
# 4 = 验证失败
```

---

## 四、AI Skill 设计

### 4.1 Skill 定义

```markdown
<!-- .claude/skills/template-ai/skill.md -->
---
name: template-ai
description: >
  模板 AI 工作流。当用户提到"模板变量"、"渲染"、"生成代码"、"模板编辑"、
  "代码生成"、"CRUD 生成"时触发。
---

# Template Studio AI 助手

## 概述

Template Studio 是一个模板管理和代码生成平台。本 Skill 提供 AI 辅助的模板操作工作流。

## CLI 命令

所有 AI 操作通过 `ts ai` 命令组执行：

### 分析变量
```bash
ts ai analyze-variables <template-path> [--format json|table|compact]
```
分析模板文件，提取所有变量并推断类型/schema。
- 输出包含变量名、类型、是否必填、推断来源
- `source: "regex"` 表示从模板语法直接提取
- `source: "inferred"` 表示 AI 推断

### 自动填充变量
```bash
ts ai fill-variables <template-path> --project <id> [--provider deepseek] [--dry-run|--write]
```
基于项目上下文（表结构、类型映射、命名规范）自动填充变量。
- `--dry-run` 只输出不写入
- `--write` 直接写入 variables.json
- 输出包含置信度和 AI 推理过程

### 渲染预览
```bash
ts ai render-preview <template-path> --vars-file <path> [--full]
```
渲染模板并预览结果。
- 默认输出文件树 + 内容摘要
- `--full` 输出完整文件内容

### 验证
```bash
ts ai validate <template-path> [--vars-file <path>] [--check-output]
```
多层验证：语法 → 变量 → 渲染输出。

### 编辑文件
```bash
ts ai edit-file <file-path> --insert <line> --content <text>
ts ai edit-file <file-path> --replace <start>-<end> --content <text>
ts ai edit-file <file-path> --delete <start>-<end>
```
行级编辑模板文件。

### 推荐模板
```bash
ts ai recommend --project <id> [--language <lang>] [--category <cat>] [--explain]
```
基于项目特征推荐合适的模板。

## 标准工作流

### 场景 1：为项目生成 CRUD 代码

1. `ts template list --category crud` — 查看可用 CRUD 模板
2. `ts ai analyze-variables <template>` — 了解需要哪些变量
3. `ts ai fill-variables <template> --project <id> --dry-run` — 预览自动填充结果
4. 确认后 `ts ai fill-variables <template> --project <id> --write` — 写入变量
5. `ts ai render-preview <template> --vars-file variables.json` — 预览渲染结果
6. `ts ai validate <template> --vars-file variables.json --check-output` — 验证输出
7. 确认后 `ts create <project-name> -T <template> --config-file variables.json -o <output>`

### 场景 2：修改模板添加功能

1. `ts template info <template> --files` — 查看模板文件结构
2. `ts ai analyze-variables <template>` — 了解现有变量
3. `ts ai edit-file <file> --insert 5 --content "@Data"` — 编辑模板
4. `ts ai validate <template>` — 验证语法
5. `ts ai render-preview <template> --vars-file variables.json` — 确认效果

### 场景 3：检查模板质量

1. `ts ai validate <template>` — 语法检查
2. `ts ai validate <template> --vars-file variables.json --check-output` — 完整验证
3. 根据错误信息修复：`ts ai edit-file ...`

### 场景 4：将现有项目转换为模板

1. `ts ai convert-to-template ./my-project -o ./my-template` — AI 扫描并转换
2. 在 VS Code 中打开 `./my-template`，review 变量标记
3. `ts ai validate ./my-template` — 验证语法
4. `ts ai render-preview ./my-template --vars-file variables.json` — 预览渲染结果
5. `ts ai validate ./my-template --check-output` — 验证渲染输出
6. 确认后 `ts template upload ./my-template` — 上传模板

**注意**：推荐使用 VS Code 扩展进行转换，支持行内变量标记和实时预览。

## 注意事项

- 变量 schema 使用 JSON Schema 格式（type/title/description/required/default/options/group/condition）
- 渲染前必须验证变量完整性
- AI 填充的变量需要人工确认，置信度仅供参考
- 编辑操作不自动保存，需要明确指定 `--write` 或通过 `ts create` 触发渲染

## 错误处理

所有命令的错误输出到 stderr，JSON 格式。常见错误：
- `TEMPLATE_NOT_FOUND` — 模板路径不存在
- `VARIABLE_MISSING` — 缺少必填变量
- `AI_SERVICE_ERROR` — AI 服务调用失败
- `VALIDATION_FAILED` — 验证未通过
```

### 4.2 Skill 的价值

| 维度 | MCP 工具描述 | AI Skill |
|------|-------------|----------|
| 上下文占用 | 25 个 JSON Schema（~2000 tokens） | 1 个 Skill 文件（~800 tokens，按需加载） |
| 描述完整性 | 一行 description | 完整工作流 + 边界条件 + 示例 |
| Agent 理解度 | 靠猜 | 靠读 |
| 工作流指导 | 无 | 标准流程 + 场景示例 |
| 错误处理 | 无 | 错误码 + 退出码 + 建议 |

---

## 五、crates/ai_agent 模块设计

### 5.1 crate 结构

```
crates/ai_agent/
  Cargo.toml
  src/
    lib.rs              → 公开 API
    config.rs           → AI 配置（提供商、模型、API Key）
    client.rs           → OpenAI 兼容 HTTP 客户端
    tools/
      mod.rs            → 工具注册表
      template.rs       → 模板操作工具
      variable.rs       → 变量管理工具
      render.rs         → 渲染工具
      file.rs           → 文件编辑工具
      project.rs        → 项目分析工具
      validate.rs       → 验证工具
    agent.rs            → Agent 循环（多轮 tool calling）
    prompts/
      mod.rs            → Prompt 模板管理
      variable.rs       → 变量推断 prompt
      validate.rs       → 验证 prompt
      recommend.rs      → 推荐 prompt
    context.rs          → 项目上下文构建
    types.rs            → 共享类型
```

### 5.2 核心 trait

```rust
/// AI 工具 trait — 所有工具实现此 trait
#[async_trait]
pub trait AiTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;  // JSON Schema
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError>;
}

/// AI 客户端 trait — 支持多种提供商
#[async_trait]
pub trait AiClient: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, AiError>;
    async fn chat_with_tools(&self, request: ChatRequest, tools: Vec<ToolDefinition>) -> Result<ChatResponse, AiError>;
    async fn chat_stream(&self, request: ChatRequest) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, AiError>>>>, AiError>;
}

/// Agent 循环
pub struct Agent {
    client: Box<dyn AiClient>,
    tools: Vec<Box<dyn AiTool>>,
    max_iterations: usize,
    system_prompt: String,
}

impl Agent {
    pub async fn run(&self, user_message: &str) -> Result<AgentResult, AgentError>;
}
```

### 5.3 依赖关系

```toml
[dependencies]
template_studio_template_core = { path = "../template_core" }
template_studio_shared = { path = "../shared" }
template_studio_infrastructure = { path = "../infrastructure" }
template_studio_services = { path = "../services" }

reqwest = { version = "0.12", features = ["json", "stream"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
regex = "1"
```

---

## 六、桌面端嵌入式 AI

### 6.1 Tauri 命令

桌面端直接调用 `ai_agent` crate，享受丰富的 UI 上下文：

```rust
// apps/desktop/src-tauri/src/commands/ai_agent.rs

#[tauri::command]
pub async fn ai_analyze_variables(
    database: tauri::State<'_, DbState>,
    template_path: String,
    provider: String,
    model: String,
) -> Result<String, String> {
    let db = database.as_ref();
    let client = create_ai_client(&provider, &model, db).await?;
    let result = ai_agent::variable::analyze_variables(&client, &template_path).await
        .map_err(|e| format!("分析失败: {}", e))?;
    serde_json::to_string(&result).map_err(|e| format!("序列化失败: {}", e))
}

#[tauri::command]
pub async fn ai_fill_variables(
    database: tauri::State<'_, DbState>,
    template_path: String,
    project_id: i64,
    provider: String,
    model: String,
    dry_run: bool,
) -> Result<String, String> {
    let db = database.as_ref();
    let client = create_ai_client(&provider, &model, db).await?;
    let project = db.get_project(project_id).await.map_err(|e| format!("获取项目失败: {}", e))?;
    let tables = db.get_project_tables(project_id).await.map_err(|e| format!("获取表失败: {}", e))?;
    let mappings = db.get_project_mappings(project_id).await.map_err(|e| format!("获取映射失败: {}", e))?;

    let context = ai_agent::context::ProjectContext { project, tables, mappings };
    let result = ai_agent::variable::fill_variables(&client, &template_path, &context).await
        .map_err(|e| format!("填充失败: {}", e))?;

    if !dry_run {
        ai_agent::variable::write_variables(&template_path, &result.filled).await
            .map_err(|e| format!("写入失败: {}", e))?;
    }
    serde_json::to_string(&result).map_err(|e| format!("序列化失败: {}", e))
}
```

### 6.2 嵌入式 AI 的优势

| 维度 | 嵌入式（Desktop） | 外部（CLI/Skill） |
|------|------------------|------------------|
| 项目上下文 | 自动获取（DbState） | 需要 `--project` 参数 |
| 表结构 | 实时查询 | 通过 API |
| 类型映射 | 直接读取 | 需要参数 |
| UI 状态 | 可感知（当前页面、选中项） | 无感知 |
| 交互方式 | 流式输出、进度条、确认弹窗 | 一次性输出 |
| 错误恢复 | UI 提示 + 重试 | stderr + 退出码 |

### 6.3 前端 AI 组件

```
src/components/ai/
  AiAssistant.vue         → 全局 AI 助手浮窗（对话式）
  AiVariablePanel.vue     → 变量自动填充面板
  AiRecommendCard.vue     → 模板推荐卡片
  AiValidationResult.vue  → 验证结果面板
```

---

## 七、CLI 集成方式

### 7.1 新增 `ai` 子命令组

在现有 CLI 架构中新增 `Commands::Ai` 变体：

```rust
// apps/cli/src/cli/mod.rs

#[derive(Subcommand, Debug)]
pub enum Commands {
    Create(CreateCommand),
    Template { template_subcommand: TemplateCommands },
    Config { config_subcommand: ConfigCommands },
    /// AI 辅助命令
    Ai {
        #[command(subcommand)]
        ai_subcommand: AiCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum AiCommands {
    /// 分析模板变量
    AnalyzeVariables {
        /// 模板路径
        path: String,
        /// 输出格式
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// 自动填充变量
    FillVariables {
        /// 模板路径
        path: String,
        /// 项目 ID
        #[arg(long)]
        project: i64,
        /// AI 提供商
        #[arg(long)]
        provider: Option<String>,
        /// AI 模型
        #[arg(long)]
        model: Option<String>,
        /// 只预览不写入
        #[arg(long)]
        dry_run: bool,
        /// 直接写入
        #[arg(long)]
        write: bool,
        /// 输出格式
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// 项目转换为模板
    ConvertToTemplate {
        /// 项目路径
        path: String,
        /// 输出目录
        #[arg(short, long)]
        output: String,
        /// 模板名称
        #[arg(long)]
        name: Option<String>,
        /// 模板分类
        #[arg(long)]
        category: Option<String>,
        /// 转换策略 (conservative/aggressive)
        #[arg(long, default_value = "conservative")]
        strategy: String,
    },
    /// 渲染预览
    RenderPreview {
        /// 模板路径
        path: String,
        /// 变量文件路径
        #[arg(long)]
        vars_file: Option<String>,
        /// 变量 JSON 字符串
        #[arg(long)]
        vars: Option<String>,
        /// 输出完整内容
        #[arg(long)]
        full: bool,
    },
    /// 验证模板
    Validate {
        /// 模板路径
        path: String,
        /// 变量文件路径
        #[arg(long)]
        vars_file: Option<String>,
        /// 检查渲染输出
        #[arg(long)]
        check_output: bool,
    },
    /// 编辑模板文件
    EditFile {
        /// 文件路径
        path: String,
        /// 在指定行后插入
        #[arg(long)]
        insert: Option<usize>,
        /// 替换行范围 (start-end)
        #[arg(long)]
        replace: Option<String>,
        /// 删除行范围 (start-end)
        #[arg(long)]
        delete: Option<String>,
        /// 追加到末尾
        #[arg(long)]
        append: Option<String>,
        /// 内容
        #[arg(long)]
        content: Option<String>,
    },
    /// 推荐模板
    Recommend {
        /// 项目 ID
        #[arg(long)]
        project: Option<i64>,
        /// 编程语言
        #[arg(long)]
        language: Option<String>,
        /// 模板分类
        #[arg(long)]
        category: Option<String>,
        /// 输出推荐理由
        #[arg(long)]
        explain: bool,
    },
    /// AI 配置管理
    Config {
        #[command(subcommand)]
        config_subcommand: AiConfigCommands,
    },
}
```

### 7.2 命令处理

```rust
// apps/cli/src/cli/commands.rs

pub async fn handle_ai(ai_cmd: AiCommands, config: Config) -> Result<()> {
    match ai_cmd {
        AiCommands::AnalyzeVariables { path, format } => {
            let client = create_ai_client(&config).await?;
            let result = ai_agent::variable::analyze_variables(&client, &path).await?;
            print_output(&result, &format)?;
        }
        AiCommands::FillVariables { path, project, provider, model, dry_run, write, format } => {
            let client = create_ai_client_with_override(&config, provider, model).await?;
            let context = build_project_context(&config, project).await?;
            let result = ai_agent::variable::fill_variables(&client, &path, &context).await?;
            if write {
                ai_agent::variable::write_variables(&path, &result.filled).await?;
            }
            print_output(&result, &format)?;
        }
        // ... 其他子命令
    }
    Ok(())
}
```

### 7.3 CLI 模块扩展

```
apps/cli/src/
  ai/
    mod.rs              → AI 命令入口
    variables.rs        → 变量分析/填充命令处理
    render.rs           → 渲染预览/导出命令处理
    validate.rs         → 验证命令处理
    edit.rs             → 文件编辑命令处理
    recommend.rs        → 推荐命令处理
    config.rs           → AI 配置命令处理
    output.rs           → 输出格式化（JSON/Table/Compact）
```

---

## 八、VS Code 扩展（模板创作）

### 8.1 定位

VS Code 扩展是**模板创作**的最优体验。核心逻辑全在 `ai_agent` + CLI，扩展只做薄 UI 层。

| 场景 | 最佳工具 | 理由 |
|------|---------|------|
| 使用模板生成代码 | Desktop / CLI | 一次性操作，不需要 IDE |
| **项目转换为模板** | **VS Code 扩展** | 行内标记变量、实时预览、迭代编辑 |
| 修改已有模板 | VS Code 扩展 / CLI | 本地文件编辑 + AI 辅助 |
| 外部 Agent 协作 | CLI + Skill | 结构化输出，工作流驱动 |

### 8.2 核心功能

#### 项目 → 模板转换

```
用户在 VS Code 中打开项目
        │
        │  Ctrl+Shift+P → "Convert to Template"
        │
        ▼
  AI 扫描项目 → 行内高亮变量候选 → 侧边栏显示变量列表
        │
        │  用户点击确认/取消每个变量标记
        │
        ▼
  替换为 {{ var }} 语法 → 生成 variables.json → 侧边栏预览渲染结果
        │
        │  确认无误
        │
        ▼
  一键上传到 Template Studio
```

#### 行内变量标记

```typescript
// 转换前（原始项目）
public class UserController {
    private UserService userService;
    // ...
}

// 转换后（VS Code 中显示）
public class {{ className }}Controller {        // ← 黄色高亮，点击可取消
    private {{ className }}Service {{ serviceName }};  // ← 黄色高亮
    // ...
}
```

- **InlineDecoration** — 变量标记处显示黄色背景
- **Hover** — 鼠标悬停显示变量信息（类型、描述、来源）
- **Click** — 点击切换变量/原始值
- **CodeLens** — 变量上方显示 "已标记为变量" 提示

#### 侧边栏面板

```
┌─────────────────────────────────────┐
│ Template Studio          [Upload ▶] │
├─────────────────────────────────────┤
│ Variables (8)                       │
│ ┌─────────────────────────────────┐ │
│ │ tableName    = "user"       [✓] │ │
│ │ className    = "User"       [✓] │ │
│ │ basePackage  = "com.example"[✓] │ │
│ │ author       = "cicbyte"   [✓] │ │
│ │ createdAt    = "2026-06-24"[✓] │ │
│ └─────────────────────────────────┘ │
├─────────────────────────────────────┤
│ Preview                             │
│ ┌─────────────────────────────────┐ │
│ │ src/main/java/com/example/      │ │
│ │   user/                         │ │
│ │     UserController.java   [342B]│ │
│ │     UserService.java      [289B]│ │
│ │     User.java             [478B]│ │
│ └─────────────────────────────────┘ │
├─────────────────────────────────────┤
│ Validation                          │
│ ✓ Syntax valid                      │
│ ✓ All variables defined             │
│ ✓ 6 files rendered successfully     │
└─────────────────────────────────────┘
```

### 8.3 命令面板命令

| 命令 | 说明 | 底层调用 |
|------|------|---------|
| `Convert to Template` | 项目转换为模板 | `ts ai convert-to-template` |
| `Analyze Variables` | 分析当前模板变量 | `ts ai analyze-variables` |
| `Preview Template` | 预览渲染结果 | `ts ai render-preview` |
| `Validate Template` | 验证模板 | `ts ai validate` |
| `Upload Template` | 上传模板到服务器 | `ts template upload` |
| `AI: Fill Variables` | AI 自动填充变量 | `ts ai fill-variables` |

### 8.4 技术方案

```
vs_plugin/
  package.json            → 扩展清单、命令注册、视图定义
  src/
    extension.ts          → 扩展入口
    commands/
      convert.ts          → Convert to Template 命令
      analyze.ts          → Analyze Variables 命令
      preview.ts          → Preview Template 命令
      validate.ts         → Validate Template 命令
      upload.ts           → Upload Template 命令
    decorations/
      variableDecoration.ts  → 行内变量高亮装饰器
    views/
      variablePanel.ts    → 变量列表 WebviewView
      previewPanel.ts     → 渲染预览 WebviewView
      validationPanel.ts  → 验证结果 WebviewView
    cli/
      executor.ts         → CLI 命令执行器（child_process）
      parser.ts           → JSON 输出解析
    utils/
      path.ts             → 路径工具
      config.ts           → 扩展配置
  webview/
    variablePanel.html    → 变量面板 UI
    previewPanel.html     → 预览面板 UI
```

**关键实现**：

```typescript
// src/cli/executor.ts — 调用 CLI 命令
import { execFile } from 'child_process';

export async function executeCli(command: string, args: string[]): Promise<any> {
  return new Promise((resolve, reject) => {
    execFile('ts', ['ai', command, ...args, '--format', 'json'], (error, stdout) => {
      if (error) reject(error);
      else resolve(JSON.parse(stdout));
    });
  });
}

// src/decorations/variableDecoration.ts — 行内高亮
const variableDecorationType = vscode.window.createTextEditorDecorationType({
  backgroundColor: 'rgba(255, 220, 100, 0.3)',
  borderRadius: '3px',
  border: '1px solid rgba(255, 220, 100, 0.6)',
});
```

### 8.5 与 CLI 的关系

VS Code 扩展是 CLI 的薄 UI 包装，不重复实现逻辑：

```
VS Code Extension (TypeScript)
        │
        │  child_process.execFile('ts', ['ai', ...])
        │
        ▼
CLI `ts ai` (Rust)
        │
        ▼
crates/ai_agent (Rust)
```

扩展只负责：
1. 调用 CLI 命令并解析 JSON 输出
2. 行内装饰（InlineDecoration）显示变量标记
3. Webview 面板展示变量列表和预览
4. 命令面板命令注册

---

## 九、外部 Agent 协作方式

### 9.1 协作模型

```
外部 Agent（Claude Code / Cursor）
        │
        │  读取 AI Skill（完整工作流描述）
        │
        ▼
  理解工作流 → 调用 CLI 命令 → 解析结构化输出 → 决策下一步
        │
        │  ts ai analyze-variables ./template
        │  ts ai fill-variables ./template --project 1 --format json
        │  ts ai render-preview ./template --vars-file vars.json
        │
        ▼
  解析 JSON 输出 → 人类确认 → 批量操作
```

### 9.2 与 MCP 方案的对比

| 维度 | MCP 25 工具 | CLI + Skill |
|------|------------|-------------|
| 上下文占用 | 高（~2000 tokens 工具描述） | 低（~800 tokens Skill，按需加载） |
| 描述完整性 | 受限（JSON Schema） | 完整（工作流 + 边界 + 示例） |
| Agent 理解度 | 靠猜工具用途 | 靠读完整文档 |
| 错误处理 | 无指导 | 错误码 + 退出码 + 建议 |
| 工作流 | 无（Agent 自行编排） | 标准流程 + 场景示例 |
| 输出格式 | 固定 JSON | 可选 JSON/Table/Compact |
| 可脚本化 | 否（MCP 协议） | 是（标准 CLI） |
| 维护成本 | 高（25 个工具定义） | 低（CLI 命令 + 1 个 Skill 文件） |

### 9.3 可选：MCP 薄包装

如果未来需要 MCP 协议支持，可以做一个薄包装层，将 CLI 命令映射为 MCP 工具：

```rust
// 可选的 MCP 适配层（不在 Phase 1 范围内）
// 将 5-8 个核心 CLI 命令暴露为 MCP 工具
// 工具描述从 Skill 文件生成，保证一致性
```

这不是优先级，CLI + Skill 已经足够覆盖外部协作需求。

---

## 十、实施路线

### Phase 1：AI 核心 + CLI 基础（2 周）

| 任务 | 工作量 | 交付物 |
|------|--------|--------|
| 创建 `crates/ai_agent` crate | 0.5 天 | crate 骨架 |
| 实现 `AiClient` trait + OpenAI 兼容客户端 | 1 天 | 可调用任意 OpenAI 兼容 API |
| 实现变量分析工具（regex + AI 推断） | 2 天 | 变量自动发现 |
| 实现变量填充工具（项目上下文 → 自动填充） | 2 天 | 变量自动填充 |
| CLI 新增 `ai analyze-variables` 命令 | 0.5 天 | CLI 变量分析 |
| CLI 新增 `ai fill-variables` 命令 | 0.5 天 | CLI 变量填充 |
| CLI 输出格式化（JSON/Table/Compact） | 0.5 天 | 结构化输出 |
| 编写 AI Skill 文件 | 0.5 天 | 外部协作文档 |

**验证**：`ts ai analyze-variables ./template` 输出完整变量 schema，`ts ai fill-variables ./template --project 1 --format json` 输出填充结果。

### Phase 2：渲染 + 验证 + 编辑 + 项目转换（2 周）

| 任务 | 工作量 | 交付物 |
|------|--------|--------|
| 实现渲染预览工具 | 1.5 天 | 渲染预览 |
| 实现渲染导出工具 | 1 天 | 渲染导出 |
| 实现验证工具（语法 + 变量 + 输出） | 2 天 | 多层验证 |
| 实现文件编辑工具（行级操作） | 2 天 | 文件编辑 |
| 实现项目转换工具（项目 → 模板） | 2 天 | 项目转换 |
| CLI 新增 render/validate/edit/convert 命令 | 1 天 | CLI 完整命令 |
| 桌面端：AI 变量面板组件 | 1.5 天 | 嵌入式变量 UI |

**验证**：完整工作流 — 分析变量 → 填充 → 渲染预览 → 验证 → 导出；项目转换 — 扫描 → 标记 → 预览 → 上传。

### Phase 3：Agent 循环 + 推荐 + 高级（1.5 周）

| 任务 | 工作量 | 交付物 |
|------|--------|--------|
| 实现 Agent 循环（多轮 tool calling） | 2 天 | Agent 引擎 |
| 实现模板推荐工具 | 1 天 | 推荐引擎 |
| 实现渲染对比工具 | 0.5 天 | 对比工具 |
| 桌面端：AI 助手对话组件 | 2 天 | 嵌入式对话 UI |
| 端到端测试 + Skill 完善 | 1 天 | 完整可用 |

**验证**：Claude Code 通过 Skill 指导完成完整的模板操作工作流。

### Phase 4：VS Code 扩展（2 周）

| 任务 | 工作量 | 交付物 |
|------|--------|--------|
| VS Code 扩展骨架 + 命令注册 | 1 天 | 扩展框架 |
| CLI 执行器 + JSON 输出解析 | 0.5 天 | CLI 调用层 |
| Convert to Template 命令 | 2 天 | 项目转换命令 |
| 行内变量装饰器（InlineDecoration） | 1.5 天 | 变量高亮标记 |
| 变量列表侧边栏面板（WebviewView） | 1.5 天 | 变量管理面板 |
| 渲染预览侧边栏面板 | 1 天 | 预览面板 |
| 验证结果面板 + 上传命令 | 1 天 | 验证 + 上传 |
| 端到端测试 + 文档 | 0.5 天 | 发布就绪 |

**验证**：在 VS Code 中打开项目 → Convert to Template → 行内高亮变量 → 侧边栏预览 → 一键上传。

---

## 十一、与现有系统的关系

### 11.1 不破坏现有功能

| 现有功能 | 影响 |
|----------|------|
| AI SQL 生成/修复 | 不变，继续使用 `commands/ai.rs` |
| 模板渲染向导 | 不变，继续使用 HTTP API |
| 数据库浏览器 | 不变 |
| 其他 99 个 Tauri 命令 | 不变 |
| CLI 现有命令 | 不变，`ai` 是新增子命令组 |
| VS Code 扩展 | 全新，不影响现有功能 |

### 11.2 渐进式增强

```
现有：用户手动操作 → 手动填变量 → 手动渲染 → 手动导出
增强：用户选择项目 → AI 自动填变量 → 自动渲染 → 一键导出
创作：VS Code 打开项目 → AI 转换为模板 → 行内标记变量 → 预览 → 上传
高级：外部 Agent 通过 Skill 编排 CLI 命令完成全流程
```

### 11.3 数据流对比

**现有**：
```
用户 → 前端 UI → Tauri 命令 → database/template_core → 结果
```

**增强后**：
```
用户 → 前端 UI → Tauri 命令 → ai_agent → AI API + template_core → 结果
                                        ↑
CLI 用户 → ts ai ... → ai_agent --------┘
                                        ↑
VS Code → 扩展 → CLI → ai_agent --------┘
                                        ↑
外部 Agent → Skill → CLI → ai_agent ----┘
```

---

## 十二、技术选型

| 组件 | 选型 | 理由 |
|------|------|------|
| AI 客户端 | `reqwest` + OpenAI 兼容 API | 支持所有主流提供商 |
| Agent 框架 | 自研（轻量） | 避免重框架依赖 |
| 流式响应 | `reqwest` + `tokio::stream` | 原生异步流 |
| 工具描述 | JSON Schema | 标准格式 |
| Prompt 管理 | 内嵌 Rust 字符串 + 模板化 | 简单直接 |
| 外部协作 | CLI + AI Skill | 完整上下文、低开销 |
| MCP | 可选薄包装 | 非优先级，CLI 足够 |

---

## 十三、总结

### 核心决策

1. **AI 必须嵌入** — `crates/ai_agent` 是核心，Desktop 通过 Tauri 命令直接调用，享受完整 UI 上下文
2. **CLI 是枢纽** — `ts ai` 子命令组是所有外部交互的统一接口，结构化输出，可脚本化
3. **VS Code 扩展专注模板创作** — 项目→模板转换的最优体验，行内变量标记，侧边栏预览
4. **AI Skill 指导外部协作** — 完整工作流描述，CLI 命令指导，边界条件说明
5. **MCP 非必需** — CLI + Skill 已足够覆盖外部协作，MCP 薄包装可选

### 架构一句话

> AI 能力嵌入 `crates/ai_agent` 核心 crate，Desktop 直接调用获得深度体验，CLI `ai` 命令组提供结构化接口，VS Code 扩展提供模板创作最优体验，AI Skill 指导外部 Agent 通过 CLI 协作。

### 各客户端定位

| 客户端 | 定位 | 核心场景 |
|--------|------|---------|
| Desktop | 模板使用 + 项目管理 | 选择项目 → AI 填充变量 → 渲染导出 |
| CLI | 统一接口 + 脚本集成 | 命令行操作、流水线、外部 Agent 调用 |
| VS Code 扩展 | 模板创作 | 项目→模板转换、行内变量标记、实时预览 |
| 外部 Agent | 大方向协作 | 通过 Skill 编排 CLI 命令完成全流程 |

### 为什么不用 MCP 作为主要接口

| MCP 方案的问题 | CLI + Skill 的优势 |
|---------------|-------------------|
| 25 个工具描述占 ~2000 tokens | 1 个 Skill 文件 ~800 tokens |
| JSON Schema 描述不完整 | 完整工作流 + 边界条件 + 示例 |
| Agent 靠猜工具用途 | Agent 靠读完整文档 |
| 无工作流指导 | 标准流程 + 场景示例 |
| 不可脚本化 | 标准 CLI，可集成到任何流水线 |
| 维护成本高（25 个工具定义） | 维护成本低（CLI 命令 + 1 个 Skill） |
