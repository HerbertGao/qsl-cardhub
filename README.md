# qsl-cardhub - Rust + Tauri 版本

> 业余无线电 QSL 卡片打印工具

## 简介

qsl-cardhub 是一款专为业余无线电爱好者设计的 QSL 卡片打印工具。Rust + Tauri 版本提供了更快的启动速度、更小的体积和更好的跨平台支持。

### 主要特性

- ✅ **快速启动**：启动时间 < 500ms
- ✅ **体积小巧**：可执行文件 < 20MB
- ✅ **跨平台支持**：Windows、macOS、Linux
- ✅ **配置管理**：支持多配置切换、导入导出
- ✅ **打印功能**：QSL 卡片打印、校准页打印
- ✅ **现代化界面**：Vue 3 + Element Plus

## 项目结构

```
qsl-cardhub/
├── src/                    # Rust 后端代码
│   ├── commands/          # Tauri Commands（API 层）
│   │   ├── platform.rs    # 平台信息
│   │   ├── profile.rs     # 配置管理
│   │   └── printer.rs     # 打印功能
│   ├── config/            # 配置管理模块
│   │   ├── models.rs      # 数据模型
│   │   └── profile_manager.rs # 配置管理器
│   ├── printer/           # 打印模块
│   │   ├── backend/       # 打印后端
│   │   │   ├── windows.rs # Windows 后端
│   │   │   ├── cups.rs    # CUPS 后端
│   │   │   └── mock.rs    # Mock 后端
│   │   ├── tspl.rs        # TSPL 生成器
│   │   └── manager.rs     # 打印管理器
│   ├── utils/             # 工具模块
│   │   └── platform.rs    # 平台检测
│   ├── error/             # 错误处理
│   └── main.rs            # 主程序入口
├── web/                   # Vue 3 前端
│   ├── src/
│   │   ├── views/         # 页面组件
│   │   └── components/    # 通用组件
│   └── vite.config.js
├── config/                # 配置文件目录
│   ├── config.toml        # 全局配置
│   ├── profiles/          # Profile 配置
│   └── templates/         # 打印模板（v0.5）
├── scripts/               # 构建和工具脚本
│   ├── build.sh          # macOS/Linux 构建脚本
│   ├── build.ps1         # Windows 构建脚本
│   ├── build.bat         # Windows CMD 入口
│   └── sync-version.sh   # 版本同步脚本
├── .github/              # GitHub 配置
│   └── workflows/        # GitHub Actions 工作流
│       ├── build.yml     # PR 构建验证
│       ├── release.yml   # 版本发布
│       └── README.md     # 工作流文档
├── output/                # Mock 打印输出
├── dist/                  # 构建产物目录
├── tests/                 # 测试文件
└── Cargo.toml            # Rust 依赖配置
```

## 技术栈

### 后端
- **Rust** - 系统编程语言
- **Tauri 2** - 桌面应用框架
- **serde** - 序列化/反序列化
- **toml** - TOML 配置文件解析
- **anyhow/thiserror** - 错误处理
- **uuid** - UUID 生成
- **chrono** - 日期时间处理

### 前端
- **Vue 3** - 渐进式前端框架
- **Element Plus** - UI 组件库
- **Vite** - 构建工具

### 平台特定依赖
- **Windows**: windows-rs (Win32 API)
- **macOS/Linux**: CUPS (lp 命令)

## 快速开始

### 前置要求

- Rust 1.70+
- Node.js 18+
- pnpm (推荐) 或 npm

**平台特定要求：**
- Windows: 无额外要求
- macOS: 已安装 Xcode Command Line Tools
- Linux: 已安装 CUPS (`sudo apt install cups` 或 `sudo yum install cups`)

### 安装依赖

```bash
# 安装 Rust 依赖
cargo build

# 安装前端依赖
cd web
pnpm install
```

### 开发模式

```bash
# 启动开发服务器（前端热重载 + Rust 后端）
cargo tauri dev
```

### 生产构建

#### 方式一：使用构建脚本（推荐）

**macOS/Linux:**
```bash
./scripts/build.sh
```

**Windows:**
```powershell
.\scripts\build.ps1
```

或使用 CMD：
```cmd
.\scripts\build.bat
```

构建脚本会自动：
- 检查依赖（Node.js、npm、Rust、cargo）
- 验证版本一致性
- 构建前端
- 打包 Tauri 应用
- 将产物复制到 `dist/` 目录

**产物命名格式：**
- macOS: `qsl-cardhub-v{version}-macos-universal.dmg`
- Windows x64: `qsl-cardhub-v{version}-windows-x64.msi`
- Windows ARM64: `qsl-cardhub-v{version}-windows-arm64.msi`

#### 方式二：手动构建

```bash
# 构建前端
cd web
pnpm run build

# 构建 Tauri 应用
cd ..
cargo tauri build
```

构建产物位于 `target/release/bundle/`

## 版本管理与发布

### 版本同步

项目使用 `Cargo.toml` 作为版本号的单一数据源。修改版本号后,使用同步脚本更新其他配置文件：

```bash
./scripts/sync-version.sh
```

这会将版本号从 `Cargo.toml` 同步到 `tauri.conf.json`。

### 发布新版本

发布流程使用 GitHub Actions 自动化：

```bash
# 1. 更新版本号
# 编辑 Cargo.toml，修改 version = "x.y.z"

# 2. 同步版本号
./scripts/sync-version.sh

# 3. 提交版本更新
git add Cargo.toml tauri.conf.json
git commit -m "chore: bump version to x.y.z"
git push origin master

# 4. 创建并推送标签
git tag vx.y.z
git push origin vx.y.z
```

推送标签后，GitHub Actions 会自动：
- 在 macOS、Windows x64 和 Windows ARM64 平台构建
- 创建 GitHub Release
- 上传构建产物
- 生成 Release Notes

**支持的平台：**
- macOS Universal (Intel + Apple Silicon)
- Windows x64
- Windows ARM64

### CI/CD 工作流

项目包含两个 GitHub Actions 工作流：

1. **build.yml** - PR 构建验证
   - 触发：Pull Request 到 master 分支
   - 功能：并行构建所有平台，上传 Artifacts
   - 产物保留时间：90 天

2. **release.yml** - 版本发布
   - 触发：推送 `v*` 格式的标签
   - 功能：构建并发布到 GitHub Release

详细说明请参考 [GitHub Actions 文档](.github/workflows/README.md)。

## 配置文件

### 配置目录位置

- **开发模式**: 项目根目录 `config/`
- **生产模式**:
  - Windows: `%APPDATA%/qsl-cardhub/`
  - macOS: `~/Library/Application Support/qsl-cardhub/`
  - Linux: `~/.config/qsl-cardhub/`

### 配置文件结构

```
config/
├── config.toml           # 全局配置（首次启动自动创建）
├── templates/            # 打印模板目录
│   └── default.toml      # 默认模板（从应用资源自动复制）
└── profiles/             # 打印配置文件目录
    ├── uuid-1.toml       # 配置 1
    └── uuid-2.toml       # 配置 2
```

**注意**：
- 生产环境下，首次启动应用时会自动从应用资源目录复制 `default.toml` 模板到用户配置目录
- `config.toml` 会由应用自动创建和管理
- 用户可以在 `templates/` 目录下添加自定义模板文件

### 示例配置

参见 `config/profiles/.example.toml` 和 `config/templates/default.toml`

## 架构设计

详细架构设计请参考：
- [提案文档](openspec/changes/migrate-to-rust-tauri/proposal.md)
- [设计文档](openspec/changes/migrate-to-rust-tauri/design.md)
- [任务清单](openspec/changes/migrate-to-rust-tauri/tasks.md)

## 开发指南

### 添加新的 Tauri Command

1. 在 `src/commands/` 中定义函数
2. 使用 `#[tauri::command]` 宏标注
3. 在 `src/main.rs` 的 `invoke_handler!` 中注册

```rust
#[tauri::command]
async fn my_command(param: String) -> Result<String, String> {
    Ok(format!("Hello, {}", param))
}
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test --package qsl-cardhub --lib config::tests
```

### 代码格式化

```bash
# 格式化 Rust 代码
cargo fmt

# 格式化前端代码
cd web
pnpm run lint
```

## 版本规划

- **v0.1** (当前): 基础功能迁移
  - 配置管理
  - TSPL 打印
  - 跨平台支持

- **v0.5**: 模板配置化
  - 打印模板配置文件
  - PDF 测试后端
  - 日志查看功能

- **v1.0**: 完整版本
  - 完整错误处理
  - 打包优化
  - 用户文档

- **v2.0**: 高级功能
  - 模板管理 UI
  - 中文字体支持
  - 批量打印
  - 网络打印

## 许可证

MIT License

## 联系方式

- 作者: Herbert Software
- 项目: https://github.com/HerbertGao/qsl-cardhub
