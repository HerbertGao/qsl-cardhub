## 上下文

阶段 1 的创始租户 `bh2ro` 写凭据是这样诞生的：迁移 SQL 里留占位符，运维手工跑 `node -e '...'` 算 hash、再手工替换。这一次性手法不可复用，占位符忘替换还会占用 `idx_tenant_credentials_active_key_hash` 的 active 唯一槽位致表驱动对真实 Key 永久 miss（阶段 1 已记此坑）。

服务端契约现状（worker `src/worker/index.js`，均已上线，**契约真源 = `openspec/specs/cloud-backend-api`**）：`resolveTenant(env,key)` 用 `sha256(trim(key))` 查 `tenant_credentials WHERE key_hash=? AND status='active'`（**不查 `scope`**）；`crossCheckTenant` 比对 `X-Tenant-Id`（401 `auth_failed` / 403 `tenant_mismatch`）；`/ping` 回显 `{tenant, fallback}`；`/sync` 必填 `client_id`（缺则 400）、走 OCC `base_version`（严格整数）/`force===true`/409/返回 `server_version`；`GET /pull` 供恢复（JSON 串/布尔列还原为对象/布尔）；`/t/<slug>/` 前缀在 `/ping`·`/sync`·`/pull` 上直接 404（非查询面 gate）；`defaultTenant(env)=env.DEFAULT_TENANT||'bh2ro'`。表结构：`tenants(tenant_id PK CHECK slug,...)` + `tenant_credentials(id PK, tenant_id, scope, key_hash, status, ...)`（`scope` 可空无默认）+ active key_hash 全局部分唯一索引。

`docs/cloud-sync-api-spec.md` 早于全部多租户改造（单一 `API_KEY`、无 `/pull`/OCC/`X-Tenant-Id`/租户、且把 `client_id` 当隔离键——正是被淘汰的反模式）——与真实契约严重脱节。

## 目标 / 非目标

**目标：**
- 一条可复用、**离线**、不落明文 Key 的命令，把「(tenant, key)」变成可直接 `wrangler d1 execute` 的 SQL。
- 把自托管 API 规范文档校正到当前真实契约，并**声明以 `cloud-backend-api` 主规范为真源**（文档跟踪它、不另立并行契约）。
- 自托管最小路径文档：区分「只实现同步 API」与「自部署本仓库 worker」两类。

**非目标：**
- 不写 turnkey 单租户产品、不做注册自助端点（注册纯线下签发）。
- 不连 D1、不替运维执行写入（与阶段 1「AI 不碰生产迁移」一致——脚本只产 SQL，运维自跑）。
- 不做 AI-facing CLI 框架（子命令/doctor/`--json`/插件打包）。
- 不改 worker、不改桌面端、无 D1 迁移、不新增运行时端点；不改 `wrangler.toml.example` 的 `DEFAULT_TENANT`（仅文档说明）。
- 读取面 PoW 旁路（`REQUIRE_POW`）、缺头收紧（`REQUIRE_TENANT_HEADER`）不在本变更，文档仅标「计划中」。
- 不引入 pepper/加盐（worker 仍 `sha256(trim(key))`，阶段 1 决策 D），只在脚本侧加最小 Key 强度门 + 文档建议高熵生成。

## 决策

- **D1 hash 必须与 worker 逐字节一致**：worker `crypto.subtle.digest('SHA-256', TextEncoder().encode(key.trim()))` → hex；脚本 `node:crypto` `createHash('sha256').update(key.trim(),'utf8').digest('hex')`。二者都对「JS `String.prototype.trim()` 后的 UTF-8 字节」做 SHA-256，输出同（已差分验证）。**禁** `tr -d '[:space:]'`（去全部空白，与 trim 仅去首尾不等价，已验证 `foo bar` 两路 hash 不同）。
- **`id` 用完整 `key_hash` 派生，不截断**：`id = '<slug>-' || key_hash`（64 位 hex 全量）。彻底消除「前 8 位 32-bit 截断 → 同租户两个合法 Key 的 hash8 相同 → 误撞 PK」（Codex/CR/RC 共同指出）。
- **重跑语义明确（不笼统称"幂等"）**：`tenants` 行 `INSERT OR IGNORE`（真幂等，已存在不动）；`tenant_credentials` 行**普通 `INSERT`，安全失败**——同一 Key（同 id + 同 key_hash）重签 → PK / active 唯一索引拒（明确报错，运维据此知道已签发，绝不静默覆写）；跨租户复用同一 Key → 全局 active 唯一索引拒（`idx_tenant_credentials_active_key_hash` 无 tenant_id 列）。脚本不用 `INSERT OR IGNORE` 写凭据（否则跨租户复用会被静默吞掉、运维不自知）。
- **不暴露 `--scope`**：worker `resolveTenant` 只 `WHERE key_hash=? AND status='active'`、**从不读 `scope`**（已核 `index.js:323-329`）。暴露一个不被强制执行的 `--scope` 是误导旋钮，且是脚本里唯一会被插值进 SQL 串字面量的自由文本（注入面）。故**移除 `--scope`**，恒写常量 `scope='sync'`（仅元数据）——一举消除注入面 + 误导。slug 经正则、Key 经 hash，输出再无自由文本插值。
- **slug 校验对齐服务端**：`^[a-z0-9-]{1,32}$`，**拒绝不转换**（大写/非法直接报错退出，不静默小写化）。与 `tenants.tenant_id` CHECK（`length BETWEEN 1 AND 32 AND NOT GLOB '*[^a-z0-9-]*'`）同语义。
- **拒空 Key + 拒弱 Key**：`key.trim()===''` → 报错退出，绝不输出 `sha256('')`（=`e3b0c442…b855`，阶段 1 安全红线）；`key.trim().length` < 阈值（默认 **32**，对齐脚本建议的 `openssl rand -hex 32`=64 字符；16 对 unsalted sha256 太低、手敲短口令可离线爆破）→ 报错。注意：min-length **仍只是长度下限、非熵保证**（`32×'a'` 仍会过、但已挡掉常见手敲短口令）——真正的强度靠文档/stderr 引导用 `openssl rand -hex 32`。
- **`--key-stdin`**：默认从参数读 Key 会进 shell history；`--key-stdin` 从 stdin 读，运维可 `printf %s "$KEY" | node mint-credential.mjs <tenant> --key-stdin`。trim 吸收 `echo` 尾随 `\n`/CRLF，与参数式同 hash（已验证）。
- **退出码语义化**：成功 0（SQL 走 stdout）；slug/空 Key/弱 Key/参数错 → 非 0 + stderr 说明（可直接 `| wrangler d1 execute --file=-`）。
- **自托管默认 slug `default` 与 `env.DEFAULT_TENANT` 不可分**：worker 内置兜底 `bh2ro`，deploy 文档红线「seed slug 必须 == `DEFAULT_TENANT` 否则裸查询面静默空结果」。故文档凡写 `default` 处必同时要求 `DEFAULT_TENANT="default"` + seed 匹配租户，并交叉引用该红线。官方云创始租户仍 `bh2ro`。
- **API 规范文档以 `cloud-backend-api` 为真源、不另立能力**：文档是 tasks 交付而非新建 OpenSpec 能力——`cloud-backend-api` 主规范已规范化 `/ping`·`/sync`OCC·`/pull`·`X-Tenant-Id`·`client_id` 全部契约；再建一个"文档准确性"能力会在归档后形成两份并行漂移的契约真源（重演阶段 4-C1 ADDED 致主规范自相矛盾的教训，4-C2 正是改用 MODIFIED 规避）。文档文首声明「以 `cloud-backend-api` 为准」，使其定义上从属、永不竞争。
- **文档范围**：聚焦写入/同步面（`/ping`、`/sync` OCC、`/pull`、`X-Tenant-Id`、租户/凭据模型、`client_id` 降级为仅溯源非归属、裸路径非 `/t/`、默认 `default`）——这是自托管者实现桌面端同步所需的全部。读取面防爬（恒 PoW + KV + `SESSION_SECRET`）属「自部署本仓库 worker」一类，文档单列并标 PoW 旁路「计划中」，不要求「只实现同步 API」一类支持。

## 风险 / 权衡

- [脚本 hash 与 worker 漂移] → 脚本内置一行**硬编码字面量**自检：`assert(sha256('qsl-mint-selfcheck') === '989e608711151a9484b398e3af86cb80f1449d3fa9824da303383c30f1d215fe')`（独立已知向量，**非**用 `createHash` 自比自——避免恒真的 vacuous assert）；与 worker `sha256(trim(key))` 同语义。
- [运维误把明文 Key 写进 SQL/history] → 脚本绝不回显 Key、输出只含 hash；推荐 `--key-stdin`；文档明示 Key 不进任何文件。
- [文档与 worker 契约再次脱节] → 不靠"更新历史"散文（正是让旧文档腐烂的机制）：文档定义上从属 `cloud-backend-api`（冲突以后者为准）；归档任务加 grep 门，证文档不复述与主规范并行的规范性断言。
- [跨租户/重签的 wrangler 报错被误读为脚本 bug] → 文档说明：凭据已存在时 `wrangler d1 execute` 抛 SQLite 约束错误（非脚本错误），表示该 Key/租户已签发。
- [自托管者把 `api_url` 配成 `/t/<slug>`] → 文档明示同步端点为裸路径，`/t/` 前缀在 `/ping`·`/sync`·`/pull` 上 worker 返 404（已核 `index.js:391-395`）。

## Migration Plan

无 D1 迁移、无部署。交付即文件落地：
1. 合并后，运维需签发新租户时本地跑脚本 → 得 SQL → `wrangler d1 execute qsl-sync --remote --file=-`。
2. 自托管者照 `cloud-sync-api-spec.md` 实现同步 API，或自部署 worker（按 deploy 文档配齐 KV/`SESSION_SECRET`）。
回滚 = 还原文件，无运行时影响。

## Open Questions

无。
