# Template Studio

> 一个强大的模板管理和代码生成平台，支持模板创建、编辑、渲染和版本管理。

<div align="center">

**[English](README_EN.md)** | **简体中文**

![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)
![Vue](https://img.shields.io/badge/Vue-3.5+-green?logo=vue.js)
![License](https://img.shields.io/badge/License-MIT-blue)
![Status](https://img.shields.io/badge/Status-Early%20Development-yellow)

[功能特性](#功能特性) • [快速开始](#快速开始) • [架构设计](#架构设计) • [开发指南](#开发指南)

</div>

---

## ⚠️ 开发状态

**本项目目前处于早期开发版本，功能仍在积极开发中。**

- ✅ 核心功能已实现
- 🚧 部分功能仍在开发中
- 🐛 可能存在已知或未知的 Bug
- 📝 文档正在完善中

欢迎参与贡献和提出建议！

---

## 功能特性

### 🎯 核心能力

- **📦 模板管理** - 支持模板的创建、编辑、分类和版本管理
- **🚀 代码生成** - 基于模板快速生成项目脚手架
- **🔧 模板引擎** - 强大的 MiniJinja/Tera 模板引擎，支持变量替换和条件渲染
- **📊 依赖分析** - 自动分析模板文件依赖关系，优化渲染顺序
- **🌐 Web 管理界面** - 基于 Naive UI 的现代化管理后台
- **💻 CLI 工具** - 支持命令行和 TUI 两种交互模式
- **🖥️ 桌面应用** - 跨平台的脚手架生成器和代码生成器

### 🏗️ 技术架构

**后端技术栈：**
- Axum - 高性能异步 Web 框架
- SQLx - 类型安全的数据库操作
- Tokio - 异步运行时
- MiniJinja/Tera - 模板引擎
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
git clone https://github.com/your-org/template-studio.git
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

#### 5. 使用 CLI 工具

```bash
# 创建新项目（交互式 TUI 模式）
cargo run -p template-studio-cli -- create

# 创建新项目（命令行模式）
cargo run -p template-studio-cli -- create my-project --template react-ts

# 列出所有可用模板
cargo run -p template-studio-cli -- list
```

### Docker 部署

```bash
# 构建并启动所有服务
docker-compose up -d

# 查看日志
docker-compose logs -f

# 停止服务
docker-compose down
```

---

## 项目架构

```
template-studio/
├── apps/                      # 应用层
│   ├── web/                   # Web 后端服务器
│   ├── cli/                   # CLI 工具
│   ├── scaffold-desktop/      # 脚手架桌面应用
│   └── codegen-desktop/       # 代码生成桌面应用
├── crates/                    # 核心库
│   ├── template_core/         # 模板引擎核心
│   ├── infrastructure/        # 基础设施层
│   ├── repositories/          # 数据访问层
│   ├── services/              # 业务逻辑层
│   └── shared/                # 共享类型和工具
├── web/                       # Web 前端
│   ├── src/
│   │   ├── api/              # API 服务层
│   │   ├── components/       # 可复用组件
│   │   ├── views/            # 页面组件
│   │   └── router/           # 路由配置
│   └── package.json
├── migrations/                # 数据库迁移文件
├── config/                    # 配置文件
├── data/                      # 数据目录
│   ├── templates/            # 模板文件存储
│   └── releases/            # 版本发布数据
└── Cargo.toml                # Rust workspace 配置
```

### 分层架构

```
┌─────────────────────────────────────────┐
│          前端层 (Vue 3 + Naive UI)        │
├─────────────────────────────────────────┤
│          应用层 (Axum HTTP Handlers)     │
├─────────────────────────────────────────┤
│       业务逻辑层 (Services)              │
│  - TemplateService                       │
│  - CategoryService                       │
│  - RenderService                         │
├─────────────────────────────────────────┤
│       数据访问层 (Repositories)          │
│  - TemplateRepository                    │
│  - CategoryRepository                    │
├─────────────────────────────────────────┤
│    基础设施层 (Infrastructure)           │
│  - Database Pool                         │
│  - Storage Manager                       │
│  - Git Service                           │
└─────────────────────────────────────────┘
```

---

## 使用方法

### Web 界面

访问 `http://localhost:3000` 使用 Web 管理界面：

1. **模板管理** - 创建、编辑、删除模板
2. **变量配置** - 配置模板变量和预设
3. **版本控制** - 管理模板版本和发布
4. **文件编辑** - 在线编辑模板文件
5. **实时预览** - 实时预览渲染结果

### CLI 工具

```bash
# 交互式创建项目（TUI 模式）
cargo run -p template-studio-cli -- create

# 使用指定模板创建项目
cargo run -p template-studio-cli -- create my-app --template vue3-ts

# 查看模板详情
cargo run -p template-studio-cli -- info vue3-ts

# 更新本地模板缓存
cargo run -p template-studio-cli -- update
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

### 前端配置 (web/.env)

```bash
VITE_API_BASE_URL=http://localhost:8080
VITE_APP_TITLE=Template Studio
```

---

## 开发指南

### Rust 后端开发

```bash
# 开发模式运行（自动重新编译）
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

# 类型检查
pnpm run type-check
```

### 桌面应用开发

```bash
cd apps/scaffold-desktop

# 开发模式
pnpm run tauri:dev

# 构建生产版本
pnpm run tauri:build
```

---

## 路线图

### v0.1.0 (当前版本)
- ✅ 基础模板管理功能
- ✅ Web 管理界面
- ✅ CLI 工具
- ✅ 模板渲染引擎
- 🚧 桌面应用完善中

### v0.2.0 (计划中)
- 🔲 模板市场
- 🔲 用户权限系统
- 🔲 模板分享和协作
- 🔲 更多内置模板

### v0.3.0 (未来)
- 🔲 插件系统
- 🔲 云端同步
- 🔲 AI 辅助模板生成
- 🔲 多语言支持

---

## 常见问题

### Q: 数据库迁移失败怎么办？

A: 检查数据库连接配置，确保数据库服务正在运行，并执行：
```bash
# 重新运行迁移
cargo run -p template-studio-web
```

### Q: 前端无法连接后端？

A: 检查 `web/.env` 中的 `VITE_API_BASE_URL` 是否正确指向后端服务地址。

### Q: 如何添加自定义模板？

A: 在 Web 界面中创建模板，或通过 API 上传模板文件到 `data/templates/` 目录。

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

- 📖 [文档](./docs/) - 详细的使用文档
- 🐛 [Issue Tracker](https://github.com/your-org/template-studio/issues) - Bug 报告和功能建议
- 💬 [Discussions](https://github.com/your-org/template-studio/discussions) - 讨论和问答

---

## 许可证

本项目采用 [MIT](LICENSE) 许可证。

---

## 致谢

- [Naive UI Admin](https://github.com/jekip/naive-ui-admin) - 优秀的前端管理模板
- [Axum](https://github.com/tokio-rs/axum) - 强大的 Rust Web 框架
- [Tauri](https://tauri.app/) - 现代化的桌面应用开发框架
- 所有贡献者 ❤️

---

<div align="center">

**⭐ 如果这个项目对你有帮助，请给一个 Star！**

Made with ❤️ by Template Studio Team

</div>
