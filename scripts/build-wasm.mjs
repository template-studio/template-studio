#!/usr/bin/env node
/**
 * 构建 template_core_wasm 并将产物复制到当前前端项目的 src/wasm/ 目录。
 *
 * 用法（在 web/ 或 ant_design_web/ 等前端目录下执行）：
 *   node ../scripts/build-wasm.mjs --check     仅检查产物是否存在，不存在时退出码 1
 *   node ../scripts/build-wasm.mjs --dev       dev 模式构建并复制
 *   node ../scripts/build-wasm.mjs --release   release 模式构建并复制（默认）
 */
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const crateDir = path.join(repoRoot, 'crates', 'template_core_wasm');
const outDir = path.join(process.cwd(), 'src', 'wasm');
const glueFile = 'template_studio_template_core_wasm.js';

const args = process.argv.slice(2);
const mode = args.includes('--check') ? 'check' : args.includes('--dev') ? 'dev' : 'release';

if (mode === 'check') {
  if (fs.existsSync(path.join(outDir, glueFile))) process.exit(0);
  console.log('WASM not found, building...');
  process.exit(1);
}

const result = spawnSync(
  'wasm-pack',
  ['build', '--target', 'web', '--out-dir', 'pkg', mode === 'dev' ? '--dev' : '--release'],
  { cwd: crateDir, stdio: 'inherit', shell: process.platform === 'win32' }
);
if (result.status !== 0) {
  console.error(`wasm-pack build 失败（退出码 ${result.status}）`);
  process.exit(result.status ?? 1);
}

const pkgDir = path.join(crateDir, 'pkg');
if (!fs.existsSync(pkgDir)) {
  console.error(`未找到构建产物目录: ${pkgDir}`);
  process.exit(1);
}
fs.mkdirSync(outDir, { recursive: true });
const files = fs.readdirSync(pkgDir).filter((f) => f.endsWith('.js') || f.endsWith('.wasm'));
for (const file of files) fs.copyFileSync(path.join(pkgDir, file), path.join(outDir, file));
console.log(`已复制 ${files.length} 个文件到 ${outDir}: ${files.join(', ')}`);
