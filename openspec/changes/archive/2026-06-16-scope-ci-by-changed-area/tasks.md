## 1. build.yml 路径范围化

- [x] 1.1 `.github/workflows/build.yml` 的 `on.pull_request.paths-ignore` 增加 `web_query_service/**`、`openspec/**`（保留既有 `**/*.md`、`docs/**`、`.github/workflows/README.md`）
- [x] 1.2 确认桌面端路径（`src/**`、`web/**`、`Cargo.*`、`tauri.conf.json`、`capabilities/**`、`build.rs`）**不在** ignore 内（denylist 默认跑、不漏）；并加「必检×path 过滤死锁」警告注释

## 2. 新增查询端 CI workflow

- [x] 2.1 新建 `.github/workflows/web-query-service.yml`：`on.pull_request` 到 master、`paths: ['web_query_service/**']`、`workflow_dispatch`；`concurrency` 取消同分支在途；`permissions: contents: read`、`timeout-minutes: 15`
- [x] 2.2 单 job（ubuntu-latest，`defaults.run.working-directory: web_query_service`）：checkout → pnpm/action-setup（`version: 10` 对齐 packageManager pnpm@10.28.2，仓根无 package.json 故显式）→ setup-node（cache=pnpm、cache-dependency-path=`web_query_service/pnpm-lock.yaml`）→ `pnpm install --frozen-lockfile` → `pnpm run test:unit` → `pnpm run build`
- [x] 2.3 **禁止**任何 `cargo`/`tauri` 步骤；smoke 不纳入（留本地）

## 3. 前置修复（review-loop 抓到）

- [x] 3.1 **lockfile 须可追踪**：`web_query_service/.gitignore` 删除 `pnpm-lock.yaml` 行 + `git add web_query_service/pnpm-lock.yaml`（否则 CI `--frozen-lockfile` 无 lockfile 必失败）
- [x] 3.2 **test:unit Node 20 兼容**：`web_query_service/package.json` 的 `test:unit` 由 `node --test "verify/**/*.test.js"`（`**` 需 Node 21+）改为 `node --test verify/*.test.js`（shell 展开、Node 版本无关；CI 钉 node 20）

## 4. 文档

- [x] 4.1 `.github/workflows/README.md` 增补 `web-query-service.yml` 说明 + build.yml 路径范围化说明 + 缓存段 npm→pnpm 更正

## 5. 验证

- [x] 5.1 yaml 静态校验：两个 workflow 经 `actionlint` 通过（exit 0）
- [x] 5.2 用 CI 精确 pnpm（`corepack pnpm@10.28.2`）复跑查询端 CI 全序列：`install --frozen-lockfile`（Done）+ `test:unit`（92/92）+ `build`（绿）；lockfile 已追踪、`git check-ignore` 不命中
- [x] 5.3 人工核对 path 模式 + 新维度扇出 grep：build.yml ignore 含 `web_query_service/**`+`openspec/**`、桌面端路径全不在；web-query-service.yml paths=`web_query_service/**`；release.yml=tag push、reusable=workflow_call → 查询端 PR 无任何 workflow 跑 Rust
- [x] 5.4 对抗 review-loop 到 APPROVE（修掉 lockfile gitignored / Node 20 glob 两个本地强锚假绿的 blocker）

## 6. 合并后实地确认

- [ ] 6.1【合并后】下一个纯查询端 PR：确认 `build.yml`（Rust 构建）**不触发**、`web-query-service.yml` 触发并全绿
- [ ] 6.2【合并后】下一个触及桌面端的 PR：确认 `build.yml` 仍照常全量构建
- [ ] 6.3 用户确认后 `openspec-cn archive scope-ci-by-changed-area`（增量并入 `build-and-packaging` 主规范）
