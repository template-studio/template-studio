# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Template Studio is a template management and code generation platform. It's a fullstack monorepo with three application targets:

- **Web backend** (`apps/web/`) — Rust/Axum API server + Vue 3/Naive UI admin frontend (`web/`)
- **CLI tool** (`apps/cli/`) — Rust CLI with TUI support
- **Desktop app** (`apps/desktop/`) — Tauri 2.x + Vue 3/Ant Design Vue

## Development Commands

### Rust Backend

```bash
cargo run -p template-studio-web          # Start API server (localhost:8080)
cargo run -p template-studio-cli -- <cmd> # Run CLI
cargo test                                 # Run all Rust tests
cargo clippy                               # Lint
cargo fmt                                  # Format
```

### Web Frontend (Vue 3 + Naive UI)

```bash
cd web
pnpm install
pnpm run dev              # Dev server at localhost:3000
pnpm build                # Production build
pnpm run lint:eslint      # ESLint
pnpm run lint:prettier    # Prettier
pnpm run lint:stylelint   # Stylelint
pnpm run type-check       # TypeScript type checking
```

### Desktop App (Tauri)

```bash
cd apps/desktop
pnpm run tauri:dev        # Start desktop app in dev mode
```

## Architecture

### Rust Workspace (9 crates)

```
apps/web/src/              → Axum handlers, routes, middleware
apps/cli/src/              → CLI commands (clap), TUI (ratatui), renderer
apps/desktop/src-tauri/src/ → Tauri commands, SQLite database layer

crates/shared/             → Types, models, constants, utils
crates/infrastructure/     → DB pool, config, git, logging, file_tree
crates/repositories/       → Data access layer (12 modules)
crates/services/           → Business logic layer (20 modules)
crates/template_core/      → Template engine (MiniJinja), conditions, filters, tree rendering
crates/template_core_wasm/ → WASM bindings for browser-side template rendering
```

**Request flow**: Handler → Service → Repository → Database (layered architecture).

### Web Frontend (`web/`)

Vue 3 + TypeScript + Naive UI admin panel. See `web/CLAUDE.md` for detailed frontend architecture.

Key patterns:
- **Alova** for HTTP client (configured in `src/utils/http/alova/`)
- **Pinia** for state management (`src/store/modules/`)
- **Dynamic routing** with permission-based filtering (FIXED or BACK mode)
- **Path aliases**: `@/` → `src/`, `/#/` → `types/`

### Database

- MySQL (primary), SQLite, PostgreSQL — all supported via SQLx feature flags
- Migrations in `migrations/` (SQL files)
- Config: `config/config.toml` (gitignored, copy from `config/config.toml.example`)

### Template Engine

`template_core` provides MiniJinja-based template rendering with conditional file generation, custom filters, built-in functions, dependency analysis, and parallel rendering. Also compiled to WASM for browser use.

## Conventions

- **Commit messages**: Always use Chinese (中文提交信息)
- **Conventional commits**: `feat:`, `fix:`, `style:`, `refactor:`, `docs:`, `chore:`
- **Rust**: Uses `anyhow`/`thiserror` for error handling, `tracing` for logging
- **Config files**: `config.toml` and `config.dev.toml` are gitignored — never commit secrets
