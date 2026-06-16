## 为什么

GitHub Actions 的 PR 构建当前对**所有** PR 一刀切触发 `build.yml` 的全平台 `cargo tauri build`（macOS/Windows 多 target 矩阵，耗时数分钟）。但本仓是**桌面端（Rust/Tauri）+ 独立查询端（`web_query_service/`，Cloudflare Worker，JS/Vue）**双栈：纯查询端、纯 `openspec/` spec、纯 docs 的 PR 跑 Rust 桌面构建**毫无意义、纯浪费**——近期查询端的 PR（如 4-A、4-C1）每次都白跑一遍 Rust 全平台构建。同时查询端本身**没有任何 CI**（其单测 `test:unit`、构建 `build` 在 PR 上从不跑）。

新需求：**CI 运行必须与改动区域相关（meaningful CI）**——改哪端跑哪端的检查。`build.yml` 的 `Build`/`Build Summary` 当前**非** master 分支保护的必需状态检查（仓主手动合并），故可直接用 path 过滤跳过、不会卡合并。

## 变更内容

- **`build.yml` 路径范围化**：`paths-ignore` 增加 `web_query_service/**` 与 `openspec/**`——纯查询端/纯 spec/纯 docs 的 PR **不再触发** Rust/Tauri 构建与质量检查。触及桌面端文件（`src/**`、`web/**`、`Cargo.*`、`tauri.conf.json`、`capabilities/**` 等）的 PR 仍照常全量构建（`paths-ignore` 仅在**全部**改动文件都命中忽略模式时才跳过）。
- **新增查询端 CI `web-query-service.yml`**：`pull_request` 到 master 且 `paths: ['web_query_service/**']` 时触发，运行查询端自己的快速检查——`pnpm install` + `pnpm run test:unit`（`node --test`）+ `pnpm run build`（`vue-tsc --noEmit && vite build`）。让查询端 PR 跑**有意义**的检查而非空白或 Rust 构建。（worker smoke 需 `wrangler dev` 较重，本期不入 CI，留本地/手动。）

## 功能 (Capabilities)

### 新增功能
<!-- 无新增能力；属既有 build-and-packaging 能力内 CI 行为修订。 -->

### 修改功能
- `build-and-packaging`: 修改「GitHub Actions 自动构建」需求——「Pull Request 构建验证」场景增加路径范围化（桌面端构建只在桌面端相关变更触发，纯查询端/spec/docs 不触发）；新增「查询端 Pull Request 验证」场景（`web_query_service/**` 变更触发查询端 CI = 单测 + 构建，不触发 Rust 构建）；钉死「CI 运行与改动区域相关」原则。

## 影响

- **CI 配置**：`.github/workflows/build.yml`（`paths-ignore` 增列）；新增 `.github/workflows/web-query-service.yml`；`.github/workflows/README.md` 增补查询端 workflow 说明。
- **无代码/运行时影响**：仅 CI 触发范围；不改桌面端/查询端任何源码或部署。
- **前提**：`Build`/`Build Summary` 非必需状态检查（仓主手动合并）；若未来设为必需，纯 paths 跳过会致必需检查 pending，需改「必检 gate job 永远跑、对无关改动 no-op」模式（本期不需要）。
- **回滚**：还原 workflow 文件即可。
