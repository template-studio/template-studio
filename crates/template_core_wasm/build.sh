#!/bin/bash
# WASM 构建脚本（Linux/macOS）
#
# 使用方法：
#   ./build.sh              # 构建发布版本
#   ./build.sh --dev        # 构建开发版本
#   ./build.sh --install    # 安装 wasm-pack

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 解析参数
BUILD_MODE="release"
INSTALL_WASM_PACK=false

for arg in "$@"; do
    case $arg in
        --dev)
            BUILD_MODE="dev"
            shift
            ;;
        --install)
            INSTALL_WASM_PACK=true
            shift
            ;;
        *)
            echo -e "${RED}Unknown argument: $arg${NC}"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}=== Template Studio WASM Build ===${NC}"
echo "Build mode: $BUILD_MODE"

# 检查 wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    if [ "$INSTALL_WASM_PACK" = true ]; then
        echo -e "${YELLOW}Installing wasm-pack...${NC}"
        cargo install wasm-pack
    else
        echo -e "${RED}Error: wasm-pack not found!${NC}"
        echo "Run with --install flag to install it automatically:"
        echo "  ./build.sh --install"
        exit 1
    fi
fi

# 设置构建时间环境变量
export BUILD_TIME=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# 切换到脚本所在目录
cd "$(dirname "$0")"

# 构建 WASM
echo -e "${YELLOW}Building WASM package...${NC}"

if [ "$BUILD_MODE" = "release" ]; then
    wasm-pack build --target web --out-dir pkg --release
else
    wasm-pack build --target web --out-dir pkg --dev
fi

# 检查构建结果
if [ -f "pkg/template_studio_template_core_wasm.js" ]; then
    echo -e "${GREEN}Build successful!${NC}"
    echo ""
    echo "Output files:"
    ls -lh pkg/*.js pkg/*.wasm 2>/dev/null || true

    # 显示 WASM 文件大小
    WASM_SIZE=$(stat -f%z "pkg/template_studio_template_core_wasm_bg.wasm" 2>/dev/null || stat -c%s "pkg/template_studio_template_core_wasm_bg.wasm" 2>/dev/null || echo "unknown")
    echo ""
    echo -e "WASM size: ${GREEN}${WASM_SIZE} bytes${NC}"

    # 如果有 wasm-opt，进行优化
    if command -v wasm-opt &> /dev/null && [ "$BUILD_MODE" = "release" ]; then
        echo ""
        echo -e "${YELLOW}Optimizing WASM with wasm-opt...${NC}"
        wasm-opt -Oz pkg/template_studio_template_core_wasm_bg.wasm -o pkg/template_studio_template_core_wasm_bg.wasm 2>/dev/null || true
        echo "Optimization complete!"
    fi
else
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}Done!${NC}"
echo "To use in JavaScript:"
echo "  import init, { render_string } from './pkg/template_studio_template_core_wasm.js';"
echo "  await init();"
