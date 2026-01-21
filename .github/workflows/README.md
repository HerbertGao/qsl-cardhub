# GitHub Actions 工作流

本目录包含 qsl-cardhub 项目的 GitHub Actions 自动化工作流。

## 工作流说明

### build.yml - 构建工作流

**触发条件**：
- Pull Request 到 master 分支
- 手动触发（workflow_dispatch）

**功能**：
- 在 macOS (ARM64 + x64)、Windows (x64 + ARM64) 平台并行构建
- 上传构建产物到 GitHub Artifacts
- 用于 PR 验证和测试

### release.yml - 发布工作流

**触发条件**：
- Git 标签推送（格式：`v*`，如 `v0.4.0`）

**功能**：
- 在 macOS (ARM64 + x64)、Windows (x64 + ARM64) 平台并行构建
- 自动创建 GitHub Release
- 上传构建产物到 Release
- 自动生成 Release Notes

## 使用说明

### PR 构建验证

创建 Pull Request 时会自动触发构建验证：

```bash
# 1. 创建分支并提交代码
git checkout -b feature/my-feature
git add .
git commit -m "feat: add new feature"
git push origin feature/my-feature

# 2. 在 GitHub 上创建 PR
# 构建将自动开始
```

### 发布新版本

发布新版本需要创建 Git 标签：

```bash
# 1. 更新版本号
# 编辑 Cargo.toml，修改 version = "0.4.0"

# 2. 同步版本号
./scripts/sync-version.sh

# 3. 提交版本更新
git add Cargo.toml tauri.conf.json
git commit -m "chore: bump version to 0.4.0"
git push origin master

# 4. 创建标签
git tag v0.4.0
git push origin v0.4.0

# 5. GitHub Actions 将自动构建并创建 Release
```

### 手动触发构建

在 GitHub Actions 页面可以手动触发构建：

1. 访问 `https://github.com/{username}/qsl-cardhub/actions`
2. 选择 "Build" 工作流
3. 点击 "Run workflow"
4. 选择分支并点击 "Run workflow"

## 构建产物

### Artifacts（PR 构建）

构建成功后，产物会上传到 GitHub Artifacts：
- 保留时间：90 天
- 下载路径：PR 页面 → Checks → Artifacts

### Release（版本发布）

发布构建会创建 GitHub Release：
- macOS ARM64: `qsl-cardhub-v{version}-macos-arm64.dmg`
- macOS x64: `qsl-cardhub-v{version}-macos-x64.dmg`
- Windows x64: `qsl-cardhub-v{version}-windows-x64.msi`
- Windows ARM64: `qsl-cardhub-v{version}-windows-arm64.msi`

## 构建环境

### macOS ARM64 (Apple Silicon)
- Runner: `macos-latest` (M1/M2/M3 芯片)
- Target: `aarch64-apple-darwin`
- Rust: 最新稳定版
- Node.js: 20.x

### macOS x64 (Intel)
- Runner: `macos-15-intel` (macOS 15 Sequoia, Intel 芯片)
- Target: `x86_64-apple-darwin`
- Rust: 最新稳定版
- Node.js: 20.x
- 注：已从 macos-13 升级，避免使用已弃用的 runner

### Windows x64
- Runner: `windows-latest`
- Target: `x86_64-pc-windows-msvc`
- Rust: 最新稳定版
- Node.js: 20.x

### Windows ARM64
- Runner: `windows-11-arm` (Public Preview)
- Target: `aarch64-pc-windows-msvc`
- Rust: 最新稳定版
- Node.js: 20.x
- 注：自 2025-04-14 起对公共仓库免费开放

## 缓存策略

为了加快构建速度，工作流使用了以下缓存：

- **Cargo 缓存**：`~/.cargo`, `target/`
- **npm 缓存**：`node_modules/`, `~/.npm`

缓存键基于 lockfile 的哈希值，依赖更新时会自动失效。

## 故障排查

### 构建失败

1. 检查 Actions 日志中的错误信息
2. 本地运行构建脚本验证：`./scripts/build.sh` 或 `.\scripts\build.bat`
3. 检查依赖版本是否兼容

### 版本号不一致

如果看到版本号不一致警告：

```bash
./scripts/sync-version.sh
```

### Release 创建失败

1. 确保标签格式正确（`v*`）
2. 确保有推送权限
3. 检查是否已存在同名 Release

## 注意事项

- Windows ARM64 runner 目前处于 public preview 阶段
- 构建时间约 3-5 分钟（缓存命中时）
- 首次构建约 10-15 分钟
