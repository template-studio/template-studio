# Template Studio Desktop 功能分析报告

> 更新日期：2026-06-24
> 分析范围：`apps/desktop/` — 144 前端文件 (27,006 行) + 28 Rust 文件 (8,205 行)

---

## 一、技术架构概览

| 层级 | 技术栈 |
|------|--------|
| 桌面壳 | Tauri 2.x |
| 前端框架 | Vue 3 + Pinia + Vue Router (hash) |
| UI 库 | Ant Design Vue 4.x (zhCN locale) |
| HTTP 客户端 | Axios（模板 API）+ Tauri IPC `invoke()`（本地数据） |
| 后端语言 | Rust (sqlx + SQLite) |
| 主题系统 | CSS 自定义属性 + `darkAlgorithm` + 系统检测 |

### 混合数据架构

- **本地数据**（项目、数据源、语言、映射、AI 配置）→ Tauri IPC → Rust/SQLite
- **远程数据**（模板、版本、变量、文件树）→ Axios → Web Server (127.0.0.1:8080)

---

## 二、路由与页面 (16 路由)

| 路由 | 页面 | 布局 |
|------|------|------|
| `/home` | 首页仪表盘 | AppLayout |
| `/templates` | 模板库（脚手架） | AppLayout |
| `/template-render` | 模板渲染 | AppLayout |
| `/datasource` | 数据源管理 | AppLayout |
| `/datasource/:id/browse` | 数据库浏览器 | AppLayout |
| `/projects` | 项目列表 | AppLayout |
| `/mappings` | 全局类型映射 | AppLayout |
| `/languages` | 编程语言管理 | AppLayout |
| `/settings/:main?/:sub?/:third?` | 三级设置 | AppLayout |
| `/help` | 帮助文档 | AppLayout |
| `/project/:id` | 项目工作台仪表盘 | ProjectWorkspaceLayout |
| `/project/:id/tables` | 表管理 | ProjectWorkspaceLayout |
| `/project/:id/preferences` | 表规范配置 | ProjectWorkspaceLayout |
| `/project/:id/mappings` | 项目类型映射 | ProjectWorkspaceLayout |
| `/:pathMatch(.*)*` | 404 页面 | AppLayout |

双布局系统：`AppLayout`（全局页面）和 `ProjectWorkspaceLayout`（项目内页面，含侧边栏导航）。

---

## 三、Tauri 命令总览 (99 个)

| 模块 | 命令数 | 职责 |
|------|--------|------|
| window | 7 | 窗口控制、文件写入、系统信息 |
| template | 10 | 模板列表、渲染、导出、下载 |
| settings | 3 | 配置读写 |
| project | 7 | 项目 CRUD、统计、最近项目 |
| datasource | 6 | 数据源 CRUD、连接测试 |
| sync | 10 | 远程数据库浏览、表导入、SQL 执行、推送同步 |
| table | 13 | 表/列 CRUD、SQL 导入、偏好设置 |
| language | 14 | 语言 CRUD、项目语言关联、字段类型管理 |
| type_mapping | 13 | 系统/项目级类型映射 CRUD、批量保存、复制 |
| ai | 16 | AI 提供商/模型管理、SQL 生成/修复、连接测试 |

---

## 四、数据库层 (SQLite)

### 模块结构

```
database/
  mod.rs          → Database 结构体 + 初始化
  models.rs       → 9 个共享模型结构体
  project.rs      → 11 方法（项目 CRUD + 统计 + 语言关联）
  datasource.rs   → 6 方法（数据源 CRUD + 连接测试）
  table.rs        → 4 方法（表 CRUD）
  column.rs       → 5 方法（列 CRUD + 排序）
  language.rs     → 10 方法（语言 CRUD + 字段类型）
  type_mapping.rs → 13 方法（类型映射两层继承体系）
  preferences.rs  → 2 方法（表规范配置）
  ai.rs           → 10 方法（AI 提供商/模型）
  import.rs       → 7 公开函数（远程数据库导入 + SQL 解析）
  migrations.rs   → 1 个入口 + 12 个迁移方法
```

**总计：61 个公开方法/函数 + 18 个内部辅助函数**

### 数据表 (12 张)

| 表名 | 用途 |
|------|------|
| `projects` | 项目（关联数据源、主语言、前端/后端语言） |
| `datasources` | 数据源连接配置（MySQL/PG/SQLite） |
| `db_tables` | 项目下的表（FK → projects, CASCADE） |
| `db_columns` | 表下的列（FK → db_tables, CASCADE） |
| `languages` | 13 个内置语言 + 用户自定义 |
| `project_languages` | 项目-语言多对多关联 |
| `language_field_types` | 语言字段类型定义 |
| `system_type_mappings` | 系统级类型映射模板 |
| `project_type_mappings` | 项目级类型映射覆盖（scoped: frontend/backend） |
| `table_preferences` | 项目表规范（PK、审计字段、软删除、命名、存储） |
| `ai_providers` | AI 提供商配置 |
| `ai_models` | AI 模型注册表 |
| `schema_migrations` | 迁移版本跟踪 |

---

## 五、前端状态管理 (6 Store)

| Store | 职责 |
|-------|------|
| `layout` | 侧边栏状态、响应式断点、全局 Footer（分页/概览） |
| `theme` | 亮/暗主题切换、系统主题检测、localStorage 持久化 |
| `config` | API URL、API Key、模板路径（从 Rust 后端加载） |
| `navigation` | 导航历史栈、后退支持 |
| `notification` | 通知中心（内存，最多 100 条，未读计数） |
| `ai-config` | AI 提供商/模型管理（16 个 Tauri 命令） |

---

## 六、功能完成度矩阵

### 核心功能模块

| 模块 | 完成度 | 关键能力 |
|------|--------|---------|
| 项目管理 | 100% | CRUD、仪表盘、统计卡片、快速操作、最近表列表 |
| 数据源管理 | 100% | CRUD、连接测试、数据库浏览器、连接状态监控 |
| 表管理 | 100% | CRUD、列管理（拖拽排序）、SQL 导入、AI 建表、批量操作、DDL 导出 |
| 表结构同步 | 100% | 双视图（总览/列对比）、双向同步（远程↔本地）、DDL 生成在 Rust 含单测 |
| 语言管理 | 100% | CRUD、字段类型管理、项目语言关联、13 个内置语言 |
| 类型映射 | 100% | 两层继承（系统→项目）、前后端分域、导入/导出、5 套预置模板 |
| 表规范配置 | 100% | PK、审计字段、软删除、命名规范、存储配置 |
| 模板库 | 100% | 网格展示、分类/语言筛选、搜索排序、推荐标记 |
| 模板渲染 | 100% | 3 步向导（详情→变量→预览导出）、变量表单、JSON 编辑器、文件树预览 |
| 数据库浏览器 | 100% | 树形导航、数据/列双视图、分页、连接池缓存 |
| AI 集成 | 100% | 多提供商（DeepSeek/GLM/MiMo/OpenAI）、SQL 生成/修复、模型管理 |
| 全局搜索 | 100% | Ctrl+K 命令面板、分类搜索（页面/项目/数据源/模板）、键盘导航 |
| 通知中心 | 100% | Navbar 铃铛 + Popover、未读徽章、操作历史、notify() 双写 |
| 设置系统 | 100% | 三级导航、常规/显示/快捷键/备份/Web 服务器/AI/关于 |
| 帮助文档 | 100% | Wiki 布局（折叠目录 + 内容区）、快速开始、功能指南、FAQ、快捷键 |
| 暗黑模式 | 100% | darkAlgorithm + CSS 变量同步 + 系统检测 + 全组件适配 |

### 页面级功能

| 页面 | 搜索 | 筛选 | 排序 | 分页 | 其他特性 |
|------|------|------|------|------|---------|
| 首页 | - | - | - | - | 统计卡片、最近项目 |
| 模板库 | ✅ | ✅ 分类+语言 | ✅ | ✅ | 推荐优先、代码预览头 |
| 项目列表 | ✅ | ✅ | ✅ | ✅ | - |
| 数据源 | ✅ | ✅ | ✅ | ✅ | 连接测试、状态监控、浏览器 |
| 语言管理 | ✅ | ✅ | ✅ | ✅ | 字段类型管理、emoji 建议 |
| 全局映射 | ✅ | - | - | ✅ | 导入/导出、5 套模板 |
| 表管理 | ✅ | ✅ 引擎 | ✅ | ✅ | SQL 导入、AI 建表、DDL 导出、列拖拽 |
| 数据库浏览器 | ✅ 表搜索 | - | - | ✅ | 树形导航、数据/列双视图 |

---

## 七、组件架构

### 公共组件 (13 个)

| 组件 | 路径 | 用途 |
|------|------|------|
| GlobalSearch | `components/common/` | Ctrl+K 命令面板 |
| NotificationCenter | `components/common/` | 通知中心弹窗 |
| SearchBar | `components/common/` | 统一搜索栏（搜索+筛选+排序） |
| Pagination | `components/common/` | 统一分页 |
| EmptyState | `components/common/` | 空状态展示 |
| ConfirmDialog | `components/common/` | 删除确认对话框 |
| AppLayout | `components/layout/` | 全局应用布局 |
| Navbar | `components/layout/` | 顶部导航栏（标题栏拖拽区） |
| Sidebar | `components/layout/` | 侧边栏导航 |
| MainContent | `components/layout/` | 主内容区 + 全局 Footer |
| ProjectWorkspaceLayout | `components/layout/` | 项目工作区布局（Header+Footer 已拆分） |
| ThemeToggle | `components/theme/` | 主题切换按钮 |
| TemplateConfig | `components/` | 模板配置（FilterSection+TemplateCard 已拆分） |

### 私有子组件 (35 个)

本次重构将 13 个大文件拆分为父组件 + 同级 `components/` 下的私有子组件：

| 父组件 | 原行数 | 现行数 | 子组件 |
|--------|--------|--------|--------|
| TemplateRenderDrawer | 1427 | 752 | TemplateDetailPanel, VariableConfigPanel, RenderPreviewPanel, ExportDialog |
| TemplateWizardDrawer | 520 | 381 | StepTemplateIntro, StepPathConfig, StepVariables, StepPreview |
| TablePreferencesManager | 645 | 348 | PrimaryKeyConfig, AuditFieldsConfig, SoftDeleteConfig, NamingConventionConfig, StorageConfig |
| ModelProviderConfig | 667 | 417 | ModelGroupList, ModelEditDialog |
| SchemaDiffDrawer | 639 | 480 | DiffOverviewView, DiffDetailView |
| Mappings/index | 498 | 398 | MappingsTable, AddMappingDialog |
| DatabaseBrowser/index | 922 | 337 | DatabaseToolbar, DatabaseTreePanel, TableContentView |
| ProjectWorkspaceLayout | 989 | 662 | WorkspaceHeader, WorkspaceFooter |
| project/index | 532 | 114 | ProjectInfoCard, StatsGrid, QuickActionsGrid, RecentTablesList |
| home/index | 503 | 215 | StatsSection, RecentProjectsList |
| languages/index | 500 | 349 | LanguageCard |
| TemplateConfig | 587 | 185 | TemplateCard, FilterSection |
| AboutSettings | 518 | 426 | TechStackGrid, SystemInfoGrid |

---

## 八、API 模块清单 (9 个)

### Tauri IPC (5 个)

| 模块 | 方法数 | 覆盖功能 |
|------|--------|---------|
| datasources.js | 8 | CRUD + 测试 + 表/列/数据查询 + 连接状态 |
| projects.js | 12 | CRUD + 表/列 CRUD + 排序 + SQL 导入 + 数据源导入 |
| languages.js | 11 | CRUD + 项目关联 + 字段类型 |
| statistics.js | 2 | 统计 + 最近项目 |
| tableConfig.js | 9 | 配置 CRUD + 代码预览/生成 + 批量生成 |

### HTTP/Axios (4 个)

| 模块 | 方法数 | 端点前缀 |
|------|--------|---------|
| templates.js | 6 | `/api/v1/studio/` |
| releases.js | 1 | `/api/v1/template/` |
| templateFiles.js | 2 | `/api/v1/template-files/` |
| templateVariables.js | 1 | `/api/v1/template-files/` |

---

## 九、Rust 后端架构

### 模块结构

```
src-tauri/src/
  lib.rs          → 入口 + 99 个命令注册
  config.rs       → YAML 配置（自动创建）
  state.rs        → DbState (Arc) + BrowserPoolCache
  ddl.rs          → DDL 生成 + 5 个单测
  commands/       → 10 个命令模块（薄包装层）
  database/       → 12 个数据库模块（61 方法 + 12 迁移）
```

### 关键架构模式

1. **Split impl** — `Database` 的方法分散在 8 个文件中，按领域划分
2. **薄命令层** — Tauri 命令只做参数提取 + 状态注入 + 序列化，业务逻辑在 Database 层
3. **两层类型映射继承** — 系统级默认 → 项目级覆盖（scoped: frontend/backend）
4. **连接池缓存** — `BrowserPoolCache` 按 URL 缓存 MySQL/PG/SQLite 连接池
5. **自定义迁移系统** — 12 个顺序迁移 + `schema_migrations` 版本表
6. **SQL 解析** — `sqlparser` crate 支持 MySQL/PG/SQLite DDL 解析
7. **AI 统一接口** — OpenAI 兼容 `/chat/completions`，支持 5 个提供商

---

## 十、Composables (3 个)

| Composable | 用途 |
|------------|------|
| `useLayout` | 响应式断点检测、防抖 resize、侧边栏切换、内容区尺寸计算 |
| `useTheme` | 主题颜色工具、CSS 变量操作、系统主题自动检测 |
| `useSettingsNavigation` | 三级设置标签导航、URL 同步 |

---

## 十一、已知问题与技术债务

### 功能缺失

| 项目 | 优先级 | 说明 |
|------|--------|------|
| 项目导出 | P2 | 无项目配置导出功能 |
| 数据源配置导入/导出 | P2 | 无批量数据源配置迁移 |
| 撤销/重做 | P3 | 无操作撤销机制 |
| Navbar 搜索框 | P2 | 中间搜索框有 `TODO` 注释，未接入功能 |

### 代码质量

| 项目 | 说明 |
|------|------|
| `navigation.js` | `canGoForward` 始终返回 `false`，前进功能未实现 |
| 硬编码颜色 | 少量页面仍有硬编码色值（大部分已通过 CSS 变量修复） |
| 错误处理 | 无全局错误边界、无网络断开检测、无自动重试 |
| 状态管理 | 项目/数据源/模板列表未全局管理，各页面独立加载 |

### 性能

| 项目 | 状态 |
|------|------|
| 列表分页 | ✅ 已通过全局 Footer 分页解决 |
| 连接池 | ✅ 已通过 BrowserPoolCache 解决 |
| 大表行数 | ✅ 已通过快速估算（TABLE_ROWS/reltuples）解决 |
| 模板加载 | ✅ 已分页显示 |
| Schema 对比 | ✅ 并行获取 + 类型标准化 |

---

## 十二、Web 后端未对接功能

以下 handler 存在于 `apps/web/` 但桌面端未对接（均为 Web 管理端功能，非桌面端职责）：

| Handler | 功能 | 桌面端必要性 |
|---------|------|-------------|
| user_management | 用户 CRUD | 无（桌面端单用户） |
| role_management | 角色管理 | 无 |
| permission_management | 权限管理 | 无 |
| email | 邮件配置 | 无 |
| system_setting | 系统设置 | 低（桌面端有自己的设置） |
| var_preset | 变量预设 | 无（变量每次渲染不同） |
| preset_subscribe | 预设订阅 | 无 |
| file_conditions | 文件条件 | 无（模板引擎内部处理） |
| editor | 模板编辑器 | 无（桌面端消费模板，不创作） |
| template_analysis | 模板分析 | 无 |
| engine | 引擎管理 | 无 |
| builtin | 内置模板管理 | 无 |
| category | 分类管理 | 无 |

---

## 十三、总结

**桌面端核心功能 100% 完成。** 99 个 Tauri 命令覆盖所有业务领域，61 个数据库方法支撑完整的 CRUD 和业务逻辑，16 路由覆盖所有页面，35 个私有子组件保证代码可维护性。

**剩余工作均为 P2/P3 级体验优化**：项目导出、撤销/重做、Navbar 搜索框接入、全局错误边界。无阻断性缺失。
