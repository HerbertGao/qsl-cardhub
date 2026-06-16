## 修改需求

### 需求： GitHub Actions 自动构建

系统必须提供 GitHub Actions 工作流，实现自动化构建和发布。**CI 运行必须与改动区域相关（meaningful CI）**：本仓为桌面端（Rust/Tauri）+ 独立查询端（`web_query_service/`，Cloudflare Worker）双栈，**禁止**对纯查询端、纯 `openspec/` 规范、纯 docs 的改动触发与之无关的 Rust/Tauri 桌面构建；改哪端跑哪端的检查。

#### 场景： Pull Request 构建验证（桌面端，按路径范围化）

- **当** 开发者创建 Pull Request 到 master 分支，且改动**包含**桌面端相关文件（如 `src/**`、`web/**`、`Cargo.*`、`tauri.conf.json`、`capabilities/**`、`build.rs` 等）
- **那么** GitHub Actions 必须自动触发 `build.yml` 工作流
- **并且** 必须在 macOS 和 Windows 平台并行构建
- **并且** 构建成功后必须将产物上传到 GitHub Artifacts
- **并且** PR 页面必须显示构建状态（成功或失败）
- **并且** 构建失败时必须显示详细的错误日志
- **并且** 当 PR 的**全部**改动文件均落在与桌面端无关的区域（`web_query_service/**`、`openspec/**`、`**/*.md`、`docs/**`）时，`build.yml`（Rust/Tauri 构建与质量检查）**必须不触发**——通过 `paths-ignore` 实现；`paths-ignore` 仅在**所有**改动文件都命中忽略模式时跳过，故同时触及桌面端与其他区域的 PR 仍照常全量构建
- **并且** 该跳过**不得卡合并**：前提是 `Build`/`Build Summary` 非分支保护必需状态检查（仓主手动合并）；若未来设为必需，**必须**改用「必检 gate job 永远运行、对无关改动 no-op 报成功」模式，**禁止**用整工作流跳过致必需检查永久 pending

#### 场景： 查询端 Pull Request 验证

- **当** 开发者创建 Pull Request 到 master 分支，且改动**触及** `web_query_service/**`
- **那么** GitHub Actions 必须自动触发查询端工作流（`web-query-service.yml`）
- **并且** 该工作流必须运行查询端自己的检查：安装依赖（`pnpm install --frozen-lockfile`）、单元测试（`pnpm run test:unit`，即 `node --test`）、构建（`pnpm run build`，即 `vue-tsc --noEmit && vite build`）
- **并且** `--frozen-lockfile` 的前置：`web_query_service/pnpm-lock.yaml` **必须**被 git 追踪（**禁止** gitignore），否则 CI 全新 checkout 无 lockfile、安装必失败（`ERR_PNPM_NO_LOCKFILE`）；pnpm 版本**必须**对齐 `web_query_service/package.json` 的 `packageManager`（仓根无 `package.json`，`pnpm/action-setup` 须显式给 `version`）
- **并且** **禁止**在该工作流中执行 `cargo tauri build` 等桌面端构建（查询端与桌面端 Rust 构建无关）
- **并且** worker 冒烟（`run_worker_smoke.sh`，依赖 `wrangler dev`）较重，**可**不纳入本工作流、留本地/手动执行；纳入与否不影响「查询端 PR 至少跑单测 + 构建」这一最低有意义检查

#### 场景： Tag 发布构建

- **当** 开发者推送 Git 标签（格式为 `v*`，如 `v0.4.0`）
- **那么** GitHub Actions 必须自动触发 `release.yml` 工作流
- **并且** 必须在 macOS 和 Windows 平台并行构建
- **并且** 构建成功后必须自动创建 GitHub Release
- **并且** Release 必须包含构建产物（DMG 和 MSI 文件）
- **并且** Release 标题必须为标签名称（如"v0.4.0"）
- **并且** Release 必须自动生成 Release Notes（基于 commits）

#### 场景： 手动触发构建

- **当** 开发者在 GitHub Actions 页面手动触发 `build.yml`
- **那么** 工作流必须执行完整的构建流程
- **并且** 必须将产物上传到 Artifacts
- **并且** 不创建 Release

#### 场景： 构建缓存优化

- **当** GitHub Actions 执行构建时
- **那么** 必须缓存 Cargo 构建产物（`~/.cargo`, `target/`）
- **并且** 必须缓存 pnpm 依赖（经 `actions/setup-node` 的 `cache: pnpm` + `cache-dependency-path` 指向对应 `pnpm-lock.yaml`；本仓用 pnpm、非 npm）
- **并且** 缓存键必须基于 `Cargo.lock` 和 `pnpm-lock.yaml` 的哈希值
- **并且** 缓存命中时，构建时间必须显著减少（< 5 分钟）

#### 场景： 构建矩阵配置

- **当** GitHub Actions 执行构建时
- **那么** 必须使用 matrix 策略并行构建多个平台
- **并且** 必须包含以下平台（各为独立 matrix 条目）：
  - macOS ARM64：`macos-latest` runner，target `aarch64-apple-darwin`
  - macOS x64：`macos-15-intel` runner，target `x86_64-apple-darwin`
  - Windows x64：`windows-latest` runner，target `x86_64-pc-windows-msvc`
  - Windows ARM64：`windows-11-arm` runner（原生 ARM64），target `aarch64-pc-windows-msvc`
- **并且** 每个平台的构建必须独立运行
- **并且** 任何平台构建失败不应影响其他平台（`fail-fast: false`）

#### 场景： Windows ARM64 原生构建

- **当** GitHub Actions 构建 Windows ARM64 版本
- **那么** 必须使用 `windows-11-arm` runner（原生 ARM64 环境）
- **并且** 必须安装 Rust 和 Node.js（ARM64 版本）
- **并且** 构建流程必须与 Windows x64 完全相同（无需特殊配置）
- **并且** 必须生成 `qsl-cardhub-v{version}-windows-arm64.msi` 文件
- **并且** Tauri 必须自动检测 ARM64 架构并使用正确的 target
