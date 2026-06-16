## 上下文

本仓双栈：桌面端（Rust/Tauri，`src/`、`web/`、`Cargo.*`、`tauri.conf.json`）+ 独立查询端（`web_query_service/`，Cloudflare Worker，JS/Vue）。`build.yml` 在 PR 触发全平台 `cargo tauri build`，`paths-ignore` 仅含 `**/*.md`/`docs/**`——纯查询端/纯 `openspec/` PR 仍白跑 Rust 全平台构建。查询端无任何 CI。

约束：`Build`/`Build Summary` 非 master 分支保护必需检查（仓主手动合并）→ 可直接用 `paths`/`paths-ignore` 跳过，不会卡合并。

## 目标 / 非目标

**目标：**
- 纯 `web_query_service/**`/`openspec/**`/docs 的 PR 不触发 Rust/Tauri 构建。
- 查询端 PR 跑有意义的检查（其单测 + 构建）。

**非目标：**
- 不改桌面端/查询端任何源码、不改 release.yml（发布走 tag、不受影响）。
- 不把 worker smoke（需 `wrangler dev`）纳入 CI（较重，留本地）。
- 不引入「必检 gate job」复杂模式（前提 build 非必需检查，本期不需要）。

## 决策

### D1：`build.yml` 用 `paths-ignore` denylist（非 `paths` allowlist）

`paths-ignore` 增 `web_query_service/**`、`openspec/**`（已有 `**/*.md`、`docs/**`）。语义：PR 的**全部**改动文件都命中忽略模式才跳过；只要触及一个桌面端文件就照常全量构建——**漏配桌面端路径的风险为零**（denylist 默认跑）。`paths` allowlist 更精确但易漏列桌面端路径致该跑不跑，故选 denylist。

### D2：查询端独立 workflow（非塞进 build.yml）

新建 `web-query-service.yml`，`paths: ['web_query_service/**']` allowlist 触发（查询端是封闭目录、allowlist 不会漏）。跑 `pnpm install --frozen-lockfile` + `test:unit`（node --test，hermetic 快）+ `build`（vue-tsc + vite）。工作目录 `web_query_service/`，pnpm 缓存键基于 `web_query_service/pnpm-lock.yaml`。独立 workflow 比塞进 build.yml 的条件 job 更清晰、与桌面端 CI 解耦。

### D3：smoke 不入 CI

`run_worker_smoke.sh` 依赖 `wrangler dev --local`（miniflare）+ 多次重启 + PoW，较重且仓主本地实测有状态/时序 flake。CI 只跑 hermetic 的 `test:unit`（92/92）+ `build`——快、稳、有意义。smoke 留本地/手动（部署前）。

## 风险 / 权衡

- **[paths-ignore 漏跑桌面端]** denylist 默认跑 → 不会漏（只会多跑，不会少跑）。✓
- **[查询端 paths allowlist 漏触发]** `web_query_service/**` 是封闭目录，查询端改动必在其下 → 不漏。若查询端将来依赖仓根共享文件需扩 paths。
- **[build 未来设为必需检查]** 则 paths 跳过致必需检查 pending 卡合并——本期前提非必需；spec 已钉死「若设必需须改 gate 模式」。
- **[两 workflow 对同一 PR 都不触发]** 纯 `openspec/`/docs PR：build.yml 被 ignore、web-query-service.yml 的 paths 不命中 → 两者都不跑（符合预期，纯文档无需 CI）。

## 验证

CI path 过滤只在真实 PR 上生效、无法本地完全复现。强锚：① yaml 语法/结构静态校验（`actionlint`/`yamllint` 若有，或 `python -c yaml.safe_load`）；② 本地复跑查询端 CI 将执行的命令（`pnpm install` + `test:unit` + `build`）确认全绿；③ 人工核对 paths/paths-ignore 模式覆盖正确（桌面端路径不在 build.yml 的 ignore 内、`web_query_service/**` 在内且命中查询端 workflow 的 paths）。合并后首个查询端 PR 实地确认 Rust 构建不跑、查询端 CI 跑。
