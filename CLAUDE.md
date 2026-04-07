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
pnpm run build            # Production build (frontend only)
cargo test -p desktop --lib  # Run Rust unit tests
```

## Architecture

### Rust Workspace (9 crates)

```
apps/web/src/              → Axum handlers, routes, middleware
apps/cli/src/              → CLI commands (clap), TUI (ratatui), renderer
apps/desktop/src-tauri/src/ → Tauri commands, SQLite database layer

crates/shared/             → Types, models, constants, utils
crates/infrastructure/     → DB pool, config, git, logging, file_tree
crates/repositories/       → Data access layer (10 modules)
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

### Desktop App (`apps/desktop/`)

Tauri 2.x desktop app with Vue 3 + Ant Design Vue frontend.

**17 routes** with dual layout system: `AppLayout` for global pages, `ProjectWorkspaceLayout` for `/project/*` routes.

Key pages:
| Route | Purpose |
|---|---|
| `/home` | Dashboard |
| `/templates` | Template management with wizard drawer |
| `/datasource` | Datasource CRUD + connection status |
| `/datasource/:id/browse` | Live database browser with data query |
| `/projects` | Project list |
| `/project/:id/tables` | Table management (core feature) |
| `/project/:id/preferences` | Table naming conventions |
| `/project/:id/mappings` | Per-project type mappings |
| `/mappings` | Global type mappings |
| `/languages` | Programming language management |
| `/settings` | Multi-level settings (general, display, keyboard, backup, AI, etc.) |

**Tables management** (`apps/desktop/src/views/project/tables/`):
- `index.vue` — Table list with search/filter/sort, batch delete, pagination
- `TableDialog.vue` — Add/edit table modal
- `ColumnsDrawer.vue` — Column CRUD with drag-to-reorder
- `SqlImportModal.vue` — Import tables from SQL DDL
- `ImportProgressModal.vue` — Import table structure from datasource
- `AiCreateTableDrawer.vue` — AI-assisted table creation (NL → SQL → preview → execute)
- `SchemaDiffDrawer.vue` — Schema diff/sync: overview (remote-new/local-new/synced) + column-level diff with bidirectional sync

**Backend** (`apps/desktop/src-tauri/src/`):
- `lib.rs` — 98 Tauri commands, DDL generation, unit tests
- `database.rs` — SQLite database layer (4400+ lines), 12 migrations, 60+ methods
- `config.rs` — App configuration

**98 Tauri commands** covering: template engine, project/datasource CRUD, remote database operations (MySQL/PostgreSQL/SQLite), table/column management, schema sync, language/type mapping CRUD, AI services, table preferences.

### Database

- **Web/CLI**: MySQL (primary), SQLite, PostgreSQL — all supported via SQLx feature flags. Migrations in `migrations/` (SQL files). Config: `config/config.toml` (gitignored, copy from `config/config.toml.example`).
- **Desktop**: Local SQLite database at `~/.cicbyte/template_studio/db/desktop.db` with 12 inline migrations in `database.rs`. Uses `sqlx` with WAL mode, 64MB cache, foreign keys enabled.

### Template Engine

`template_core` provides MiniJinja-based template rendering with conditional file generation, custom filters, built-in functions, dependency analysis, and parallel rendering. Also compiled to WASM for browser use.

## Key Patterns

### Tauri Command Pattern

```rust
// Rust side: serialize to JSON string, errors in Chinese
#[tauri::command]
async fn db_xxx(database: tauri::State<'_, DbState>, ...) -> Result<String, String> {
    let db = database.as_ref();
    let result = db.some_method(...).await.map_err(|e| format!("错误描述: {}", e))?;
    serde_json::to_string(&result).map_err(|e| format!("序列化失败: {}", e))
}

// Frontend side: invoke + JSON.parse
import { invoke } from '@tauri-apps/api/core'
const data = JSON.parse(await invoke('db_xxx', { id }))
```

### Vue 3 Composition API

All desktop components use `<script setup>`:
- `ref()`, `reactive()`, `computed()`, `watch()` for state
- `defineProps()` / `defineEmits()` for component contracts
- `useLayoutStore()` for global pagination/footer
- `v-model:open` pattern for drawers/modals

### Database Layer

- `Database` struct wraps `SqlitePool`, shared via `DbState(Arc<Database>)`
- Raw SQL via `sqlx::query()` / `sqlx::query_scalar()` / `sqlx::query_as()` (no ORM)
- Remote DB connection pooling via `BrowserPoolCache` (keyed by URL)
- DDL generation in `generate_create_table_ddl()` (lib.rs)

### UI Framework

Ant Design Vue 4.x with Chinese locale (`zhCN`), CSS custom properties for theming, CodeMirror 6 for SQL editing with diff view (`@codemirror/merge`).

## Conventions

- **Commit messages**: Always use Chinese (中文提交信息)
- **Conventional commits**: `feat:`, `fix:`, `style:`, `refactor:`, `docs:`, `chore:`
- **Rust**: Uses `anyhow`/`thiserror` for error handling, `tracing` for logging
- **Config files**: `config.toml` and `config.dev.toml` are gitignored — never commit secrets
