# 模板渲染功能设计文档

> 作者：Claude Code
> 日期：2026-06-16

---

## 一、功能概述

### 1.1 定位

**模板渲染**是一个通用的模板执行入口，不拘泥于脚手架、CRUD 或任何特定场景。用户可以：

- 选择任意已下载的模板
- 根据模板定义的变量 schema 自动生成表单输入
- 在"高级模式"下直接编辑 JSON 变量
- 可选注入当前项目的表信息作为上下文
- 实时预览渲染结果，一键导出到指定目录

### 1.2 与现有功能的关系

| 现有功能 | 关系 |
|---------|------|
| `TableConfigDrawer` | 专为单表 CRUD 代码生成设计，配置结构固定（Basic/Fields/Options/Extra）。模板渲染是通用入口，不限于表场景 |
| `AiCreateTableDrawer` | AI 辅助建表，输出是 SQL DDL。模板渲染的输出是任意文件 |
| `render_template_preview` | 后端已有的渲染管道，模板渲染直接复用 |
| 模板向导（TemplatesView） | 模板向导是项目创建流程，模板渲染是独立的轻量渲染工具 |

### 1.3 核心设计原则

1. **schema 驱动**：表单字段完全由模板的 `variables.json` 决定，不硬编码任何字段
2. **双模式**：简单模式（表单）和高级模式（JSON 编辑器）可随时切换，数据双向同步
3. **上下文注入**：可选将项目表信息、数据源信息等注入变量，模板可通过 `{{ tables }}` 等访问
4. **实时预览**：变量变更后自动触发渲染预览（防抖 500ms）

---

## 二、用户流程

```
┌─────────────────────────────────────────────────────────┐
│                    模板渲染 Drawer                        │
│                                                          │
│  ┌── Step 1: 选择模板 ──────────────────────────────────┐ │
│  │  搜索/筛选已下载模板 → 选择模板 → 选择版本            │ │
│  └──────────────────────────────────────────────────────┘ │
│                        ↓                                  │
│  ┌── Step 2: 填写变量 ──────────────────────────────────┐ │
│  │  [简单模式]  自动生成的表单（基于 variables.json）     │ │
│  │  [高级模式]  JSON 编辑器（CodeMirror）                │ │
│  │  [上下文]    ☑ 注入项目表信息  ☑ 注入数据源信息       │ │
│  └──────────────────────────────────────────────────────┘ │
│                        ↓                                  │
│  ┌── Step 3: 预览结果 ──────────────────────────────────┐ │
│  │  左侧：渲染后的文件树                                │ │
│  │  右侧：文件内容预览（语法高亮）                       │ │
│  │  错误提示（变量缺失、渲染失败）                       │ │
│  └──────────────────────────────────────────────────────┘ │
│                        ↓                                  │
│  ┌── Step 4: 导出 ─────────────────────────────────────┐ │
│  │  选择输出目录 → 确认导出                             │ │
│  └──────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

---

## 三、变量 Schema 结构

模板变量定义在 `.meta/variables/variables.json` 中，Web 后端通过 `GET /api/v1/template-files/variables` 返回 `fieldSchemaJson`。

### 3.1 Schema 格式推断

根据 Web 后端 `template_files.rs` 的实现，`fieldSchemaJson` 是一个 JSON 字符串，解析后用于生成表单。需要支持的字段类型：

| type | 表单组件 | 说明 |
|------|---------|------|
| `string` | `a-input` | 单行文本 |
| `text` | `a-textarea` | 多行文本 |
| `number` | `a-input-number` | 数字 |
| `boolean` | `a-switch` | 开关 |
| `select` | `a-select` | 下拉选择，需 `options` |
| `multi-select` | `a-select mode="multiple"` | 多选 |
| `date` | `a-date-picker` | 日期 |
| `color` | 颜色选择器 | 颜色 |
| `json` | JSON 编辑器 | 嵌套对象/数组 |

### 3.2 Schema 字段结构

```json
{
  "fields": [
    {
      "name": "project_name",
      "label": "项目名称",
      "type": "string",
      "default": "my-project",
      "required": true,
      "placeholder": "请输入项目名称",
      "description": "生成的项目目录名",
      "group": "基础信息"
    },
    {
      "name": "author",
      "label": "作者",
      "type": "string",
      "default": "",
      "required": false,
      "group": "基础信息"
    },
    {
      "name": "use_database",
      "label": "启用数据库",
      "type": "boolean",
      "default": true,
      "group": "功能选项"
    },
    {
      "name": "database_type",
      "label": "数据库类型",
      "type": "select",
      "options": ["mysql", "postgresql", "sqlite"],
      "default": "mysql",
      "condition": { "field": "use_database", "operator": "eq", "value": true },
      "group": "功能选项"
    }
  ]
}
```

### 3.3 条件字段

支持字段级别的条件显隐（与模板引擎的 condition 系统对齐）：

- `eq` / `ne` — 等于/不等于
- `in` / `not_in` — 包含/不包含
- `gt` / `lt` / `gte` / `lte` — 数值比较

条件字段在简单模式下自动显隐，在高级模式下无影响（JSON 中始终存在）。

---

## 四、上下文注入

### 4.1 注入内容

当用户勾选"注入项目表信息"时，自动将以下数据注入变量：

```json
{
  "__context": {
    "project": {
      "name": "my-project",
      "database_name": "mydb",
      "datasource_type": "mysql"
    },
    "tables": [
      {
        "name": "user",
        "comment": "用户表",
        "engine": "InnoDB",
        "columns": [
          {
            "name": "id",
            "data_type": "bigint",
            "length": 20,
            "is_nullable": false,
            "is_primary_key": true,
            "comment": "主键"
          }
        ]
      }
    ],
    "datasource": {
      "type": "mysql",
      "host": "localhost",
      "port": 3306,
      "database": "mydb"
    }
  }
}
```

### 4.2 模板侧使用

模板中可通过 `{{ __context.project.name }}`、`{% for table in __context.tables %}` 等方式访问上下文数据。这使得同一模板可以：

- 生成单表 CRUD（遍历 `__context.tables`）
- 生成数据库文档（使用 `__context.datasource`）
- 生成项目配置（使用 `__context.project`）

### 4.3 注入选项

| 选项 | 默认 | 说明 |
|------|------|------|
| 注入项目信息 | ☑ | 项目名、数据库名、数据源类型 |
| 注入表信息 | ☐ | 所有表的结构（可能数据量大） |
| 注入指定表 | ☐ | 用户选择的表（弹出表选择器） |
| 注入数据源信息 | ☐ | 连接信息（注意安全，脱敏处理） |

---

## 五、组件设计

### 5.1 组件结构

```
src/views/project/
  └── template-render/
      └── index.vue                  # 模板渲染主页面（或 Drawer）

src/components/templateRender/
  ├── TemplateRenderDrawer.vue       # 主 Drawer（入口）
  ├── TemplateSelector.vue           # Step 1: 模板选择
  ├── VariableForm.vue               # Step 2: 简单模式表单
  ├── VariableJsonEditor.vue         # Step 2: 高级模式 JSON 编辑器
  ├── ContextInjector.vue            # Step 2: 上下文注入选项
  ├── RenderPreview.vue              # Step 3: 渲染结果预览
  └── ExportDialog.vue               # Step 4: 导出确认
```

### 5.2 TemplateRenderDrawer.vue

主容器组件，管理步骤流程和数据流。

**Props:**
- `open: Boolean` — 控制显示
- `project: Object` — 当前项目（用于上下文注入）
- `templateId: String` — 可选，预选模板 ID

**Emits:**
- `update:open` — 关闭
- `exported` — 导出完成

**State:**
```js
const step = ref(1)                    // 当前步骤
const selectedTemplate = ref(null)     // 选中的模板
const selectedVersion = ref(null)      // 选中的版本
const schema = ref(null)               // variables.json 解析后的 schema
const variables = ref({})              // 用户填写的变量值
const rawJson = ref('')                // 高级模式的 JSON 字符串
const mode = ref('simple')             // 'simple' | 'advanced'
const contextOptions = ref({           // 上下文注入选项
  project: true,
  tables: false,
  selectedTables: [],
  datasource: false
})
const renderResult = ref(null)         // 渲染结果
const renderError = ref('')            // 渲染错误
const rendering = ref(false)           // 渲染中
```

### 5.3 TemplateSelector.vue

模板选择组件，展示已下载模板列表。

**功能:**
- 搜索模板（名称、描述）
- 按分类筛选
- 显示模板缩略图/图标、名称、描述、版本列表
- 选择模板后自动加载最新版本的 `variables.json`
- 如果模板无 `variables.json`，提示"此模板无可配置变量，可直接渲染"

### 5.4 VariableForm.vue

简单模式：根据 schema 自动生成表单。

**核心逻辑:**
1. 解析 `schema.fields`，按 `group` 分组
2. 根据 `type` 渲染对应的 Ant Design 组件
3. 根据 `condition` 控制字段显隐
4. 字段变更时 emit `update:variables`
5. 支持 `required` 校验

**渲染规则:**
```js
const componentMap = {
  string: 'a-input',
  text: 'a-textarea',
  number: 'a-input-number',
  boolean: 'a-switch',
  select: 'a-select',
  'multi-select': 'a-select[multiple]',
  date: 'a-date-picker',
  json: 'JsonEditor'  // 嵌套 JSON 用 CodeMirror
}
```

### 5.5 VariableJsonEditor.vue

高级模式：直接编辑 JSON 变量。

**功能:**
- CodeMirror JSON 编辑器，语法高亮 + 自动补全
- 从简单模式切换时，自动填充当前变量值的 JSON
- 从高级模式切回简单模式时，解析 JSON 更新表单
- JSON 格式校验
- 提供 schema 字段名的自动补全提示

### 5.6 ContextInjector.vue

上下文注入配置。

**功能:**
- 开关：注入项目信息 / 注入表信息 / 注入数据源信息
- 表选择器：勾选注入哪些表（a-checkbox-group）
- 预览注入的 JSON 结构
- 脱敏提示：数据源密码等敏感字段自动替换为 `***`

### 5.7 RenderPreview.vue

渲染结果预览。

**功能:**
- 左侧：文件树（`a-tree`），显示渲染后的目录结构
- 右侧：文件内容预览（CodeMirror 只读模式，语法高亮）
- 文件数/总大小统计
- 渲染错误高亮（单个文件失败不影响其他文件）
- 支持搜索文件名

### 5.8 ExportDialog.vue

导出确认对话框。

**功能:**
- 输出目录选择（默认为项目目录下的 `generated/`）
- 导出文件列表预览
- 冲突检测：已存在文件时提示覆盖/跳过/重命名
- 导出进度
- 导出完成后的"打开目录"按钮

---

## 六、后端设计

### 6.1 复用的现有命令

| 命令 | 用途 |
|------|------|
| `list_templates` | 获取已下载模板列表 |
| `get_template_variables` | 获取模板变量 schema（需改造） |
| `render_template_preview` | 渲染模板（已有完整管道） |
| `db_get_project_tables` | 获取项目表列表（上下文注入） |
| `db_get_table_columns` | 获取表列信息（上下文注入） |
| `db_get_datasource` | 获取数据源信息（上下文注入） |
| `write_file` | 写入导出文件 |

### 6.2 需要改造的命令

#### `get_template_variables` — 从 Web API 获取真实 schema

当前是 mock 实现。改造方案：

```rust
#[tauri::command]
async fn get_template_variables(
    template_id: String,
    version: Option<String>,
    config: tauri::State<'_, AppConfig>,
) -> Result<String, String> {
    // 1. 检查模板是否已下载
    let template_path = config.get_template_path(&template_id, &version.unwrap_or_default());
    let variables_file = template_path.join(".meta/variables/variables.json");

    // 2. 如果本地有 variables.json，直接读取
    if variables_file.exists() {
        let content = fs::read_to_string(&variables_file)
            .map_err(|e| format!("读取变量文件失败: {}", e))?;
        return Ok(content);
    }

    // 3. 如果本地没有，调用 Web API
    let web_url = config.get_web_server_url();
    let url = format!("{}/api/v1/template-files/variables?templateId={}", web_url, template_id);
    let resp = reqwest::get(&url).await
        .map_err(|e| format!("请求失败: {}", e))?;
    let body: serde_json::Value = resp.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    // 4. 返回 fieldSchemaJson
    Ok(body["fieldSchemaJson"].as_str().unwrap_or("{}").to_string())
}
```

#### `render_template_preview` — 支持直接传入变量 JSON

当前需要从配置文件加载变量。改造为支持直接传入：

```rust
#[tauri::command]
async fn render_template_preview(
    template_id: String,
    version: Option<String>,
    variables_json: String,  // 新增：直接传入 JSON
    // ... 其他参数
) -> Result<String, String> {
    // 直接使用 variables_json 构建 Variables
    let render_vars = Variables::from_json(&variables_json)
        .map_err(|e| format!("变量解析失败: {}", e))?;

    // 后续渲染逻辑不变
    let rendered = render_tree(template_files, &render_vars)?;
    serde_json::to_string(&rendered).map_err(|e| format!("序列化失败: {}", e))
}
```

### 6.3 新增命令（可选）

#### `cmd_render_and_export` — 渲染并导出到目录

如果需要在 Rust 端完成"渲染 + 写文件"的一体化操作：

```rust
#[tauri::command]
async fn cmd_render_and_export(
    template_id: String,
    version: Option<String>,
    variables_json: String,
    output_dir: String,
    overwrite_mode: String,  // "overwrite" | "skip" | "rename"
) -> Result<ExportResult, String> {
    // 1. 渲染
    let rendered = render_tree(...)?;

    // 2. 写入文件
    let mut result = ExportResult::default();
    for file in rendered {
        let path = Path::new(&output_dir).join(&file.file_path);
        // 冲突处理
        if path.exists() && overwrite_mode == "skip" {
            result.skipped += 1;
            continue;
        }
        // 写入
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, file.file_content.unwrap_or_default())?;
        result.exported += 1;
    }
    Ok(result)
}
```

---

## 七、路由设计

### 方案 A：Drawer 形式（推荐）

在项目工作区侧边栏添加"模板渲染"入口，点击打开 Drawer。

**优点：** 不离开当前页面，轻量
**入口位置：** 项目工作区侧边栏菜单、项目工作台仪表盘快速操作

```
侧边栏菜单项：
  工作台
  表管理
  规范配置
  类型映射
  模板渲染  ← 新增
```

### 方案 B：独立页面

新增路由 `/project/:id/template-render`。

**优点：** 更大的工作空间，适合复杂模板
**缺点：** 增加路由层级

### 建议

采用**方案 A（Drawer）**，宽度 900px（可拖拽调整），与现有 Drawer 风格一致。如果用户反馈需要更大空间，后续可升级为独立页面。

---

## 八、数据流

```
用户选择模板
    ↓
加载 variables.json → 解析 schema
    ↓
┌─────────────────────────────────┐
│  简单模式            高级模式    │
│  VariableForm ←→ VariableJson  │
│  (表单)            (JSON编辑器) │
│       ↕ 双向同步 ↕              │
└─────────────────────────────────┘
    ↓ 变量变更（防抖 500ms）
合并上下文注入 → variables_json
    ↓
调用 render_template_preview
    ↓
渲染结果 → RenderPreview 展示
    ↓
用户确认导出 → cmd_render_and_export
    ↓
写入文件 → 通知完成
```

---

## 九、与 TableConfigDrawer 的共存

`TableConfigDrawer` 保持不变，它仍然是表级别"配置 → 生成代码"的专用工具。

模板渲染是通用工具，两者互补：

| 场景 | 推荐使用 |
|------|---------|
| 为某张表生成 CRUD 代码 | `TableConfigDrawer`（已有字段配置、类型推断） |
| 生成项目脚手架 | 模板渲染 |
| 生成数据库文档 | 模板渲染（注入表信息） |
| 生成 API 接口代码 | 模板渲染（注入表信息 + 自定义变量） |
| 生成任意自定义文件 | 模板渲染 |

---

## 十、实现优先级

### Phase 1：核心功能（MVP）

1. `TemplateRenderDrawer.vue` — 主容器 + 步骤流程
2. `TemplateSelector.vue` — 模板选择（复用 `list_templates`）
3. `VariableForm.vue` — 基于 schema 的表单生成
4. `RenderPreview.vue` — 渲染结果预览（复用 `render_template_preview`）
5. 改造 `get_template_variables` — 支持读取本地 `variables.json`

### Phase 2：高级模式

6. `VariableJsonEditor.vue` — JSON 编辑器（CodeMirror）
7. 简单/高级模式双向同步
8. `ContextInjector.vue` — 上下文注入

### Phase 3：导出

9. `ExportDialog.vue` — 导出确认 + 冲突处理
10. `cmd_render_and_export` — 后端一体化导出命令
11. 导出历史记录

### Phase 4：增强

12. 模板收藏/最近使用
13. 变量预设（保存常用变量组合）
14. 批量渲染（多模板组合）
15. 渲染结果 diff（与已有文件对比）

---

## 十一、技术要点

### 11.1 Schema 解析容错

`variables.json` 可能不存在或格式不规范，需要：

- 文件不存在 → 跳过表单，直接进入高级模式（空 JSON）
- 格式错误 → 提示错误，降级为高级模式
- 字段类型未知 → 降级为 `string` 输入框

### 11.2 变量类型映射

MiniJinja 支持的类型（string/number/bool/array/object）与表单类型的映射：

- `string` / `text` / `select` / `date` / `color` → JSON string
- `number` → JSON number
- `boolean` → JSON boolean
- `multi-select` → JSON array of strings
- `json` → JSON object/array（递归）

### 11.3 渲染性能

- 模板文件扫描 + 渲染可能耗时较长（大模板 100+ 文件）
- 使用防抖（500ms）避免频繁渲染
- 渲染在 Rust 端并行执行（rayon，≥50 文件时自动并行）
- 前端显示 loading 状态

### 11.4 错误处理

- 变量缺失（Strict 模式）→ 渲染结果中标记错误文件，其他文件正常显示
- 模板语法错误 → 同上
- 磁盘写入失败 → 逐文件处理，报告成功/失败数量

---

## 十二、文件清单

### 新建文件

| 文件 | 行数（估） | 说明 |
|------|-----------|------|
| `src/components/templateRender/TemplateRenderDrawer.vue` | ~200 | 主容器 |
| `src/components/templateRender/TemplateSelector.vue` | ~100 | 模板选择 |
| `src/components/templateRender/VariableForm.vue` | ~200 | 表单生成 |
| `src/components/templateRender/VariableJsonEditor.vue` | ~120 | JSON 编辑器 |
| `src/components/templateRender/ContextInjector.vue` | ~80 | 上下文注入 |
| `src/components/templateRender/RenderPreview.vue` | ~150 | 渲染预览 |
| `src/components/templateRender/ExportDialog.vue` | ~100 | 导出确认 |
| `src/components/templateRender/index.js` | ~10 | 统一导出 |

### 修改文件

| 文件 | 修改内容 |
|------|---------|
| `src-tauri/src/lib.rs` | 改造 `get_template_variables`，新增 `cmd_render_and_export` |
| `src/views/project/index.vue` | 侧边栏添加"模板渲染"菜单项 |
| `src/components/layout/ProjectWorkspaceLayout.vue` | 添加菜单项 |
| `src/router/index.js` | 可选：新增路由（如果采用方案 B） |

### 不修改

| 文件 | 原因 |
|------|------|
| `src/components/tableConfig/*` | 保持现有表配置功能不变 |
| `crates/template_core/*` | 模板引擎无需改动 |
| `src/api/templateVariables.js` | 已有 API 可复用 |
