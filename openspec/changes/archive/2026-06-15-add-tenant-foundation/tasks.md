## 1. 前置校验与量级摸底

- [x] 1.1 对**每张业务表 + `sync_meta`** 跑 `SELECT client_id, COUNT(*) … GROUP BY client_id`，确认单一所有者（`sync_meta` 因 worker 从不删旧行、换机/重装会累积多行，必须纳入）；非单一则停下，按既定步骤（确认仅一台机器同步过 / 择 `received_at` 最新一条、余删）人工处置后重来
- [x] 1.2 **先确认该 D1 计划档（Free 50 / Paid 1000 子请求/调用）**；跑 `SELECT COUNT(*)` 量各业务表真实行数，按**兜底最坏路径**把**单次 `/sync` 全部子请求**计入预算（`resolveTenant` 凭据查询 + 兜底计数器 UPDATE〔此二者为 batch 外独立 `.run()`、计入单次调用子请求总数但不占 batch 内语句额度〕+ `DELETE×5` + `N×INSERT` + `sync_meta` upsert）评估是否进单 `DB.batch`；设明确阈值（Paid 下总语句 ≤~850 走真单 batch；Free 50 下小数据量也会撞限须分块/影子表）
- [x] 1.3 `wrangler d1 export` 远端全量导出（默认含 schema+data），留作迁移回滚点；**记录当前生产 worker 部署版本号（Cloudflare Deployments）或对应 git commit** 作为回滚步骤④的目标；迁移窗口内冻结桌面端写入

## 2. schema.sql 演进（新建 D1 的源 schema）

- [x] 2.1 新增 `tenants`（`tenant_id TEXT NOT NULL PRIMARY KEY CHECK (length(tenant_id) BETWEEN 1 AND 32 AND tenant_id NOT GLOB '*[^a-z0-9-]*')`——否定字符类，**禁用** `GLOB '[a-z0-9-]*'`（其 `*` 只约束首字符、`abc!` 会通过）；显式 `NOT NULL`）、`name`、`tier`、`status`、`created_at`
- [x] 2.2 新增 `tenant_credentials`（`id`/`tenant_id`/`scope`/`key_hash`/`status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active','revoked'))`/`created_at`/`last_used_at`）+ 部分唯一索引 `CREATE UNIQUE INDEX … ON tenant_credentials(key_hash) WHERE status='active'`
- [x] 2.3 新增 `tenant_routes`（`route_key TEXT NOT NULL PRIMARY KEY`——rowid 表 TEXT PK 不隐含 NOT NULL 故显式加，`tenant_id TEXT NOT NULL`）
- [x] 2.8 新增兜底计数表 `service_counters (name TEXT PRIMARY KEY, count INTEGER NOT NULL DEFAULT 0)`（承载 `resolveTenant` 兜底命中计数；进 schema.sql 源，确保重建 D1 也有此表）
- [x] 2.4 业务表 `projects`/`cards`/`sf_senders`/`sf_orders`/`app_settings`：`client_id` → `tenant_id`，主键改 `(tenant_id, id)`（`app_settings` 为 `(tenant_id, key)`）
- [x] 2.5 **完整重建索引清单**（删 client_id 后旧 5 个 `idx_*_client` **弃用不重建**；下列业务索引必须在新表上重建，否则读路径退化全表扫）：`cards` 加 `(tenant_id, callsign COLLATE NOCASE)`（取代旧单列 `idx_cards_callsign`）+ 保留 `project_id`、`created_at`；`sf_orders` 保留 `order_id`、`waybill_no`、`card_id`（route-push 反查依赖）；`projects` 保留 `created_at`；**`sf_senders` 无非-client 业务索引、无需额外索引**；**`app_settings` 主键 `(tenant_id, key)` 已覆盖按 key 查、无需额外索引**（显式记录，避免被读成"漏了这两表"）
- [x] 2.6 `sync_meta` 改为 PK `tenant_id` + `last_client_id` + `server_version INTEGER NOT NULL DEFAULT 0` + `sync_time`/`received_at`
- [x] 2.7 更新 `schema.sql` 顶部注释（隔离键由 `client_id` 改述为 `tenant_id`）

## 3. 迁移 SQL 文件（生产 D1 一次性）

- [x] 3.1 新建 `web_query_service/migrations/0001_tenant_foundation.sql`：建三张租户表（含部分唯一索引、status CHECK、tenant_id CHECK）+ **`CREATE TABLE service_counters (name TEXT PRIMARY KEY, count INTEGER NOT NULL DEFAULT 0)`**（迁移文件**必须自建**此表，列定义与 2.8 schema.sql 源逐字一致；生产 D1 是按旧 schema 建的、本无此表，遗漏则 3.4 的 seed INSERT 会 `no such table` 致整文件回滚——schema.sql 与迁移文件是两条独立路径须双写）
- [x] 3.2 逐业务表「建新表（含 `tenant_id`/新 PK/**第2.5节全部索引一并建好**）→ `INSERT…SELECT` 回填 `tenant_id='default'` → DROP 旧表 → RENAME」；**文件内禁写 `BEGIN`/`COMMIT`/`SAVEPOINT`**（D1 不支持、会 `SQLITE_AUTH` 失败），原子性靠 `--file` 全文件失败回滚
- [x] 3.3 重建 `sync_meta` 为终态并回填：`last_client_id` 取最新一行 `client_id`，用确定性 `ORDER BY received_at DESC, client_id DESC LIMIT 1`（并列由 client_id 兜底；`received_at` 可字典序==时序由 `serverTime()` 定宽零填充 ISO 保证、已核，改其格式须同步校验）（或置 NULL），`server_version=0`，**禁止**照搬多行致 PK 冲突
- [x] 3.4 seed：`INSERT INTO tenants … 'default'` + `INSERT INTO tenant_credentials … key_hash='<占位:离线 sha256(trim(API_KEY))>'`（占位符，执行前由用户替换为真实 64 位小写 hex）+ `INSERT INTO service_counters(name, count) VALUES ('auth_fallback', 0)`（兜底计数初始行，避免验收读到「行不存在」）
- [x] 3.5 文件**末尾**加执行后自检 `SELECT key_hash FROM tenant_credentials WHERE key_hash LIKE '<占位%'`（须 0 行；注：此自检仅防占位残留、**不**证 hash 算对，hash 正确性由部署后 7.3「兜底计数==0」兜住，故 7.3 与迁移属同一不可分交付单元）；文件头注释写明前置：先 `d1 export` 备份、先跑 1.1 单一所有者校验、冻结写入、离线算 hash 命令 `node -e 'process.stdout.write(require("crypto").createHash("sha256").update((process.env.API_KEY||"").trim()).digest("hex"))'`（与 worker JS `trim()` 逐字符同语义；**禁用** `tr -d '[:space:]'`），均交用户在自己终端执行
- [x] 3.6 文件头注释写明回滚剧本（**有序 fail-closed、前滚优先**）：既不可单独退 worker（旧 worker 读 client_id 整站 500）、也不可单独退数据（恢复旧 dump 后新 worker 查无租户表整站 500）；按序：①新 worker 下线（部署返 503 临时版本 或 dashboard 解绑路由）→ ②清空迁移后 D1 全部表（**推荐建新空 D1 + 改 `wrangler.toml` 绑定**，天然规避漏表；若逐表 DROP 须含全部 12 张：三租户表 + `service_counters` + 业务 5 表 + `sync_meta` + 全局 `callsign_openid_bindings`/`sf_route_log`，漏清全局表 → import 撞 PK 失败）→ ③import 步骤 1.3 的 dump → ④部署回旧 worker → ⑤恢复路由（任一中间态皆 fail-closed 500/503、非裸写）

## 4. worker：写入侧（/sync）与 /ping

- [x] 4.1 实现 `resolveTenant(env, key)`：`sha256(trim(key))` 查 `tenant_credentials`（`status='active'`）得 `tenant_id`；未命中且 `trim(key) === trim(env.API_KEY)` 且 `env.API_KEY` 非空 → 返回 `default` 并**递增 D1 计数行**（`UPDATE service_counters SET count=count+1 WHERE name='auth_fallback'`，**禁用 KV**，KV fail-open 会吞递增；迁移已建表+seed 该行）；写失败判据**用 `.run()` 返回的 `result.meta.changes`（D1 受影响行数），`=== 0`**（行缺失：漏 seed）**或**写抛错（漏建表 → `no such table`）→ 视为写失败、`/sync` 返非 200、**不静默吞**（**禁用** SQLite `SELECT changes()` 那种多一次往返/易误实现的写法）。此 UPDATE 是 `resolveTenant` 内**独立 `.run()`、不进 4.4 数据 batch**，但仍计入单次调用子请求预算（见 1.2）；`env.API_KEY` 未配置或仍不命中 → 401（**禁止**沿用现状「env.API_KEY 空即放行」；「空」统一判为 `!env.API_KEY || trim(env.API_KEY)===''`，纯空白 secret 经 trim 成空串也算空，与 4.6 同源）
- [x] 4.2 `/sync` 改用 `resolveTenant` 解析 `tenant_id`，**删除现状 `index.js:239` 的 `token!==env.API_KEY` 前置 401 门**（保留它会让多 Key→同租户的表驱动凭据被旧门 401 架空，鉴权须统一由 `resolveTenant` 命中决定）；**保留** `client_id` 存在性校验作请求形态契约（缺失 400），但只把它写入 `sync_meta.last_client_id` 溯源、不参与归属（`client_id` 是客户端可控字段，落 `last_client_id` 前**必须**长度归一 ≤128：**超长截断**（不因超长拒绝合法桌面端；溯源列截断可接受），防超长串污染溯源列）
- [x] 4.3 **整段删除** `/sync` 内联 `CREATE TABLE IF NOT EXISTS`（`index.js:259-297`）——迁移后库结构由迁移文件保证，缺表时让 `/sync` 显式报错而非静默重建无索引空表
- [x] 4.4 将 `DELETE … WHERE tenant_id=?`（五业务表）+ 全部 `INSERT`（绑 `tenant_id`）+ `sync_meta` upsert 合入单个 `DB.batch` 原子执行；按 1.2 阈值，超阈则分块且 `DELETE + 首块` 同 batch（并在 spec 如实标注跨块非原子）
- [x] 4.5 `sync_meta` 写入按 `tenant_id` upsert，落 `last_client_id`/`sync_time`/`received_at`（`server_version` 本期不改写），并入 4.4 同 batch
- [x] 4.6 `/ping` 凭据比对：仅在 `env.API_KEY` 侧补 `trim`（token 侧 `getBearerToken` 已 trim），消除 secret 含尾随空白时桌面端「测试连接」401；并与 `/sync` 一致——`env.API_KEY` 空时 `/ping` **也返回 401**（删除现状 `index.js:226` 的「env 空即放行」fail-open）

## 5. worker：读取侧与顺丰 join

- [x] 5.1 `/api/query`、`/api/callsigns/:callsign` 注入 `WHERE c.tenant_id = ? AND c.callsign = ? COLLATE NOCASE`，`tenant_id` 取服务端常量 `default`；`projects` join 改为 `p.tenant_id = c.tenant_id AND p.id = c.project_id`（**保留现状 `LEFT JOIN` 语义**——勿改 INNER，否则无项目卡片行被丢、字段/行集变化破坏「移动端不变」；保留 `COLLATE NOCASE` 以命中 NOCASE 索引）
- [x] 5.2 顺丰 route-push 的 `sf_orders` join `cards`（`index.js:559`/`:567`，现状 `ON o.client_id=c.client_id AND o.card_id=c.id`）：**保留业务连接键 `o.card_id = c.id`**，仅把隔离键 `o.client_id=c.client_id` 替换为 `o.tenant_id = c.tenant_id`，并加 `WHERE o.tenant_id = ?` 绑服务端常量 `default`（非仅同租户自洽；漏 `card_id=c.id` 会变笛卡尔积错号）；代码注释标注「按 order 派生确定租户属阶段 4」
- [x] 5.3 grep 审查无任何读/写路径从前端/请求体参数取 `tenant_id`（覆盖 `/api/query`、`/api/callsigns`、**route-push 路径**）

## 6. 本地验证（优先 --remote 临时/preview D1）

- [x] 6.1 对一份含 `client_id` 的旧结构 D1 跑迁移 SQL，断言：业务表无 `client_id`、PK `(tenant_id,id)`、数据全 `tenant_id='default'`、`id` 不变、**全部业务索引存在**（`EXPLAIN QUERY PLAN` 断言 `cards` 呼号查询命中 `idx_cards_tenant_callsign`、route-push 命中 `idx_sf_orders_order_id`）。注：`--local` 可能对 DDL/RENAME 报 `SQLITE_AUTH`，本地通过不等于 remote，必要时用一次性 `--remote` 临时库验证
- [ ] 6.2 迁移 SQL 中段人为注入失败，断言整文件回滚、不留半迁移表——**确认性实测（上生产硬门）**：Cloudflare 文档已保证 `--file` 失败回滚到原状，本步确认当前 wrangler 版本行为一致；若实测为逐语句提交则改影子表（`cards_v2` 校验后 RENAME 切换）〔归档说明：未做人为失败注入实测；依赖 Cloudflare 文档的 `--file` 整文件回滚保证 + 2026-06-15 生产一次性迁移成功（423 changes 单文件落地），接受为 out-of-scope〕
- [x] 6.3 用现有全局 API_KEY 打 `/sync`：表驱动凭据命中、**兜底计数行存在且 `count===0`**（严格相等+存在性双检，缺失/不可读判 inconclusive 非 pass）、200、数据落 `default`；错误 Key → 401；`env.API_KEY` 置空 → 401（不放行）；额外对抗用例：在 `env.API_KEY` 尾部加空白，断言表驱动仍命中且兜底计数==0
- [x] 6.4 `/sync` 注入失败模拟，断言单 batch 回滚（不出现已删未写空表）
- [x] 6.5 手工 DROP 一张业务表后打 `/sync`，断言显式报错、**不**按旧结构静默重建
- [x] 6.6 `/api/query?callsign=…` 返回 `default` 数据、字段集与现状一致；构造 `?tenant_id=other` 注入尝试，**自动化断言**参数被忽略、不跨租户
- [x] 6.7 `/ping` 用含尾随空白的 `env.API_KEY` 断言仍 200（trim 生效）

## 7. 生产迁移与部署（用户执行）与验收

- [x] 7.1 向用户展示完整迁移操作清单（备份 → 单一所有者校验含 sync_meta → 冻结写入 → 离线算 hash 替换占位 → `wrangler d1 execute --file 0001_tenant_foundation.sql --remote` → 执行后占位符自检 0 行），由用户在自己终端执行
- [x] 7.2 部署 worker 新版本（含 4.x/5.x 全部改动 + 兜底计数器）
- [x] 7.3 验收（与迁移同一不可分交付单元）：现有桌面端 `/sync` 200 且落 `default`、**兜底计数行存在且 `count===0`**（表驱动真命中，>0 或计数行缺失/不可读视为 seed 失败须排查、判 inconclusive，非可接受）；桌面端「测试连接」`/ping` 200；现有移动端裸域查询照常字段集不变
- [x] 7.4 量化判据满足后（连续 N 次/T 窗口表驱动命中且兜底计数恒 0）记录「撤兜底」为后续清理项；阶段 4 上线第二真实租户前必须先迁 `callsign_openid_bindings`/`sf_route_log`（本期已知二次迁表点）
