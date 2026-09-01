# 工作日志

## 2026-08-29 修复并统一前端 WASM 构建脚本

**变更内容：**
- 新增跨平台共享脚本 `scripts/build-wasm.mjs`：负责 wasm-pack 构建 `template_core_wasm` 并将产物（*.js/*.wasm）复制到调用方前端的 `src/wasm/` 目录，支持 `--check` / `--dev` / `--release` 三种模式，替代原先内嵌在 package.json 中的 PowerShell 命令。
- 修复 `web/`、`ant_design_web/` 两个前端 package.json 中 `build:wasm*` 系列脚本的问题：复制目标相对路径错误（`../web/src/wasm/` 实际解析到不存在的 `crates/web/`），且不自动创建被 gitignore 的目标目录，导致新克隆环境下 `pnpm run dev` 失败。两个前端统一改为调用共享脚本。
- `web/pnpm-workspace.yaml`：`allowBuilds.less` 的占位符值改为 `false`（less 的 postinstall 会下载 Playwright 浏览器，不应执行），修复 pnpm v11 安装前检查报错退出的问题。

**涉及文件：** `scripts/build-wasm.mjs`（新增）、`web/package.json`、`ant_design_web/package.json`、`web/pnpm-workspace.yaml`

**验收结果：** 删除两个前端 `src/wasm/` 产物模拟新环境后，`pnpm run build:wasm:check` 自动触发构建并正确复制（退出码 0）；两个前端 `pnpm run dev` 均正常启动，页面与 WASM 资源请求均返回 HTTP 200。

## 2026-08-29 输出 ant_design_web 迁移现状报告

**变更内容：** 明确 `web/` 后续移除、Web 端与桌面端统一 Ant Design Vue 技术栈的方向后，将两前端对比审计结果整理为现状报告（总体结论、P0-P3 分级问题清单、已确认等价项、移除 web/ 前待办），作为迁移工作基线。

**涉及文件：** `dev-docs/antd-web-migration-status.md`（新增，dev-docs 目录为首建）

**验收结果：** 报告中所有问题均带文件行号证据，关键 bug（引擎切换 Event 错传、树图标 isExpanded 字段、登录默认密码不一致）经二次人工复核确认。

## 2026-08-29 ant_design_web 首轮修复（P0-P3）

**变更内容：** 按迁移报告完成首轮修复——引擎切换 `@change` 参数错传（Event→e.target.value）、文件树目录图标 `isExpanded`→`expanded`、Table 行内编辑 customRender 签名包装、web 版登录默认密码改为 12345678（经 API 实测确认为正确凭据）、页脚 Naive UI 文案（代码默认值 + 后端存量 footer/powered_by 配置双修）、Footer 链接与版权行改指本项目、`treeToNaive` 更名 `convertToAntTree`、死代码清理（renderConditionIndicator / getTypeTagType）、package.json 元数据改为项目标识。决策记录：放弃锁屏功能；window.$loading 死代码维持现状。

**涉及文件：** `ant_design_web/src/views/editor/components/AdvancedDrawer.vue`、`FullRenderDrawer.vue`、`TemplateFileTree.vue`、`QuickDesignDrawer/utils/componentTemplates.js`、`src/components/Table/src/hooks/useColumns.ts`、`src/components/FooterBar.vue`、`src/components/SimpleVarPresetEditor.vue`、`src/layout/components/Footer/index.vue`、`src/views/login/index.vue`（web 侧）、`ant_design_web/package.json`；后端 `footer/powered_by` 配置经管理 API 更新；`dev-docs/antd-web-migration-status.md` 状态同步更新

**验收结果：** 全部改动模块经 vite 编译验证通过（200、无编译错误）；首页运行时复验渲染正常、页脚文案已显示「Powered by Ant Design Vue」；后端配置 GET 复查生效。

## 2026-08-29 修复编辑器无法加载问题（两前端共有）

**变更内容：** 登录后全路由走查发现 `/editor/:id` 两版均卡首屏。根因：根 `.gitignore` 的 `data/` 规则误伤任意层级 data 目录，导致 `src/views/editor/data/templateSyntax.ts`（模板语法参考数据）从未入库，编辑器异步组件编译失败。修复：gitignore 规则锚定为 `/data/`；按使用方约定重建 templateSyntax.ts（MiniJinja 语法参考，六类 27 项）放入 web 与 ant_design_web。

**涉及文件：** `.gitignore`、`web/src/views/editor/data/templateSyntax.ts`（新增）、`ant_design_web/src/views/editor/data/templateSyntax.ts`（新增）

**验收结果：** 两版 `/editor/:id` 实测正常挂载；模板目录缺失时显示「暂无数据（右键新建）」空状态。另完成登录后全路由走查（15 路由渲染正常），结论与数据迁移注意事项已更新至 `dev-docs/antd-web-migration-status.md`。

## 2026-08-29 条件编译（文件生成条件）全端接入

**变更内容：** 此前条件排除仅在 Web 服务端渲染生效，本次补齐其余各端——
1. `template_core`：将 `TreeBuilder.filter_by_conditions` 的过滤逻辑抽为自由函数 `filter_files_by_conditions` 并导出，供各端复用（语义不变：无条件默认生成、评估失败放行、目录级联剔除）。
2. WASM 绑定：`WasmTemplateFile` 增加 `condition` 可选字段，`render_tree` 在渲染前调用统一过滤函数，浏览器端渲染与服务端语义对齐。
3. Web 前端：`TemplateFile` 增加 `generateCondition` 结构化条件字段（`services/types.ts`），`WasmEngine` 透传给 WASM；修复 `BackendEngine.renderTree` 调用错位（原请求缺必需的 templateId 且按错误的响应结构解析，现按真实契约传 `{templateId, variables}` 并解析 `data.tree` 嵌套树）；`RenderEngine` 接口及 EngineManager/RenderService/composable 的 `renderTree` 签名统一增加 templateId 参数。
4. 桌面端/CLI：渲染前加载模板目录 `.meta/variables/conditions.yml` 并应用统一过滤（路径按 `/` 规范化匹配），本地渲染支持条件排除。
5. 修复存量 bug：前端操作符常量 `NOT_IN: 'not_in'` 与后端 serde 规范 `notin` 不一致导致「不包含于」条件保存必失败；常量改为 `notin`，后端 `NotIn` 增加 `not_in` 反序列化别名容错。

**涉及文件：** `crates/template_core/src/{tree_builder,conditions,lib}.rs`、`crates/template_core_wasm/src/lib.rs`、`apps/desktop/src-tauri/src/commands/template.rs`、`apps/cli/src/renderer/mod.rs`、`web/src/services/types.ts`、`web/src/services/render/{WasmEngine,BackendEngine,EngineManager,RenderService}.ts`、`web/src/composables/useRenderService.ts`、`web/src/api/conditions/index.ts`（另含 rustfmt 对 template_core 子模块的格式化）

**验收结果：** `cargo test -p template_studio_template_core` 45 用例全过；desktop/cli/wasm 编译通过。运行时双链路实测：浏览器内加载新 WASM 包验证条件过滤（无条件默认生成、单文件条件、目录级联均正确）；后端对真实模板设条件后 `preview-tree` 按变量值正确排除/包含目标文件（28/29 节点）。测试条件已清理。

## 2026-08-29 输出项目深度审计报告

**变更内容：** 四路并行审计（后端安全 / 健壮性 / 引擎与跨端一致性 / 桌面端与 CLI）加工程化自查，关键结论全部实测复核（无 token 删除模板与切换推荐状态成功、fork 恒 500、truncate 过滤器无效、default 第二参数失效、自动转义缺失均经运行时验证）。产出按 P0-P3 分级的修复路线图。

**涉及文件：** `dev-docs/project-deep-audit-2026-08.md`（新增）

**验收结果：** 报告共 12 项 P0 安全问题（8 项经实测确认或结构确认）、功能缺失/bug 风险/性能/工程化四类问题清单，全部带文件行号证据。探测均使用不存在的 ID，未影响真实数据。

## 2026-08-29 P0 安全修复 1/7：JWT secret 移出源码

**变更内容：** 深度审计修复序列第一步。`JwtConfig` 移除硬编码 `Default` 实现，改为 `from_env()`：优先读环境变量 `TEMPLATE_STUDIO_JWT_SECRET`；未配置时 debug 构建沿用开发默认值（本地开发零影响），release 构建生成随机临时密钥兜底并告警（重启后登录态失效）。`config.toml.example` 补充配置说明。

**涉及文件：** `crates/shared/src/models/auth.rs`、`apps/web/src/main.rs`、`config/config.toml.example`

**验收结果：** 新增单测验证 env 读取与兜底分支（通过）；web 应用编译通过。本地 dev 重启后已有登录态不受影响（debug 分支密钥不变）。

## 2026-08-29 P0 安全修复 2/7：模板路由读写分离

**变更内容：** `/api/v1/template` 路由组拆分为公开只读（types/templateList/detail/export/releases 列表/版本下载）与认证写操作（add/edit/del/toggle-featured/fork/analyze-variables/发布/回滚/重置/弃用）两组，写操作挂 auth 中间件。fork 随之移入认证组，修复其因 `AuthUser` extractor 挂在公开路由导致的恒 500。GET 直链下载类接口（export、版本下载）因 `<a href>` 无法携带 token 头暂保持公开，已留 TODO（后续支持 `?token=` 或下载签名）。

**涉及文件：** `apps/web/src/main.rs`

**验收结果：** 实测无 token 调用 5 个写接口全部 401（fork 由 500 变 401），读接口 200；带 token 写操作通过认证层（toggle-featured 同值 no-op 成功、fork 到达业务层）；浏览器实测前台模板广场（匿名）正常渲染。

## 2026-08-29 P0 安全修复 3/7：editor 与 backup 路由组补认证

**变更内容：** `/api/v1/editor`（文件树/增删改/上传/渲染/文件条件管理）与 `/api/v1/backup`（创建/预览/恢复）整组挂认证中间件；studio 公开目录组与 template-files 公开渲染组保持不变（前台与桌面端匿名使用）。匿名用户打开编辑器页不再能读取任意模板文件。

**涉及文件：** `apps/web/src/main.rs`

**验收结果：** 实测无 token 调 editor 四类接口与 backup 两接口全部 401；studio/index 与生成器 preview-tree（公开组）仍 200；浏览器登录后编辑器文件树完整加载（12 节点），匿名打开编辑器被拒（预期行为）。注：编辑器前端路由 ignoreAuth 的登录跳转体验为后续优化项。

## 2026-08-29 P0 安全修复 4/7：管理接口角色校验

**变更内容：** 新增 `admin_auth_middleware`（认证 + super_admin 角色查库校验，角色变更即时生效；PAT 令牌拒绝访问管理端），`/api/v1/admin` 路由按性质拆分：用户自助路由（auth/info、password、tokens、profile、avatar、my/templates——前端个人中心与我的模板实际使用）保持仅登录；管理路由（分类/语言/模板审核/变量预设/统计/设置/用户/角色/权限/邮件测试）要求 super_admin。AuthService 新增 `get_user_role_names` 公共方法；认证中间件重构出公共 `authenticate` 函数与统一错误响应构造。

**涉及文件：** `apps/web/src/middleware/auth.rs`、`apps/web/src/routes/admin.rs`、`apps/web/src/main.rs`、`crates/services/src/auth_service.rs`

**验收结果：** 实测矩阵：无 token→401；admin token→管理接口 200；普通注册用户→管理接口 403、自助接口（auth/info、my/templates）200；admin 管理页浏览器渲染正常。测试用户已清理。遗留：模板写接口的「属主校验」（普通用户可改他人模板）为独立后续项。

## 2026-08-29 P0 安全修复 5/7：路径穿越统一防护

**变更内容：** 新增共享路径校验工具 `shared::utils::path`（`validate_relative_path`/`safe_join`，组件级校验拒绝 `..`、反斜杠变体、绝对路径、盘符，附单测）。接入全部穿越点：`StorageManager::get_release_path` 改为校验 version 后返回 Result（覆盖全部按版本定位的 6 处调用方）；`render_file_from_path` 的 filePath；编辑器文件读/删/建/上传的 parent_path 与 file_name；upload_zip 解压与备份恢复解压的 zip 条目名（zip-slip）。

**涉及文件：** `crates/shared/src/utils/path.rs`（新增）及 `mod.rs`、`crates/infrastructure/src/config/storage.rs`、`crates/services/src/{template_render_service,backup_service,release_service}.rs`、`apps/web/src/handlers/{template,template_files}.rs`

**验收结果：** 单测 3 项通过；实测攻击全部被拒——公开 preview 的 `../` 与反斜杠穿越、version 穿越（400 非法版本号）、编辑器 content 接口 filePath 穿越（400），正常路径渲染/读取均不受影响；浏览器端到端编辑器打开 main.go 正常。

## 2026-08-29 P0 安全修复 6/7：模板查询 SQL 参数化

**变更内容：** `list_user_templates` 与 `list_public_templates` 的 visibility/keyword/category_id 条件由 `format!` 拼接（手工单引号转义可被 MySQL 反斜杠绕过）改为 sqlx 参数绑定；新增 `like_pattern` 辅助函数以 `ESCAPE '|'` 显式转义用户输入中的 `%`/`_`/`|`，顺带修复 LIKE 通配符污染匹配语义的问题。列表与 COUNT 两条查询同步参数化。repositories 层其余 SQL 复查无拼接残留。

**涉及文件：** `crates/repositories/src/template_repository.rs`

**验收结果：** 实测注入 payload（`' OR '1'='1`）与通配符 payload（`100%`）均返回 200 且按字面量匹配（0 条、无 SQL 错误、不全表泄漏）；templateList 正常返回 7 条。对比验证发现 `my/templates/list` 返回 0 条为**改动前已存在**的现象（疑与新库元数据迁移的 owner 关联有关），非本次引入，记为遗留疑点。

## 2026-08-29 P0 安全修复 7/7：接口限速与 CORS 收紧

**变更内容：** 新增滑动窗口限速中间件（`middleware/rate_limit.rs`，进程内单例，带单测）：认证组（登录/注册/忘记密码）每 IP 每分钟 20 次，公开重型接口组（preview/generate/generate-zip/clear-cache）每 IP 每分钟 60 次，客户端 IP 优先取代理头、否则 ConnectInfo（serve 改为 `into_make_service_with_connect_info`）。CORS 由 `permissive()` 改为谓词式：配置 `server.cors_origins` 时仅放行配置来源，未配置时放行 localhost/127.0.0.1/[::1] 开发来源，任意方法与头放行（前端自定义 token 头需要）。

**涉及文件：** `apps/web/src/middleware/rate_limit.rs`（新增）及 `middleware/mod.rs`、`apps/web/src/main.rs`、`apps/web/src/routes/public.rs`、`crates/infrastructure/src/config/settings.rs`

**验收结果：** 实测登录突发 25 次→20×401+5×429；重型接口突发 65 次→44 正常+21×429；CORS 预检 localhost:8001 回显 allow-origin、恶意来源无该头；浏览器端到端编辑器文件树正常（12 节点，自定义头预检问题已修复）。P0 序列（审计二节 #1-#10）至此全部完成。

## 2026-08-29 「我的模板」疑点排查（勘误）与头像上传白名单

**变更内容（排查）：** 此前记录的「my/templates/list 返回 0 条」疑点经复查为**探测脚本解析信封字段错误**（该接口返回 `{code, result}`，探测误读 `data`），功能本身完全正常（admin 返回 7 条、关键词搜索精确命中）。审计报告疑点注记已更正。此事实证了「API 信封三套混用」的危害，建议提升该项修复优先级。

**变更内容（修复）：** 头像上传（`upload_avatar`）增加扩展名白名单（jpg/jpeg/png/gif/webp，防 `.html` 存储型 XSS）与图片魔数校验（PNG/JPEG/GIF/WebP，防内容伪装）。

**涉及文件：** `apps/web/src/handlers/auth.rs`、`dev-docs/project-deep-audit-2026-08.md`（疑点更正）

**验收结果：** 实测 `.html` 上传 400 拒绝、伪装 `.png`（HTML 内容）400 魔数拦截、真实 PNG 上传成功且以 image/png 提供。

## 2026-08-29 错误响应内部路径泄漏修复（审计 P0 尾巴收官）

**变更内容：** 四处会把含完整服务器路径的错误透传给客户端的位置改为「内部日志保留全量、对外脱敏」：文件树服务（模板目录不存在）、模板渲染服务两处读文件 IO 错误、storage 层 `read_json_file`。

**涉及文件：** `crates/infrastructure/src/file_tree/service.rs`、`crates/infrastructure/src/config/storage.rs`、`crates/services/src/template_render_service.rs`

**验收结果：** 实测不存在的模板 fileTree 响应仅返回「模板目录不存在，模板可能尚未初始化」（无路径），后端日志仍保留完整路径供排查；正常 fileTree 200；渲染接口对不存在文件仅回显用户自身输入。至此审计 P0 全部关闭（桌面凭据加密为独立桌面端事项，留待后续）。

## 2026-08-29 安全修复系列全量回归

**变更内容：** 无代码改动。对 P0 安全修复系列（路由读写分离、三层中间件、SQL 参数化、路径防护、限速 CORS、错误脱敏）做全量回归。

**验收结果：** API 层 14 项检查全过（匿名公开读 5 项 200、匿名写操作 3 项 401、admin 正常路径 5 项 200、穿越攻击复核 400）；浏览器 14 条路由全部正常渲染、控制台零错误；编辑器深流程（文件树 12 节点、打开文件、变量侧栏 5 页签 15 分类）正常。未发现任何回归。

## 2026-08-29 P1 修复 1：文件监听三重失效修复

**变更内容：** `file_watcher` 此前从未真正工作过，共三层问题一并修复：① watcher 留在函数作用域、返回即被 drop，监听静默失效——现 move 进监听线程并保持存活；② 监听线程内直接 `tokio::spawn` 在非 runtime 上下文首个事件即 panic——改为经传入的 `tokio::runtime::Handle` 派发；③（修复①②后暴露的深层问题）监视路径为相对路径而 notify 在 Windows 返回 `\\?\C:\...` 绝对路径事件，前缀匹配失配导致所有事件被静默过滤——监视前先 `canonicalize`。

**涉及文件：** `apps/web/src/file_watcher.rs`、`apps/web/src/main.rs`（调用点传 Handle::current()）

**验收结果：** 实测修改模板目录文件后，后端日志出现「模板 1779081291997 缓存已失效（文件变化）」且依赖缓存 DEBUG 失效日志同步出现——文件变化→缓存自动失效链路首次真正打通，全程无 panic。

## 2026-08-29 P1 修复 2：发布与回滚事务化

**变更内容：** `release_service` 的发布流程（旧版本 is_latest 置 false + 新版本 INSERT）与回滚流程（两步 UPDATE）分别包入数据库事务，任一步失败整体回滚，消除「模板无 latest 版本 / 所有版本 is_latest=false」的数据损坏窗口。

**涉及文件：** `crates/services/src/release_service.rs`

**验收结果：** 真实模板实测完整周期：发布 0.1.0-test → 发布 0.2.0-test（版本列表恰好一个 latest=0.2.0）→ 回滚到 0.1.0-test（恰好 latest=0.1.0，get_latest_version 正确应答）。遗留说明：模板 1779081291997 现存两个 test 版本（最终 latest=0.1.0-test），内容与工作区一致，可在管理界面弃用或删除。

## 2026-08-29 P1 修复 3：请求路径 panic 与 UB 清理

**变更内容：** ① `template_files.rs` 六处 `n.as_i64().unwrap()` 改为 `unwrap_or(0)`（用户传浮点数不再打崩进程，与相邻字符串分支风格一致，落 0 后走下游不存在错误）；② PAT 创建的 `checked_add_signed().unwrap()` 改为优雅错误（极端过期天数）；③ PAT 验证的用户可控 token 头字节切片改为 `chars().take()`（多字节字符跨边界与长度不足均不再 panic/越界）；④ 邮件发件人回退分支二次 unwrap 改为错误传播；⑤ `static mut GIT_INIT_FN`（unsafe 数据竞争）改为 `OnceLock<GitInitFn>`。

**涉及文件：** `apps/web/src/handlers/{template_files,template}.rs`、`crates/services/src/{pat_service,email_service}.rs`

**验收结果：** 实测浮点 templateId 返回业务错误（修复前 panic）、多字节与超短 PAT 头均 401（修复前 panic/越界）、正常请求 200、后端日志零 panic；编译通过。

## 2026-08-29 P1 修复 4：删除模板事务与孤儿数据清理

**变更内容：** 仓库层 `delete` 事务化并补齐关联清理（template_languages、template_versions 无外键级联需显式删除；template_reviews 有 CASCADE 自动处理）；服务层删除时同步清理磁盘上的发布快照目录 `releases/<id>/`（此前会成为孤儿）。

**涉及文件：** `crates/repositories/src/template_repository.rs`、`crates/services/src/template_service.rs`

**验收结果：** 端到端实测：创建测试模板（目录落盘）→ 删除（成功）→ 模板列表无残留、存储目录无残留、后端日志「删除模板目录成功/删除模板成功」。测试数据已随删除清理。

## 2026-08-29 P1 修复 5：桌面端 SQLite 连接配置修正

**变更内容：** 桌面端数据库初始化改用 `SqliteConnectOptions` 连接级配置：PRAGMA（journal_mode/synchronous/foreign_keys/cache_size/temp_store）对池内全部连接生效（此前 `pool.execute` 只命中单个连接）；新增 `busy_timeout=5s` 避免 WAL 下并发写直接报 database is locked；显式 `max_connections(10)` 保持池规模。

**涉及文件：** `apps/desktop/src-tauri/src/database/mod.rs`

**验收结果：** desktop crate 编译零错误零新增警告，`cargo test -p desktop --lib` 5 用例全过。注：migration_005 的 DROP 重建升级风险为独立迁移设计问题，需专门设计升级迁移，未在本项处理。

## 2026-08-29 P2 引擎修复：truncate/default/length 语义修复与四个假内置函数实现

**变更内容：**
1. `truncate` 修复实测无效问题（根因：关键字参数 `truncate(length=5)` 不进位置参数槽，长度恒为默认 50）——改为 minijinja `Kwargs` 显式接收，位置与关键字传参均支持，附 `end` 自定义结尾参数；按字符截断消除多字节 panic 隐患。
2. `default` 补齐 Jinja2 的 `default("x", true)` boolean 语义（空串/0/false 也启用默认值）。
3. `length` 对无长度值的字符串按字符数计（原按字节数，中文算 6）。
4. 实现宣传已久但从未注册的四个内置能力：`now()` 函数（Unix 时间戳）、`date` 过滤器（时间戳格式化，本地时区）、`number_format`（千位分隔）、`slugify`（URL 友好化）；builtin.rs 的「需要自定义注册」标注更新为「已实现」，过滤器信息列表同步。
5. template_core 新增 chrono 依赖；新增 7 个过滤器单测（crate 总测试 46+6 全过）。

**涉及文件：** `crates/template_core/src/{filters,builtin}.rs`、`crates/template_core/Cargo.toml`

**验收结果：** 浏览器加载重建后的 WASM 包实测全部用例：`truncate(length=5)`→"hello..."（修复前不截断）、中文截断字符安全、`default("x",true)`→"x"（修复前返回空串）、中文 length→2、`now/date/number_format/slugify` 全部按预期输出；后端已重启同步生效。

## 2026-08-29 引擎修复（继承分析项 1/4）：渲染环境真缓存

**变更内容：** 落实 `dev-docs/engine-inheritance-analysis.md` §3.5。TEMPLATE_CACHE 由「只写不读的无上限 HashMap（存源码副本）」重构为「模板集哈希 → `Arc<Environment>`」的 LRU 真缓存（容量 32，新增 lru workspace 依赖）：命中直接复用整棵已编译模板环境，消除并行渲染下每文件重建 Environment + 重注册过滤器的 CPU/内存放大；缓存键为模板集内容哈希，内容变化自然换键、天然自失效。因 minijinja `add_template` 借用源字符串无法装入 'static 缓存，改用 `set_loader` 按名加载；主模板经 `render_str` 一次性渲染。`render_simple` 消除重复读锁；`/template-files/clear-cache` 端点接线 `clear_template_cache`（此前服务端无调用点）。

**涉及文件：** `crates/template_core/src/engine.rs`、`crates/template_core/Cargo.toml`、`apps/web/src/handlers/template_files.rs`

**验收结果：** 新增专项单测（命中不新增条目、继承跨命中正确、内容变化自失效、清空生效），crate 总测试 47+6 全过；WASM 重建后浏览器实测连续三次整树渲染继承全部正确且 get_cache_size 稳定为 1（缓存真实命中）；服务端重启后正常渲染。

## 2026-08-29 引擎修复（继承分析项 2/4）：双键注册与全文件继承

**变更内容：** 落实分析文档 §3.1/§3.2。`render_tree` 的模板映射改为双键注册：file_path 相对路径为主键（`extends "layouts/base.html"` 路径引用三端可解析，跨目录同名文件天然消歧），basename 为兼容键且仅在全树唯一时注册（保持简写可用，同名冲突时明确失败而非旧的随机覆盖）；`render_single_file` 移除 .html/.htm 分流，所有文件统一走支持继承的渲染（`.j2/.md/.txt` 中的 extends/include 可用），性能由上一项的环境缓存吸收。新增 3 个单测（路径+basename 双引用、同名消歧、非 HTML 继承）。

**涉及文件：** `crates/template_core/src/tree.rs`

**验收结果：** crate 总测试 50+6 全过；WASM 重建后浏览器复验当初失败场景——路径引用 `extends "layouts/base.html"` 现输出 `<html>B</html>`（修复前 template not found）、basename 简写保持可用、`.txt` 文件 include HTML 片段正常；服务端已重启同步生效。

## 2026-08-29 引擎修复（继承分析项 3/4）：依赖分析器补语法与预览一致性

**变更内容：** 落实分析文档 §3.3。① 依赖分析器全部正则（extends/import/include 三类五种）支持单双引号（MiniJinja 支持 `{% extends 'x' %}`，此前只认双引号导致漏识别）；新增 `{% from "macros.html" import a, b %}` 语法识别（from-import 的导入符号记入 namespace，依赖收集只需路径）。② 单文件预览（`render_file_from_path`）从「仅收集同目录 HTML」改为递归收集整棵模板树并经共享的 `build_template_map`（自 template_core 导出，整树渲染与预览共用）构建双键映射——预览与最终渲染的继承/include 解析从此一致；跳过 .git/.meta，读取失败的文件（二进制/编码）静默跳过。

**涉及文件：** `crates/template_core/src/{dependency_analyzer,tree,lib}.rs`、`crates/services/src/template_render_service.rs`

**验收结果：** 新增单测覆盖单引号 extends/include 与 from-import（crate 总测试 51+6 全过）；真实模板端到端实测——写入跨目录继承文件（pages/child extends layouts/base 路径引用）后经公开预览接口正确渲染 `<div class="layout">CHILD</div>`（修复前同目录限制必失败）；测试文件已清理。

## 2026-08-29 引擎修复（继承分析项 4/4）：按扩展名自动转义（方案 A）

**变更内容：** 落实分析文档 §3.4。新增 `render_string_named`（带模板名渲染）并成为文件内容渲染的标准入口：`.html/.htm/.xml` 结尾时 `{{ var }}` 输出自动 HTML 转义、`| safe` 豁免，其余扩展名与无名渲染（文件名/路径内部渲染、WASM 单文件入口）保持不转义。实现方式：全局环境与缓存环境均设置 `set_auto_escape_callback`，主模板经 `render_named_str` 携带真实文件名参与决策。接入点：整树渲染 `render_single_file`、编辑器单文件渲染 `render_file`、预览 `render_file_from_path`。builtin.rs 中「Tera 默认会转义」的错误文档同步修正为准确描述。AI 工具的展示性渲染与 WASM 单字符串入口维持不转义（非部署产物场景）。

**涉及文件：** `crates/template_core/src/{engine,tree,lib,builtin}.rs`、`crates/services/src/template_render_service.rs`

**验收结果：** 单测覆盖（HTML 转义、safe 豁免、非 HTML 不转义、无名不转义），crate 总测试 52+6 全过；浏览器 WASM 实测树渲染中 .html 输出 `&lt;b&gt;&amp;` 且 `| safe` 原样、.md 不转义；存量模板回归：gin-vue-base 无 .html 文件（转义零影响），全量渲染 failedFiles=11 均为预存的「空变量 + Strict 模式」undefined 错误（非本次引入，早期响应样本即含同类 renderError）。

## 2026-08-29 新增 CI 流水线（仅手动触发）

**变更内容：** 新增 GitHub Actions workflow（`.github/workflows/ci.yml`）：rust job（cargo fmt --check + clippy 警告不阻断 + workspace 测试，排除 wasm crate）、frontend job（web/ 安装依赖 + eslint；因未安装 vue-tsc 无 type-check 脚本，类型检查留待引入后启用）、wasm job（wasm32 目标 cargo check）。按约定仅 `workflow_dispatch` 手动触发，不做强制门禁，不含发版/部署。

**涉及文件：** `.github/workflows/ci.yml`（新增）

**验收结果：** YAML 语法经 js-yaml 解析通过；job 内引用的包名（template-studio-template-core-wasm）与前端脚本（lint:eslint）已与实际文件核对（发现并绕开了不存在的 type-check 脚本——CLAUDE.md 中该说明为旧版前端遗留，又一处文档漂移）。首次真实运行待推送到 GitHub 后手动触发验证。

## 2026-08-29 API 信封统一深度调研（文档输出）

**变更内容：** 量化盘点信封现状（后端 code:0+data 102 处 vs code:200+result 18 处、ApiResponse 死代码；前端双客户端——request/axios 绑阵营 A 返回完整 response、Alova 绑阵营 B 返回解包 result，另有 isReturnNativeResponse 第三种用法与 code:912 魔法数遗留）。产出四步迁移方案：前端拦截器双信封兼容兜底 → 后端 18 处收敛并启用 ApiResponse → 全量回归 → 前端收紧清理。

**涉及文件：** `dev-docs/api-envelope-analysis.md`（新增）

**验收结果：** 全部论断经 grep 量化与链路追踪实证（登录链路经 isReturnNativeResponse 绕开解包的机制已澄清；Alova 仅 4 个 api 文件使用且 menu/table 为模板残留）。

## 2026-08-29 API 信封统一第 1/4 步：前端双信封兼容兜底

**变更内容：** 落实 `dev-docs/api-envelope-analysis.md` 第①步。两个前端拦截器放宽成功判定以兼容双信封：Alova 拦截器 `code∈{0,200}` 均视为成功、业务负载取 `result ?? data`；request（axios）拦截器成功判定同步放宽（成功返回值保持完整 response 不变，不影响 89 处调用方）。`code:912` 魔法数与 `isReturnNativeResponse` 用法暂保留，待第④步收紧。

**涉及文件：** `web/src/utils/http/alova/index.ts`、`web/src/utils/request.ts`

**验收结果：** vite 编译通过；浏览器冒烟四类信封场景页面（模板广场 A 信封/我的模板 B 信封/仪表盘 A/个人中心 B）全部正常渲染、控制台零错误。

## 2026-08-29 API 信封统一第 2/4 步：后端 18 处收敛为 code:0+data

**变更内容：** 落实分析文档第②步。后端 auth.rs（7 处）、email.rs（3 处）、template.rs 用户模板系列（8 处）由 `{code:200, result}` 机械收敛为 `{code:0, data}`。前端同步更新四类消费方（过渡安全写法 `data ?? result` / `code===0 || code===200`）：user store 的 login/getInfo、登录页登录与注册判定、BasicUpload 上传成功判定；Alova 默认解包路径已由第 1 步的拦截器兼容覆盖。决策说明：本轮采用机械替换而非 ApiResponse 构造器重构以控制风险，ApiResponse 启用作为后续增量项。

**涉及文件：** `apps/web/src/handlers/{auth,email,template}.rs`、`web/src/store/modules/user.ts`、`web/src/views/login/index.vue`、`web/src/components/Upload/src/BasicUpload.vue`

**验收结果：** curl 复验登录/auth/info/我的模板均返回 code:0+data；浏览器全新登录周期端到端通过（清 storage → 登录 → 跳转 /admin/dashboard → 个人中心显示 admin 与令牌管理 → 我的模板正常）。过程中发现并修复自查引入的回归一处：getInfo 方法尾部两处未替换的 `result` 引用导致 ReferenceError（现象为路由守卫 catch 后静默登出弹回登录页），已补齐并全文件复扫清零。

## 2026-08-29 API 信封统一第 3/4 步：全量回归

**变更内容：** 无代码改动。对信封收敛做三层回归。

**验收结果：** ① API 电池 8/8——登录/用户信息/PAT 列表/我的模板/模板广场/忘记密码（防枚举）/注册新用户/新用户登录全部返回 code:0 且无 result 残留字段（当初踩坑的统一解析方式现已直接正确）；② 浏览器 14 路由全过、控制台零错误；③ 新用户注册→登录→删除全链路通过。测试用户已清理。第②步引入的 getInfo 回归经完整登录周期复验已确认修复。

## 2026-08-29 API 信封统一第 4/4 步：前端收紧为单信封（系列收官）

**变更内容：** 落实分析文档第④步。移除全部过渡兼容：两个拦截器收紧为仅 `code:0` 成功、负载仅取 `data`；`ResultEnum.SUCCESS` 由 200 改为 0；删除 `code:912` 魔法数分支与注释；store/登录页/BasicUpload 的 `?? result`、`|| code===200` 双判定收紧为单信封；全局残留扫描补获并修复重置密码页的 `code === 200` 漏网判定；删除零引用的模板残留假 api `api/table/list.ts`（`api/system/menu.ts` 因被路由生成器引用而保留，属路由机制遗留非信封问题）。

**涉及文件：** `web/src/enums/httpEnum.ts`、`web/src/utils/http/alova/index.ts`、`web/src/utils/request.ts`、`web/src/store/modules/user.ts`、`web/src/views/login/index.vue`、`web/src/views/client/reset-password/index.vue`、`web/src/components/Upload/src/BasicUpload.vue`、删除 `web/src/api/table/list.ts`

**验收结果：** 全部改动文件 vite 编译 200；全局 `912/===200/?? result` 残留扫描清零；浏览器全新登录周期通过（跳转 dashboard、token 写入）且五类关键路由（我的模板/个人中心/仪表盘/模板广场/编辑器）正常渲染。至此 API 信封全链路（后端 102+18 处、前端双客户端）统一为 `{code:0, message, data}`。

## 2026-08-29 修复 README.md 样式损坏

**变更内容：** 提交 5dfde15 精简头部徽章区时误删 `</div>` 闭合标签（开头 `<div align="center">` 成孤立标签，Markdown 渲染器将后续内容视为 HTML 块导致全文样式崩坏）；且文件尾部在 `<div align="center">` 开标签处截断，居中结尾块（Star 号召/署名/闭合）整体丢失。修复：删除头部孤立 div；从 git 历史（189b8cd）恢复尾部居中块及闭合标签。

**涉及文件：** `README.md`

**验收结果：** div 开闭配对 1:1（python 复核），头部与尾部结构恢复正常。README_EN.md 经检查 div 配对完好无需处理。注：README 内容层面的漂移（仍描述 Naive UI 前端、启动说明未更新）为已登记的待办项，本次仅修结构损坏。

## 2026-08-29 README 内容刷新（对齐项目现状）

**变更内容：** 12+1 处内容校准：Web 前端描述与技术栈由 Naive UI 改为 Ant Design Vue（与桌面端统一）；环境要求补 wasm-pack 与 wasm32 编译目标、Node 18+；数据库配置补 URL 格式警示（非 Go DSN）与 JWT 密钥环境变量说明；后端启动补「仓库根目录运行」与默认管理员凭据；前端端口 3000→8001（两处）并说明首次自动构建 WASM；项目结构图更新 web/ 描述并补 scripts/build-wasm.mjs 与 dev-docs/；API 示例改为真实默认凭据与统一信封 `{code:0,data}` 说明、补 token 头用法；配置示例补 cors_origins；前端开发指南移除不存在的 type-check 脚本并注明 vue-tsc 待引入；分层架构图去除 Naive UI。

**涉及文件：** `README.md`

**验收结果：** 全文扫描残留过时描述清零（致谢区 naive-ui-admin 模板来源引用保留属正常）；div 配对保持 1:1；diff 统计 +33/-20。README_EN.md 仍为旧内容，列为后续待办。

## 2026-08-29 README_EN.md 同步更新

**变更内容：** 英文版与中文版刷新内容对齐（13 处）：Ant Design Vue 技术栈描述、环境要求（wasm-pack/wasm32/Node 18）、数据库 URL 格式与 JWT 环境变量说明、后端根目录运行与默认凭据、前端端口 8001 与 WASM 自动构建说明、结构图（web 描述/scripts/dev-docs）、认证 API 示例（真实凭据 + 统一信封 + token 头）、cors_origins 配置示例、开发指南移除 type-check 并注明 vue-tsc 待引入。EN 版保留自身的徽章/语言切换头部结构（中文版此前已由用户精简，两版头部形态有意不同）。

**涉及文件：** `README_EN.md`

**验收结果：** 残留扫描清零（致谢区模板来源引用保留）、div 配对 1:1、代码围栏 32 偶数配对。过程中修正一处脚本拼接引入的代码块围栏错误（缺 ```bash 开头与续行符丢失），已重写该段并复核。

## 2026-08-29 CLAUDE.md 对齐项目现状

**变更内容：** 文档漂移收尾：前端描述 Naive UI→Ant Design Vue（含单前端说明）、移除幽灵 `type-check` 脚本与不存在的 `web/CLAUDE.md` 引用、前端端口 3000→8001、后端补根目录运行与默认凭据、workspace 9→10 crate（补 ai_agent）、桌面命令数 98→100、database.rs→database/ 模块目录。新增「Key backend conventions」小节沉淀当日确立的约定：统一信封、token 头认证与路由组权限布局、路径安全工具、JWT 环境变量、WASM 共享构建脚本、元数据与文件双存储。

**涉及文件：** `CLAUDE.md`

**验收结果：** 漂移扫描清零（Naive UI/type-check/3000/web::CLAUDE 引用）。至此 README（中英）与 CLAUDE.md 三份文档全部与现状对齐。

## 2026-08-29 修复桌面端 lodash-es 幽灵依赖

**变更内容：** `TemplateWizardDrawer.vue` 引用了未在 package.json 声明的 `lodash-es`（仅为一个 debounce 函数），导致 vite 导入解析失败。将 AppLayout.vue 中的内联 debounce 实现提取为共享工具 `apps/desktop/src/utils/debounce.ts`（补 TS 泛型与类型），两处统一引用，移除对 lodash-es 的依赖需求。

**涉及文件：** `apps/desktop/src/utils/debounce.ts`（新增）、`apps/desktop/src/components/layout/AppLayout.vue`、`apps/desktop/src/views/templates/components/TemplateWizardDrawer.vue`

**验收结果：** 桌面端运行中的 dev server（14200）实测三个改动模块编译均 200 且无解析错误；全项目 `lodash-es` 引用仅剩工具文件注释。

## 2026-08-29 模板属主校验（第一批：模板管理面 8 个接口）

**变更内容：** 新增 `handlers/access.rs` 的 `ensure_template_access`（super_admin 直通，否则查库校验属主，403「无权操作他人的模板」）与 `template_service.is_template_owner` 转发。接入 8 个模板管理接口：templates/edit、del、toggle-featured、analyze-variables、releases 发布/回滚/重置/弃用——普通登录用户此前可增删改发布任何人的模板。编辑器文件操作组与文件条件组（templateFiles/content、add、del、edit、rename、uploadCode、uploadZip、restore、file-conditions 系列）为第二批待接。

**涉及文件：** `apps/web/src/handlers/{access,mod,template,releases,template_analysis}.rs`、`crates/services/src/template_service.rs`

**验收结果：** 编译零错误；双用户实测矩阵——普通用户对 admin 模板的编辑/删除/发布/回滚/重置/弃用/推荐切换/变量分析全部 403，admin 同值操作正常（编辑 422 为字段校验、发布列表 200、toggle 成功）；测试用户已清理。

## 2026-08-29 模板属主校验（第二批：编辑器文件面 13 个接口）

**变更内容：** `ensure_template_access` 接入编辑器文件操作全组：文件树/内容读取、新建/删除/编辑/重命名（move）、uploadCode/uploadZip（multipart 在 templateId 解析后校验）、restore，以及 file-conditions 六接口（查/设/删/试评/导入/导出）。至此普通登录用户无法读写删他人模板的文件与条件配置。过程中一处脚本插入将 upload_zip 的 `pub` 劈裂产生语法碎片，经按行修复与三处复核（guard 落点、回读核实、编译）收敛。

**涉及文件：** `apps/web/src/handlers/{template,editor,file_conditions,access}.rs`

**验收结果：** 编译零错误零警告；双用户实测——普通用户对 admin 模板的文件树/内容读取 403、新建（合法请求体）/删除/设置条件/导出条件 403（首轮两个 422 为测试载荷字段不全导致的反序列化先行失败，非绕过，合法体复测均 403）；admin 同操作正常（新建+删除成功）。测试用户已清理。

## 2026-08-29 实现 get_template_languages（模板详情语言字段）

**变更内容：** 服务层 TODO 桩（恒返回空列表，模板详情 languages 字段因此永远为空）落地：仓库层新增 `get_template_language_details`（template_languages JOIN languages，含名称/显示名/图标/颜色/主语言标记，按主语言优先排序），服务层桩改为转发；`TemplateLanguageInfo` 补 `sqlx::FromRow` derive。

**涉及文件：** `crates/repositories/src/template_repository.rs`、`crates/services/src/template_service.rs`、`crates/shared/src/models/template.rs`

**验收结果：** 实测模板 1779081291997 详情返回 languages=[{name: go, displayName: go, isPrimary: 1}]（修复前恒为空数组）。

## 2026-08-29 统计接口真实化（仪表盘五指标去伪造）

**变更内容：** statistics.rs 五个接口全部改为真实数据——总览的 totalFiles 由 `模板数×5` 臆造改为各模板最新发布版本 file_count 汇总；分类分布/语言热度由 `count*100/total%10` 伪造改为 GROUP BY 真实聚合（含真实百分比）；复杂度由硬编码 5/8/3 改为按模板类型分档 + 解析各模板 variables.json 字段数分档（0/1-10/>10）；使用趋势由 `(i%10)+1` 伪造改为按 created_at 真实聚合（补零保持时间轴连续）。仓库层新增四个聚合查询，service 层转发。

**涉及文件：** `apps/web/src/handlers/statistics.rs`（重写）、`crates/repositories/src/template_repository.rs`、`crates/services/src/template_service.rs`

**验收结果：** 实测：分类分布 web:6/cli:1（真实）、语言热度 go:4/vue:2/rust:2/python:1（真实）、复杂度 6 scaffold+1 datadriven 且变量分档来自真实 variables.json、趋势 400 天窗口显示 7 个真实创建日（近 7 天全零为正确行为）。过程中发现并修复 MySQL `SUM(INT)` 返回 DECIMAL 导致 i64 解码失败被 unwrap_or(0) 吞掉的问题（CAST AS SIGNED），totalFiles 由 0 修正为 323。

## 2026-08-29 桌面端两个 mock 命令落地

**变更内容：** ① `list_templates`：由返回硬编码假模板（Go Web Service/Rust CLI Tool）改为本地优先策略——扫描本地模板存储目录列出已下载模板（离线可用，配合桌面端离线定位），本地为空时回退 Web 服务端公开模板列表（5 秒超时，离线静默返回空由前端提示）。② `render_template`：由按 ID 硬编码模拟内容改为复用 `render_template_preview` 的真实本地渲染链路（扫描 + 条件过滤 + render_tree），把树形结果转换为前端期望的扁平文件列表（过滤目录节点）。

**涉及文件：** `apps/desktop/src-tauri/src/commands/template.rs`

**验收结果：** desktop 编译零错误、5 个单测全过。注：两个命令的完整 UI 级验证需启动 Tauri 桌面应用进行，本机验证以编译与单测为界；render_template 走 render_template_preview 已验证过的渲染管线。

## 2026-08-30 migration_005 升级数据保护（备份-重建-回填）

**变更内容：** 桌面端 migration_005 原实现直接 DROP 旧表重建（v4 及更早版本升级时用户的 projects/datasources/db_tables/db_columns 数据全部清空）。重写为备份-重建-回填：①幂等保护（检测新列已存在则只补版本号）；②带数据的旧表 RENAME 暂存（无数据的直接 DROP）；③原重建逻辑不变；④回填——datasources 同名列直迁，projects 以首个数据源兜底关联+database_type 占位，db_tables/db_columns 按项目名/表名+项目名关联迁回；⑤最后清理暂存表（回填失败的数据保留在暂存表可人工恢复）。新增集成测试 `tests/mig005_test.rs`（对真实库副本模拟 v4 结构与数据，执行完整迁移链后断言数据保留/新结构就位/暂存表清理），Database 增加 `from_pool` 测试构造与 `run_migrations_for_test` 公开包装。

**涉及文件：** `apps/desktop/src-tauri/src/database/{migrations,mod}.rs`、`apps/desktop/src-tauri/src/lib.rs`、`apps/desktop/src-tauri/tests/mig005_test.rs`（新增）、`apps/desktop/src-tauri/Cargo.toml`（dev-deps tokio）

**验收结果：** 集成测试通过——v4 模拟库（1 项目/1 数据源/1 表/2 列）升级后全部数据保留、新结构（datasource_id 列）就位、暂存表清理干净；desktop 全部测试（5 单测 + 1 集成）通过。过程中修正回填列名与新表 schema 的不匹配（extra/updated_at 列新表不存在）。

## 2026-08-30 桌面端凭据加密存储

**变更内容：** 新增 `database/credential.rs`：机器绑定密钥 + AES-256-GCM 加密。密钥优先存 OS 凭据管理器（keyring crate，实测已写入 Windows Credential Manager `local-db-encryption-key.template-studio-desktop`），不可用时回退用户目录密钥文件（0600）；密文格式 `v1:base64(nonce||ciphertext+tag)`；非 v1 前缀的历史明文解密时原样返回（不破坏存量），下次写入自动升级为密文。接入两个明文存储点：数据源密码（datasource.rs 的 create/update 加密、get_all/get_datasource 解密）、AI api_key（ai.rs 的 upsert 加密、两处查询解密）。

**涉及文件：** `apps/desktop/src-tauri/src/database/{credential,datasource,ai,mod}.rs`（新增+接线）、`apps/desktop/src-tauri/Cargo.toml`（aes-gcm/base64/rand/keyring 依赖）

**验收结果：** credential 单测 2 项通过（中英文+emoji 密码往返、随机 nonce 密文唯一性、历史明文兼容）；desktop 全部 7 单测通过；Windows 凭据管理器确认密钥落位、回退文件未触发。存量明文数据在下次读取时兼容、保存时自动加密升级。

## 2026-08-30 直链下载的 ?token= 认证支持（审计 P0 收尾）

**变更内容：** `extract_token` 扩展：token 请求头缺失且方法为 GET 时回退读取 `?token=` 查询参数（JWT/PAT 字符集 URL 安全，要求不做额外编码）；路由调整——`templates/:id/export` 与 `templates/:id/releases/:version/download` 从公开组移入认证组（导出含未发布草稿内容，版本下载统一语义），消除此前「无法带请求头所以公开」的妥协；前端 `exportTemplate` 的直链 URL 拼 `&token=`。过程中修复路由重复注册 panic（同路径 GET 在两组各留一份导致 Overlapping method route，公开组移除后恢复）。

**涉及文件：** `apps/web/src/middleware/auth.rs`、`apps/web/src/main.rs`、`web/src/api/templates/index.ts`

**验收结果：** 实测矩阵——无 token 直链/版本下载 401、`?token=` 有效令牌 200（导出 19KB、下载 19.6KB）、假 token 401；前端模块编译 200。

## 2026-08-30 优雅停机与 /health 真实化

**变更内容：** ① `/health` 由恒返回静态 OK 改为真实数据库连通探测（AppState 增加 db_pool 字段），返回 `{status, database, timestamp}` JSON，数据库不可达时 HTTP 503（此前数据库挂了探针仍通过）。② 优雅停机：`with_graceful_shutdown` 接入 SIGTERM（Unix）/Ctrl+C 信号处理，停止接收新连接后关闭数据库连接池再退出，供容器滚动更新避免硬切在途请求；关停用 db_pool 句柄在路由 move 前克隆，Arc 解构后调用 close(self)。

**涉及文件：** `apps/web/src/main.rs`、`crates/infrastructure/src/database/pool.rs`（既有 health_check 首次被调用）

**验收结果：** /health 实测返回 `{"database":true,"status":"healthy",...}`（真实探测）；优雅停机编译就位——Windows 下 SIGTERM 信号语义受限无法本机完整验证停机时序，Unix 部署环境（K8s/docker）将正常触发，属已声明的验证边界。

## 2026-08-30 vue-tsc 引入与类型检查落地

**变更内容：** 安装 vue-tsc 1.8.27（Vue 3.5 + TS 4.9 兼容版），新增 `type-check` 脚本；tsconfig 调整（noUnusedLocals/Parameters 暂关——存量代码 24 处未使用告警属风格债、include 排除 build/ 与 vite.config.ts——构建脚本依赖类型不在 devDeps）。首跑 97 错，治理后余 30：api 层默认参数批量标注（Record<string,any>）、近期新代码补类型（user store 登录/getInfo、FooterBar target 收窄、useRenderService 的 renderTree 签名与信封统一时的新签名对齐、AdvancedDrawer 的 localSettings/backupState）、build/vite/proxy.ts 类型修正。剩余 30 个为通用组件层（Table/Form）移植期类型摩擦，产出 `dev-docs/type-debt-inventory.md` 清单（分布/错误类型/修复策略/完整列表）。CI 前端 job 启用 type-check（continue-on-error 非阻断，清单清零后转阻断）。

**涉及文件：** `web/package.json`、`web/tsconfig.json`、`web/src/api/**`（3 文件批量标注）、`web/src/store/modules/user.ts`、`web/src/components/FooterBar.vue`、`web/src/composables/useRenderService.ts`、`web/src/views/editor/components/AdvancedDrawer.vue`、`web/build/vite/proxy.ts`、`.github/workflows/ci.yml`、`dev-docs/type-debt-inventory.md`（新增）

**验收结果：** type-check 可稳定执行（97→30）；关键修复经 vite 编译验证（composable 200）；YAML 校验通过；错误清单落档供专项清理。

## 2026-08-30 CLI 三个 stub 落地

**变更内容：** ① `config set`：支持 server.url / server.api_key / user.author / user.email 四个配置项的写入与持久化（Config::save 落 ~/.cicbyte/template_studio/config/config.toml），未知配置项给出支持列表。② `ai config` 三子命令：show（持久化配置优先，未配置时展示环境变量状态并给出指引）、set（增量更新——未指定字段保留现有值，写回 config 的 [ai] 段，Config 结构新增可选 AiSection）、test（真实 HTTP 连通测试——对 chat/completions 发 8-token 探测请求，成功/HTTP 错误/网络错误三种结果都有明确输出，未配置时回退 AI_API_KEY 环境变量）。③ `ai recommend --project <id>`：CLI 不连数据库按 ID 无从解析，从「暂未实现」改为明确的指引性报错（附正确用法示例）。README 承诺的命令至此全部可用。

**涉及文件：** `apps/cli/src/cli/commands.rs`、`apps/cli/src/config/mod.rs`

**验收结果：** 实测——config set 写入后 config show 正确回显、未知项报支持列表；ai config show 未配置态/配置态两种输出正确；ai config set 持久化 glm 配置后 show 显示（key 脱敏）；ai config test 用假 key 对真实端点发起请求，返回真实的 HTTP 401 与服务端错误信息（链路完整）；recommend 给出明确指引。测试产物已从配置文件清理。

## 2026-08-30 输出剩余工作清单

**变更内容：** 汇总两轮审计、四份专项分析文档与全部 worklog 遗留注记，核对仓库现状（避免已完成项误列），产出分类清单：数据清理（1 项）/ 验证性收尾（3 项）/ 前端类型债（3 项）/ 后端工程质量（8 项）/ 引擎（4 项）/ 桌面端产品缺口（3 项）/ 安全增强（3 项）/ 文档维护（2 项），共 27 项，附优先级建议与详细出处。

**涉及文件：** `dev-docs/remaining-work-inventory.md`（新增）

**验收结果：** 全部条目经仓库现状核对（属主校验 18 处已接、测试版本残留已确认存在、dev-docs 文档清单核对），无已完成项误列。

## 2026-08-30 清单 #1：清理测试版本数据

**变更内容：** 删除模板 1779081291997 的两个测试版本（0.1.0-test/0.2.0-test，事务验证时发布）：先经 API 弃用标记，再 pymysql 直连删除版本表 2 行 + 磁盘发布快照两个目录；API 复验版本数归零。过程中发现无单版本删除接口（仅删模板时级联清理），已将「版本删除接口」作为潜在项归入后端工程质量包。

**涉及文件：** `dev-docs/remaining-work-inventory.md`（#1 标记完成）

**验收结果：** DB 剩余版本 0、快照目录清空、releases API 返回空列表。

## 2026-08-30 前端类型债清零（30→0）与 CI 转阻断

**变更内容：** 清单 #5/#6 完成。零散视图 13 处：ellipsis `{tooltip:true}`→`true` 语义转换（naive 的 tooltip 属性 antd CellEllipsisType 不含，4 处 columns + useColumns 默认值）、App.vue colorPrimary 断言、login register 返回 any、columns 占位函数 null→''、reset-password 查询参数收窄、smtp 异步响应 any、Header eventObject 断言、downloadFile blob null 守卫、AdvancedDrawer 备份 Blob 断言。组件层 17 处：BasicForm componentProps 可选链、useForm 去泛型、editable 的 dataIndex 数组扁平化与 key 回退、EditableCell 点击包装、ActionItem extends Omit（type/onClick 与 antd ButtonProps 冲突）、TableAction/BasicUpload/Modal 的 any 断言、Table getProps 经 unknown 双重转换、maxHeight 动态属性、ColumnSetting checkList 数组化。ci.yml 的 type-check 移除 continue-on-error 转阻断。

**涉及文件：** `web/src/` 下 16 个文件（见各修复点）、`.github/workflows/ci.yml`

**验收结果：** `pnpm run type-check` 退出码 0、零错误；全部改动模块经 vite 编译 200；YAML 校验通过。

## 2026-08-30 fork 反模式重构与 git2 非 Send 修复（清单 #10 部分 + #12 fork 部分）

**变更内容：** 清单 #10/#12 的 fork 相关项：① 移除请求路径内嵌套整个 tokio Runtime 的反模式（`Runtime::new().unwrap()` + block_on 在 async fn 里二次阻塞）——模板名查询与 git 克隆改为当前 async 上下文直接 await，spawn_blocking 包装一并拆除（git 服务自身按需 spawn_blocking）。② 修复 git2 非 Send 类型跨 await：`Repository`/`Signature`（裸指针包装）在 await 期间存活导致 future 不满足 axum Handler 的 Send 约束（此前被嵌套 Runtime 掩盖，拉平后立即暴露）。重构 git 服务的 init_repository/configure_repository：init 后立即取 workdir 并 drop 仓库句柄，configure 改按路径接收（内部需要时重开），Signature 创建移到纯同步提交段。③ release_service 的 `dest.parent().unwrap()` 加守卫。

**涉及文件：** `apps/web/src/handlers/template.rs`、`crates/infrastructure/src/git/service.rs`、`crates/services/src/release_service.rs`

**验收结果：** 编译零错误（#[axum::debug_handler] 辅助定位后移除）；模板核心 52 测试全过；端到端实测 fork 全链路（合法载荷 → 新模板 1788083580182 创建 + 目录落盘 + git 仓库初始化含 HEAD/config）后删除清理、目录无残留。

## 2026-08-30 迁移目录化（清单 #11）：017-021 运行时迁移废除

**变更内容：** main.rs 里约 100 行的运行时迁移块（017 templates 可见性字段 / 018 审核表 / 019 密码重置令牌表 / 020 users.bio / 021 PAT scopes）整体移除——此前以运行时 ALTER/CREATE + `.ok()` 吞错实现，迁移失败无感知且不记录版本号。补齐缺失的 019/020 SQL 文件（019 加 IF NOT EXISTS 防御），migrations/ 目录自此 001-021 完整、由 scripts/migrate.py 统一执行与记录版本。当前库的版本记录缺失项（001-007 历史建表无记录、017-021 运行时迁移无记录）已按实际状态补登（表/字段均已存在的实证核对）。

**涉及文件：** `apps/web/src/main.rs`（移除运行时迁移块）、`migrations/019_create_password_reset_tokens.sql`（新增）、`migrations/020_alter_users_add_bio.sql`（新增）

**验收结果：** migrate.py dry-run 显示「所有迁移已执行」；后端重启日志确认无运行时迁移输出、/health 正常；版本记录表 001-021 完整。

## 2026-08-30 错误码表与 ApiResponse 强化（清单 #8/#9 奠基）

**变更内容：** shared 新增 `ErrorCode` 枚举（Ok/BadRequest/Unauthorized/Forbidden/NotFound/Conflict/TooManyRequests/Internal，`code()` 与 HTTP 语义同步、附 as_str 标识），ApiResponse 补 `success_msg`/`success_with_message` 构造与文档注释（唯一信封的构造入口，禁手写 json! 的约束写入注释）；`access.rs` 属主校验的两处错误信封改经 ErrorCode 产生（示范接入点）；auth 中间件的错误信封加注释说明 401/403 保留 result 字段的前端兼容原因。附 3 个单测（成功信封、错误码-HTTP 映射、序列化形态）。

**涉及文件：** `crates/shared/src/utils/response.rs`、`apps/web/src/handlers/access.rs`、`apps/web/src/middleware/auth.rs`

**验收结果：** shared 7 测试全过；后端重启正常；无 token 的属主校验返回标准 401 信封。存量约 120 处 json! 手写信封的批量迁移（机械替换为 ApiResponse 构造）为独立后续项——本项已把「入口、错误码表、单测」三要素就位。

## 2026-08-30 账号级登录失败锁定（清单 #24）

**变更内容：** IP 级限速之外的账号级第二道闸：连续 5 次密码错误锁定账号 15 分钟（锁定期内正确密码也拒绝并提示剩余时间），登录成功即清零计数。实现：迁移 022 加 `failed_login_count`/`locked_until` 列（migrate.py 执行）；User 模型与 find_by_username 补两列；user_repository 新增 record_login_failure/clear_login_failures/lock_user_until；auth_service.login 接入锁定检查（锁定中拒绝）/失败累计（达阈值锁定并告警）/成功清零。

**涉及文件：** `migrations/022_alter_users_add_login_lockout.sql`（新增）、`crates/shared/src/models/user.rs`、`crates/repositories/src/user_repository.rs`、`crates/services/src/auth_service.rs`

**验收结果：** 完整周期实测——5 次错误密码（每次正确报错）→ 第 5 次触发锁定 → 正确密码被拒并提示「约 15 分钟后再试」→ DB 确认计数 5/锁定时间 → 手动解锁 → 正确密码登录成功且计数清零。过程中修复 sqlx 不支持单 execute 多语句的问题（UPDATE+SELECT 拆两条）。JWT 有效期缩短（#23）因前端「记住登录」依赖 7 天存储时长需联动设计，未在本轮盲改。

## 2026-08-30 操作审计日志（清单 #25）

**变更内容：** 迁移 023 建 audit_logs 表（user_id/username 冗余/action/resource_type/resource_id/detail/ip/user_agent/created_at，四组索引）；新增 `audit_service`（record 失败仅告警不阻断业务——审计是旁路关注点；list 按动作/资源类型过滤分页查询单测级实现）；AppState 注入 audit_service；四个关键操作接入审计：template.delete、release.publish、release.rollback、user.delete（user_management 的 delete_user 顺带补上 Extension<AuthUser> 提取）。

**涉及文件：** `migrations/023_create_audit_logs.sql`（新增）、`crates/services/src/audit_service.rs`（新增）及 lib.rs、`apps/web/src/main.rs`、`apps/web/src/handlers/{template,releases,user_management}.rs`

**验收结果：** 端到端实测——创建临时模板→删除→audit_logs 表出现 (admin, template.delete, template, 1788085851173, 时间戳)；编译零错误。审计查询 API（admin 界面展示）未在本轮（record 链路为先），作为后续增量。

## 2026-08-30 列表接口 N+1 治理（清单 #13）

**变更内容：** 仓库层新增 `get_languages_for_templates`（一次 IN 查询批量取多模板语言关联，按 template_id 分组）；服务层四个逐行查语言的循环点改为两阶段（批量取 + 内存组装）：list_templates（templates_list）、list_public_templates、list_user_templates、get_featured_templates（featured 循环同时治理分类逐行查询——分类按 ID 去重后预取缓存）。每行 2 个 owner 子查询因已在单条 SQL 内（数据库端关联，非应用层 N+1）保持不动。

**涉及文件：** `crates/repositories/src/template_repository.rs`、`crates/services/src/template_service.rs`

**验收结果：** 编译零错误、模板核心 52 测试全过；三个列表接口回归（templateList 7 条/公开 4 条/我的 7 条），语言关联数量与改造前一致。核查确认列表返回的 language name 为 null 属存量行为（TemplateLanguageItem 本无 name 字段），非本次引入——完整语言名填充可复用此前已实现的 get_template_language_details，作为后续增强。

## 2026-08-30 文档维护（清单 #26/#27）

**变更内容：** ① `template-studio-dev` skill 新增「关键机制」段：大修后的八条约定（统一信封+ApiResponse/ErrorCode、属主校验必须接入、token 头+?token=+限速+账号锁定、审计旁路写入、type-check CI 阻断、safe_join 路径安全、迁移只走 migrate.py 禁运行时迁移、统计保持真实聚合）。② 审计报告修复进度全量回填：P0×12 全完成、P1×5 全完成、P2 引擎全完成、P3 部分完成（N+1 ✅、ApiResponse 奠基、git 深度异步化待），另列审计后新增的修复批次与疑点勘误。

**涉及文件：** `.zcode/skills/template-studio-dev/SKILL.md`（skill 文件不随版本管理，无提交）、`dev-docs/project-deep-audit-2026-08.md`、`dev-docs/remaining-work-inventory.md`

**验收结果：** skill 内容与当前代码核对一致；审计报告进度与 worklog 逐项对得上。

## 2026-08-30 ApiResponse 批量迁移（清单 #8 主体完成）

**变更内容：** 120 处手写 json! 成功信封批量迁移为 ApiResponse 构造（success_with_message 75 处 + success_msg 38 处，21 个 handler 文件），Json<Value> 签名的 handler 经 serde_json::to_value 桥接。多轮机械转换的边角修正：裸对象字面量补 json! 包裹（34 处）、format! 条件消息的两处所有权顺序、悬空多行匹配。剩余 7 处 json! 信封为特殊形态（multipart 上传响应、builtin/engine 常量构造、SSE 流），保持手写合理。过程中发现并修复 find_by_id 漏带锁定策略新列导致的用户信息 500（上一轮加列时只改了 find_by_username）。

**涉及文件：** `apps/web/src/handlers/*.rs`（21 个）、`crates/repositories/src/user_repository.rs`

**验收结果：** 编译零错误；信封回归实测——登录（code:0 + data 含 roles/token）、公开分类、用户信息（修复后 code:0）、统计总览均正常。新增代码从「禁手写 json!」的约定变为存量已基本遵循。

## 2026-08-30 编译警告清零与死代码清理

**变更内容：** ① 清理 5 处 unused import（services 的 render_string、access.rs 的 State、main.rs 的 IntoResponse/Response、routes/auth.rs 的 put）；② 删除 git service 的两个死方法（async create_initial_commit 委托壳——sync 版在用；clone_repository_local——被 clone_and_clean 的 Send 安全重写替代）。

**涉及文件：** `crates/infrastructure/src/git/service.rs`、`crates/services/src/template_render_service.rs`、`apps/web/src/handlers/access.rs`、`apps/web/src/main.rs`、`apps/web/src/routes/auth.rs`

**验收结果：** `cargo check -p template-studio-web` 零警告零错误（含全部依赖 crate）；后端重启正常、/health 通过。

## 2026-08-30 clippy 清理（60→23）

**变更内容：** ① `cargo clippy --fix` 自动修复 27 条（needless_return、redundant clone 等）；② 手工修复 10 条：`if let Some(_) =` → `is_some()`（3 文件 5 处）、冗余 `as i64` cast（bind 参数）、useless `format!`、变量分析服务的**循环内正则编译**提升为 `LazyLock` 静态（模板分析每次循环重复编译正则，分析大模板时的隐性热点）。③ 误删恢复：`total as u32` 的 cast 不是冗余的（COUNT 返回 i64，PagedResponse 要 u32），编译期抓到后恢复。剩余 23 条为 &PathBuf→&Path 签名改动（4 处，影响面广）、too many arguments（2 处，需重构参数结构体）、少量 or_insert_with 等低价值项——保留给日常迭代消化。

**涉及文件：** `crates/services/src/{category,language,var_preset,template_analysis}_service.rs`、`crates/repositories/src/template_repository.rs`、`apps/web/src/{main,handlers/template}.rs`（自动修复涉及面更广）

**验收结果：** 全部测试通过（template_core 52 + services 2 + web 3）；编译零警告零错误；变量分析接口回归 200（正则提升后功能不变）。

## 2026-08-31 桌面端模板编辑器立项（方案 A）+ 阶段1：API 客户端与认证打通

**变更内容：** ① 立项文档 `dev-docs/desktop-editor-plan.md`（现状盘点、三项关键决策：PAT+token 头双客户端并存 / Tauri 内存渲染引擎 / 独立顶层路由，六阶段拆分）；② 新增 `apps/desktop/src/utils/apiRequest.js`——语义与 web 端 `utils/request.ts` 对齐（`token` 头注入、`code!==0` 拦截、blob 透传、qs 序列化、401 引导到设置页而非跳登录），存量 `utils/request.js` 契约不动；③ 设置页 API 密钥字段改为 API Token 说明（指引 Web 端「个人中心」创建 ts_pat_ 令牌）；④ 修复桌面端幽灵依赖：`qs` 新增、`lodash-es` 与 `@lezer/highlight` 补声明（后者导致 vite build 失败）。

**涉及文件：** `dev-docs/desktop-editor-plan.md`、`apps/desktop/src/utils/apiRequest.js`（新增）、`apps/desktop/src/views/settings/WebServerSettings.vue`、`apps/desktop/package.json`

**验收结果：** PAT 全链路实测——创建 PAT（全 7 scope）→ `token` 头调编辑器真实端点 `/api/v1/editor/templateFiles/fileTree` 返回 200（12 文件），无 token 401；`pnpm build`（桌面前端）通过。

## 2026-08-31 桌面端编辑器阶段2：Tauri 内存渲染引擎

**变更内容：** ① Rust 侧新增三个 Tauri 命令：`render_files`（内存文件集整树渲染——先按生成条件过滤再 render_tree，与 WASM/服务端语义一致）、`render_string_content`（单字符串渲染）、`get_render_engine_info`（版本/过滤器/内置函数）；引擎 `initialize()` 经 `Once` 保证单次注册。② `template_core` 新增 `VERSION` 常量导出（引擎信息上报引擎自身版本而非宿主版本）。③ 前端新增 `apps/desktop/src/services/render/TauriEngine.ts`（implements RenderEngine，与 WasmEngine 同构；错误类型字段为 core 原生 `type` 而非 WASM 包装层 `error_type`）+ 移植 `services/types.ts`。④ 修复存量 bug：桌面 `render_template` 命令按驼峰读 `isDirectory`/`filePath`/`fileContent`，而 core::RenderedFile 序列化为 snake_case，导致 PreviewPane 预览永远为空——改为 snake_case 字段名。

**涉及文件：** `apps/desktop/src-tauri/src/commands/template.rs`、`apps/desktop/src-tauri/src/lib.rs`、`crates/template_core/src/lib.rs`、`apps/desktop/src/services/types.ts`（新增）、`apps/desktop/src/services/render/TauriEngine.ts`（新增）

**验收结果：** 命令级单测 4/4 通过（render_files 变量渲染+条件剔除、单文件语法错误不破坏整树、render_string_content 结果形状、引擎信息含过滤器注册）；`cargo check`（desktop）零错误；桌面前端 `pnpm build` 通过。

## 2026-08-31 桌面端编辑器阶段3：编辑器 API 模块移植

**变更内容：** ① web 端 8 个编辑器 API 模块移植至 `apps/desktop/src/api/editor/`（templates / templateFiles / templateExpose / templateVariablePresets / conditions / releases / builtinFunctions / backup），请求层统一换 `utils/apiRequest.js`（与 web 端信封语义一致）；与桌面存量旧客户端同名模块（templates/releases/templateFiles）以子目录隔离。② `templates/exportTemplate` 适配桌面差异：页面与 API 不同源，导出直链改为拼接服务端绝对地址、token 取自设置页 API Token。③ `templates/contribution.ts`（我的模板管理，Alova 客户端）编辑器未引用，留待阶段5 入口需要时移植。

**涉及文件：** `apps/desktop/src/api/editor/**`（8 个模块新增）

**验收结果：** 全部模块 TS 语法批检通过；每模块抽一个端点带 PAT 实测——detail(fileTree/variables/data/variables/test/preset-variables/file-conditions/releases/builtin-functions) 均 200 code:0，backup 以缺参请求证实路由与认证可达（400 missing templateId）。发现并记录：`templateExpose` 模块内 `expose/versions` 函数指向后端不存在的路由（web 端遗留，编辑器视图未引用）。

## 2026-08-31 桌面端编辑器阶段4：services 层与编辑器视图族移植

**变更内容：** ① services 层：`EngineManager` 桌面版（TauriEngine 占据 web 版 WASM 的本地引擎槽位，公共 API 沿用 wasm 命名以零改动复用视图）、`BackendEngine` 适配（服务端地址从 configStore 动态解析、请求带 token 头）、`RenderService`/`types`/`useRenderService` 原样移植；`storage/`（IndexedDB WASM 缓存）不适用桌面未移植。② 视图族 28 文件整体复制至 `apps/desktop/src/views/editor/`，导入改写：`@/api/*` → `@/api/editor/*`、`@/store/modules/templateFileStore` → `@/stores/templateFileStore`。③ 桌面适配三处：`App.vue` 增加独立全屏页分支（编辑器不套 AppLayout）、编辑器关闭统一返回 `/templates`（桌面无后台管理页）、AdvancedDrawer 引擎名判断与文案（WASM→本地引擎）。④ 补齐 4 个幽灵依赖：@codemirror/search、@codemirror/autocomplete、js-yaml、file-saver。

**涉及文件：** `apps/desktop/src/services/**`、`apps/desktop/src/views/editor/**`（28 文件）、`apps/desktop/src/icons/ionicons5.ts`、`apps/desktop/src/stores/templateFileStore.ts`、`apps/desktop/src/composables/useRenderService.ts`、`apps/desktop/src/router/index.js`、`apps/desktop/src/App.vue`、`apps/desktop/package.json`

**验收结果：** `pnpm build` 全量通过；浏览器冒烟（vite dev + hash 路由 `/#/editor/1770799783109`）：编辑器完整渲染（文件树/变量/设置侧栏、编辑面板、空态），无 Tauri 环境下 401 引导文案精确显示——验证了路由、独立布局分支、运行时无导入错误、CORS 放行、信封错误路径与引擎回退。数据成功路径待真实 Tauri 环境（阶段6）。

## 2026-08-31 桌面端编辑器阶段5：模板广场编辑入口

**变更内容：** ① `contribution.ts`（我的模板 API：列表/创建/更新/删除/提交审核）移植到 `api/editor/templates/contribution.ts`（apiRequest 版，路径 `/api/v1/admin/my/templates/*`）。② 模板广场（`views/templates/index.vue`）新增编辑入口：工具栏「新建模板」按钮 + 卡片「编辑」图标，均以 `configStore.hasApiKey`（设置页已配置 API Token）为显隐条件。③ 新建模板弹窗：名称/类型（`getTemplateTypes` 动态加载）/分类/主语言/描述，创建成功取 `data.id` 跳转 `/editor/:id`；载荷对齐后端 `CreateTemplateRequest`（visibility private、languages 数组）。

**涉及文件：** `apps/desktop/src/api/editor/templates/contribution.ts`（新增）、`apps/desktop/src/views/templates/index.vue`

**验收结果：** 表单载荷实测往返——创建（code:0 得到新模板 id）→ 删除（code:0）验证载荷形状正确；类型接口返回 `templateTypes: [{value,label,description}]` 与下拉渲染匹配；浏览器验证广场正常渲染（分类/语言/4 张模板卡片）、无 PAT 时新入口按设计隐藏。带 PAT 的入口展示待真实 Tauri 环境（阶段6）。

## 2026-08-31 桌面端「我的模板」管理页（用户反馈：登录态专属菜单）

**变更内容：** ① 新增 `views/my-templates/index.vue`（参照 web 前台我的模板页重写）：状态筛选（全部/草稿/待审核/已发布）、卡片列表（状态徽章 + 悬浮操作：编辑内容/修改信息/Fork）、右键菜单（提交审核/撤回/删除）、创建/编辑/Fork 三个弹窗；样式改用桌面 CSS 变量适配深色主题。② 侧边栏 `NavigationMenu.vue` 新增「我的模板」菜单项，以 `configStore.hasApiKey` 显隐（与模板广场入口同一登录态判定）。③ 路由 `/my-templates`。④ 移植修复：web 版「撤回」操作误用弹窗里可能过期的 `formData` 构造载荷，桌面版改为基于菜单命中的模板对象构造。

**涉及文件：** `apps/desktop/src/views/my-templates/index.vue`（新增）、`apps/desktop/src/components/layout/NavigationMenu.vue`、`apps/desktop/src/router/index.js`

**验收结果：** `pnpm build` 通过（期间修正一处图标名：`GitForkOutlined` 在 icons-vue 7.x 不存在，改 `ForkOutlined`）；`vite preview` 实测页面渲染完整（标题/筛选/空态/创建按钮），无 PAT 时菜单项与数据请求按设计降级（菜单隐藏 + 401 引导文案）；列表与 fork 接口响应形状带 PAT 实测核对（`{templatesList,total}`、fork 返回裸 id）。

## 2026-08-31 桌面端编辑器阶段6：真实环境回归与收尾修复

**变更内容：** tauri:dev 真实环境回归中暴露并修复三个问题：① 「我的模板」列表解析层级错误——`res?.data?.templatesList` 应为 `res?.data?.data?.templatesList`（apiRequest 返回 axios response，业务数据在信封第二层），请求成功但解析取空，页面呈误导性「还没有创建模板」空态；全项目排查同层级错误，编辑器 28 视图（`response.data.data`）与广场旧客户端（单层 `.data`）均正确，仅此一处。② 我的模板卡片样式与模板广场对齐：140px 深色渐变视觉区 + 微光动画、语言标签行、按主语言生成代码片段（Rust/Go/Python/JS/TS 分支）、5 列网格、状态徽章改玻璃质感。③ 加载 Spin 贴顶：a-spin 包空网格高度塌陷，容器加 320px 最小高度垂直居中（模板广场存在同类存量问题，未在本期处理）。

**涉及文件：** `apps/desktop/src/views/my-templates/index.vue`

**验收结果：** 用户实测数据加载正常（8 模板）；修复后构建通过、tauri 会话日志零 error/panic；模板广场/设置/项目等存量页面回归无异常。桌面端模板编辑链路（清单 #20）至此完整落地：PAT 认证 → 我的模板管理 → 编辑器（文件树/内存渲染/发布）→ 广场消费。

## 2026-08-31 模板广场加载 Spin 贴顶修复

**变更内容：** 模板广场页与「我的模板」页存在同款存量问题——`a-spin` 包裹的网格在加载时为空、高度塌陷导致 spinner 贴顶；补同款修复（`.ant-spin-nested-loading`/`.ant-spin-container` 加 320px 最小高度，spinner 在内容区垂直居中）。

**涉及文件：** `apps/desktop/src/views/templates/index.vue`

**验收结果：** `pnpm build` 通过；Tauri 运行窗口经 HMR 生效，加载态 spinner 居中。

## 2026-08-31 设置页 UI 壳持久化（清单 #22）

**变更内容：** ① 新增 `stores/uiSettings.js`：localStorage 持久化的 pinia store（key `ui-settings-v1`），含默认值深合并（新增字段自动补默认、不丢用户已有设置）与 resetAll；经 pinia 插件 `$subscribe`（detached）实现任意变更自动落盘，插件在 `stores/index.js` 注册。② 5 个纯 UI 壳设置页接线到 store（安全/网络/行为/调试/实验性，共 16 个开关绑定），模板层零改动、仅替换 script 的本地 reactive。勘误：KeyboardShortcutsSettings 原本就有 localStorage 持久化，清单误记为壳。

**涉及文件：** `apps/desktop/src/stores/uiSettings.js`（新增）、`apps/desktop/src/stores/index.js`、`apps/desktop/src/views/settings/{AdvancedSecurity,AdvancedNetwork,GeneralBehavior,AdvancedDeveloperDebug,AdvancedDeveloperExperimental}Settings.vue`

**验收结果：** 浏览器实测闭环——默认值渲染 → 点击开关 localStorage 立即写入完整状态（含全部 5 个分区）→ 刷新后开关状态保留；`pnpm build` 通过。注：部分开关（沙盒/CSP/代理/调试端口）当前仅存储偏好，生效逻辑待功能实现时接线。

## 2026-08-31 小批三件：死代码清理 / 连接池调优（#14）/ 请求日志与 trace-id（#15）

**变更内容：** ① 清理 `getTemplateExposeVersions` 死函数——指向后端不存在的 `/expose/versions` 路由且无调用方，web 与桌面两份模块同步删除。② 连接池显式调优：`MySqlPool::connect`（全默认）改为 `MySqlPoolOptions`——配置里的 `max_connections` 首次真正生效（此前被静默忽略），另加 acquire 5s 超时、连接最长寿命 30min、空闲 10min 回收。③ 新增 `request_log` 中间件：每请求生成 trace-id（UUID v4）放入 `request` span（handler 日志自动内联关联）并经 `x-trace-id` 响应头回传，记录 method/path/status/耗时；`/health` 不记日志；CORS 补 `Access-Control-Expose-Headers` 让跨域前端可读取该头；中间件注册在 CORS 层之外（认证/CORS 拒绝的请求也有日志）。

**涉及文件：** `web/src/api/templateExpose/index.ts`、`apps/desktop/src/api/editor/templateExpose/index.ts`（删除死函数）；`crates/infrastructure/src/database/pool.rs`；`apps/web/src/middleware/request_log.rs`（新增）、`apps/web/src/middleware/mod.rs`、`apps/web/src/main.rs`

**验收结果：** cargo check 零错误、web 测试 3/3 通过；后端重启实测——响应头 `x-trace-id` 返回且同 ID 出现在 handler 日志 span、请求日志含 `status=200 elapsed_ms`、`access-control-expose-headers: x-trace-id` 生效、/health 无日志噪音。

## 2026-08-31 桌面端视觉重构：AgentHub/HiFox 视觉语言

**变更内容：** 引入 `dev-docs/prototype`（AgentHub 原型）的视觉语言，仅桌面端。① **令牌换血**：`variables.css` 整体重写——暖灰画布（浅 `#f1f1ee`/暗 `#141519`）+ 白色面板 + 单色主操作（浅黑 `#1b1c1f`/暗反转白 `#f2f2ef`）+ 品牌绿 `#16a34a`（仅强调）；状态色对齐原型；新增画布/品牌/面板阴影（resting/breathe/hover 三态，暗色对应翻转）。变量名全部保留只换值，零组件改名。② **AntD 主题令牌**（App.vue）：colorPrimary 单色化、暗色补 `colorTextLightSolid` 反转（白底主按钮黑字，AntD 默认白字会消失）、borderRadius 8。③ **壳层浮卡**（AppLayout）：画布底 + 顶栏/侧栏/内容三张悬浮圆角卡（8px 栏间缝）+ `panelBreathe` 8s 呼吸动画（关键帧用令牌驱动以支持暗色翻转）。④ **全局细节**：细滚动条（两主题）、品牌绿选区与焦点环。⑤ **蓝色残留清理**：18 个文件 28 处硬编码旧蓝——logo→品牌绿、语法/调色板→info 蓝、推荐徽标→黑、微光/悬停中性化、useTheme/DisplaySettings 同步新令牌。

**涉及文件：** `apps/desktop/src/assets/styles/{variables,themes}.css`、`src/App.vue`、`src/components/layout/AppLayout.vue` + 蓝色清理涉及的 18 个组件/视图

**验收结果：** `pnpm build` 通过；计算样式实测浅/暗两主题的关键令牌（画布/面板/主操作/品牌绿）与浮卡几何（三卡 top 8px、圆角 14、动画运行）全部正确；视觉走查发现的「推荐徽标蓝残留」已修。遗留：ProjectWorkspaceLayout（/project/* 工作区）暂未浮卡化，见 `dev-docs/desktop-visual-refactor.md`。

## 2026-08-31 侧边栏重构：自绘导航替换 AntD 菜单

**变更内容：** 按原型侧栏模式重写导航。① `NavigationMenu.vue` 整体重写：抛弃 `a-menu`（此前靠约百行 `!important` 覆盖 AntD 默认样式，呈现"两不像"），改为自绘导航——三个小节（工作台/代码生成/系统，11px 小节标题）+ 平铺条目（30px 行高、7px 圆角胶囊、图标灰/选中深、单色语言）；「代码生成器」折叠子菜单拍平为小节分组（菜单量少，常显优于折叠）；「设置」并入系统小节；折叠态收为图标条目 + tooltip。② `Sidebar.vue` 简化：logo 去掉双层文字/渐变蓝（#2196f3 残留）/浮动动画，底部动作条只留帮助 + 主题切换（设置按钮移除，避免与导航重复）。③ 侧栏底色调为原型的 `#fbfbfa`（暗色 `#1c1d21`），与内容卡的纯白拉开微妙层次。

**涉及文件：** `apps/desktop/src/components/layout/NavigationMenu.vue`（重写）、`apps/desktop/src/components/layout/Sidebar.vue`、`apps/desktop/src/assets/styles/variables.css`

**验收结果：** DOM 实测——三小节 8 条目结构正确、路由选中态（/templates → 脚手架）正确、nav-item 规格 30px/7px 符合原型、无 AntD 菜单残留、折叠态 8 个图标条目且小节标题隐藏、宽度 60px 可还原；`pnpm build` 通过。

## 2026-08-31 项目工作区布局适配视觉语言

**变更内容：** `ProjectWorkspaceLayout.vue`（/project/* 路由的独立壳层）对齐主壳层浮卡语言：① 壳层改为画布底 + 侧栏/顶栏/内容三张悬浮圆角卡（8px 缝隙、panelBreathe 呼吸、侧栏 240/60px 折叠保留）；侧栏从 fixed 定位改回文档流（去掉 main-area 的内联 marginLeft 补偿）。② 菜单 a-menu → 自绘导航（与主侧栏同规格 30px/7px 胶囊），selectedKeys + watch 路由同步逻辑简化为 activeKey 计算属性；logo 同步简化（去双层/渐变蓝残留）。③ 底部动作条/页脚跟随新样式。

**涉及文件：** `apps/desktop/src/components/layout/ProjectWorkspaceLayout.vue`

**验收结果：** DOM 实测——画布色 #f1f1ee、三张浮卡（圆角 14、sidebar 240px 文档流、呼吸动画运行）、4 个导航条目且路由选中态（/project/:id/tables → 表管理）正确、无 AntD 菜单残留；`pnpm build` 通过。数据链路（项目加载）在真实 Tauri 环境随日常使用观察。

## 2026-08-31 导航选中/悬浮态定稿：柔和层次（调研驱动）

**变更内容：** 用户反馈黑色实心选中胶囊"太突兀"，调研 Notion/Linear/Vercel 的侧栏选中范式后定稿：**低透明度底色 + 文字加重表达位置，不用色块**。落地：新增导航专用令牌 `--color-nav-hover`（约 5% 灰，浅 `#ececea`/暗 `#2c2e34`）与 `--color-nav-active`（约 9% 灰，浅 `#e3e3df`/暗 `#373941`）——不挪用全局 hover/active（被按钮等组件共用）；选中态字重 550、悬浮轻于选中一档、过渡 120ms、无位移缩放。主侧栏与工作区侧栏两处同步。

**涉及文件：** `apps/desktop/src/assets/styles/variables.css`、`apps/desktop/src/components/layout/{NavigationMenu,ProjectWorkspaceLayout}.vue`

**验收结果：** 计算样式实测两主题选中色正确（浅 `#e3e3df`/暗 `#373941`，120ms 过渡）；期间发现的"暗色不翻转"经最小探针证实为内嵌浏览器渲染同步假象（CSSOM 与变量链路均正确），非代码问题。

## 2026-08-31 AI 品牌图标：SVG 字标替换机器人图标

**变更内容：** 用户提供「✦ AI」品牌 SVG（48x22，currentColor），新增组件 `components/icons/AiIcon.vue`（size 属性控制高度、宽度按比例、颜色跟随文字色），替换全部 9 处机器人图标：AI 助手面板（触发按钮/头部/消息头像/发送位）、设置主侧栏「AI 服务」入口、设置子侧栏 AI 提供商图标映射（glm 与 default 兜底分支）、表管理页「AI 建表」按钮、模板渲染 AI 变量面板。

**涉及文件：** `apps/desktop/src/components/icons/AiIcon.vue`（新增）、`components/AiAssistant.vue`、`components/settings/{SettingsSidebar,SettingsSubSidebar}.vue`、`views/project/tables/index.vue`、`views/template-render/components/AiVariablePanel.vue`

**验收结果：** `pnpm build` 通过；全仓 `RobotOutlined` 残留清零；设置页实测 AiIcon 渲染正确（14px 高按 48:22 比例、currentColor 生效）。

## 2026-08-31 主题切换移顶栏 + 底部条收敛（含误恢复找回）

**变更内容：** ① 新增 `ThemeToggleButton.vue`：自绘太阳/月亮细线条 SVG（currentColor、16px、1.8 描边），接入主顶栏右上角（通知中心左侧）与工作区顶栏（窗口控制左侧），浅色显太阳、深色显月亮，tooltip 提示目标主题。② 侧栏底部收敛：主侧栏移除「帮助」与「主题切换」条目（帮助整体去掉，主题移至顶栏），底部仅剩登录态身份卡，未配置 Token 时整个底部区隐藏；工作区侧栏保留「设置」（其导航无此项）+ 身份卡，同样移除帮助/主题。③ 清理随之失效的图标导入与死代码（openHelp/goToHelp/toggleTheme/isDark/themeStore 引用）。④ 找回此前 git 误恢复的损失：`</>` logo（两处侧栏）与身份卡接线（两处）重新应用。

**涉及文件：** `apps/desktop/src/components/layout/ThemeToggleButton.vue`（新增）、`Navbar.vue`、`workspace/WorkspaceHeader.vue`、`Sidebar.vue`、`ProjectWorkspaceLayout.vue`

**验收结果：** `pnpm build` 通过；实测：主题按钮位于顶栏右上（浅色态太阳图形）、底部帮助/主题条目清零、身份卡隐藏逻辑保持、logo 为干净字标。

## 2026-08-31 卡片样式全局统一

**变更内容：** 用户反馈「卡片样式全局不一致（模板渲染页不一样、有的有作者头像有的没有）」。盘点全应用卡片家族后统一到以模板广场为规范的基准：① 模板渲染页卡片对齐规范（圆角 8→令牌 12、视觉区 120→140、内容内距/字号全套对齐、修掉未定义变量 `--color-bg-container` 与残留蓝 hover 背景）；② 悬浮交互统一（projects/datasource 的 -6px 位移→规范 -4px，补齐 hover 边框强调 border-strong）；③ 卡片静止阴影全部移除（统一由边框承托，悬浮时出阴影——原先 6px16/12px24/2px8/8px24 四种写法并存）；④ 硬编码圆角（8px/12px）收敛到令牌。作者头像策略：仅模板广场（市场语义，展示属主）保留，渲染页结构同源保留，我的模板（全是本人）不展示——此为有意设计而非遗漏。

**涉及文件：** `views/template-render/index.vue`、`views/projects/index.vue`、`views/datasource/index.vue`、`views/languages/components/LanguageCard.vue`、`views/home/components/{StatsSection,RecentProjectsList}.vue`、`views/project/components/StatsGrid.vue`

**验收结果：** `pnpm build` 通过；模板渲染页卡片与广场卡同构（视觉区/内距/字号/圆角一致）。

## 2026-09-01 编辑器暗色主题修复：变量族接管全部硬编码

**变更内容：** 用户反馈暗色下编辑器（含变量栏）仍有白底。首轮已定义 `--editor-*` 变量族映射应用令牌，但编辑器视图还有约 180 处 web 移植带来的硬编码色。本轮按语义全量映射（仅 background/border/color 声明位，彩色语义标记与渐变不动）：浅底（#fafafa/#f5f5f5/#f1f5f9/#f8f9fa/#fafbfc/#f0f0f0）→ 新增 `--editor-inset-bg`；白底 → panel；边框灰（#e0e0e0/#e2e8f0/#e8e8e8）→ border；文字灰（#333/#1e293b → primary，#666/#64748b/#999/#94a3b8 → muted）。24 个文件映射 + 4 处边角（滚动条/标签/hover）补齐；有意保留的彩色设计（蓝色激活标签、深色 settings-card）不动。

**涉及文件：** `apps/desktop/src/assets/styles/variables.css`、`apps/desktop/src/views/editor/**`（28 文件）

**验收结果：** `pnpm build` 通过；全量正则核查编辑器内浅色声明已全部走变量（含兜底）；暗色下编辑器各面板（文件树/变量栏/预览/抽屉）随主题翻转。

## 2026-09-01 编辑器暗色收尾三件：染色底 / 抽屉传送选择器 / 简写边框

**变更内容：** ① 变量栏带色相的浅染色底（分组标题浅灰渐变、悬浮浅蓝、预设浅紫、必填浅红/浅绿徽章）改为双主题安全形式：中性走编辑器变量、彩色走半透明 rgba。② 全量渲染抽屉错误面板压平（浅琥珀底 → 半透明 + 压掉 collapse 默认 10px 圆角盒）；期间发现并修复隐蔽 CSS 编译坑：`:deep(.a .b)` 要求 `.a` 祖先带作用域属性，抽屉 teleport 场景下永不匹配，改为 `.a :deep(.b)` 形式。③ 简写形式的边框硬编码（`border-bottom: 1px solid #e8e8e8`）此前正则漏网（只匹配了颜色直接跟随声明的形式），22 个文件补齐映射——这是暗色下"白线"的来源。

**涉及文件：** `views/editor/**`（约 25 文件）、`assets/styles/variables.css`

**验收结果：** `pnpm build` 通过；抽屉内分隔线全量枚举验证：旧 #e8e8e8 全部收敛到编辑器边框变量（浅 #e9e9e7/暗 #2c2e35），残留未包裹计数 0。

## 2026-09-01 快速设计器（Variable Studio）颜色体系收编

**变更内容：** 用户反馈快速设计器（Variable Studio，QuickDesignDrawer 组件族）暗色下白色浅色混合割裂。审计发现两类残留：CSS 声明里的非灰阶色（绿色文字/边框 #18a058、蓝边框 #007acc 等此前灰阶映射不覆盖）与模板内联样式（此前正则只扫 CSS 块）。专项清理：绿色 → `--editor-accent`、蓝 → `--color-info`、灰阶文字/边框/浅底 → 对应编辑器变量、浅绿底 → 半透明绿，共 12 处替换 + 2 处收尾（JS 对象样式、空状态图标）。语义色（红 #ff4d4f/琥珀 #faad14）保留。

**涉及文件：** `views/editor/components/QuickDesignDrawer/**`（7 文件）

**验收结果：** `pnpm build` 通过；目录内非语义色声明全部走变量（含内联样式）。

## 2026-09-01 Variable Studio 三栏布局重构与面板宽度再平衡

**变更内容：** 用户反馈 Variable Studio（快速设计器）中变量栏被挤压 / 设计模式下面板过宽。根因：`calculateColumnWidth()` 把三栏一律 JS 三等分（每栏 33.33%），设计列被压到 426px，内部面板连环挤压。重构为角色分配：设计列 `flex:1` 弹性生长（保底 420px），Schema/表单预览列固定 300px，删除三处内联三等分宽度；内部面板再平衡——变量树 240px 不可压缩（顺带修复其未消费父组件 width prop 导致拖拽宽度刷新丢失的 bug）、组件库 240→220px 不可压缩、属性面板 320→260px 可压缩（保底 200px）、Schema 编辑器补 min-width:0 自行滚动。

**涉及文件：** `views/editor/components/QuickDesignDrawer/index.vue`、`components/{VariableTree,ComponentLibrary,PropertyPanel}.vue`

**验收结果：** 实测设计模式分配：设计列 680（库 220 / 属性 260 / 画布 199→弹性）+ 预览列各 300；变量树模式：树 240 稳定 + Schema 编辑器弹性。窄窗口下牺牲顺序为预览栏→属性面板，工作区最后。

## 2026-09-01 Variable Studio 列宽可拖拽 + 空画布拖放区

**变更内容：** 用户多轮反馈列宽不合适（组件库/变量栏"太大"、面板间"间距"），静态数值调整无法收敛，改为用户自调节：① Schema 列与表单预览列增加左缘拖拽手柄（5px 热区、悬停品牌绿高亮、拖拽范围 220-520px，默认 Schema 260/表单 300）；② 属性面板同款手柄（范围 200-420，默认 260）；③ 组件库 220→190px；④ 空画布改为整幅虚线拖放区（原居中小空态在宽画布中呈现为莫名空白——用户感知的"属性与预览之间的间距"实为空画布/空内容列）。设计模式默认仅设计区全宽（Schema/表单按需点开）。

**涉及文件：** `views/editor/components/QuickDesignDrawer/index.vue`、`components/{PropertyPanel,ComponentLibrary,DesignCanvas}.vue`

**验收结果：** `pnpm build` 通过；实测抽屉内 3 个拖拽手柄就位、Schema 列内联 width 260 生效、拖拽逻辑（mousedown/mousemove/mouseup + 范围钳制 + body 光标）完整。

## 2026-09-01 Variable Studio 布局体系定稿：v-show 修复 + 全栏可拖 + 内容驱动默认值

**变更内容：** 多轮宽度反馈后的系统性定稿。① 修复关键 bug：`.layout-column { display: flex !important }` 压过 v-show 的内联 display:none，导致点掉 Schema/表单后列壳残留占位、空间不重分配（用户反馈"凭空消失"）——去掉该 !important。② 全栏可拖拽：变量树（右缘手柄，160-420）、属性面板（180-440）、Schema 列与表单列（左缘手柄，160-560）。③ 内容驱动默认值：变量树 240→190（节点=缩进+图标+短名，190 充裕）、属性 300→280（label+控件+内距）、Schema 列 260→200（纯文本预览）、表单预览 300→320（需按真实表单宽度渲染）、组件库 220（卡片单列）、画布弹性吃余量。④ 设计模式默认收起 Schema/表单两栏（设计优先），空画布为整幅虚线拖放区。

**涉及文件：** `views/editor/components/QuickDesignDrawer/index.vue`、`components/{VariableTree,PropertyPanel,ComponentLibrary,DesignCanvas}.vue`

**验收结果：** `pnpm build` 通过；实测点掉两栏后设计列 720→1280 吃满全宽（display:none 真生效、offsetWidth 0）；四个拖拽手柄与范围钳制就位。

## 2026-09-01 拖拽动态上限：单列调整不再破坏整体布局

**变更内容：** 用户反馈拖拽影响整体布局。根因：四个可拖列的上限各自独立写死（如 Schema/表单各 560），同时拖宽时总需求远超抽屉宽度（约 2200 > 1280），布局溢出错乱。修复：拖拽上限改为**动态计算**——开始拖拽时取「所在布局总宽 − 兄弟列实际宽 − 弹性区保底（设计列 440 / 画布 220 / Schema 编辑器 220）」，与原静态上限取小。效果：拖任何一列只会压缩弹性画布到保底为止，其他列宽度永不被挤、整体永不溢出。

**涉及文件：** `views/editor/components/QuickDesignDrawer/index.vue`、`components/{PropertyPanel,VariableTree}.vue`

**验收结果：** `pnpm build` 通过。

## 2026-09-01 表单预览改按需抽屉 + Schema 工具栏溢出修复

**变更内容：** ① 应用户建议，表单预览从常驻列改为**按需抽屉**（顶栏「表单」按钮开关，左侧滑出 460px、无遮罩），三栏布局简化为两栏，永久释放 320px 空间；表单预览在抽屉中获得比原窄列更大的渲染宽度。② Schema 列内容溢出修复：工具栏 6 按钮（重置/格式化/复制/导入/导出/同步）加 `wrap` 换行，列宽再窄按钮也只是换行不再被裁；默认宽 200→240。③ 拖拽上限计算修正（弹性兄弟按保底计入而非当前宽，修复预览列永远拖不大）；设计列保底 420→660（组件库+画布最小+属性，防极限拖宽时属性被裁）。

**涉及文件：** `views/editor/components/QuickDesignDrawer/index.vue`、`components/SchemaEditor.vue`

**验收结果：** `pnpm build` 通过。
