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























