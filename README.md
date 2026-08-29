# Template Studio

> 一个强大的模板管理和代码生成平台，支持模板创建、编辑、渲染和版本管理。

---

## 功能特性

### 🎯 核心能力

- **📦 模板管理** - 支持模板的创建、编辑、分类、版本管理和发布审核流程
- **🚀 代码生成** - 基于模板快速生成项目脚手架，支持模板生成器在线使用
- **🔧 模板引擎** - 基于 MiniJinja 的模板引擎，支持变量替换、条件渲染、自定义 filter 和 builtin 函数
- **📊 依赖分析** - 自动分析模板文件依赖关系，优化渲染顺序
- **🔐 RBAC 权限系统** - 基于角色的访问控制，支持 JWT 认证和个人访问令牌 (PAT)
- **🤖 AI 辅助** - 集成 AI 服务，支持变量分析、智能填充、项目转换、SQL 生成
- **🗃️ 数据库同步** - 支持远程 MySQL/PostgreSQL/SQLite 数据库浏览、表结构导入和同步
- **🔀 类型映射** - 系统级和项目级的数据库类型到编程语言类型映射
- **🌐 Web 管理界面** - 基于 Naive UI 的现代化管理后台 + 面向用户的前台界面
- **💻 CLI 工具** - 支持命令行和 TUI 两种交互模式，集成 AI 命令
- **🖥️ 桌面应用** - 基于 Tauri 2.x 的跨平台桌面应用，支持离线使用

### 🏗️ 技术架构

**后端技术栈：**

- Axum - 高性能异步 Web 框架
- SQLx - 类型安全的数据库操作（支持 MySQL / SQLite / PostgreSQL）
- Tokio - 异步运行时
- MiniJinja - 模板引擎（支持编译到 WASM 用于浏览器端渲染）
- Git2 - Git 版本控制集成

**前端技术栈：**

- Vue 3 + Composition API
- Naive UI - 企业级 UI 组件库
- Pinia - 状态管理
- Alova - 带缓存的 HTTP 客户端
- CodeMirror 6 - 代码编辑器

**桌面应用：**

- Tauri 2.x - 轻量级桌面应用框架
- Ant Design Vue - UI 组件库
- CodeMirror 6 - 多语言代码编辑器（SQL/JS/Go/Rust/Python 等）

---

## 快速开始

### 环境要求

- Rust 1.70+
- Node.js 16+
- MySQL 5.7+ / SQLite 3.x / PostgreSQL 12+
- pnpm 7+

### 安装与运行

#### 1. 克隆项目

```bash
git clone https://github.com/template-studio/template-studio.git
cd template-studio
```

#### 2. 配置数据库

复制配置文件并修改数据库连接：

```bash
cp config/config.toml.example config/config.toml
# 编辑 config/config.toml，修改数据库连接信息
```

#### 3. 启动后端服务

```bash
cargo run -p template-studio-web
```

服务将在 `http://localhost:8080` 启动

#### 4. 启动前端界面

```bash
cd web
pnpm install
pnpm run dev
```

前端将在 `http://localhost:3000` 启动

#### 5. 启动桌面应用

```bash
cd apps/desktop
pnpm install
pnpm run tauri:dev
```

#### 6. 使用 CLI 工具

```bash
# 创建新项目（交互式 TUI 模式）
cargo run -p template-studio-cli -- create

# 创建新项目（命令行模式）
cargo run -p template-studio-cli -- create my-project --template react-ts

# 列出所有可用模板
cargo run -p template-studio-cli -- template list

# AI 变量分析
cargo run -p template-studio-cli -- ai analyze-variables ./my-template

# AI 变量填充
cargo run -p template-studio-cli -- ai fill-variables ./my-template --project 1

# 项目转模板
cargo run -p template-studio-cli -- ai convert-to-template ./my-project --output ./templates

# 渲染预览
cargo run -p template-studio-cli -- ai render-preview ./my-template

# 模板验证
cargo run -p template-studio-cli -- ai validate ./my-template
```

---

## 项目架构

```
template-studio/
├── apps/                          # 应用层
│   ├── web/                       # Axum Web 后端服务器（24 个 handler 模块）
│   ├── cli/                       # CLI 工具（支持 TUI + AI 命令）
│   └── desktop/                   # Tauri 桌面应用
│       ├── src/                   # Vue 3 前端（15+ 页面）
│       └── src-tauri/             # Tauri Rust 后端（80+ 命令）
├── crates/                        # 核心 Rust 库（7 个 crate）
│   ├── shared/                    # 共享类型、模型、工具
│   ├── infrastructure/            # 基础设施层（数据库池、配置、Git、日志）
│   ├── repositories/              # 数据访问层（12 个仓库模块）
│   ├── services/                  # 业务逻辑层（20 个服务模块）
│   ├── template_core/             # 模板引擎核心（基于 MiniJinja）
│   ├── template_core_wasm/        # 模板引擎 WASM 绑定（浏览器端渲染）
│   └── ai_agent/                  # AI Agent 模块（变量分析、填充、转换）
├── web/                           # Vue 3 Web 前端（Naive UI）
│   └── src/
│       ├── api/                   # API 服务层（17 个模块）
│       ├── components/            # 可复用组件
│       ├── views/                 # 页面组件
│       │   ├── admin/             # 后台管理（仪表盘、模板、分类、语言、变量预设、设置）
│       │   ├── client/            # 前台界面（首页、模板广场、个人中心、模板生成器）
│       │   └── editor/            # 模板编辑器
│       ├── store/                 # Pinia 状态管理
│       └── router/                # 路由配置（admin/client/editor 模块）
├── migrations/                    # SQL 数据库迁移文件（19 个迁移）
├── config/                        # 配置文件
├── data/                          # 运行时数据（模板、版本、头像）
├── docs/                          # 项目文档（7 个文档）
└── Cargo.toml                     # Rust workspace 配置（10 个成员 crate）
```

### 分层架构

```
┌─────────────────────────────────────────────────────────┐
│          前端层 (Vue 3 + Naive UI / Ant Design Vue)      │
│    Web 管理后台 + 前台界面 + Tauri 桌面应用               │
├─────────────────────────────────────────────────────────┤
│          应用层 (Axum HTTP Handlers / Tauri Commands)     │
│    24 个 Web Handler + 80+ 个 Tauri 命令                  │
├─────────────────────────────────────────────────────────┤
│          业务逻辑层 (Services)                            │
│    TemplateService / AuthService / RBACService            │
│    TemplateRenderService / TemplateAnalysisService        │
│    FileConditionsService / ReleaseService / BackupService │
│    CategoryService / LanguageService / VarPresetService   │
├─────────────────────────────────────────────────────────┤
│          数据访问层 (Repositories)                        │
│    TemplateRepository / UserRepository / RoleRepository   │
│    CategoryRepository / PermissionRepository / PatRepo   │
│    LanguageRepository / VarPresetRepository / ...         │
├─────────────────────────────────────────────────────────┤
│          基础设施层 (Infrastructure)                      │
│    Database Pool (MySQL/SQLite/PG) / Storage Manager      │
│    Git Service / Logging / File Tree                      │
├─────────────────────────────────────────────────────────┤
│          模板引擎 (template_core)                         │
│    MiniJinja 渲染 / 条件文件生成 / 依赖分析               │
│    WASM 绑定（浏览器端渲染）                              │
├─────────────────────────────────────────────────────────┤
│          AI Agent (ai_agent)                              │
│    变量分析 / 智能填充 / 项目转换 / SQL 生成               │
└─────────────────────────────────────────────────────────┘
```

---

## 使用方法

### Web 界面

访问 `http://localhost:3000` 使用 Web 管理界面：

**前台功能：**

1. **首页** - 平台概览和快速入口
2. **模板广场** - 浏览和搜索公开模板
3. **模板生成器** - 在线选择模板、填写变量、生成项目
4. **个人中心** - 管理个人信息和我的模板
5. **用户主页** - 查看其他用户的公开模板

**后台管理：**

1. **仪表盘** - 数据统计和分析
2. **模板管理** - 创建、编辑、删除、审核模板
3. **分类管理** - 模板分类的增删改查
4. **语言管理** - 编程语言配置
5. **变量预设** - 预设变量的设计和管理
6. **权限管理** - 用户和角色管理
7. **系统设置** - 全局配置

### 桌面应用

桌面应用提供以下功能：

1. **项目管理** - 创建和管理项目
2. **表管理** - 数据库表结构设计，支持列的拖拽排序
3. **数据源管理** - 配置远程数据库连接（MySQL/PostgreSQL/SQLite）
4. **数据库浏览器** - 浏览远程数据库表结构和数据
5. **类型映射** - 系统级和项目级的数据库类型到编程语言类型映射
6. **语言管理** - 管理编程语言和字段类型
7. **模板渲染** - 基于项目数据渲染模板
8. **AI 辅助** - AI 驱动的 SQL 生成和修复
9. **设置** - 通用、显示、快捷键、备份、网络、安全等配置

### CLI 工具

```bash
# 模板管理
cargo run -p template-studio-cli -- template list              # 列出模板
cargo run -p template-studio-cli -- template info <name>       # 查看详情
cargo run -p template-studio-cli -- template search <keyword>  # 搜索模板

# 项目创建
cargo run -p template-studio-cli -- create                     # TUI 模式
cargo run -p template-studio-cli -- create my-app --template vue3-ts

# AI 命令
cargo run -p template-studio-cli -- ai analyze-variables <path>           # 变量分析
cargo run -p template-studio-cli -- ai fill-variables <path> --project 1  # 智能填充
cargo run -p template-studio-cli -- ai convert-to-template <path> -o out  # 项目转模板
cargo run -p template-studio-cli -- ai render-preview <path>              # 渲染预览
cargo run -p template-studio-cli -- ai validate <path>                    # 模板验证
cargo run -p template-studio-cli -- ai recommend                          # 模板推荐
cargo run -p template-studio-cli -- ai edit-file <path> --insert 1 -c "..." # 文件编辑

# 配置管理
cargo run -p template-studio-cli -- config show                # 查看配置
cargo run -p template-studio-cli -- ai config show             # 查看 AI 配置
```

### API 使用

```bash
# 获取模板列表
curl http://localhost:8080/api/v1/template/templateList

# 获取模板详情
curl http://localhost:8080/api/v1/template/templates/detail?id=1

# 渲染模板
curl -X POST http://localhost:8080/api/v1/editor/templateFiles/render \
  -H "Content-Type: application/json" \
  -d '{
    "templateId": 1,
    "variables": {"projectName": "my-project"},
    "fileTree": [...]
  }'

# 认证相关
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "password"}'
```

---

## 配置说明

### 后端配置 (config/config.toml)

```toml
[server]
host = "127.0.0.1"
port = 8080

[database]
url = "mysql://user:password@localhost:3306/template_studio"
max_connections = 10

[storage]
type = "local"
base_path = "./data"

[git]
auto_init = true
default_branch = "main"
```

### 前端配置

**Web 前端 (web/):**

- `web/.env.development` - 开发环境配置
- `web/.env.production` - 生产环境配置

**桌面应用 (apps/desktop/):**

- 配置存储在本地 SQLite 数据库中
- 通过应用内设置界面管理

---

## 开发指南

### Rust 后端开发

```bash
# 开发模式运行
cargo run -p template-studio-web

# 运行测试
cargo test

# 代码检查
cargo clippy

# 格式化代码
cargo fmt
```

### Vue 前端开发

```bash
cd web

# 开发服务器
pnpm run dev

# 代码检查
pnpm run lint:eslint
pnpm run lint:prettier
pnpm run lint:stylelint

# 类型检查
pnpm run type-check
```

### 桌面应用开发

```bash
cd apps/desktop

# 安装依赖
pnpm install

# 开发模式
pnpm run tauri:dev

# 构建生产版本
pnpm run build
```

### CLI 工具开发

```bash
# 运行 CLI
cargo run -p template-studio-cli -- <command>

# 测试 AI 功能
cargo run -p template-studio-cli -- ai analyze-variables ./path/to/template
```

---

## 路线图

### 已实现功能

- ✅ 模板管理（创建、编辑、分类、版本管理）
- ✅ Web 管理后台 + 前台界面
- ✅ CLI 工具（TUI + 命令行模式）
- ✅ 模板渲染引擎（MiniJinja + WASM）
- ✅ RBAC 权限系统 + JWT 认证 + PAT
- ✅ 桌面应用（Tauri 2.x）
- ✅ 模板审核流程
- ✅ 变量预设系统
- ✅ 模板生成器（在线使用）
- ✅ 数据库同步（MySQL/PostgreSQL/SQLite）
- ✅ 类型映射（系统级 + 项目级）
- ✅ AI 辅助（变量分析、填充、转换、SQL 生成）
- ✅ 数据备份与恢复

### 计划中功能

- 🔲 模板市场和分享
- 🔲 模板协作编辑
- 🔲 更多内置模板
- 🔲 插件系统
- 🔲 云端同步

---

## 常见问题

### Q: 数据库迁移失败怎么办？

A: 检查数据库连接配置，确保数据库服务正在运行。Web 后端启动时会自动执行迁移。

### Q: 前端无法连接后端？

A: 检查 `web/.env.development` 和 `web/.env.production` 中的 API 地址配置是否正确。

### Q: 如何添加自定义模板？

A: 可以通过以下方式：

1. Web 界面：后台管理 → 模板管理 → 创建模板
2. CLI 工具：`cargo run -p template-studio-cli -- ai convert-to-template <project-path>`
3. API：上传模板文件到 `data/templates/` 目录

### Q: 桌面应用如何配置 AI 功能？

A: 在桌面应用的设置页面中配置 AI 提供商（支持 DeepSeek、OpenAI 等），设置 API Key 即可使用 AI 功能。

---

## 贡献指南

我们欢迎所有形式的贡献！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feat/amazing-feature`)
3. 提交更改 (`git commit -m 'feat: add amazing feature'`)
4. 推送到分支 (`git push origin feat/amazing-feature`)
5. 创建 Pull Request

请确保：

- 代码通过 `cargo clippy` 和 `cargo test` 检查
- 前端代码通过 `pnpm run lint` 检查
- 提交信息遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范

---

## 技术支持

- 📖 [文档](./docs/) - 项目文档
  - `desktop-ai-architecture-plan.md` - 桌面应用 AI 架构设计
  - `desktop-ai-integration-research.md` - AI 集成调研
  - `desktop-missing-features-analysis.md` - 功能缺失分析
  - `desktop-rust-refactor-plan.md` - Rust 重构计划
  - `rbac-implementation-plan.md` - RBAC 实现方案
  - `template-contribution-plan.md` - 模板投稿系统设计
  - `template-render-design.md` - 模板渲染设计
- 🐛 [Issue Tracker](https://github.com/template-studio/template-studio/issues) - Bug 报告和功能建议
- 💬 [Discussions](https://github.com/template-studio/template-studio/discussions) - 讨论和问答

---

## 许可证

本项目采用 [Apache License 2.0](LICENSE) 许可证。

---

## 致谢

- [Naive UI Admin](https://github.com/jekip/naive-ui-admin) - 优秀的前端管理模板
- [Axum](https://github.com/tokio-rs/axum) - 强大的 Rust Web 框架
- [Tauri](https://tauri.app/) - 现代化的桌面应用开发框架
- [MiniJinja](https://github.com/mitsuhiko/minijinja) - 灵活的模板引擎
- [Ant Design Vue](https://www.antdv.com/) - 企业级 UI 组件库
- 所有贡献者 ❤️

---

<div align="center">

**⭐ 如果这个项目对你有帮助，请给一个 Star！**

Made with ❤️ by Template Studio Team

</div>
