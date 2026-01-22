#!/bin/bash

# 生成 Tauri Updater 签名密钥对
# 用途：生成用于签名自动更新包的密钥对

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

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Tauri Updater 签名密钥生成工具"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 检查 tauri CLI 是否安装
if ! command -v cargo-tauri &> /dev/null && ! cargo tauri --version &> /dev/null 2>&1; then
    print_error "未安装 Tauri CLI"
    echo ""
    echo "请先安装 Tauri CLI："
    echo "  cargo install tauri-cli"
    exit 1
fi

# 创建 keys 目录
KEYS_DIR=".tauri-keys"
mkdir -p "$KEYS_DIR"

print_info "密钥将保存到 $KEYS_DIR/ 目录"
echo ""

# 生成密钥对
print_info "正在生成 Ed25519 密钥对..."
echo ""

# 运行 Tauri 签名生成命令
# 注意：这个命令会交互式询问密码，也可以使用 -w 指定输出文件
cargo tauri signer generate -w "$KEYS_DIR/qsl-cardhub.key"

echo ""
print_success "密钥生成完成！"
echo ""

# 显示公钥
if [ -f "$KEYS_DIR/qsl-cardhub.key.pub" ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "公钥内容（添加到 tauri.conf.json 的 plugins.updater.pubkey）："
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cat "$KEYS_DIR/qsl-cardhub.key.pub"
    echo ""
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "下一步操作："
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. 将公钥添加到 tauri.conf.json："
echo "   plugins.updater.pubkey = \"<公钥内容>\""
echo ""
echo "2. 将私钥添加到 GitHub Secrets："
echo "   - 名称: TAURI_SIGNING_PRIVATE_KEY"
echo "   - 值: $KEYS_DIR/qsl-cardhub.key 文件内容"
echo ""
echo "3. 如果设置了密码，添加密码到 GitHub Secrets："
echo "   - 名称: TAURI_SIGNING_PRIVATE_KEY_PASSWORD"
echo "   - 值: 你设置的密码"
echo ""
print_warning "重要：请妥善保管私钥文件，不要提交到 Git 仓库！"
echo ""

# 确保 .tauri-keys 在 .gitignore 中
if ! grep -q ".tauri-keys" .gitignore 2>/dev/null; then
    echo ".tauri-keys/" >> .gitignore
    print_success "已将 .tauri-keys/ 添加到 .gitignore"
fi
