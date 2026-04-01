#!/usr/bin/env pwsh
# WASM 构建脚本（Windows PowerShell）
#
# 使用方法：
#   .\build.ps1              # 构建发布版本
#   .\build.ps1 -Dev         # 构建开发版本
#   .\build.ps1 -Install     # 安装 wasm-pack

param(
    [switch]$Dev,
    [switch]$Install
)

$ErrorActionPreference = "Stop"

# 颜色函数
function Write-Success { param($text) Write-Host $text -ForegroundColor Green }
function Write-Info { param($text) Write-Host $text -ForegroundColor Cyan }
function Write-Warning { param($text) Write-Host $text -ForegroundColor Yellow }
function Write-Error { param($text) Write-Host $text -ForegroundColor Red }

Write-Success "=== Template Studio WASM Build ==="

# 确定构建模式
$BuildMode = if ($Dev) { "dev" } else { "release" }
Write-Info "Build mode: $BuildMode"

# 检查 wasm-pack
$wasmPack = Get-Command wasm-pack -ErrorAction SilentlyContinue

if (-not $wasmPack) {
    if ($Install) {
        Write-Warning "Installing wasm-pack..."
        cargo install wasm-pack
    } else {
        Write-Error "Error: wasm-pack not found!"
        Write-Host "Run with -Install flag to install it automatically:"
        Write-Host "  .\build.ps1 -Install"
        exit 1
    }
}

# 设置构建时间环境变量
$env:BUILD_TIME = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

# 切换到脚本所在目录
Set-Location $PSScriptRoot

# 构建 WASM
Write-Warning "Building WASM package..."

if ($BuildMode -eq "release") {
    wasm-pack build --target web --out-dir pkg --release
} else {
    wasm-pack build --target web --out-dir pkg --dev
}

# 检查构建结果
$jsFile = "pkg\template_studio_template_core_wasm.js"
$wasmFile = "pkg\template_studio_template_core_wasm_bg.wasm"

if (Test-Path $jsFile) {
    Write-Success "`nBuild successful!`n"

    Write-Info "Output files:"
    Get-ChildItem pkg\*.js, pkg\*.wasm -ErrorAction SilentlyContinue | ForEach-Object {
        $size = "{0:N2} KB" -f ($_.Length / 1KB)
        Write-Host "  $($_.Name) - $size"
    }

    # 显示 WASM 文件大小
    if (Test-Path $wasmFile) {
        $wasmSize = (Get-Item $wasmFile).Length
        $wasmSizeKB = "{0:N2}" -f ($wasmSize / 1KB)
        Write-Host "`nWASM size: " -NoNewline
        Write-Success "$wasmSizeKB KB"
    }
} else {
    Write-Error "Build failed!"
    exit 1
}

Write-Host ""
Write-Success "Done!"
Write-Host "To use in JavaScript:"
Write-Host "  import init, { render_string } from './pkg/template_studio_template_core_wasm.js';"
Write-Host "  await init();"
