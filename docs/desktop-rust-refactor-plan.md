# 桌面端 Rust 代码重构计划

> 日期: 2026-06-18
> 目标: 将 `database.rs` (4469行) 和 `lib.rs` (3544行) 拆分为模块化结构，单文件控制在 800 行以内

## 一、现状分析

### 1.1 代码规模

| 文件 | 行数 | 职责 |
|------|------|------|
| `database.rs` | 4469 | SQLite 数据层：60 个 pub 方法 + 12 个 migration + 8 个 pub 函数 + 9 个私有函数 + 15 个结构体 |
| `lib.rs` | 3544 | Tauri 入口：98 个 command + 辅助结构体 + DDL 生成 + 连接池缓存 + 测试 |
| `config.rs` | 131 | 配置管理（正常） |
| `main.rs` | 6 | 入口（正常） |
| **合计** | **8150** | |

### 1.2 核心问题

- **单文件过大**: 4000+ 行文件中定位一个方法全靠搜索，认知负担高
- **职责混杂**: `lib.rs` 同时承担 Tauri 入口、命令注册、连接池管理、DDL 生成、文件扫描
- **可维护性差**: 新增功能需要在 4000 行文件中找到正确位置插入
- **合并冲突**: 多人协作时单文件冲突概率高

## 二、重构方案

### 2.1 目标目录结构

```
apps/desktop/src-tauri/src/
├── main.rs                    # 入口 (不变)
├── config.rs                  # 配置 (不变)
├── lib.rs                     # 精简：run() + generate_handler![] + mod 声明 (~200行)
├── state.rs                   # DbState, BrowserPoolCache (~120行)
├── ddl.rs                     # PushColumnDef + generate_create_table_ddl + tests (~200行)
├── database/
│   ├── mod.rs                 # Database 结构体、init()、pool()、run_migrations (~200行)
│   ├── migrations.rs          # 12 个 migration 方法 (~800行)
│   ├── models.rs              # 所有公开数据结构体 (~200行)
│   ├── project.rs             # 项目 CRUD + 项目语言关系 (~350行)
│   ├── datasource.rs          # 数据源 CRUD + 测试连接 (~250行)
│   ├── table.rs               # 表 CRUD (~250行)
│   ├── column.rs              # 列 CRUD + 排序 (~250行)
│   ├── language.rs            # 语言 CRUD + 字段类型 (~400行)
│   ├── type_mapping.rs        # 系统/项目类型映射 CRUD (~500行)
│   ├── preferences.rs         # 表命名偏好 (~100行)
│   ├── ai.rs                  # AI 提供者/模型 CRUD (~400行)
│   └── import.rs              # 远程表导入 + SQL 解析 (~900行)
└── commands/
    ├── mod.rs                 # re-export 所有 command (~50行)
    ├── template.rs            # 模板相关 commands (~400行)
    ├── project.rs             # 项目 commands (~200行)
    ├── datasource.rs          # 数据源 commands (~150行)
    ├── table.rs               # 表/列 commands (~300行)
    ├── sync.rs                # 远程数据库操作 commands (~600行)
    ├── language.rs            # 语言 commands (~250行)
    ├── type_mapping.rs        # 类型映射 commands (~350行)
    ├── ai.rs                  # AI commands (~500行)
    ├── settings.rs            # 设置 commands (~100行)
    └── window.rs              # 窗口控制 commands (~50行)
```

### 2.2 文件行数预估

| 模块 | 预估行数 | 说明 |
|------|---------|------|
| `lib.rs` (精简后) | ~200 | 仅保留 run()、mod 声明、generate_handler![] |
| `state.rs` | ~120 | DbState + BrowserPoolCache |
| `ddl.rs` | ~200 | DDL 生成 + 测试 |
| `database/mod.rs` | ~200 | 结构体定义、init、migrations 入口 |
| `database/migrations.rs` | ~800 | 12 个 migration |
| `database/models.rs` | ~200 | 9 个公开结构体 |
| `database/project.rs` | ~350 | 13 个方法 |
| `database/datasource.rs` | ~250 | 6 个方法 |
| `database/table.rs` | ~250 | 4 个方法 |
| `database/column.rs` | ~250 | 5 个方法 |
| `database/language.rs` | ~400 | 5 个方法 + 5 个字段类型方法 |
| `database/type_mapping.rs` | ~500 | 13 个方法 |
| `database/preferences.rs` | ~100 | 2 个方法 |
| `database/ai.rs` | ~400 | 10 个方法 |
| `database/import.rs` | ~900 | 8 个 pub 函数 + 9 个私有函数 + 6 个辅助结构体 |
| `commands/mod.rs` | ~50 | re-export |
| `commands/template.rs` | ~400 | 10 个 command |
| `commands/project.rs` | ~200 | 7 个 command |
| `commands/datasource.rs` | ~150 | 5 个 command |
| `commands/table.rs` | ~300 | 9 个 command (table + column + import) |
| `commands/sync.rs` | ~600 | 8 个 command |
| `commands/language.rs` | ~250 | 10 个 command |
| `commands/type_mapping.rs` | ~350 | 19 个 command |
| `commands/ai.rs` | ~500 | 16 个 command |
| `commands/settings.rs` | ~100 | 4 个 command |
| `commands/window.rs` | ~50 | 5 个 command |

## 三、重构步骤（分阶段执行）

### 阶段一：提取 models 和 state（低风险）

**目标**: 将数据结构体和状态管理从大文件中分离，不改变任何逻辑。

#### 步骤 1.1: 创建 `database/models.rs`

将 `database.rs` 中所有 `pub struct` 移入:

```
移出的结构体 (9个):
- Project (line 3002)
- Datasource (line 3028)
- DatasourceParams (line 2989)
- TestConnectionParams (line 2976)
- DbTable (line 3044)
- DbColumn (line 3058)
- Language (line 3074)
- Statistics (line 4397)
- RecentProject (line 4406)
```

#### 步骤 1.2: 创建 `state.rs`

从 `lib.rs` 移出:

```
移出的内容:
- DbState struct + Clone + AsRef impls (lines 18-34)
- BrowserPool enum (lines 38-43)
- BrowserPoolCache struct + methods (lines 44-95)
```

#### 步骤 1.3: 创建 `ddl.rs`

从 `lib.rs` 移出:

```
移出的内容:
- PushColumnDef struct (line 1247)
- generate_create_table_ddl() fn (line 1258)
- mod tests (line 3447) — DDL 相关测试
```

### 阶段二：拆分 database/ 模块（中等风险）

**目标**: 将 `database.rs` 改为 `database/` 目录模块。

#### 步骤 2.1: 创建 `database/mod.rs`

```rust
// database/mod.rs
mod migrations;
mod models;
mod project;
mod datasource;
mod table;
mod column;
mod language;
mod type_mapping;
mod preferences;
mod ai;
mod import;

pub use models::*;

use sqlx::{SqlitePool, Row};
use std::path::PathBuf;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn init() -> Result<Self, sqlx::Error> { ... }
    pub fn pool(&self) -> &SqlitePool { ... }
}
```

#### 步骤 2.2: 迁移 migration 到 `database/migrations.rs`

```
迁移内容:
- run_migrations() 方法
- migration_001 ~ migration_012 共 12 个方法
- schema_migrations 表相关逻辑
```

#### 步骤 2.3: 按领域迁移 Database impl 方法

每个子模块中使用 `impl super::Database` 来扩展方法:

```rust
// database/project.rs
use super::Database;

impl Database {
    pub async fn create_project(&self, ...) -> Result<i64, String> { ... }
    pub async fn get_all_projects(&self) -> Result<Vec<Project>, String> { ... }
    // ...
}
```

迁移顺序（按依赖关系）:

| 顺序 | 文件 | 方法数 | 来源行范围 |
|------|------|--------|-----------|
| 1 | `project.rs` | 13 | 1228-1398, 2297-2348, 4417-4469 |
| 2 | `datasource.rs` | 6 | 1747-1880 |
| 3 | `table.rs` | 4 | 1954-2105 |
| 4 | `column.rs` | 5 | 2018-2174 |
| 5 | `language.rs` | 10 | 2186-2286, 2380-2471 |
| 6 | `type_mapping.rs` | 13 | 1409-1719 |
| 7 | `preferences.rs` | 2 | 2792-2827 |
| 8 | `ai.rs` | 10 | 2514-2765 |
| 9 | `import.rs` | ~25 | 2962-4381 (所有 pub/私有函数 + 辅助结构体) |

#### 步骤 2.4: 删除旧 `database.rs`

确认所有方法已迁移后删除原文件。

### 阶段三：拆分 commands/ 模块（中等风险）

**目标**: 将 `lib.rs` 中 98 个 command 按领域拆分。

#### 步骤 3.1: 创建 `commands/mod.rs`

```rust
// commands/mod.rs
pub mod template;
pub mod project;
pub mod datasource;
pub mod table;
pub mod sync;
pub mod language;
pub mod type_mapping;
pub mod ai;
pub mod settings;
pub mod window;
```

#### 步骤 3.2: 按领域迁移 commands

每个子模块中定义 command 函数，使用 `use crate::*` 引入依赖:

```rust
// commands/project.rs
use crate::{DbState, database::Project};
use tauri::State;

#[tauri::command]
pub async fn db_get_all_projects(database: State<'_, DbState>) -> Result<String, String> { ... }
```

迁移顺序:

| 顺序 | 文件 | command 数 | 来源行范围 |
|------|------|-----------|-----------|
| 1 | `window.rs` | 5 | 97, 103, 109, 116, 127 |
| 2 | `settings.rs` | 4 | 680-715, 3197 |
| 3 | `project.rs` | 7 | 742-884 |
| 4 | `datasource.rs` | 5 | 898-974 |
| 5 | `table.rs` | 9 | 1814-2003, 2020 |
| 6 | `sync.rs` | 12 | 988-1852, 3091-3141 |
| 7 | `template.rs` | 10 | 134-665, 3207 |
| 8 | `language.rs` | 10 | 2040-2262 |
| 9 | `type_mapping.rs` | 19 | 2197-2483, 3044-3076 |
| 10 | `ai.rs` | 16 | 2503-3015 |

#### 步骤 3.3: 精简 `lib.rs`

重构后的 `lib.rs` 仅保留:

```rust
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod config;
mod database;
mod state;
mod ddl;
mod commands;

use config::Config;
use database::Database;
use state::{DbState, BrowserPoolCache};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(BrowserPoolCache::new())
        .setup(|app| {
            // ... 初始化数据库
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 从 commands 模块引用
            commands::window::greet,
            commands::window::write_text_file,
            commands::project::db_get_all_projects,
            // ... 全部 98 个
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## 四、模块依赖关系

```
lib.rs
  ├── state.rs          (DbState, BrowserPoolCache)
  ├── ddl.rs            (DDL 生成)
  ├── config.rs         (配置)
  ├── database/
  │   ├── mod.rs        (Database struct)
  │   ├── models.rs     (数据结构体)
  │   ├── migrations.rs (数据库迁移)
  │   ├── project.rs    ← depends on models.rs
  │   ├── datasource.rs ← depends on models.rs
  │   ├── table.rs      ← depends on models.rs
  │   ├── column.rs     ← depends on models.rs
  │   ├── language.rs   ← depends on models.rs
  │   ├── type_mapping.rs
  │   ├── preferences.rs
  │   ├── ai.rs
  │   └── import.rs     ← depends on models.rs, datasource.rs
  └── commands/
      ├── mod.rs        (re-export)
      ├── template.rs   ← depends on database, config, ddl
      ├── project.rs    ← depends on database, state
      ├── datasource.rs ← depends on database, state
      ├── table.rs      ← depends on database, state, ddl
      ├── sync.rs       ← depends on database, state, ddl
      ├── language.rs   ← depends on database, state
      ├── type_mapping.rs ← depends on database, state
      ├── ai.rs         ← depends on database, state, config
      ├── settings.rs   ← depends on config
      └── window.rs     ← depends on tauri::Manager
```

## 五、迁移规则

### 5.1 Database impl 拆分规则

Rust 允许同一结构体在不同文件中定义 `impl` 块。每个子模块使用:

```rust
use super::Database;

impl Database {
    // 该领域的方法
}
```

**注意**: `mod.rs` 中需要 `pub mod xxx;` 来引入子模块，子模块的 `impl` 才会生效。

### 5.2 Command 函数迁移规则

每个 command 函数需要:
1. 保留在原文件中的 `#[tauri::command]` 注解
2. 保持 `pub` 可见性（`generate_handler![]` 需要访问）
3. `use crate::state::DbState` 替代原来的局部引用
4. `use crate::database::XxxStruct` 替代原来的局部引用

### 5.3 generate_handler! 更新规则

`tauri::generate_handler![]` 中的路径需要从:
```rust
greet, write_text_file, ...
```
改为:
```rust
commands::window::greet, commands::window::write_text_file, ...
```

## 六、风险控制

### 6.1 每阶段验证

每个阶段完成后执行:

```bash
cargo clippy -p desktop --lib    # 零警告
cargo test -p desktop --lib      # 测试通过
pnpm run build                   # 前端构建通过
cargo build -p desktop           # Tauri 构建通过
```

### 6.2 回滚策略

- 使用 git 分支: `refactor/database-split`、`refactor/commands-split`
- 每个步骤一个 commit，方便 bisect
- 如遇问题可 `git revert` 单个 commit

### 6.3 不做的事情

- **不改业务逻辑**: 纯移动代码，不修改任何函数签名或实现
- **不改前端代码**: Tauri command 名称不变，前端无感知
- **不改数据库 schema**: 不新增 migration
- **不一次性重构**: 分阶段执行，每阶段独立可用

## 七、预期收益

| 指标 | 重构前 | 重构后 |
|------|--------|--------|
| 最大单文件行数 | 4469 行 | ~900 行 |
| 平均文件行数 | 2037 行 | ~300 行 |
| 文件数量 | 4 个 | 27 个 |
| 定位功能所需时间 | 搜索 4000 行 | 直接打开对应模块 |
| 新增功能改动文件数 | 1-2 个大文件 | 2-3 个小文件 |
| 合并冲突概率 | 高 | 低 |
