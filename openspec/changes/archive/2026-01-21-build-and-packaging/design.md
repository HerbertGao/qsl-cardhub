# 构建和打包系统 - 设计文档

## 概述

本文档描述 qsl-cardhub 项目的构建和打包系统设计，包括手动构建脚本和 GitHub Actions 自动化流程。

---

## 设计决策

### 决策 1：构建脚本语言选择

**问题**：选择什么语言编写构建脚本？

**选项**：
- **Shell Script (bash)**：macOS/Linux 原生支持
- **PowerShell**：Windows 原生支持
- **Python**：跨平台，但需要额外安装 Python
- **Node.js**：跨平台，但需要额外安装 Node.js

**决定**：
- macOS/Linux 使用 **Bash Script**
- Windows 使用 **PowerShell** + **Batch 包装器**

**理由**：
1. 使用系统原生脚本语言，无需额外依赖
2. Bash 在 macOS/Linux 上普遍可用
3. PowerShell 在 Windows 10+ 上默认安装
4. Batch 包装器提供向后兼容性（用户可以双击 `.bat` 文件）

---

### 决策 2：构建产物命名规范

**问题**：如何命名构建产物？

**决定**：使用统一的命名格式

**格式**：
```
qsl-cardhub-v{version}-{platform}-{arch}.{ext}
```

**示例**：
- `qsl-cardhub-v0.4.0-macos-universal.dmg`
- `qsl-cardhub-v0.4.0-windows-x64.msi`

**理由**：
1. 包含版本号，便于识别和管理
2. 包含平台和架构信息，避免混淆
3. 统一格式，便于自动化处理

---

### 决策 3：版本号管理策略

**问题**：版本号存储在哪里？如何同步？

**决定**：
- **单一来源**：`Cargo.toml` 中的 `version` 字段
- **自动同步**：使用脚本同步到 `tauri.conf.json`

**理由**：
1. `Cargo.toml` 是 Rust 项目的标准配置文件
2. Tauri 构建时会读取 `tauri.conf.json` 中的版本号
3. 避免手动维护多个文件中的版本号

**同步流程**：
```bash
# 1. 手动更新 Cargo.toml 版本号
version = "0.4.0"

# 2. 运行同步脚本
./scripts/sync-version.sh

# 3. 脚本自动更新 tauri.conf.json
"version": "0.4.0"
```

---

### 决策 4：GitHub Actions 触发条件

**问题**：何时触发自动构建？

**决定**：
- **build.yml**：PR to master、手动触发
- **release.yml**：tag 推送（`v*`）

**理由**：
1. PR 验证确保代码质量
2. Tag 推送表明正式发布意图
3. 手动触发用于临时测试

**不触发情况**：
- 普通 commit 推送到非 master 分支（避免浪费资源）
- 文档修改（可选优化：添加 path 过滤）

---

### 决策 5：构建缓存策略

**问题**：如何优化 CI 构建速度？

**决定**：缓存以下内容
- Cargo 构建缓存（`~/.cargo`, `target/`）
- npm 模块缓存（`node_modules/`, `~/.npm`）

**缓存键**：
```yaml
# Cargo 缓存
key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

# npm 缓存
key: ${{ runner.os }}-npm-${{ hashFiles('**/package-lock.json') }}
```

**理由**：
1. Cargo 编译耗时长，缓存能显著提速
2. npm 依赖安装耗时，缓存能节省时间
3. 使用 lockfile 哈希作为缓存键，确保依赖更新时重新构建

**预期效果**：
- 首次构建：10-15 分钟
- 缓存命中：3-5 分钟

---

### 决策 6：构建矩阵配置

**问题**：如何配置跨平台构建？

**决定**：使用 GitHub Actions 的 matrix 策略

```yaml
strategy:
  matrix:
    include:
      - platform: macos-latest
        target: universal-apple-darwin
      - platform: windows-latest
        target: x86_64-pc-windows-msvc
      - platform: windows-11-arm  # 原生 ARM64 runner (public preview)
        target: aarch64-pc-windows-msvc
```

**理由**：
1. 并行构建，提高效率
2. 统一配置，减少重复
3. 易于添加新平台（如 Linux）

**Windows ARM64 原生构建**：
- GitHub Actions 提供 `windows-11-arm` 原生 ARM64 runner（自 2025-04-14 public preview）
- 无需交叉编译，直接在 ARM64 环境构建
- 构建流程与 x64 完全相同，只是 runner 不同
- 更可靠、更快速（无需额外的 target 安装）

---

### 决策 7：Release 创建策略

**问题**：如何创建 GitHub Release？

**决定**：使用 `softprops/action-gh-release` Action

**配置**：
```yaml
- uses: softprops/action-gh-release@v1
  with:
    files: |
      dist/*.dmg
      dist/*.msi
    generate_release_notes: true
    draft: false
    prerelease: false
```

**理由**：
1. 成熟的第三方 Action，维护活跃
2. 支持自动生成 Release Notes
3. 支持上传多个文件
4. 支持 draft 和 prerelease 模式

---

### 决策 8：构建验证策略

**问题**：如何确保构建产物质量？

**决定**：多层验证

**验证层次**：
1. **编译验证**：Cargo 编译成功
2. **文件存在验证**：检查产物文件存在
3. **文件大小验证**：检查文件大小在合理范围（> 5MB, < 100MB）
4. **格式验证**：检查文件扩展名正确

**脚本实现**：
```bash
# 检查文件存在
if [ ! -f "$OUTPUT_FILE" ]; then
    echo "错误：构建产物不存在"
    exit 1
fi

# 检查文件大小
FILE_SIZE=$(stat -f%z "$OUTPUT_FILE")
if [ $FILE_SIZE -lt 5242880 ]; then  # 5MB
    echo "警告：文件大小过小，可能构建不完整"
fi
```

**理由**：
1. 早期发现构建问题
2. 避免发布损坏的安装包
3. 提供清晰的错误信息

---

## 技术架构

### 构建流程

```
┌─────────────┐
│ 更新版本号   │
│ Cargo.toml  │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ 同步版本号   │
│ sync-version │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ 前端构建     │
│ npm build   │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Tauri 打包   │
│ cargo tauri │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ 产物整理     │
│ dist/       │
└─────────────┘
```

### GitHub Actions 流程

```
┌──────────────┐
│  Tag 推送     │
│  v{version}  │
└───────┬──────┘
        │
        ▼
┌───────────────────┐
│  触发 CI 构建     │
│  release.yml      │
└─────────┬─────────┘
          │
    ┌─────┴─────┐
    ▼           ▼
┌────────┐  ┌────────┐
│ macOS  │  │Windows │
│ 构建    │  │ 构建    │
└────┬───┘  └───┬────┘
     │          │
     └────┬─────┘
          ▼
  ┌───────────────┐
  │ 上传产物      │
  │ Artifacts     │
  └───────┬───────┘
          │
          ▼
  ┌───────────────┐
  │ 创建 Release  │
  │ GitHub        │
  └───────────────┘
```

---

## 脚本设计

### macOS 构建脚本（build.sh）

**结构**：
```bash
#!/bin/bash

# 1. 检查依赖
check_dependencies() {
    # 检查 Node.js, npm, Rust, cargo
}

# 2. 构建前端
build_frontend() {
    cd web
    npm install
    npm run build
    cd ..
}

# 3. 打包应用
build_app() {
    cargo tauri build
}

# 4. 整理产物
organize_output() {
    mkdir -p dist
    cp target/release/bundle/dmg/*.dmg dist/
}

# 5. 验证产物
verify_output() {
    # 检查文件存在和大小
}

# 主流程
main() {
    check_dependencies
    build_frontend
    build_app
    organize_output
    verify_output
}

main
```

**特性**：
- 彩色输出（成功绿色、错误红色、警告黄色）
- 进度提示（当前步骤 X/Y）
- 错误处理（任何步骤失败立即退出）
- 构建时间统计

### Windows 构建脚本（build.ps1）

**结构**：类似 macOS 脚本，使用 PowerShell 语法

**特性**：
- 彩色输出（使用 `Write-Host -ForegroundColor`）
- 错误处理（使用 `$ErrorActionPreference = "Stop"`）
- 兼容性检查（检查 PowerShell 版本）

---

## 配置文件

### tauri.conf.json 优化

**优化项**：
```json
{
  "productName": "qsl-cardhub",
  "version": "0.4.0",
  "identifier": "com.herbertgao.qsl-cardhub",
  "bundle": {
    "active": true,
    "targets": ["dmg", "msi"],
    "icon": [
      "assets/icons/32x32.png",
      "assets/icons/128x128.png",
      "assets/icons/icon.icns",
      "assets/icons/icon.ico"
    ],
    "copyright": "Copyright © 2026 Herbert Software",
    "shortDescription": "业余无线电 QSL 卡片打印工具",
    "longDescription": "qsl-cardhub 是一款专为业余无线电爱好者设计的 QSL 卡片打印工具。"
  }
}
```

---

## 安全考虑

### 1. 代码签名（未实现）

**当前状态**：未配置代码签名

**影响**：
- macOS: 用户需要在"系统偏好设置 → 安全性与隐私"中允许
- Windows: SmartScreen 会显示警告

**未来改进**：
- 申请 Apple Developer 账号，配置代码签名和公证
- 申请 Code Signing Certificate，签名 Windows 应用

### 2. Secrets 管理

**当前状态**：无敏感信息

**如果添加代码签名**：
- 使用 GitHub Secrets 存储证书
- 不提交证书到代码库

---

## 性能优化

### 构建时间优化

**目标**：
- 本地构建 < 10 分钟
- CI 构建（首次）< 15 分钟
- CI 构建（缓存）< 5 分钟

**优化措施**：
1. 使用 Cargo 缓存（`~/.cargo`, `target/`）
2. 使用 npm 缓存（`node_modules/`）
3. 并行构建（macOS 和 Windows 并行）
4. 编译优化（`opt-level = "z"`, `lto = true`）

### 产物体积优化

**目标**：
- macOS DMG < 30MB
- Windows MSI < 30MB

**已配置优化**：
```toml
[profile.release]
opt-level = "z"     # 优化体积
lto = true          # 链接时优化
strip = true        # 移除符号信息
```

---

## 错误处理

### 脚本错误处理

**策略**：任何步骤失败立即退出

**实现**：
```bash
# Bash
set -e  # 任何命令失败立即退出

# PowerShell
$ErrorActionPreference = "Stop"
```

**错误信息**：
- 清晰说明失败的步骤
- 提供可能的解决方案
- 显示完整的错误日志

### CI 错误处理

**策略**：
- 构建失败时，CI 状态标记为失败
- 保留 Artifacts 用于调试
- 发送通知（可选）

---

## 测试策略

### 本地测试

**测试内容**：
1. 脚本能检测依赖
2. 构建流程完整
3. 产物可安装和运行
4. 应用功能正常

**测试频率**：每次修改脚本后

### CI 测试

**测试内容**：
1. PR 构建验证
2. Tag 发布验证
3. Release 创建验证

**测试频率**：每次 PR 和 Release

---

## 文档要求

### README 更新

**添加章节**：
- 构建说明（本地）
- 发布流程
- 下载和安装

### 脚本文档

**内容**：
- 脚本功能说明
- 使用方法
- 依赖要求
- 常见问题

---

## 未来扩展

### 0. Windows ARM64 支持现状（已包含在提案中）

**已实现**：
- 使用 GitHub Actions `windows-11-arm` 原生 runner
- 无需交叉编译，构建流程与 x64 完全相同
- 自 2025-04-14 起对公共仓库免费开放（public preview）

**注意事项**：
- `windows-11-arm` runner 目前处于 public preview 阶段
- 需要关注 GitHub 的稳定性和可用性公告
- 打印机驱动需要在实际 ARM64 设备上测试

### 1. Linux 支持

**待实现**：
- Linux 构建脚本
- AppImage / Deb / RPM 打包
- GitHub Actions Linux 构建（可使用 `ubuntu-latest`）

### 2. 自动更新

**待实现**：
- 应用内检查更新
- 自动下载和安装更新
- 使用 Tauri Updater 插件

### 3. 代码签名

**待实现**：
- macOS 代码签名和公证
- Windows 代码签名

### 4. 构建通知

**待实现**：
- 构建成功/失败通知（邮件、Slack）
- 构建状态 Badge
