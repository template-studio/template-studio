# AI 建表功能实现总结

## 项目信息

**项目名称：** codegen-desktop
**功能模块：** AI 智能建表
**实现日期：** 2025-02-09
**状态：** ✅ 开发完成，已编译通过

## 功能架构

```
┌─────────────────────────────────────────────────────────────┐
│                       前端 Vue 3                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │ AI 服务配置   │  │ 默认服务设置 │  │  AI 建表UI  │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
└────────────────────────┬────────────────────────────────────┘
                         │ invoke()
                         ↓
┌─────────────────────────────────────────────────────────────┐
│                    Tauri 2.0 命令层                          │
│  ┌────────────────────────────────────────────────────┐     │
│  │ • ai_generate_sql  - AI 生成 SQL                    │     │
│  │ • ai_fix_sql       - AI 修复 SQL                    │     │
│  │ • parse_ai_sql     - 解析 SQL（仅预览）             │     │
│  │ • execute_ai_sql   - 执行 SQL（创建表）             │     │
│  │ • ai_*_provider    - AI 提供商管理                  │     │
│  │ • ai_*_model       - AI 模型管理                    │     │
│  └────────────────────────────────────────────────────┘     │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ↓
┌─────────────────────────────────────────────────────────────┐
│                      Rust 后端逻辑                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │ reqwest HTTP │→ │sqlparser-rs  │→ │    sqlx      │       │
│  │ AI API 调用   │  │  SQL 解析    │  │  数据库操作  │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
└─────────────────────────────────────────────────────────────┘
```

## 实现的功能

### 1. AI 服务管理（已完成）

#### 后端命令
| 命令 | 功能 | 文件位置 |
|-----|------|---------|
| `ai_get_all_providers` | 获取所有 AI 提供商 | lib.rs:1180 |
| `ai_get_provider` | 获取单个 AI 提供商 | lib.rs:1194 |
| `ai_save_provider` | 保存 AI 提供商配置 | lib.rs:1210 |
| `ai_toggle_provider` | 启用/禁用 AI 提供商 | lib.rs:1251 |
| `ai_delete_provider` | 删除 AI 提供商 | lib.rs:1267 |
| `ai_get_provider_models_grouped` | 获取提供商的模型列表 | lib.rs:1282 |
| `ai_add_model` | 添加 AI 模型 | lib.rs:1297 |
| `ai_delete_model` | 删除 AI 模型 | lib.rs:1331 |
| `ai_update_model` | 更新 AI 模型 | lib.rs:1346 |

#### 前端组件
| 组件 | 路径 | 功能 |
|-----|------|-----|
| `ModelProviderConfig.vue` | components/settings/ | 单个提供商配置界面 |
| `DefaultAIService.vue` | components/settings/ | 默认服务选择界面 |
| `SettingsSubSidebar.vue` | components/settings/ | AI 服务侧边栏菜单 |
| `useAIConfigStore.js` | stores/ | AI 配置状态管理 |

### 2. AI SQL 生成和修复（已完成）

#### 后端命令
| 命令 | 功能 | 文件位置 |
|-----|------|---------|
| `ai_generate_sql` | 调用 AI 生成建表 SQL | lib.rs:1378 |
| `ai_fix_sql` | 调用 AI 修复 SQL 错误 | lib.rs:1435 |

**实现细节：**
- 使用 `reqwest` 调用 OpenAI 兼容 API
- 支持自定义 Temperature 和 Max Tokens
- 自动从数据库获取 API 密钥
- 支持的提供商：DeepSeek、GLM、LongCat

#### AI 提示词模板

**生成 SQL：**
```
请根据以下描述生成 {DIALECT} 建表 SQL 语句：

{用户描述}

要求：
1. 使用标准 {DIALECT} 语法
2. 包含完整的字段定义（名称、类型、长度、约束）
3. 添加适当的索引和主键
4. 只返回 CREATE TABLE 语句，不要其他内容
5. 如果有多个表，依次生成多个 CREATE TABLE 语句

请直接输出 SQL 语句（不要用 markdown 代码块包裹）：
```

**修复 SQL：**
```
以下 SQL 执行时出现错误：

{SQL}

错误信息：
{ERROR}

请分析错误原因并修复 SQL 语句。要求：
1. 保持原有的表结构和字段定义
2. 只修复导致错误的部分
3. 确保语法符合 {DIALECT} 标准
4. 只返回修复后的完整 SQL，不要其他解释

请直接输出修复后的 SQL：
```

### 3. SQL 解析和执行（已完成）

#### 后端命令
| 命令 | 功能 | 文件位置 |
|-----|------|---------|
| `parse_ai_sql` | 解析 SQL，返回表结构（不创建） | lib.rs:1504 |
| `execute_ai_sql` | 执行 SQL，在数据库中创建表 | lib.rs:1517 |

**支持的 SQL 方言：**
- MySQL（使用 `MySqlDialect`）
- PostgreSQL（使用 `PostgreSqlDialect`）
- SQLite（使用 `SQLiteDialect`）

**解析实现：**
- 使用 `sqlparser-rs` 库进行 SQL 解析
- 提取表名、字段名、数据类型、约束等信息
- 返回 JSON 格式的表结构

**字段解析能力：**
- ✅ 字段名称
- ✅ 数据类型（INT, VARCHAR, TEXT, DECIMAL 等）
- ✅ 类型长度（VARCHAR(255) → 255）
- ✅ 是否可空（NULL / NOT NULL）
- ✅ 主键约束（PRIMARY KEY）
- ✅ 唯一约束（UNIQUE）
- ✅ 默认值（DEFAULT）
- ✅ 字段位置（ORDINAL POSITION）

#### 数据库函数
| 函数 | 功能 | 文件位置 |
|-----|------|---------|
| `parse_sql_only` | 只解析 SQL，返回 JSON | database.rs:2786 |
| `parse_and_create_from_sql` | 解析并创建表和字段 | database.rs:2854 |

### 4. 前端 UI 实现（已完成）

#### TablesView.vue 新增内容

**AI 建表对话框：**
- 4 步向导流程
- 步骤指示器显示当前进度
- 每个步骤的上一步/下一步按钮
- 加载状态和错误处理

**步骤 1：输入描述**
- SQL 类型选择（MySQL/PostgreSQL/SQLite）
- 多行文本输入框用于表描述
- 生成 SQL 按钮

**步骤 2：SQL 预览**
- 只读文本框显示 AI 生成的 SQL
- 支持手动编辑
- 上一步/下一步按钮

**步骤 3：字段预览**
- 表格显示解析后的字段信息
- 列：字段名、类型、长度、可空、主键、唯一、默认值、位置
- 支持多表预览（Tab 切换）

**步骤 4：执行结果**
- 成功/失败状态提示
- 错误信息显示
- AI 修复按钮

**按钮位置：**
- 页面顶部工具栏，"导入表结构" 按钮旁边
- 图标：`<RobotOutlined />`
- 文本："AI 建表"

## 数据流

### 生成 SQL 流程
```
用户输入描述
    ↓
前端：generateAISQL()
    ↓
invoke('ai_generate_sql', { provider, model, prompt })
    ↓
后端：ai_generate_sql()
    ↓
获取提供商配置（API 密钥、端点）
    ↓
HTTP POST 到 AI API
    ↓
返回生成的 SQL
    ↓
前端显示在步骤 2
```

### 解析 SQL 流程
```
用户点击"下一步"
    ↓
前端：parseAISQL()
    ↓
invoke('parse_ai_sql', { projectId, sql, dialect })
    ↓
后端：parse_sql_only()
    ↓
根据方言选择 Parser（MySQL/PostgreSQL/SQLite）
    ↓
解析 CREATE TABLE 语句
    ↓
提取表和字段信息
    ↓
返回 JSON 格式
    ↓
前端显示在步骤 3（字段预览表格）
```

### 执行 SQL 流程
```
用户点击"完成"
    ↓
前端：executeAISQL()
    ↓
invoke('execute_ai_sql', { projectId, sql, dialect })
    ↓
后端：parse_and_create_from_sql()
    ↓
解析 SQL（同上）
    ↓
循环每个 CREATE TABLE：
    - 在 db_tables 表中插入表记录
    - 循环每个字段：
      - 在 db_columns 表中插入字段记录
    ↓
返回执行结果
    ↓
前端显示在步骤 4（成功/失败）
```

### AI 修复流程
```
执行失败，显示错误
    ↓
用户点击"AI 修复"
    ↓
前端：fixAISQL()
    ↓
invoke('ai_fix_sql', { provider, model, sql, error, dialect })
    ↓
后端：ai_fix_sql()
    ↓
构建修复提示词（包含原 SQL 和错误信息）
    ↓
HTTP POST 到 AI API
    ↓
返回修复后的 SQL
    ↓
前端更新 SQL 文本框
    ↓
自动调用 parseAISQL() 重新解析
    ↓
跳转到步骤 3
```

## 数据库表结构

### ai_providers
```sql
CREATE TABLE ai_providers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider_name TEXT UNIQUE NOT NULL,
    display_name TEXT NOT NULL,
    provider_type TEXT NOT NULL,
    api_key TEXT,
    api_endpoint TEXT,
    is_enabled INTEGER DEFAULT 0,
    is_default INTEGER DEFAULT 0,
    temperature REAL,
    max_tokens INTEGER,
    timeout_seconds INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### ai_models
```sql
CREATE TABLE ai_models (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    model_id TEXT NOT NULL,
    model_name TEXT NOT NULL,
    provider_name TEXT NOT NULL,
    group_id TEXT DEFAULT 'chat',
    description TEXT,
    max_tokens INTEGER,
    supports_functions INTEGER DEFAULT 0,
    supports_vision INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (provider_name) REFERENCES ai_providers(provider_name)
);
```

## 技术栈

### 前端
- **框架：** Vue 3 (Composition API)
- **UI 库：** Ant Design Vue
- **状态管理：** Pinia
- **构建工具：** Vite

### 后端
- **框架：** Tauri 2.0
- **语言：** Rust
- **异步运行时：** Tokio
- **数据库：** SQLite (sqlx)
- **HTTP 客户端：** reqwest
- **SQL 解析：** sqlparser-rs

## 文件清单

### 新增文件
| 文件路径 | 说明 | 行数 |
|---------|------|-----|
| `apps/codegen-desktop/docs/ai-table-creation-feature.md` | 功能使用指南 | ~400 |
| `apps/codegen-desktop/docs/ai-table-creation-checklist.md` | 测试清单 | ~350 |
| `apps/codegen-desktop/docs/ai-table-creation-implementation.md` | 技术实现总结（本文件） | ~600 |

### 修改文件
| 文件路径 | 修改类型 | 主要改动 |
|---------|---------|---------|
| `src-tauri/src/lib.rs` | 新增命令 | 新增 4 个 AI SQL 命令 |
| `src-tauri/src/database.rs` | 新增函数 | 新增 `parse_sql_only` 函数 |
| `src/components/settings/DefaultAIService.vue` | 新建组件 | 默认服务选择界面 |
| `src/components/settings/SettingsSubSidebar.vue` | 修改 | 添加"默认服务"菜单项 |
| `src/stores/ai-config.js` | 新建 Store | AI 配置状态管理 |
| `src/views/project/TablesView.vue` | 修改 | 新增 AI 建表对话框和逻辑 |

### 代码统计
- **后端新增代码：** ~350 行（Rust）
- **前端新增代码：** ~500 行（Vue/JavaScript）
- **文档：** ~1350 行（Markdown）
- **总计：** ~2200 行

## 编译和运行

### 编译
```bash
cd apps/codegen-desktop/src-tauri
cargo check
cargo build
```

### 运行开发模式
```bash
cd apps/codegen-desktop
npm run tauri dev
```

### 构建发布版本
```bash
npm run tauri build
```

## 测试方法

1. **配置 AI 服务**
   - 打开设置 → AI 服务
   - 配置 DeepSeek/GLM/LongCat 的 API 密钥
   - 设置默认服务

2. **创建项目**
   - 新建项目，关联数据源
   - 进入表管理页面

3. **测试 AI 建表**
   - 点击"AI 建表"按钮
   - 输入表描述
   - 生成 SQL
   - 预览字段
   - 执行创建

详细测试步骤见：`ai-table-creation-checklist.md`

## 已知限制

1. **AI API 调用限制**
   - 依赖第三方 AI 服务，需要网络连接
   - API 调用可能失败或超时
   - 生成的 SQL 质量取决于 AI 模型

2. **SQL 解析限制**
   - 不支持所有 SQL 语法（如触发器、存储过程）
   - 复杂的外键约束可能解析不完整
   - 某些数据库特定语法可能不支持

3. **数据库执行限制**
   - 需要数据源连接正常
   - 表已存在时会报错
   - 权限不足时无法创建表

## 未来改进计划

### 短期（1-2 周）
- [ ] 添加 SQL 语法高亮
- [ ] 支持保存常用表描述模板
- [ ] 添加历史记录功能
- [ ] 优化 AI 提示词，提高生成质量

### 中期（1-2 月）
- [ ] 支持外键关系可视化
- [ ] 支持从现有表反向生成描述
- [ ] 支持 SQL 优化建议
- [ ] 支持批量表编辑

### 长期（3-6 月）
- [ ] 支持更多数据库类型（Oracle、SQL Server）
- [ ] 支持生成完整的 ER 图
- [ ] 支持表关系的自动推断
- [ ] 支持 AI 生成查询语句

## 相关文档

- **使用指南：** `ai-table-creation-feature.md`
- **测试清单：** `ai-table-creation-checklist.md`
- **AI 服务设置设计：** `ai-models-settings-design.md`

## 贡献者

- **开发：** Claude Code
- **设计：** 用户需求驱动
- **测试：** 待测试

## 许可证

与项目主体一致

---

**最后更新：** 2025-02-09
**版本：** 1.0.0
