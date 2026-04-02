# 模板投稿系统改造方案

## 背景

当前模板创建/编辑仅限后台管理（`/admin`），普通用户无法贡献模板。本方案实现：
1. 客户端用户模板投稿（创建/编辑/发布）
2. 后台管理审核发布流程
3. 模板编辑器组件在管理端与客户端复用

---

## 一、数据库改造

### 1.1 templates 表新增字段

```sql
-- migrations/017_alter_templates_add_visibility.sql

ALTER TABLE templates
  ADD COLUMN owner_id BIGINT DEFAULT NULL COMMENT '模板所有者用户ID，NULL表示系统模板',
  ADD COLUMN visibility VARCHAR(20) DEFAULT 'private' COMMENT 'private=私有 draft=草稿 public=公开 pending=待审核',
  ADD COLUMN status VARCHAR(20) DEFAULT 'active' COMMENT 'active=正常 rejected=被拒 disabled=下架',
  ADD COLUMN reviewed_at DATETIME DEFAULT NULL COMMENT '审核时间',
  ADD COLUMN reviewed_by BIGINT DEFAULT NULL COMMENT '审核人ID',
  ADD COLUMN download_count INT DEFAULT 0 COMMENT '下载/使用次数',
  ADD INDEX idx_owner_id (owner_id),
  ADD INDEX idx_visibility (visibility),
  ADD INDEX idx_status (status);
```

**字段说明：**

| 字段 | 值域 | 含义 |
|------|------|------|
| `owner_id` | BIGINT / NULL | NULL = 后台创建的系统模板，有值 = 用户创建 |
| `visibility` | `private` / `draft` / `pending` / `public` | `private`=仅自己可见，`draft`=草稿，`pending`=待审核，`public`=已发布公开 |
| `status` | `active` / `rejected` / `disabled` | `active`=正常，`rejected`=审核被拒，`disabled`=被管理员下架 |
| `reviewed_at` | DATETIME | 审核时间 |
| `reviewed_by` | BIGINT | 审核人 |
| `download_count` | INT | 下载/使用计数 |

**状态流转：**

```
用户创建 → private (私有草稿)
    ↓ 编辑完成提交
    → pending (待审核)
    ↓ 管理员审核
    → public (公开发布) / rejected (审核拒绝，附原因)
    ↓ 管理员下架
    → disabled
```

### 1.2 模板审核记录表（可选，用于记录审核历史）

```sql
-- migrations/018_create_template_reviews.sql

CREATE TABLE template_reviews (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    template_id BIGINT NOT NULL COMMENT '模板ID',
    reviewer_id BIGINT NOT NULL COMMENT '审核人ID',
    action VARCHAR(20) NOT NULL COMMENT 'approve=通过 reject=拒绝',
    reason VARCHAR(500) DEFAULT '' COMMENT '审核备注/拒绝原因',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_template_id (template_id),
    FOREIGN KEY (template_id) REFERENCES templates(id) ON DELETE CASCADE
);
```

---

## 二、后端改造

### 2.1 新增依赖

无新增依赖，复用现有的 auth middleware 和 RBAC 体系。

### 2.2 Shared 模型扩展

**`crates/shared/src/models/template.rs` 新增：**

```rust
/// 模板可见性
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR")]
#[sqlx(rename_all = "snake_case")]
pub enum Visibility {
    Private,  // 仅自己可见
    Draft,    // 草稿
    Pending,  // 待审核
    Public,   // 已发布
}

/// 模板状态
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR")]
#[sqlx(rename_all = "snake_case")]
pub enum TemplateStatus {
    Active,    // 正常
    Rejected,  // 审核拒绝
    Disabled,  // 已下架
}

/// 扩展 CreateTemplateRequest
pub struct CreateTemplateRequest {
    // ... 现有字段 ...
    pub visibility: Option<String>,  // 新增：默认 private
}

/// 扩展 UpdateTemplateRequest
pub struct UpdateTemplateRequest {
    // ... 现有字段 ...
    pub visibility: Option<String>,  // 新增
}

/// 发布审核请求
pub struct PublishTemplateRequest {
    pub template_id: i64,
}

/// 审核请求（管理员用）
pub struct ReviewTemplateRequest {
    pub template_id: i64,
    pub action: String,   // approve / reject
    pub reason: String,   // 审核备注
}

/// 用户模板列表查询（客户端）
pub struct UserTemplateListQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub keyword: Option<String>,
    pub visibility: Option<String>,
    pub owner_id: Option<i64>,
}
```

### 2.3 Repository 层扩展

**`crates/repositories/src/template_repository.rs` 新增方法：**

```rust
impl TemplateRepository {
    // 现有方法保持不变 ...

    /// 创建用户模板（带 owner_id 和 visibility）
    pub async fn create_user_template(&self, owner_id: i64, req: &CreateTemplateRequest) -> Result<Template>

    /// 列出用户的模板
    pub async fn list_user_templates(&self, owner_id: i64, query: &UserTemplateListQuery) -> Result<(Vec<TemplateItem>, i64)>

    /// 获取待审核模板列表（管理员）
    pub async fn list_pending_templates(&self, page: u32, page_size: u32) -> Result<(Vec<TemplateItem>, i64)>

    /// 更新模板可见性
    pub async fn update_visibility(&self, template_id: i64, visibility: &str) -> Result<()>

    /// 审核模板（更新 visibility + status + reviewed_at/by）
    pub async fn review_template(&self, template_id: i64, reviewer_id: i64, action: &str, reason: &str) -> Result<()>

    /// 获取公开模板列表（客户端，只查 visibility=public 且 status=active）
    pub async fn list_public_templates(&self, query: &UserTemplateListQuery) -> Result<(Vec<TemplateItem>, i64)>

    /// 验证模板所有者
    pub async fn is_owner(&self, template_id: i64, user_id: i64) -> Result<bool>

    /// 增加下载计数
    pub async fn increment_download_count(&self, template_id: i64) -> Result<()>
}
```

### 2.4 Service 层扩展

**`crates/services/src/template_service.rs` 新增：**

```rust
impl TemplateService {
    // 现有方法保持不变 ...

    /// 用户创建模板
    pub async fn create_user_template(&self, user_id: i64, req: &CreateTemplateRequest) -> Result<Template>

    /// 用户更新自己的模板
    pub async fn update_user_template(&self, user_id: i64, template_id: i64, req: &UpdateTemplateRequest) -> Result<Template>

    /// 用户删除自己的模板
    pub async fn delete_user_template(&self, user_id: i64, template_id: i64) -> Result<()>

    /// 提交审核（private → pending）
    pub async fn submit_for_review(&self, user_id: i64, template_id: i64) -> Result<()>

    /// 管理员审核模板
    pub async fn review_template(&self, reviewer_id: i64, req: &ReviewTemplateRequest) -> Result<()>

    /// 管理员下架模板
    pub async fn unpublish_template(&self, reviewer_id: i64, template_id: i64, reason: &str) -> Result<()>
}
```

### 2.5 Handler 层新增

**`apps/web/src/handlers/template.rs` 新增处理函数：**

```rust
// ===== 用户模板操作 =====

/// 创建用户模板
pub async fn create_user_template(
    State(state), Extension(auth_user): Extension<AuthUser>, Json(req)
) -> Result<Json<Value>>

/// 更新用户模板
pub async fn update_user_template(
    State(state), Extension(auth_user): Extension<AuthUser>, Path(id), Json(req)
) -> Result<Json<Value>>

/// 删除用户模板
pub async fn delete_user_template(
    State(state), Extension(auth_user): Extension<AuthUser>, Path(id)
) -> Result<Json<Value>>

/// 提交审核
pub async fn submit_for_review(
    State(state), Extension(auth_user): Extension<AuthUser>, Path(id)
) -> Result<Json<Value>>

/// 获取我的模板列表
pub async fn list_my_templates(
    State(state), Extension(auth_user): Extension<AuthUser>, Query(query)
) -> Result<Json<Value>>

/// 获取公开模板列表（客户端，不需要认证）
pub async fn list_public_templates(
    State(state), Query(query)
) -> Result<Json<Value>>

// ===== 管理员审核 =====

/// 获取待审核模板列表
pub async fn list_pending_templates(State(state), Query(query)) -> Result<Json<Value>>

/// 审核模板
pub async fn review_template(
    State(state), Extension(auth_user): Extension<AuthUser>, Json(req)
) -> Result<Json<Value>>
```

### 2.6 路由注册

**`apps/web/src/routes/admin.rs` 新增：**

```rust
// 在 admin_routes 中（已有认证中间件）：
.nest("/templates/pending", template_review_routes())

fn template_review_routes() -> Router<AppState> {
    Router::new()
        .route("/list", get(list_pending_templates))
        .route("/review", post(review_template))
}
```

**`apps/web/src/routes/client.rs`（或 auth 路由组）新增：**

```rust
// 需要认证的用户模板路由
.nest("/my/templates", user_template_routes())

fn user_template_routes() -> Router<AppState> {
    Router::new()
        .route("/list", get(list_my_templates))
        .route("/add", post(create_user_template))
        .route("/:id", put(update_user_template))
        .route("/:id", delete(delete_user_template))
        .route("/:id/submit-review", post(submit_for_review))
}
```

---

## 三、前端改造

### 3.1 模板编辑器组件抽离

将现有后台 `web/src/views/admin/template/index.vue` 中的编辑弹窗逻辑抽离为独立组件：

```
web/src/components/template-editor/
├── TemplateFormModal.vue      ← 基本信息编辑（名称/分类/语言/描述/Markdown详细介绍）
├── TemplateFileEditor.vue     ← 文件树 + 代码编辑器（从现有 editor 页面复用）
└── useTemplateForm.ts         ← 表单状态管理 composable
```

**TemplateFormModal.vue 核心接口：**

```vue
<script setup>
// Props
const props = defineProps<{
  mode: 'create' | 'edit'
  templateData?: Template       // 编辑时传入现有数据
  categories: Category[]        // 分类选项
  languages: Language[]         // 语言选项
}>()

// Emits
const emit = defineEmits<{
  (e: 'save', data: CreateTemplateRequest): void
  (e: 'cancel'): void
}>()
</script>
```

**复用策略：**

| 消费方 | 用法 |
|--------|------|
| 后台管理 `admin/template/index.vue` | 在 n-modal 中直接引入 `<TemplateFormModal />` |
| 客户端个人中心 | 在新页面 `client/my-templates/index.vue` 中引入，外层换成客户端布局 |
| 后台模板编辑器 | 现有 `admin/template/editor.vue` 的文件编辑功能保持不变，两端共享 |

### 3.2 新增客户端页面

#### 3.2.1 "我的模板" 页面

**`web/src/views/client/my-templates/index.vue`**

功能：
- 我的模板列表（卡片布局，区分私有/草稿/待审核/已发布状态）
- 创建新模板按钮
- 编辑/删除/提交审核操作
- 状态筛选（全部/草稿/待审核/已发布）

```
页面布局：
┌──────────────────────────────────────────────┐
│  我的模板                      [+ 创建模板]    │
│                                              │
│  [全部] [草稿] [待审核] [已发布]               │
│                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ 模板A     │  │ 模板B     │  │ 模板C     │  │
│  │ [草稿]    │  │ [待审核]  │  │ [已发布]  │  │
│  │ 编辑 删除 │  │ 撤回      │  │ 编辑 统计 │  │
│  └──────────┘  └──────────┘  └──────────┘  │
└──────────────────────────────────────────────┘
```

#### 3.2.2 模板编辑页面

**`web/src/views/client/my-templates/editor.vue`**

功能：
- 基本信息编辑（复用 `TemplateFormModal` 的表单部分）
- 文件树管理 + 代码编辑（复用后台编辑器的 API 和逻辑）
- 变量配置
- 预览
- 保存草稿 / 提交审核

```
页面布局：
┌──────────────────────────────────────────────┐
│  ← 返回    编辑模板: xxx    [保存] [提交审核] │
├──────────────────────────────────────────────┤
│  [基本信息] [文件管理] [变量配置]              │
│                                              │
│  （Tab 内容区）                               │
│                                              │
└──────────────────────────────────────────────┘
```

#### 3.2.3 模板详情页增强

修改现有 `web/src/views/client/template-generator/StepIntro.vue`：
- 显示模板作者信息（owner_id 关联的用户名）
- 显示下载次数
- 对于自己的模板，显示"编辑"按钮

### 3.3 路由配置

**`web/src/router/modules/client.ts` 新增：**

```typescript
{
  path: '/my-templates',
  name: 'MyTemplates',
  component: () => import('@/views/client/my-templates/index.vue'),
  meta: { title: '我的模板', requiresAuth: true },
},
{
  path: '/my-templates/editor/:id?',
  name: 'MyTemplateEditor',
  component: () => import('@/views/client/my-templates/editor.vue'),
  meta: { title: '编辑模板', requiresAuth: true },
},
```

### 3.4 导航入口

**`web/src/components/NavBar.vue` 新增"我的模板"链接：**

在导航栏 menubar 中添加"我的模板"入口（仅登录用户可见）。

### 3.5 前端 API 新增

**`web/src/api/templates/index.ts` 新增：**

```typescript
// 用户模板
export function listMyTemplates(params) { return Alova.Get('/v1/my/templates/list', { params, cacheFor: 0 }) }
export function createUserTemplate(data) { return Alova.Post('/v1/my/templates/add', data) }
export function updateUserTemplate(id, data) { return Alova.Put(`/v1/my/templates/${id}`, data) }
export function deleteUserTemplate(id) { return Alova.Delete(`/v1/my/templates/${id}`) }
export function submitForReview(id) { return Alova.Post(`/v1/my/templates/${id}/submit-review`) }

// 公开模板（不需要认证）
export function listPublicTemplates(params) { return Alova.Get('/v1/public/templates/list', { params, cacheFor: 60 * 1000 }) }

// 管理员审核
export function listPendingTemplates(params) { return Alova.Get('/v1/admin/templates/pending/list', { params, cacheFor: 0 }) }
export function reviewTemplate(data) { return Alova.Post('/v1/admin/templates/pending/review', data) }
```

### 3.6 后台管理新增审核页面

**`web/src/views/admin/template/review.vue`**

在后台"模板管理"下新增"审核"子页面：
- 待审核模板列表
- 审核操作（通过/拒绝+原因）
- 审核历史记录

---

## 四、权限矩阵

| 操作 | 游客 | 登录用户 | 管理员 | 超级管理员 |
|------|------|---------|--------|-----------|
| 浏览公开模板 | ✅ | ✅ | ✅ | ✅ |
| 使用模板生成项目 | ❌ | ✅ | ✅ | ✅ |
| 创建模板 | ❌ | ✅（私有） | ✅ | ✅ |
| 编辑自己的模板 | ❌ | ✅ | ✅ | ✅ |
| 删除自己的模板 | ❌ | ✅ | ✅ | ✅ |
| 提交审核 | ❌ | ✅ | ✅ | ✅ |
| 审核模板 | ❌ | ❌ | ✅ | ✅ |
| 编辑任意模板 | ❌ | ❌ | ✅ | ✅ |
| 下架模板 | ❌ | ❌ | ✅ | ✅ |
| 删除任意模板 | ❌ | ❌ | ❌ | ✅ |

---

## 五、编辑器共享方案详解

### 5.1 现有代码分析

当前后台模板编辑涉及两层：

1. **模板元数据编辑**（`admin/template/index.vue` 弹窗）：名称、分类、语言、描述、Markdown 介绍
2. **模板文件编辑**（`admin/template/editor.vue` 页面）：文件树、代码编辑器、文件上传

这两层功能不同，共享策略也不同。

### 5.2 元数据编辑 → 抽离为独立组件

```
web/src/components/template-editor/
├── TemplateMetaForm.vue       ← 纯表单组件（Naive UI），不含弹窗壳
└── useTemplateForm.ts         ← 表单验证 + 分类/语言数据加载逻辑
```

**TemplateMetaForm.vue** 接收 props：
- `modelValue`（v-model 双向绑定表单数据）
- `categories`、`languages`（外部传入选项数据）
- `mode`（create / edit，编辑时禁用类型选择）

**后台管理消费：**
```vue
<!-- admin/template/index.vue 的 n-modal 中 -->
<n-modal v-model:show="showModal">
  <n-card title="添加模板">
    <TemplateMetaForm v-model="formData" :categories="categories" :languages="languages" />
  </n-card>
</n-modal>
```

**客户端消费：**
```vue
<!-- client/my-templates/editor.vue 的基本信息 tab 中 -->
<TemplateMetaForm v-model="formData" :categories="categories" :languages="languages" />
```

### 5.3 文件编辑 → 共享编辑器路由

后台模板编辑器 `admin/template/editor.vue` 是一个完整的页面（左侧文件树 + 右侧代码编辑器）。

**方案：将编辑器页面移至公共路由，两端通过不同入口跳转。**

```
修改前：
  /admin/template/editor/:id  → admin/template/editor.vue（需要 admin 权限）

修改后：
  /editor/:id  → shared/template-editor/index.vue（需要登录 + 所有权校验）
```

**权限校验逻辑（前端路由守卫）：**
```typescript
// router/guards.ts
if (route.path.startsWith('/editor/')) {
  const templateId = route.params.id;
  // 管理员直接放行
  if (userStore.isAdmin) return next();
  // 普通用户检查是否为模板所有者
  const isOwner = await checkTemplateOwnership(templateId);
  if (!isOwner) return redirect('/403');
}
```

**后端 API 层校验（编辑器文件操作）：**
```rust
// 编辑器现有 handler 中加入所有权检查
async fn edit_template_file(State(state), Extension(auth): Extension<AuthUser>, ...) {
    let template = state.template_repo.find_by_id(template_id).await?;
    // 管理员可编辑所有模板
    if !auth.roles.contains(&"super_admin".to_string()) {
        // 普通用户只能编辑自己的
        if template.owner_id != Some(auth.user_id) {
            return error_response(StatusCode::FORBIDDEN, "无权编辑此模板");
        }
    }
    // ... 现有编辑逻辑
}
```

### 5.4 共享总结

| 模块 | 共享方式 | 消费方 |
|------|---------|--------|
| 模板元数据表单 | 组件抽离 `TemplateMetaForm.vue` | 后台弹窗 + 客户端页面 |
| 文件树 + 代码编辑器 | 共享路由 `/editor/:id` | 后台入口 + 客户端入口 |
| 分类/语言数据加载 | composable `useTemplateForm.ts` | 两端复用 |
| 模板 CRUD API | 同一套 API，后端做权限区分 | 两端复用 |

---

## 六、文件清单

### 新增文件

| # | 路径 | 说明 |
|---|------|------|
| 1 | `migrations/017_alter_templates_add_visibility.sql` | templates 表新增字段 |
| 2 | `migrations/018_create_template_reviews.sql` | 审核记录表 |
| 3 | `web/src/components/template-editor/TemplateMetaForm.vue` | 模板元数据表单组件 |
| 4 | `web/src/components/template-editor/useTemplateForm.ts` | 表单 composable |
| 5 | `web/src/views/client/my-templates/index.vue` | 客户端"我的模板"页面 |
| 6 | `web/src/views/client/my-templates/editor.vue` | 客户端模板编辑页面 |
| 7 | `web/src/api/templates/user.ts` | 用户模板 API |
| 8 | `web/src/api/templates/review.ts` | 审核管理 API |

### 修改文件

| # | 路径 | 修改内容 |
|---|------|---------|
| 1 | `crates/shared/src/models/template.rs` | 新增 Visibility/TemplateStatus 枚举，扩展 Request 结构体 |
| 2 | `crates/repositories/src/template_repository.rs` | 新增用户模板和审核相关方法 |
| 3 | `crates/services/src/template_service.rs` | 新增用户模板和审核业务逻辑 |
| 4 | `apps/web/src/handlers/template.rs` | 新增用户模板和审核 handler |
| 5 | `apps/web/src/routes/admin.rs` | 新增审核路由组 |
| 6 | `apps/web/src/routes/client.rs`（或 `main.rs`） | 新增用户模板路由和公开模板路由 |
| 7 | `apps/web/src/middleware/auth.rs` | 编辑器 API 增加所有权校验 |
| 8 | `web/src/views/admin/template/index.vue` | 弹窗改用 `TemplateMetaForm` 组件 |
| 9 | `web/src/views/admin/template/editor.vue` | 抽离为共享组件或保持路由跳转 |
| 10 | `web/src/views/admin/template/review.vue`（新增 tab） | 后台审核管理页面 |
| 11 | `web/src/router/modules/client.ts` | 新增 my-templates 路由 |
| 12 | `web/src/components/NavBar.vue` | 导航新增"我的模板"入口 |
| 13 | `web/src/views/client/home/index.vue` | 公开模板列表改用新 API |
| 14 | `web/src/views/client/templates-public/index.vue` | 模板卡片显示作者信息 |

---

## 七、实现顺序

1. **数据库迁移** → 017、018
2. **Shared 模型** → template.rs 扩展
3. **Repository** → template_repository.rs 新增方法
4. **Service** → template_service.rs 新增方法
5. **Handler** → template.rs 新增 handler
6. **路由注册** → admin.rs + client routes
7. **编辑器组件抽离** → TemplateMetaForm.vue + useTemplateForm.ts
8. **客户端页面** → my-templates/index.vue + editor.vue
9. **导航入口** → NavBar.vue + client router
10. **后台审核页面** → review.vue
11. **集成测试** → 创建 → 编辑 → 提交审核 → 审核 → 公开发布

---

## 八、API 响应格式

### 用户模板 API（使用 Alova，`code: 200` 成功，返回 `result`）

```
POST /api/v1/my/templates/add
Request:  { name, description, categoryId, templateType, languages, visibility }
Response: { code: 200, result: { id, name, ... } }

GET  /api/v1/my/templates/list?page=1&pageSize=10&visibility=private
Response: { code: 200, result: { list: [...], total: N } }

PUT  /api/v1/my/templates/:id
Request:  { name, description, categoryId, languages, ... }
Response: { code: 200, result: { id, name, ... } }

DELETE /api/v1/my/templates/:id
Response: { code: 200, message: "删除成功" }

POST /api/v1/my/templates/:id/submit-review
Response: { code: 200, message: "已提交审核" }
```

### 公开模板 API（无需认证）

```
GET /api/v1/public/templates/list?page=1&pageSize=12&keyword=xxx&categoryId=1
Response: { code: 200, result: { list: [...], total: N } }
```

### 审核 API（管理员，`code: 0` 成功）

```
GET  /api/v1/admin/templates/pending/list?page=1&pageSize=20
Response: { code: 0, result: { list: [...], total: N } }

POST /api/v1/admin/templates/pending/review
Request:  { templateId, action: "approve"|"reject", reason: "..." }
Response: { code: 0, message: "审核完成" }
```
