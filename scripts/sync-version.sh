#!/bin/bash

# qsl-cardhub 版本同步脚本
# 用途：从 Cargo.toml 读取版本号并同步到 tauri.conf.json

set -e

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

# 验证 semver 格式
validate_semver() {
    local version=$1
    if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        return 1
    fi
    return 0
}

# 从 Cargo.toml 读取版本号
get_cargo_version() {
    grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/'
}

# 更新 tauri.conf.json 中的版本号
update_tauri_version() {
    local new_version=$1

    # 使用 sed 更新版本号
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$new_version\"/" tauri.conf.json
    else
        # Linux
        sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$new_version\"/" tauri.conf.json
    fi
}

# 主函数
main() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  qsl-cardhub 版本同步工具"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # 检查文件存在
    if [ ! -f "Cargo.toml" ]; then
        print_error "Cargo.toml 不存在"
        exit 1
    fi

    if [ ! -f "tauri.conf.json" ]; then
        print_error "tauri.conf.json 不存在"
        exit 1
    fi

    # 读取版本号
    CARGO_VERSION=$(get_cargo_version)

    if [ -z "$CARGO_VERSION" ]; then
        print_error "无法从 Cargo.toml 读取版本号"
        exit 1
    fi

    # 验证版本号格式
    if ! validate_semver "$CARGO_VERSION"; then
        print_error "版本号格式不正确: $CARGO_VERSION"
        echo ""
        echo "版本号必须符合 semver 格式: X.Y.Z"
        echo "例如: 0.1.0, 1.2.3"
        exit 1
    fi

    print_info "从 Cargo.toml 读取版本号: $CARGO_VERSION"

    # 读取当前 tauri.conf.json 中的版本号
    TAURI_VERSION=$(grep '"version"' tauri.conf.json | sed 's/.*"version": "\(.*\)".*/\1/')
    print_info "tauri.conf.json 当前版本: $TAURI_VERSION"

    # 更新版本号
    if [ "$CARGO_VERSION" = "$TAURI_VERSION" ]; then
        print_success "版本号已一致，无需同步"
    else
        echo ""
        print_info "正在更新 tauri.conf.json..."
        update_tauri_version "$CARGO_VERSION"
        print_success "版本号已同步: $TAURI_VERSION → $CARGO_VERSION"
    fi

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print_success "同步完成！"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
}

main
