## 为什么

阶段 4-C 已让桌面端申报租户（4-C2）、服务端交叉校验 `X-Tenant-Id`（4-C1）、首启引导与标题栏徽章（4-C3）全部落地，但**运维如何签发一个新租户的写凭据**始终没有工具：阶段 1 的 `bh2ro` 凭据是手工把占位符替换进迁移 SQL 得到的，易错、不可复用，且占位符忘替换会占用 active 唯一槽位致表驱动对真实 Key 永久 miss。同时面向自托管者的 `docs/cloud-sync-api-spec.md` **仍停留在多租户改造之前**（单一 `API_KEY`、无 `/pull`、无 OCC `base_version`/409、无 `X-Tenant-Id`、无租户模型），任何想自建后端的人都会照着过时契约实现、与桌面端对不上。

本变更补齐阶段 4-C 的最后一块：一个**离线**的凭据签发 CLI + 把自托管 API 规范与部署文档更新到当前真实契约。注册纯线下签发、无公开自助端点，因此这是工具与文档，不引入任何运行时端点、不改任何运行时行为。

## 变更内容

- **新增** `web_query_service/scripts/mint-credential.mjs`：离线（不连 D1、不落明文 Key）计算 `sha256(trim(key))`，输出 SQL（`INSERT OR IGNORE INTO tenants` + `INSERT INTO tenant_credentials`），供运维 `wrangler d1 execute --remote --file=-` 自行执行。
  - 校验 slug 对齐服务端 CHECK（`^[a-z0-9-]{1,32}$`，拒绝不转换）、拒绝空 Key、拒绝过短/低熵 Key（提示用 `openssl rand -hex 32` 生成）；`id` 用完整 `key_hash` 派生避免截断撞 PK；依赖 `idx_tenant_credentials_active_key_hash` 全局唯一索引防跨租户复用同一 Key。
  - **重跑语义明确**：`tenants` 行幂等（`OR IGNORE`）；`tenant_credentials` 行**安全失败**——同一 Key 重签报明确错误（不静默覆写），靠唯一约束兜底。
  - **不暴露 `--scope`**：worker 鉴权只查 `key_hash AND status='active'`、**从不读 `scope`**，故脚本恒写常量 `scope='sync'`（仅元数据），不给运维一个不被强制执行的误导旋钮。
  - 支持 `--key-stdin` 让 Key 不进 shell history。
  - ponytail：这是单用途离线脚本，**不是** AI-facing CLI——不做子命令/doctor/插件打包等重型框架。
- **更新** `docs/cloud-sync-api-spec.md`：重写到当前契约，并在文首声明**以 `cloud-backend-api` 主规范为契约真源**（本文档跟踪它、冲突以它为准），避免又一份会漂移的并行契约。覆盖 `Authorization: Bearer <key>` + 可选 `X-Tenant-Id`（不一致 403 `tenant_mismatch`、缺头向后兼容）、`GET /ping`（回显 `{tenant, fallback}`）、`POST /sync`（必填 `client_id`、OCC `base_version` 严格整数 / `force===true` / 409 / 返回 `server_version`）、`GET /pull`（object/boolean 还原）；端点为**裸路径**（`/t/<slug>/` 前缀在 `/ping`·`/sync`·`/pull` 上 worker 返 404）。
- **更新** `docs/web-query-service-deploy.md` 新增「新增租户与自托管」节：mint 脚本签发流程；并**区分两类自托管**——(a) 只实现同步 API 的自定义后端（仅需 Bearer 鉴权，无需 PoW/会话/KV）；(b) 自部署本仓库 worker（公共查询面恒 PoW，需 `SESSION_SECRET` + `RATE_LIMIT` KV）。
- 自托管默认 slug 用 `default`，但**必须**与 `env.DEFAULT_TENANT` 一致（worker 内置兜底是 `bh2ro`）——文档明示并交叉引用 deploy 文档既有红线「seed slug 必须 == `DEFAULT_TENANT` 否则裸查询面静默空结果」。

## 功能 (Capabilities)

### 新增功能
- `credential-minting`: 离线租户写凭据签发工具的行为契约（不连库、不落明文、`tenants` 幂等 / `credentials` 安全失败、slug 校验拒空 Key、拒弱 Key、hash 与 worker 逐字节一致、全 hash 派生 id、`--key-stdin`、跨租户复用撞码安全）。

### 修改功能
<!-- 无。本变更不改任何运行时行为。文档（cloud-sync-api-spec.md / 自托管节）作为本变更的 tasks 交付，并声明以 `cloud-backend-api` 主规范为契约真源——故不新建会与 `cloud-backend-api` 并行漂移的"文档准确性"能力（避免阶段 4-C1 归档后主规范自相矛盾的教训重演）。 -->

## 影响

- 新增 `web_query_service/scripts/mint-credential.mjs`（离线 Node 脚本，无新依赖，用 `node:crypto`）。
- 重写 `docs/cloud-sync-api-spec.md`（文首声明以 `cloud-backend-api` 为真源）；扩 `docs/web-query-service-deploy.md`。
- **无** worker 代码改动、**无**桌面端代码改动、**无** D1 迁移、**无**新增运行时端点；不改任何线上行为。
- 推迟项：读取面 `REQUIRE_POW` 自托管免 PoW 开关、`REQUIRE_TENANT_HEADER` 全量升级后收紧——均不属本变更，文档仅标「计划中」。`tenant-isolation` 主规范把 `tenants.status` 的 `CHECK(status IN (...))` 枚举约束前瞻地交给「4-C4 线下签发工具」，但该约束属 schema 变更（需 D1 迁移，本变更显式不含）——故仍推迟到后续带迁移的变更；本工具恒写 `status='active'`，仅消除签发侧误写（如 `'Active'`/`' active'`）的运营误差向量。
