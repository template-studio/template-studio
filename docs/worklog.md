# 工作日志

## 2026-08-29 修复并统一前端 WASM 构建脚本

**变更内容：**
- 新增跨平台共享脚本 `scripts/build-wasm.mjs`：负责 wasm-pack 构建 `template_core_wasm` 并将产物（*.js/*.wasm）复制到调用方前端的 `src/wasm/` 目录，支持 `--check` / `--dev` / `--release` 三种模式，替代原先内嵌在 package.json 中的 PowerShell 命令。
- 修复 `web/`、`ant_design_web/` 两个前端 package.json 中 `build:wasm*` 系列脚本的问题：复制目标相对路径错误（`../web/src/wasm/` 实际解析到不存在的 `crates/web/`），且不自动创建被 gitignore 的目标目录，导致新克隆环境下 `pnpm run dev` 失败。两个前端统一改为调用共享脚本。
- `web/pnpm-workspace.yaml`：`allowBuilds.less` 的占位符值改为 `false`（less 的 postinstall 会下载 Playwright 浏览器，不应执行），修复 pnpm v11 安装前检查报错退出的问题。

**涉及文件：** `scripts/build-wasm.mjs`（新增）、`web/package.json`、`ant_design_web/package.json`、`web/pnpm-workspace.yaml`

**验收结果：** 删除两个前端 `src/wasm/` 产物模拟新环境后，`pnpm run build:wasm:check` 自动触发构建并正确复制（退出码 0）；两个前端 `pnpm run dev` 均正常启动，页面与 WASM 资源请求均返回 HTTP 200。
