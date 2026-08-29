# 工作日志

## 2026-08-29 修复并统一前端 WASM 构建脚本

**变更内容：**
- 新增跨平台共享脚本 `scripts/build-wasm.mjs`：负责 wasm-pack 构建 `template_core_wasm` 并将产物（*.js/*.wasm）复制到调用方前端的 `src/wasm/` 目录，支持 `--check` / `--dev` / `--release` 三种模式，替代原先内嵌在 package.json 中的 PowerShell 命令。
- 修复 `web/`、`ant_design_web/` 两个前端 package.json 中 `build:wasm*` 系列脚本的问题：复制目标相对路径错误（`../web/src/wasm/` 实际解析到不存在的 `crates/web/`），且不自动创建被 gitignore 的目标目录，导致新克隆环境下 `pnpm run dev` 失败。两个前端统一改为调用共享脚本。
- `web/pnpm-workspace.yaml`：`allowBuilds.less` 的占位符值改为 `false`（less 的 postinstall 会下载 Playwright 浏览器，不应执行），修复 pnpm v11 安装前检查报错退出的问题。

**涉及文件：** `scripts/build-wasm.mjs`（新增）、`web/package.json`、`ant_design_web/package.json`、`web/pnpm-workspace.yaml`

**验收结果：** 删除两个前端 `src/wasm/` 产物模拟新环境后，`pnpm run build:wasm:check` 自动触发构建并正确复制（退出码 0）；两个前端 `pnpm run dev` 均正常启动，页面与 WASM 资源请求均返回 HTTP 200。

## 2026-08-29 新增 wasm-build 项目 skill

**变更内容：** 将 WASM 模板引擎的构建接入流程与排障经验固化为项目级 skill（新前端接入步骤、共享脚本用法、常见报错对照、验证协议）。

**涉及文件：** `.zcode/skills/wasm-build/SKILL.md`（新增）

**验收结果：** skill 内容与当前实际代码（scripts/build-wasm.mjs、两个前端 package.json / vite.config / .gitignore）核对一致。

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




