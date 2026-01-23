#!/bin/bash

# qsl-cardhub 版本管理脚本
# 用途：升级版本号并同步到所有配置文件
# 使用：./scripts/version.sh [major|minor|patch|x.y.z|check]

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

print_warning() {
    echo -e "${YELLOW}!${NC} $1"
}

# 切换到项目根目录
cd "$(dirname "$0")/.."

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
    grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
}

# 更新 Cargo.toml 中的版本号
update_cargo_version() {
    local new_version=$1
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/^version = \"[^\"]*\"/version = \"$new_version\"/" Cargo.toml
    else
        sed -i "s/^version = \"[^\"]*\"/version = \"$new_version\"/" Cargo.toml
    fi
}

# 更新 tauri.conf.json 中的版本号（如果存在 version 字段）
update_tauri_version() {
    local new_version=$1
    if [ -f "tauri.conf.json" ]; then
        if grep -q '"version"' tauri.conf.json 2>/dev/null; then
            if [[ "$OSTYPE" == "darwin"* ]]; then
                sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$new_version\"/" tauri.conf.json
            else
                sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$new_version\"/" tauri.conf.json
            fi
            print_success "已更新 tauri.conf.json"
        fi
    fi
}

# 更新 web/package.json 中的版本号
update_package_version() {
    local new_version=$1
    if [ -f "web/package.json" ]; then
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$new_version\"/" web/package.json
        else
            sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$new_version\"/" web/package.json
        fi
        print_success "已更新 web/package.json"
    fi
}

# 计算新版本号
calculate_new_version() {
    local current=$1
    local bump_type=$2

    local major minor patch
    IFS='.' read -r major minor patch <<< "$current"

    case $bump_type in
        major)
            echo "$((major + 1)).0.0"
            ;;
        minor)
            echo "${major}.$((minor + 1)).0"
            ;;
        patch)
            echo "${major}.${minor}.$((patch + 1))"
            ;;
        *)
            echo ""
            ;;
    esac
}

# 检查版本一致性
check_versions() {
    local cargo_version=$(get_cargo_version)
    local tauri_version=""
    local package_version=""
    local all_match=true

    if [ -f "tauri.conf.json" ] && grep -q '"version"' tauri.conf.json 2>/dev/null; then
        tauri_version=$(grep '"version"' tauri.conf.json | head -1 | sed 's/.*"version": "\([^"]*\)".*/\1/')
    fi

    if [ -f "web/package.json" ]; then
        package_version=$(grep '"version"' web/package.json | head -1 | sed 's/.*"version": "\([^"]*\)".*/\1/')
    fi

    echo ""
    echo "版本号检查："
    echo "  Cargo.toml:      $cargo_version"
    [ -n "$tauri_version" ] && echo "  tauri.conf.json: $tauri_version"
    [ -n "$package_version" ] && echo "  package.json:    $package_version"
    echo ""

    if [ -n "$tauri_version" ] && [ "$cargo_version" != "$tauri_version" ]; then
        all_match=false
    fi

    if [ -n "$package_version" ] && [ "$cargo_version" != "$package_version" ]; then
        all_match=false
    fi

    if $all_match; then
        print_success "所有版本号一致"
        return 0
    else
        print_warning "版本号不一致，建议运行: ./scripts/version.sh sync"
        return 1
    fi
}

# 同步版本号（从 Cargo.toml 同步到其他文件）
sync_versions() {
    local cargo_version=$(get_cargo_version)

    print_info "从 Cargo.toml 同步版本号: $cargo_version"
    echo ""

    update_tauri_version "$cargo_version"
    update_package_version "$cargo_version"

    echo ""
    print_success "版本同步完成"
}

# 显示使用帮助
show_usage() {
    echo ""
    echo "用法: ./scripts/version.sh [命令]"
    echo ""
    echo "命令:"
    echo "  (无参数)     显示当前版本号"
    echo "  major        升级主版本号 (1.0.0 → 2.0.0)"
    echo "  minor        升级次版本号 (1.0.0 → 1.1.0)"
    echo "  patch        升级补丁版本号 (1.0.0 → 1.0.1)"
    echo "  x.y.z        设置自定义版本号"
    echo "  check        检查所有文件的版本一致性"
    echo "  sync         从 Cargo.toml 同步版本到其他文件"
    echo "  help         显示此帮助信息"
    echo ""
    echo "示例:"
    echo "  ./scripts/version.sh patch    # 0.3.0 → 0.3.1"
    echo "  ./scripts/version.sh 1.0.0    # 设置版本为 1.0.0"
    echo ""
}

# 显示当前版本
show_current_version() {
    local cargo_version=$(get_cargo_version)

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  qsl-cardhub 版本信息"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "  当前版本: v$cargo_version"

    # 显示最近的 Git 标签
    if git describe --tags --abbrev=0 2>/dev/null; then
        local latest_tag=$(git describe --tags --abbrev=0 2>/dev/null)
        echo "  最新标签: $latest_tag"
    fi

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
}

# 主函数
main() {
    # 检查 Cargo.toml 存在
    if [ ! -f "Cargo.toml" ]; then
        print_error "Cargo.toml 不存在，请在项目根目录运行此脚本"
        exit 1
    fi

    local command=${1:-}

    case $command in
        "")
            show_current_version
            ;;
        help|--help|-h)
            show_usage
            ;;
        check)
            check_versions
            ;;
        sync)
            sync_versions
            ;;
        major|minor|patch)
            local current_version=$(get_cargo_version)
            local new_version=$(calculate_new_version "$current_version" "$command")

            echo ""
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo "  qsl-cardhub 版本升级"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo ""
            print_info "当前版本: $current_version"
            print_info "新版本:   $new_version"
            echo ""

            # 更新所有文件
            update_cargo_version "$new_version"
            print_success "已更新 Cargo.toml"

            update_tauri_version "$new_version"
            update_package_version "$new_version"

            echo ""
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            print_success "版本升级完成: $current_version → $new_version"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo ""
            ;;
        *)
            # 检查是否是自定义版本号
            if validate_semver "$command"; then
                local current_version=$(get_cargo_version)
                local new_version=$command

                echo ""
                echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                echo "  qsl-cardhub 版本设置"
                echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                echo ""
                print_info "当前版本: $current_version"
                print_info "新版本:   $new_version"
                echo ""

                # 更新所有文件
                update_cargo_version "$new_version"
                print_success "已更新 Cargo.toml"

                update_tauri_version "$new_version"
                update_package_version "$new_version"

                echo ""
                echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                print_success "版本设置完成: $current_version → $new_version"
                echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                echo ""
            else
                print_error "无效的命令或版本号格式: $command"
                echo ""
                echo "版本号必须符合 semver 格式: X.Y.Z (例如: 1.0.0, 0.2.1)"
                show_usage
                exit 1
            fi
            ;;
    esac
}

main "$@"
