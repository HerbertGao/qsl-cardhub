#!/bin/bash

# qsl-cardhub 构建脚本 (macOS/Linux)
# 用途：自动化构建 Tauri 应用，包括前端构建和应用打包

set -e  # 任何命令失败立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_step() {
    echo -e "\n${BLUE}==>${NC} $1"
}

# 获取版本号
get_version() {
    grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/'
}

# 检查命令是否存在
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# 检查依赖
check_dependencies() {
    print_step "步骤 1/5: 检查依赖"

    local missing_deps=()

    # 检查 Node.js
    if command_exists node; then
        NODE_VERSION=$(node --version)
        print_success "Node.js: $NODE_VERSION"
    else
        missing_deps+=("Node.js")
        print_error "Node.js 未安装"
    fi

    # 检查 npm
    if command_exists npm; then
        NPM_VERSION=$(npm --version)
        print_success "npm: v$NPM_VERSION"
    else
        missing_deps+=("npm")
        print_error "npm 未安装"
    fi

    # 检查 Rust
    if command_exists rustc; then
        RUST_VERSION=$(rustc --version | awk '{print $2}')
        print_success "Rust: v$RUST_VERSION"
    else
        missing_deps+=("Rust")
        print_error "Rust 未安装"
    fi

    # 检查 cargo
    if command_exists cargo; then
        CARGO_VERSION=$(cargo --version | awk '{print $2}')
        print_success "cargo: v$CARGO_VERSION"
    else
        missing_deps+=("cargo")
        print_error "cargo 未安装"
    fi

    # 如果有缺失的依赖，提示并退出
    if [ ${#missing_deps[@]} -gt 0 ]; then
        echo ""
        print_error "缺少以下依赖：${missing_deps[*]}"
        echo ""
        echo "请安装缺失的依赖："
        for dep in "${missing_deps[@]}"; do
            case $dep in
                "Node.js"|"npm")
                    echo "  - Node.js: https://nodejs.org/"
                    ;;
                "Rust"|"cargo")
                    echo "  - Rust: https://rustup.rs/"
                    ;;
            esac
        done
        exit 1
    fi

    echo ""
}

# 检查版本号一致性
check_version_consistency() {
    CARGO_VERSION=$(get_version)
    TAURI_VERSION=$(grep '"version"' tauri.conf.json | sed 's/.*"version": "\(.*\)".*/\1/')

    if [ "$CARGO_VERSION" != "$TAURI_VERSION" ]; then
        print_warning "版本号不一致："
        echo "  Cargo.toml: $CARGO_VERSION"
        echo "  tauri.conf.json: $TAURI_VERSION"
        echo ""
        print_info "建议运行: ./scripts/sync-version.sh"
        echo ""
    fi
}

# 构建前端
build_frontend() {
    print_step "步骤 2/5: 构建前端"

    cd web

    # 检查是否需要安装依赖
    if [ ! -d "node_modules" ]; then
        print_info "安装前端依赖..."
        npm install
    fi

    print_info "构建前端..."
    npm run build

    cd ..

    if [ -d "web/dist" ]; then
        print_success "前端构建完成"
    else
        print_error "前端构建失败"
        exit 1
    fi

    echo ""
}

# 打包应用
build_app() {
    print_step "步骤 3/5: 打包 Tauri 应用"

    print_info "开始 Tauri 打包..."
    cargo tauri build

    print_success "Tauri 打包完成"
    echo ""
}

# 整理产物
organize_output() {
    print_step "步骤 4/5: 整理构建产物"

    VERSION=$(get_version)
    OUTPUT_NAME="qsl-cardhub-v${VERSION}-macos-universal.dmg"

    # 创建 dist 目录
    mkdir -p dist

    # 查找 DMG 文件
    DMG_PATH=$(find target/release/bundle/dmg -name "*.dmg" 2>/dev/null | head -n 1)

    if [ -z "$DMG_PATH" ]; then
        print_error "未找到 DMG 文件"
        exit 1
    fi

    # 复制到 dist 目录
    cp "$DMG_PATH" "dist/$OUTPUT_NAME"

    print_success "产物已复制到: dist/$OUTPUT_NAME"
    echo ""
}

# 验证产物
verify_output() {
    print_step "步骤 5/5: 验证构建产物"

    VERSION=$(get_version)
    OUTPUT_FILE="dist/qsl-cardhub-v${VERSION}-macos-universal.dmg"

    # 检查文件存在
    if [ ! -f "$OUTPUT_FILE" ]; then
        print_error "构建产物不存在: $OUTPUT_FILE"
        exit 1
    fi

    # 检查文件大小
    FILE_SIZE=$(stat -f%z "$OUTPUT_FILE" 2>/dev/null || stat -c%s "$OUTPUT_FILE" 2>/dev/null)
    FILE_SIZE_MB=$((FILE_SIZE / 1024 / 1024))

    if [ "$FILE_SIZE" -lt 5242880 ]; then  # 5MB
        print_warning "文件大小过小 (${FILE_SIZE_MB}MB)，可能构建不完整"
    elif [ "$FILE_SIZE" -gt 104857600 ]; then  # 100MB
        print_warning "文件大小过大 (${FILE_SIZE_MB}MB)，可能包含不必要的文件"
    else
        print_success "文件大小正常: ${FILE_SIZE_MB}MB"
    fi

    print_success "构建产物验证通过"
    echo ""
}

# 打印构建总结
print_summary() {
    VERSION=$(get_version)
    OUTPUT_FILE="dist/qsl-cardhub-v${VERSION}-macos-universal.dmg"
    FILE_SIZE=$(stat -f%z "$OUTPUT_FILE" 2>/dev/null || stat -c%s "$OUTPUT_FILE" 2>/dev/null)
    FILE_SIZE_MB=$((FILE_SIZE / 1024 / 1024))

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print_success "构建完成！"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "版本号:   $VERSION"
    echo "产物路径: $OUTPUT_FILE"
    echo "文件大小: ${FILE_SIZE_MB}MB"
    echo "构建用时: $SECONDS 秒"
    echo ""
}

# 主函数
main() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  qsl-cardhub 构建脚本 (macOS/Linux)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    check_dependencies
    check_version_consistency
    build_frontend
    build_app
    organize_output
    verify_output
    print_summary
}

# 启动构建
START_TIME=$SECONDS
main
