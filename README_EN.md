# Template Studio

> A powerful template management and code generation platform with support for template creation, editing, rendering, and version management.

<div align="center">

**English** | **[简体中文](README.md)**

![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)
![Vue](https://img.shields.io/badge/Vue-3.5+-green?logo=vue.js)
![License](https://img.shields.io/badge/License-MIT-blue)
![Status](https://img.shields.io/badge/Status-Early%20Development-yellow)

[Features](#features) • [Quick Start](#quick-start) • [Architecture](#architecture) • [Development Guide](#development-guide)

</div>

---

## ⚠️ Development Status

**This project is currently in early development and actively evolving.**

- ✅ Core features implemented
- 🚧 Some features still under development
- 🐛 Known and unknown bugs may exist
- 📝 Documentation is being improved

Contributions and suggestions are welcome!

---

## Features

### 🎯 Core Capabilities

- **📦 Template Management** - Create, edit, categorize, and version templates
- **🚀 Code Generation** - Generate project scaffolding based on templates
- **🔧 Template Engine** - Powerful MiniJinja/Tera engine with variable substitution and conditional rendering
- **📊 Dependency Analysis** - Automatically analyze template file dependencies and optimize rendering order
- **🌐 Web Management Interface** - Modern admin panel built with Naive UI
- **💻 CLI Tools** - Both CLI and TUI interaction modes supported
- **🖥️ Desktop Applications** - Cross-platform scaffold generator and code generator

### 🏗️ Technical Architecture

**Backend Stack:**
- Axum - High-performance async web framework
- SQLx - Type-safe database operations
- Tokio - Async runtime
- MiniJinja/Tera - Template engines
- Git2 - Git version control integration

**Frontend Stack:**
- Vue 3 + Composition API
- Naive UI - Enterprise UI component library
- Pinia - State management
- Alova - HTTP client with caching
- CodeMirror 6 - Code editor

**Desktop Apps:**
- Tauri 2.x - Lightweight desktop app framework
- Ant Design Vue - UI component library

---

## Quick Start

### Prerequisites

- Rust 1.70+
- Node.js 16+
- MySQL 5.7+ / SQLite 3.x / PostgreSQL 12+
- pnpm 7+

### Installation & Running

#### 1. Clone the Repository

```bash
git clone https://github.com/template-studio/template-studio.git
cd template-studio
```

#### 2. Configure Database

Copy the configuration file and modify the database connection:

```bash
cp config/config.toml.example config/config.toml
# Edit config/config.toml to update database connection
```

#### 3. Start Backend Service

```bash
cargo run -p template-studio-web
```

Service will start at `http://localhost:8080`

#### 4. Start Frontend Interface

```bash
cd web
pnpm install
pnpm run dev
```

Frontend will start at `http://localhost:3000`

#### 5. Use CLI Tools

```bash
# Create new project (Interactive TUI mode)
cargo run -p template-studio-cli -- create

# Create new project (CLI mode)
cargo run -p template-studio-cli -- create my-project --template react-ts

# List all available templates
cargo run -p template-studio-cli -- list
```

---

## Architecture

```
template-studio/
├── apps/                      # Application layer
│   ├── web/                   # Web backend server
│   ├── cli/                   # CLI tools
│   ├── scaffold-desktop/      # Scaffold desktop app
│   └── codegen-desktop/       # Code generator desktop app
├── crates/                    # Core libraries
│   ├── template_core/         # Template engine core
│   ├── infrastructure/        # Infrastructure layer
│   ├── repositories/          # Data access layer
│   ├── services/              # Business logic layer
│   └── shared/                # Shared types and utilities
├── web/                       # Web frontend
│   ├── src/
│   │   ├── api/              # API service layer
│   │   ├── components/       # Reusable components
│   │   ├── views/            # Page components
│   │   └── router/           # Route configuration
│   └── package.json
├── migrations/                # Database migration files
├── config/                    # Configuration files
├── data/                      # Data directory
│   ├── templates/            # Template file storage
│   └── releases/            # Version release data
└── Cargo.toml                # Rust workspace config
```

### Layered Architecture

```
┌─────────────────────────────────────────┐
│     Frontend Layer (Vue 3 + Naive UI)    │
├─────────────────────────────────────────┤
│       Application Layer (Axum Handlers)  │
├─────────────────────────────────────────┤
│       Business Logic Layer (Services)    │
│  - TemplateService                       │
│  - CategoryService                       │
│  - RenderService                         │
├─────────────────────────────────────────┤
│       Data Access Layer (Repositories)   │
│  - TemplateRepository                    │
│  - CategoryRepository                    │
├─────────────────────────────────────────┤
│    Infrastructure Layer                  │
│  - Database Pool                         │
│  - Storage Manager                       │
│  - Git Service                           │
└─────────────────────────────────────────┘
```

---

## Usage

### Web Interface

Visit `http://localhost:3000` to use the web management interface:

1. **Template Management** - Create, edit, and delete templates
2. **Variable Configuration** - Configure template variables and presets
3. **Version Control** - Manage template versions and releases
4. **File Editor** - Edit template files online
5. **Live Preview** - Preview rendering results in real-time

### CLI Tools

```bash
# Interactive project creation (TUI mode)
cargo run -p template-studio-cli -- create

# Create project with specified template
cargo run -p template-studio-cli -- create my-app --template vue3-ts

# View template details
cargo run -p template-studio-cli -- info vue3-ts

# Update local template cache
cargo run -p template-studio-cli -- update
```

### API Usage

```bash
# Get template list
curl http://localhost:8080/api/v1/template/templateList

# Get template details
curl http://localhost:8080/api/v1/template/templates/detail?id=1

# Render template
curl -X POST http://localhost:8080/api/v1/editor/templateFiles/render \
  -H "Content-Type: application/json" \
  -d '{
    "templateId": 1,
    "variables": {"projectName": "my-project"},
    "fileTree": [...]
  }'
```

---

## Configuration

### Backend Configuration (config/config.toml)

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

### Frontend Configuration (web/.env)

```bash
VITE_API_BASE_URL=http://localhost:8080
VITE_APP_TITLE=Template Studio
```

---

## Development Guide

### Rust Backend Development

```bash
# Run in development mode (auto-reload)
cargo run -p template-studio-web

# Run tests
cargo test

# Code linting
cargo clippy

# Format code
cargo fmt
```

### Vue Frontend Development

```bash
cd web

# Development server
pnpm run dev

# Code linting
pnpm run lint:eslint
pnpm run lint:prettier

# Type checking
pnpm run type-check
```

### Desktop App Development

```bash
cd apps/scaffold-desktop

# Development mode
pnpm run tauri:dev

# Build production version
pnpm run tauri:build
```

---

## Roadmap

### v0.1.0 (Current Version)
- ✅ Basic template management
- ✅ Web management interface
- ✅ CLI tools
- ✅ Template rendering engine
- 🚧 Desktop apps in progress

### v0.2.0 (Planned)
- 🔲 Template marketplace
- 🔲 User permission system
- 🔲 Template sharing and collaboration
- 🔲 More built-in templates

### v0.3.0 (Future)
- 🔲 Plugin system
- 🔲 Cloud sync
- 🔲 AI-assisted template generation
- 🔲 Multi-language support

---

## FAQ

### Q: Database migration failed?

A: Check database connection configuration, ensure the database service is running, and execute:
```bash
# Re-run migrations
cargo run -p template-studio-web
```

### Q: Frontend cannot connect to backend?

A: Check `VITE_API_BASE_URL` in `web/.env` and ensure it points to the correct backend service address.

### Q: How to add custom templates?

A: Create templates through the web interface, or upload template files via API to the `data/templates/` directory.

---

## Contributing

We welcome all forms of contributions!

1. Fork this repository
2. Create a feature branch (`git checkout -b feat/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feat/amazing-feature`)
5. Create a Pull Request

Please ensure:
- Code passes `cargo clippy` and `cargo test` checks
- Frontend code passes `pnpm run lint` checks
- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/) specification

---

## Support

- 📖 [Documentation](./docs/) - Detailed usage documentation
- 🐛 [Issue Tracker](https://github.com/template-studio/template-studio/issues) - Bug reports and feature requests
- 💬 [Discussions](https://github.com/template-studio/template-studio/discussions) - Q&A and discussions

---

## License

This project is licensed under the [MIT](LICENSE) License.

---

## Acknowledgments

- [Naive UI Admin](https://github.com/jekip/naive-ui-admin) - Excellent frontend admin template
- [Axum](https://github.com/tokio-rs/axum) - Powerful Rust web framework
- [Tauri](https://tauri.app/) - Modern desktop app development framework
- All contributors ❤️

---

<div align="center">

**⭐ If this project helps you, please give us a Star!**

Made with ❤️ by Template Studio Team

</div>
