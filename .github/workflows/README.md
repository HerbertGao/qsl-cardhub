# GitHub Actions 工作流

本目录包含 qsl-cardhub 项目的 GitHub Actions 自动化工作流。

## 工作流说明

### build.yml - 桌面端构建工作流

**触发条件**：
- Pull Request 到 master 分支（**路径范围化**：纯 `web_query_service/**`、`openspec/**`、docs 的改动**不触发**——见下）
- 手动触发（workflow_dispatch）

**功能**：
- 在 macOS (ARM64 + x64)、Windows (x64 + ARM64) 平台并行构建桌面端（Rust/Tauri）
- 上传构建产物到 GitHub Artifacts
- 用于桌面端 PR 验证和测试

**路径范围化**：`paths-ignore` 含 `**/*.md`、`docs/**`、`web_query_service/**`、`openspec/**`。仅当 PR 的**全部**改动文件都落在这些区域时才跳过 Rust 构建；只要触及一个桌面端文件（`src/**`、`web/**`、`Cargo.*`、`tauri.conf.json`、`capabilities/**`、`build.rs` 等）就照常全量构建。前提：`Build` 非分支保护必需检查（仓主手动合并），跳过不卡合并。

### web-query-service.yml - 查询端 CI 工作流

**触发条件**：
- Pull Request 到 master 分支且改动触及 `web_query_service/**`
- 手动触发（workflow_dispatch）

**功能**：
- 单 job（ubuntu-latest）跑查询端自己的快速检查：`pnpm install` + `pnpm run test:unit`（`node --test`）+ `pnpm run build`（`vue-tsc + vite`）
- **不**执行 `cargo tauri build` 等桌面端构建
- worker 冒烟（`run_worker_smoke.sh`，依赖 `wrangler dev`）较重，不入 CI，留本地/部署前手动

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

- **Cargo 缓存**（桌面端）：`~/.cargo`, `target/`
- **pnpm 缓存**：经 `actions/setup-node` 的 `cache: pnpm` + `cache-dependency-path` 指向对应 `pnpm-lock.yaml`（桌面端 `web/pnpm-lock.yaml`、查询端 `web_query_service/pnpm-lock.yaml`）。本仓用 pnpm、非 npm。

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
