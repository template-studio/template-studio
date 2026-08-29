# Template Studio

> A powerful template management and code generation platform with support for template creation, editing, rendering, and version management.

<div align="center">

**English** | **[简体中文](README.md)**

![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)
![Vue](https://img.shields.io/badge/Vue-3.5+-green?logo=vue.js)
![License](https://img.shields.io/badge/License-Apache%202.0-blue)
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
- **🔧 Template Engine** - MiniJinja-based engine with variable substitution, conditional rendering, custom filters and builtin functions
- **📊 Dependency Analysis** - Automatically analyze template file dependencies and optimize rendering order
- **🔐 RBAC Permission System** - Role-based access control with JWT authentication and Personal Access Tokens (PAT)
- **🌐 Web Management Interface** - Modern admin panel built with Ant Design Vue (unified tech stack with the desktop app)
- **💻 CLI Tools** - Both CLI and TUI interaction modes supported
- **🖥️ Desktop Application** - Cross-platform Tauri 2.x desktop app with offline support

### 🏗️ Technical Architecture

**Backend Stack:**
- Axum - High-performance async web framework
- SQLx - Type-safe database operations (MySQL / SQLite / PostgreSQL)
- Tokio - Async runtime
- MiniJinja - Template engine (compiles to WASM for browser-side rendering)
- Git2 - Git version control integration

**Frontend Stack:**
- Vue 3 + Composition API
- Ant Design Vue - Enterprise UI component library (unified with desktop app)
- Pinia - State management
- Alova - HTTP client with caching
- CodeMirror 6 - Code editor

**Desktop App:**
- Tauri 2.x - Lightweight desktop app framework
- Ant Design Vue - UI component library

---

## Quick Start

### Prerequisites

- Rust 1.70+ (with wasm32 target: `rustup target add wasm32-unknown-unknown`)
- wasm-pack (required to build the browser-side rendering engine: `cargo install wasm-pack`)
- Node.js 18+
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

Notes:
- The database `url` must be a standard URL like `mysql://user:pass@host:port/db` (Go-style DSN `tcp(host:port)` is NOT supported)
- For production, set the JWT secret via the `TEMPLATE_STUDIO_JWT_SECRET` environment variable (if unset, release builds generate an ephemeral secret on each start, invalidating sessions on restart)

#### 3. Start Backend Service

```bash
# Must run from the repository root (config and ./data storage use relative paths)
cargo run -p template-studio-web
```

Service will start at `http://localhost:8080`. Default admin account: `admin / 12345678`

#### 4. Start Frontend Interface

```bash
cd web
pnpm install
pnpm run dev
```

The first start automatically checks and builds the WASM rendering engine (~1-2 min on first run, ~1s on cache hit). Frontend will start at `http://localhost:8001`

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
│   ├── web/                   # Axum web backend server
│   ├── cli/                   # CLI tool (with TUI mode)
│   └── desktop/               # Tauri desktop app
│       ├── src/               # Vue 3 frontend
│       └── src-tauri/         # Tauri Rust backend
├── crates/                    # Core Rust libraries
│   ├── shared/                # Shared types, models, utilities
│   ├── infrastructure/        # Infrastructure (DB pool, config, git, logging)
│   ├── repositories/          # Data access layer (12 modules)
│   ├── services/              # Business logic layer (20 modules)
│   ├── template_core/         # Template engine core (MiniJinja)
│   └── template_core_wasm/    # Template engine WASM bindings (browser-side rendering)
├── web/                       # Vue 3 + Ant Design Vue web frontend
│   └── src/
│       ├── api/               # API service layer
│       ├── components/        # Reusable components
│       ├── views/             # Page components
│       ├── store/             # Pinia state management
│       └── router/            # Route configuration
├── migrations/                # SQL database migration files
├── scripts/build-wasm.mjs     # Shared WASM build script (reused by all frontends)
├── config/                    # Configuration files
├── data/                      # Runtime data (templates, versions, avatars; migrate together with the DB)
├── docs/                      # Design docs + worklog.md
├── dev-docs/                  # Status / migration / audit reports
└── Cargo.toml                 # Rust workspace config (10 member crates)
```

### Layered Architecture

```
┌─────────────────────────────────────────┐
│     Frontend Layer (Vue 3 + Ant Design Vue) │
├─────────────────────────────────────────┤
│       Application Layer (Axum Handlers)  │
├─────────────────────────────────────────┤
│       Business Logic Layer (Services)    │
│  - TemplateService / CategoryService     │
│  - AuthService / RBACService             │
│  - RenderService / ReviewService         │
├─────────────────────────────────────────┤
│       Data Access Layer (Repositories)   │
│  - TemplateRepository / UserRepository   │
│  - CategoryRepository / RoleRepository   │
├─────────────────────────────────────────┤
│    Infrastructure Layer                  │
│  - Database Pool (MySQL/SQLite/PG)       │
│  - Storage Manager / Git Service         │
├─────────────────────────────────────────┤
│    Template Engine (template_core)       │
│  - MiniJinja rendering / conditional     │
│  - WASM bindings (browser-side)          │
└─────────────────────────────────────────┘
```

---

## Usage

### Web Interface

Visit `http://localhost:8001` to use the web management interface:

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

```bash
# Authentication (default admin: admin / 12345678)
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "12345678"}'
# Response: {"code": 0, "message": "Login successful", "data": {"token": "...", "roles": [...]}}

# Authenticated endpoints use the custom `token` header
curl http://localhost:8080/api/v1/admin/auth/info \
  -H "token: <token from the previous step>"
```

Unified response envelope: success is `{"code": 0, "message": "...", "data": ...}`; on failure the HTTP status code matches the semantics and the body carries the corresponding code.

---

## Configuration

### Backend Configuration (config/config.toml)

```toml
[server]
host = "127.0.0.1"
port = 8080
# For production, explicitly allow frontend origins (defaults to localhost dev origins only)
# cors_origins = ["https://your-frontend.example.com"]

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
```

### Desktop App Development

```bash
cd apps/desktop

# Development mode
pnpm run tauri:dev

# Build production version
pnpm run tauri:build
```

---

## Roadmap

### v0.1.0 (Current Version)
- 🚧 Basic template management
- 🚧 Web management interface
- 🚧 CLI tools
- 🚧 Template rendering engine (MiniJinja)
- 🚧 RBAC permission system + JWT authentication
- 🚧 Desktop application (Tauri 2.x)
- 🚧 Template review workflow

### v0.2.0 (Planned)
- 🔲 Template marketplace
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

This project is licensed under the [Apache License 2.0](LICENSE) License.

---

## Acknowledgments

- [Naive UI Admin](https://github.com/jekip/naive-ui-admin) - Excellent frontend admin template
- [Axum](https://github.com/tokio-rs/axum) - Powerful Rust web framework
- [Tauri](https://tauri.app/) - Modern desktop app development framework