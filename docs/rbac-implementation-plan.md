# RBAC 登录认证 + 用户管理系统 实现规划

## Context

当前后台管理系统完全没有后端认证：前端使用 mock 数据模拟登录（`web/src/store/modules/user.ts` 中 `import.meta.env.DEV` 分支），后端没有用户表、没有 JWT、没有中间件、没有权限校验。所有 admin API 裸露，任何人可访问。

**目标**：实现完整的 JWT 认证 + RBAC 角色权限 + 用户管理，替换前端 mock 登录，保护后端 admin API。

**技术选型**：
- 密码哈希：bcrypt
- Token 方案：JWT（无状态）
- 权限粒度：菜单级（前端路由 + 后端路由组）
- 初始账号：迁移种子数据

---

## 一、数据库设计（6 个迁移文件）

### 010_create_users.sql
```sql
CREATE TABLE users (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    username VARCHAR(50) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    email VARCHAR(100) DEFAULT '',
    avatar VARCHAR(500) DEFAULT '',
    status TINYINT NOT NULL DEFAULT 1 COMMENT '1=启用 0=禁用',
    last_login_at TIMESTAMP NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_status (status)
);
```

### 011_create_roles.sql
```sql
CREATE TABLE roles (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(50) NOT NULL UNIQUE,
    display_name VARCHAR(100) NOT NULL,
    description VARCHAR(255) DEFAULT '',
    sort INT DEFAULT 0,
    status TINYINT NOT NULL DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

### 012_create_permissions.sql
```sql
CREATE TABLE permissions (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(100) NOT NULL UNIQUE COMMENT '如 dashboard, template, category',
    display_name VARCHAR(100) NOT NULL COMMENT '如 仪表盘, 模板管理',
    type VARCHAR(20) NOT NULL DEFAULT 'menu' COMMENT 'menu=菜单 button=按钮',
    parent_id BIGINT DEFAULT NULL,
    sort INT DEFAULT 0,
    status TINYINT NOT NULL DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_parent (parent_id)
);
```

### 013_create_user_roles.sql
```sql
CREATE TABLE user_roles (
    user_id BIGINT NOT NULL,
    role_id BIGINT NOT NULL,
    PRIMARY KEY (user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
);
```

### 014_create_role_permissions.sql
```sql
CREATE TABLE role_permissions (
    role_id BIGINT NOT NULL,
    permission_id BIGINT NOT NULL,
    PRIMARY KEY (role_id, permission_id),
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
);
```

### 015_seed_rbac_data.sql
```sql
-- 默认管理员账号: admin / admin123 (bcrypt hash)
INSERT INTO users (username, password_hash, email, status) VALUES
('admin', '$2b$12$LJ3m4ys3NzBJSdVg8VPVMuHCFsDGZbsSSTjGBnpfEjJGvOLMKeWm6', 'admin@templatestudio.com', 1);

-- 超级管理员角色
INSERT INTO roles (name, display_name, description, sort) VALUES
('super_admin', '超级管理员', '拥有所有权限', 0),
('admin', '管理员', '常规管理权限', 1),
('viewer', '观察者', '只读权限', 2);

-- 菜单权限（与前端路由对应）
INSERT INTO permissions (name, display_name, type, sort) VALUES
('dashboard', '仪表盘', 'menu', 0),
('template', '模板管理', 'menu', 1),
('category', '分类管理', 'menu', 2),
('language', '语言管理', 'menu', 3),
('var_preset', '变量预设', 'menu', 4),
('settings', '系统设置', 'menu', 5),
('user_management', '用户管理', 'menu', 6),
('role_management', '角色管理', 'menu', 7);

-- 超级管理员拥有所有权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r CROSS JOIN permissions p WHERE r.name = 'super_admin';

-- 管理员拥有除用户/角色管理外的权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r CROSS JOIN permissions p
WHERE r.name = 'admin' AND p.name NOT IN ('user_management', 'role_management');

-- admin 用户分配超级管理员角色
INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id FROM users u CROSS JOIN roles r
WHERE u.username = 'admin' AND r.name = 'super_admin';
```

---

## 二、后端 Rust 实现

### 2.1 新增依赖

**`Cargo.toml` (workspace) 添加**：
```toml
jsonwebtoken = "9"
bcrypt = "0.15"
```

**`apps/web/Cargo.toml` 添加**：
```toml
jsonwebtoken = { workspace = true }
bcrypt = { workspace = true }
```

### 2.2 Shared 数据模型

**`crates/shared/src/models/user.rs`** - User, CreateUserRequest, UpdateUserRequest, LoginRequest, LoginResponse
**`crates/shared/src/models/role.rs`** - Role, CreateRoleRequest, UpdateRoleRequest, RoleWithPermissions
**`crates/shared/src/models/permission.rs`** - Permission, PermissionTree
**`crates/shared/src/models/auth.rs`** - Claims (JWT), AuthUser (中间件提取)

### 2.3 Repository 层

**`crates/repositories/src/user_repository.rs`**：
- `find_by_username`, `find_by_id`, `list_users`, `create_user`, `update_user`, `delete_user`
- `get_user_roles`, `get_user_permissions` (JOIN 查询)
- `assign_roles`, `remove_roles`

**`crates/repositories/src/role_repository.rs`**：
- `find_by_id`, `list_roles`, `create_role`, `update_role`, `delete_role`
- `get_role_permissions`, `assign_permissions`, `remove_permissions`

**`crates/repositories/src/permission_repository.rs`**：
- `list_permissions`, `get_permission_tree`

### 2.4 Service 层

**`crates/services/src/auth_service.rs`**：
- `login(username, password) -> LoginResponse` — 验证密码，生成 JWT
- `verify_token(token) -> Claims` — 验证并解析 JWT
- `get_current_user(claims) -> UserWithPermissions` — 获取当前用户完整信息

**`crates/services/src/user_service.rs`**：
- CRUD 用户 + 密码哈希 + 角色分配

**`crates/services/src/role_service.rs`**：
- CRUD 角色 + 权限分配

**`crates/services/src/permission_service.rs`**：
- 获取权限列表/树

### 2.5 JWT 中间件

**`apps/web/src/middleware/auth.rs`**：
- `auth_middleware` — axum 中间件层，从 `token` header 提取 JWT，验证后注入 `AuthUser` 到请求扩展
- `require_permission(permission_name)` — 用于特定路由的权限检查

### 2.6 Handler 层

**`apps/web/src/handlers/auth.rs`**：
- `POST /api/v1/auth/login` — 登录（公开）
- `GET /api/v1/auth/info` — 获取当前用户信息+权限（需认证）
- `POST /api/v1/auth/logout` — 登出（前端清 token 即可，后端可选黑名单）
- `PUT /api/v1/auth/password` — 修改自己的密码

**`apps/web/src/handlers/user_management.rs`**：
- `GET /api/v1/admin/users/list` — 用户列表
- `POST /api/v1/admin/users/add` — 创建用户
- `PUT /api/v1/admin/users/edit` — 更新用户
- `DELETE /api/v1/admin/users/del/:id` — 删除用户
- `PUT /api/v1/admin/users/:id/roles` — 分配角色

**`apps/web/src/handlers/role_management.rs`**：
- `GET /api/v1/admin/roles/list` — 角色列表
- `POST /api/v1/admin/roles/add` — 创建角色
- `PUT /api/v1/admin/roles/edit` — 更新角色
- `DELETE /api/v1/admin/roles/del/:id` — 删除角色
- `PUT /api/v1/admin/roles/:id/permissions` — 分配权限

**`apps/web/src/handlers/permission_management.rs`**：
- `GET /api/v1/admin/permissions/list` — 权限列表
- `GET /api/v1/admin/permissions/tree` — 权限树

### 2.7 路由改造

**`apps/web/src/routes/admin.rs`**：
- 现有 admin 路由组全部包裹在 auth middleware layer 中
- 新增 `/users`, `/roles`, `/permissions` 子路由组

**新增 `apps/web/src/routes/auth.rs`**：
- `/api/v1/auth/*` 公开认证路由（login 不需认证）

**`apps/web/src/main.rs`**：
- AppState 新增: `auth_service`, `user_service`, `role_service`, `permission_service`
- DI 注入所有新 repository/service
- auth 路由挂载在 `/api/v1/auth`

---

## 三、前端改造

### 3.1 前端 API

**修改 `web/src/api/system/user.ts`**：
- `login(params)` → `POST /api/v1/auth/login`
- `getUserInfo()` → `GET /api/v1/auth/info`
- `logout()` → 前端清 token 即可

**新增 `web/src/api/system/role.ts`**：
- 角色列表、创建、更新、删除、分配权限

**新增 `web/src/api/system/permission.ts`**：
- 权限列表、权限树

**新增 `web/src/api/admin/user.ts`**：
- 用户列表、创建、更新、删除、分配角色（admin CRUD）

### 3.2 Store 改造

**修改 `web/src/store/modules/user.ts`**：
- 移除 `import.meta.env.DEV` mock 分支
- `login()` 直接调用后端 API
- `getInfo()` 从 `/api/v1/auth/info` 获取真实权限列表
- 权限格式对齐：后端返回 `[{value: 'dashboard', label: '仪表盘'}, ...]`

### 3.3 新增管理页面

**`web/src/views/admin/settings/users.vue`** — 用户管理页面：
- 用户列表（n-data-table）+ 新增/编辑弹窗
- 角色分配弹窗
- 状态切换

**`web/src/views/admin/settings/roles.vue`** — 角色管理页面：
- 角色列表 + 新增/编辑弹窗
- 权限分配（树形 checkbox）

### 3.4 路由更新

**修改 `web/src/router/modules/admin.ts`**：
- 新增权限守卫：路由 meta 增加 `permissions` 字段
- `user_management` 权限 → 用户管理路由
- `role_management` 权限 → 角色管理路由
- 设置页面左侧 tab 新增"用户管理"和"角色管理"

---

## 四、文件清单

### 新增文件（20 个）

| # | 路径 | 说明 |
|---|------|------|
| 1 | `migrations/010_create_users.sql` | 用户表 |
| 2 | `migrations/011_create_roles.sql` | 角色表 |
| 3 | `migrations/012_create_permissions.sql` | 权限表 |
| 4 | `migrations/013_create_user_roles.sql` | 用户-角色关联表 |
| 5 | `migrations/014_create_role_permissions.sql` | 角色-权限关联表 |
| 6 | `migrations/015_seed_rbac_data.sql` | 种子数据 |
| 7 | `crates/shared/src/models/user.rs` | User 模型 |
| 8 | `crates/shared/src/models/role.rs` | Role 模型 |
| 9 | `crates/shared/src/models/permission.rs` | Permission 模型 |
| 10 | `crates/shared/src/models/auth.rs` | JWT Claims |
| 11 | `crates/repositories/src/user_repository.rs` | 用户 Repository |
| 12 | `crates/repositories/src/role_repository.rs` | 角色 Repository |
| 13 | `crates/repositories/src/permission_repository.rs` | 权限 Repository |
| 14 | `crates/services/src/auth_service.rs` | 认证 Service |
| 15 | `crates/services/src/user_service.rs` | 用户 Service |
| 16 | `crates/services/src/role_service.rs` | 角色 Service |
| 17 | `crates/services/src/permission_service.rs` | 权限 Service |
| 18 | `apps/web/src/middleware/auth.rs` | JWT 认证中间件 |
| 19 | `apps/web/src/middleware/mod.rs` | 中间件模块 |
| 20 | `apps/web/src/routes/auth.rs` | 认证路由 |

### 修改文件（15 个）

| # | 路径 | 修改内容 |
|---|------|----------|
| 1 | `Cargo.toml` (workspace) | 添加 jsonwebtoken, bcrypt 依赖 |
| 2 | `apps/web/Cargo.toml` | 引入 jsonwebtoken, bcrypt |
| 3 | `crates/shared/src/models/mod.rs` | 注册 user, role, permission, auth 模块 |
| 4 | `crates/repositories/src/lib.rs` | 注册 + re-export 3 个新 repository |
| 5 | `crates/services/src/lib.rs` | 注册 + re-export 4 个新 service |
| 6 | `apps/web/src/handlers/mod.rs` | 注册 auth, user_management, role_management, permission_management |
| 7 | `apps/web/src/handlers/auth.rs` (新增) | 认证 Handler |
| 8 | `apps/web/src/handlers/user_management.rs` (新增) | 用户管理 Handler |
| 9 | `apps/web/src/handlers/role_management.rs` (新增) | 角色管理 Handler |
| 10 | `apps/web/src/handlers/permission_management.rs` (新增) | 权限管理 Handler |
| 11 | `apps/web/src/routes/admin.rs` | 添加 auth middleware layer + 新路由组 |
| 12 | `apps/web/src/main.rs` | DI 注入 + AppState 字段 + auth 路由 |
| 13 | `web/src/api/system/user.ts` | API 端点指向真实后端 |
| 14 | `web/src/store/modules/user.ts` | 移除 mock，使用真实 API |
| 15 | `web/src/router/modules/admin.ts` | 添加用户管理/角色管理路由 |

---

## 五、实现顺序

1. **Cargo.toml 依赖** → 添加 jsonwebtoken, bcrypt
2. **数据库迁移** → 010-015 六个 SQL 文件
3. **Shared 模型** → user, role, permission, auth (4 个文件)
4. **Repository 层** → user, role, permission (3 个文件)
5. **Service 层** → auth, user, role, permission (4 个文件)
6. **中间件** → JWT 认证中间件
7. **Handler 层** → auth, user_management, role_management, permission_management
8. **路由注册** → auth 路由 + admin 路由添加 middleware
9. **main.rs** → DI 注入所有新组件
10. **前端 API** → 修改 user.ts，新增 role.ts, permission.ts
11. **前端 Store** → 移除 mock，对接真实 API
12. **前端页面** → 用户管理、角色管理页面
13. **前端路由** → 新增菜单项 + 权限守卫
14. **cargo build 验证** → 编译通过
15. **运行迁移** → python scripts/migrate.py
16. **端到端测试** → 登录 → 权限控制 → 用户/角色管理

---

## 六、关键参考文件

- 后端 Handler 模式：`apps/web/src/handlers/category.rs`
- 后端路由注册：`apps/web/src/routes/admin.rs`
- DI 注入模式：`apps/web/src/main.rs`（L54-L110）
- 前端 HTTP 客户端：`web/src/utils/http/alova/index.ts`（token header）
- 前端 Store：`web/src/store/modules/user.ts`
- 前端路由守卫：`web/src/router/guards.ts`
- 前端路由配置：`web/src/router/modules/admin.ts`
- 前端 API 模式：`web/src/api/system/user.ts`

---

## 七、JWT 配置

```toml
# config/config.toml 新增
[jwt]
secret = "your-secret-key-change-in-production"
expire_hours = 72
```

---

## 八、API 响应格式

前端 Alova 期望 `code: 200` 为成功，认证 API 需遵循：

```json
// POST /api/v1/auth/login 成功
{ "code": 200, "message": "登录成功", "result": { "token": "jwt-token-here" } }

// GET /api/v1/auth/info 成功
{
  "code": 200,
  "result": {
    "username": "admin",
    "email": "admin@example.com",
    "avatar": "",
    "permissions": [
      { "value": "dashboard", "label": "仪表盘" },
      { "value": "template", "label": "模板管理" }
    ]
  }
}
```

admin CRUD API 使用 axios/request，期望 `code: 0` 为成功（与现有 category/template API 一致）。

---

## 九、验证方式

1. `cargo build` 编译通过
2. `python scripts/migrate.py` 执行 010-015 迁移
3. `cargo run -p template-studio-web` 启动后端
4. 前端 `pnpm dev` 启动开发服务器
5. 访问 `/admin` → 自动跳转登录页
6. 使用 admin / admin123 登录 → 成功进入后台
7. 访问"系统设置"→"用户管理"→ 能创建/编辑/删除用户
8. 访问"系统设置"→"角色管理"→ 能创建角色并分配权限
9. 创建一个 viewer 角色用户，登录后只能看到部分菜单
10. 直接请求 `/api/v1/admin/categories/list` 不带 token → 返回 401
