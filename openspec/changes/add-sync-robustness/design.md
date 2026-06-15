## 上下文

阶段 1（已上生产、已归档 `2026-06-15-add-tenant-foundation`）的地基：

- `sync_meta(tenant_id PK, last_client_id, server_version INTEGER NOT NULL DEFAULT 0, sync_time, received_at)`——`server_version` 列已建、**值恒 0、无任何读写逻辑**（阶段 1 明确「只占列、OCC 留阶段 2」）。
- `/sync`（`web_query_service/src/worker/index.js`）已是：`resolveTenant(env, token)` 按 Key 解析 `tenant_id` → 单个 `DB.batch([DELETE×5, INSERT×N, sync_meta upsert])` 按租户全量替换。`sync_meta` upsert 当前**不写 `server_version`**（由 DEFAULT 保留 0）。
- 桌面端（Rust/Tauri）：`SyncConfig{api_url, client_id, last_sync_at}`（`src/sync/config.rs`，持久化于 `sync.toml`）；`sync_data()`（`src/sync/client.rs`）发 `{client_id, sync_time, data}`；`export_database()` 存在、**无 import/从云端重建本地库的路径**。

D1 / Workers 运行时约束（沿用阶段 1 review 钉死的事实）：

1. **`DB.batch()` 是原子单元（全或无），但不能在某条语句 `changes==0` 时中途中止**——没有 batch 内的条件分支/`RAISE` 短路。
2. D1 不支持用户侧 `BEGIN/COMMIT/SAVEPOINT`（写了 `SQLITE_AUTH`）；原子性只能靠单个 `DB.batch()` 或单个 `--file`。
3. SQLite 写事务**库级串行**：两个并发 `/sync` 的 batch 不会交错，后者等前者提交后再跑。唯一竞态窗口是「batch 外的预读」与「batch」之间。
4. 写失败/影响行数判据用 `result.meta.changes`（**禁** `SELECT changes()`——`result.meta.changes` 按语句精确归属于对应的 prepared statement 结果，`SELECT changes()` 读「最近一条 DML 的影响行数」在多语句 batch 中归属易错位）。D1 `DB.batch()` 返回的结果数组**与语句数组位置一一对应**（官方保证），故可按位置读特定语句的 `meta.changes`。
5. **D1 batch 内每条 prepared statement 各计 1 次查询**；上限 **1000(Paid) / 50(Free) 是单次 Worker invocation 的查询总数**（与单条语句返回/影响的行数无关，非「按 batch 计 1 次」）。语句条数（**batch 内**）：**守卫路径 /sync = `6 + N`**（5 DELETE + N INSERT + 1 CAS；409 分支再 +1 次 batch 后 SELECT 读当前版本）；**无条件路径 /sync = `7 + N`**（5 DELETE + N INSERT + 1 upsert + 1 读回 SELECT）；**/pull = 6**（5 业务表 SELECT + 1 `sync_meta` SELECT）。**另：单次 Worker invocation 的查询总数还须计入 `resolveTenant`**——表驱动凭据查询 +1、命中 `env.API_KEY` 兜底再 +1（计数器 UPDATE），即真实 invocation 总数 ≈ 上述 batch 语句数 + 1~2。N = **五表行数之和**（projects+cards+sf_senders+sf_orders+app_settings；当前 cards≈415 为主、余表少量，守卫 /sync invocation 总计 ≈ `6 + N + resolveTenant(1~2)` < Paid 1000，有余量但**数据翻倍即逼近**）。另有「每条查询最多 100 个绑定参数」上限：单行守卫 INSERT 的绑定参数 ≈ 列数+2（cards 13 个），远低于 100——**单行守卫 INSERT 反而比多行 `VALUES` 批插更安全**（后者易超 100 参数）。
6. D1 的 read-your-writes、写恒命中 primary、单 batch 内顺序提交等强一致保证，**仅在默认 primary-only 路径成立**（不启用 read replication、不使用 Sessions API）。本期据此假设；见风险段。

## 目标 / 非目标

**目标：**
- `/sync` 加乐观并发护栏：陈旧/并发抢先 → 409 且**零数据改动**；命中 → 全量替换 + `server_version` 单调 +1，**原子**。
- `force=true` 逃生门：跳过比较、无条件覆盖、版本推进到「当前 +1」。
- 新增 `GET /pull`：按写入 Key 返回该租户全量快照 + 当前 `server_version`。
- 桌面端：持久化并正确更新 `base_version`；409 引导（下载或强制覆盖）；从云端恢复入口。
- **未升级桌面端零改动继续工作**（缺 `base_version` → 兼容降级为旧式无条件覆盖）。

**非目标：**
- 多端双向合并 / CRDT（与「全量覆盖」模型相悖，见设计蓝本「明确不做」）。
- 防爬、读取侧动态签名（阶段 3）；host/path → 租户路由、多租户前端（阶段 4）。
- 读取侧 `/api/query` 行为变更（本期不动）；`server_version` 与公开查询无关。
- 表结构变更（`server_version` 列阶段 1 已建；本期**无 D1 迁移**）。

## 决策

### 决策 1：OCC 原子性——「守卫每条写语句 + 版本递增放最后 + 读最后一句 changes 判 409」

这是本阶段的核心，也是最易实现错的点。约束 1 决定了**朴素方案行不通**：

- ❌ **「CAS 放 batch 第一句，changes==0 就当 409」**：batch 不会因第一句 0 行而中止，后面的 `DELETE+INSERT` 照样跑——数据已被覆盖，再返回 409 是假的（数据已丢）。
- ❌ **「batch 外预读 `server_version` 比对，不等就 409，等就跑 batch」**：预读与 batch 之间存在 TOCTOU（约束 3 的竞态窗口）。两端都读到 `base=5` 都通过预检，串行跑两个无条件 batch → 后者仍静默覆盖前者。
- ❌ **触发器 `RAISE(ABORT)`**：D1 对用户触发器支持不稳定、且把控制流藏进 DDL，可观测性差、回滚难。

**采用：守卫式条件写**。一个 batch 内：

```
-- ① 5 张业务表 DELETE，每条都加版本守卫子查询（绑 base_version）
-- 占位符一律用位置匿名 ? + .bind() 按顺序绑定；**禁混用 ?1/?2 与 ?**
-- （SQLite 中匿名 ? 与编号 ?NNN 共享同一参数命名空间，混用会让 ?1 指向第一个 ?、绑定错位）
DELETE FROM cards WHERE tenant_id = ?
  AND (SELECT server_version FROM sync_meta WHERE tenant_id = ?) = ?;
  -- .bind(tenant_id, tenant_id, base_version)
-- ②（projects / sf_senders / sf_orders / app_settings 同构）
-- ③ N 条 INSERT，用 INSERT…SELECT…WHERE 守卫（每行 +1 子查询，不增加语句条数）
INSERT INTO cards (tenant_id, id, …) SELECT ?,?,… 
  WHERE (SELECT server_version FROM sync_meta WHERE tenant_id = ?) = ?;
  -- .bind(<该行各列值…>, tenant_id, base_version)
-- ④ 最后一条：CAS 递增版本（放在 batch 末尾，使前面所有守卫都看到原始 base）
UPDATE sync_meta
   SET server_version = server_version + 1, last_client_id = ?, sync_time = ?, received_at = ?
 WHERE tenant_id = ? AND server_version = ?;
  -- .bind(last_client_id, sync_time, received_at, tenant_id, base_version)
```

**正确性论证**（依赖约束 3 串行 + 约束 1 全或无）：

- **命中**（无并发或本端先到）：版本递增在最后，故 ①②③ 全程看到 `server_version = base` → 全部执行；④ 命中 `=base` → `changes=1`，版本 `base→base+1`。
- **陈旧/被抢先**（并发对手已提交、版本变 `base+k`）：本 batch 串行在其后跑，①②③ 的守卫 `=base` 全部为假 → **0 行 DELETE、0 行 INSERT**（`INSERT…SELECT…WHERE false` 不写入）；④ `WHERE server_version=base` → `changes=0`。**整个 batch 净零改动**（约束 1 保证要么全做要么全不做，此处「全不做」）。
- **判定**：读 `DB.batch()` 返回数组**最后一个结果**的 `result.meta.changes`：`1` → 200；`0` → 409，且此前已数学保证零改动。

代价：每条 INSERT 多一个相关子查询（约 415 行 → 415 次对 `sync_meta` 主键的 O(1) 点查；**不增加语句条数**，故对约束 5 的「查询条数」预算无额外影响——子查询是该语句查询计划内的额外读，不另计 1 次查询）。可接受。

> **正确性已本机 SQLite 实测**（review 阶段）：陈旧时守卫 DELETE 0 行、`INSERT…SELECT…WHERE false` 0 行、末条 CAS `changes==0` 且版本不变；命中时全写、CAS `changes==1` 版本 +1；`sync_meta` 行缺失时守卫读得 NULL → `NULL=base` 为假 → 净 0 行、CAS `changes==0` → 409（见风险段「行缺失」）。`INSERT INTO t(...) SELECT <常量列表> WHERE <谓词>` 在 SQLite 中：谓词真→插 1 行、假/NULL→插 0 行（不会插入 NULL 行）。绑定参数顺序（`SELECT` 的值参数 + 守卫的 `tenant_id`/`base`）是实现易错点，须有回归测试（tasks 2.3a/6.x）。

### 决策 2：`force` 与「缺 base_version」合并为「无条件覆盖 + 单调递增」一条路径

- **`force=true`** 或 **请求未带 `base_version`**（旧桌面端）→ 走**不守卫**路径：`DELETE×5`（仅 `WHERE tenant_id=?`）+ `INSERT×N`（无守卫）+ 末尾用 **upsert 单调递增** sync_meta：

  ```
  INSERT INTO sync_meta (tenant_id, server_version, last_client_id, sync_time, received_at)
  VALUES (?, 1, ?, ?, ?)
  ON CONFLICT(tenant_id) DO UPDATE SET
    server_version = sync_meta.server_version + 1,
    last_client_id = excluded.last_client_id, sync_time = excluded.sync_time, received_at = excluded.received_at;
  ```

  这条**永远成功**（changes≥1）、永远推进版本，且兼容「行不存在」的引导场景。
- 二者合并的理由：语义都是「无视基线、用本机快照覆盖」。旧桌面端缺 `base_version` 等价于「我不参与并发协议」——给它旧行为（无条件覆盖）即「零改动继续工作」；护栏只在桌面端升级、开始回传 `base_version` 后对该客户端生效。**这是为兼容刻意接受的：阶段 2 不强制全网升级，护栏渐进生效。**

### 决策 3：响应必回传新 `server_version`；桌面端每次都用它刷新 `base_version`

- `/sync` 200 响应体加 `server_version`（新版本）。守卫路径确定为 `base+1`；无条件路径**在同一 `DB.batch` 末尾追加一条 `SELECT server_version FROM sync_meta WHERE tenant_id=?`**，读该 batch 返回数组对应位置的结果——同事务 read-your-writes 确定读到本次写入值，**消除「batch 外再读」与并发写之间的微竞态**（取代原「batch 后一次 `.first()`」，亦省一次 D1 往返）。注意：此时无条件路径的「最后一条」是该 SELECT，而守卫路径的 409 判定仍读其 CAS 那条结果——实现按路径分别取对应结果位置，**勿一律读末元素**。
- 桌面端 200 后**必须**把 `config.base_version` 更新为响应的 `server_version` 并落盘。否则下次上传仍带旧 `base` → 自找 409。**这是客户端侧 load-bearing 的一步**，spec 与 tasks 都钉死。
- 409 响应体回传服务端**当前** `server_version`，供 UI 提示「云端已到版本 N」。**版本来源**：守卫路径的 CAS 结果只给 `changes`（命中与否）、**不给当前版本值**，故 409 分支**必须**在 batch 提交后补一次 `SELECT server_version FROM sync_meta WHERE tenant_id=?`（`.first()`）读取当前版本回传（仅 409 路径多一次读；batch 已以零改动提交，此后读安全）。**边界**：若 `sync_meta` 行不存在（守卫读得 NULL 也走 409，该补读也得空），409 体的 `server_version` **必须**为 `null`（或省略），**禁止**回 `undefined`/`NaN`（否则客户端把 `base_version` 写坏）。现役活跃租户行恒存在，此分支主要防阶段 4 新租户漏 seed / 行被误删。

### 决策 4：`GET /pull` 用写入 Key 鉴权、返回全量快照 + 版本

- 鉴权复用 `resolveTenant(env, Bearer)`（**不**用公开查询 key）：`/pull` 返回含 PII 的全租户数据，只能给持写凭据的属主。未命中 → 401。
- 响应：`{ success, server_version, data:{ projects, cards, sf_senders, sf_orders, app_settings }, last_client_id, sync_time }`。**关键序列化往返**：`data` 各表字段必须逐字段等于桌面端 `export_database()` 产出（**对象/布尔**形态），而非 D1 原始存储形态——worker 入库把 `cards.metadata`/`sf_orders.sender_info`/`sf_orders.recipient_info` 存为 **JSON 字符串**、`sf_senders.is_default` 存为**整数**；`/pull` 读出后**必须** `JSON.parse` 这三个字符串字段（NULL/空串容错）、`!!is_default` 转布尔，否则桌面端 `PullResponse`（`Vec<Card>`/`Vec<SFOrder>`，字段是对象/布尔）反序列化**直接失败**、恢复整功能不可用（review blocker）。
- 行预算：6 条 SELECT（5 业务表各一条 `SELECT <业务列，排除 tenant_id> WHERE tenant_id=?` 走 `(tenant_id, id/key)` 复合主键最左前缀范围扫；1 条 `sync_meta` 走单列主键 `tenant_id` 等值点查、单行，取 `server_version`/`last_client_id`/`sync_time`），**各计 1 次查询 = 6 次**（与返回行数无关），远低于约束 5 上限。
- 路由未引入故 tenant 仍由 Key 解析（恒 `bh2ro`/兜底），为阶段 4 多租户预留。

### 决策 5：桌面端 `base_version` 持久化与「从云端恢复」

- `SyncConfig` 加 `base_version: Option<i64>`（默认 `None`，存 `sync.toml`）。`None` = 从未与新协议同步过 → 首发走无条件路径（兼容），200 后写入真实版本。
- 「从云端恢复」= 调 `/pull` → 在**本地一个 rusqlite 事务**内「清空 5 张业务表 + 按快照 INSERT」（新增 `src/db` 导入路径，镜像 `export_database()` 的表集）→ 成功后 `base_version = 拉回的 server_version`。该操作**销毁本地未上传改动**，必须前置确认对话框。
- 409 的桌面端流程：`sync_data` 返回携带服务端当前版本的**类型化结果**（非笼统 Err 字符串），命令层转成前端可分辨的结果；前端弹「云端有更新」对话框 → 选「下载云端最新」（走恢复）或「强制覆盖」（`force=true` 重发）。

## 风险 / 权衡

- **[每行 INSERT 子查询开销]** → 当前数据量（数百行）可忽略；若未来逼近 D1 单 batch 上限须分块，则**分块与 OCC/原子性叠加冲突**（跨块非原子，阶段 1 已把分块列为 out-of-scope）。缓解：本期 spec 显式声明「单 batch 容纳前提」，超限须改影子表/版本切换方案，不在本期。
- **[护栏渐进生效，非全网即时]** → 缺 `base_version` 的旧桌面端仍可无条件覆盖，期间两台旧端之间仍可能互相覆盖。缓解：这是兼容的刻意取舍；护栏对「至少一端已升级且回传 base」的组合即生效；文档说明「需双端升级才完全闭合」。
- **[`force` 是设计性数据丢失]** → 强制覆盖会丢云端较新数据。缓解：UI 二次确认 + 文案明示「用本机覆盖云端」；`/pull` 恢复入口给「反悔前先下载」的出口。
- **[客户端忘记刷新 base_version → 自发 409]** → 决策 3 钉死「200 后必更新并落盘」；tasks 加「连续两次同步（中间无他端写）第二次应 200 而非 409」回归断言。
- **[无条件路径读回版本的微竞态]** → 已消除：读回改为 batch 内末条 `SELECT`（决策 3），同事务 read-your-writes 拿到确定版本，不再有 batch 外再读的竞态窗口。
- **[D1 read replication / Sessions API]** → 本设计的「无 TOCTOU / read-your-writes / `/pull` 读到最新已提交版本」强保证**仅在默认 primary-only 路径成立**。若未来该 D1 库在 dashboard 启用 read replication 且代码改用 Sessions API，`/pull` 的 SELECT 可能命中**滞后副本** → 客户端据此把 `base_version` 设成陈旧值 → 下次 /sync 立刻 409，恢复入口反而制造冲突循环（OCC 的 CAS 走 primary 不受影响，受影响的是 /pull 的版本+数据一致性）。缓解：**本期约束「不启用 read replication、不使用 Sessions API、读写走 primary」**（写进约束 6 + tasks 1.7）；未来开副本须 `/pull` 与 OCC 读改用 Sessions API 并携带最新 bookmark。
- **[sync_meta 行缺失的引导场景]** → 现有活跃租户（创始 `bh2ro`、及 seeded `default`；当前 `env.API_KEY` 兜底解析为 `bh2ro`）行恒存在；守卫路径对「行不存在」会 409（`SELECT` 得 NULL），无条件/`force`/upsert 路径可引导建行。阶段 4 租户开通须 seed `sync_meta` 行（已在阶段 1 的「二次迁表点」语境记录），本期不处理新租户引导。
- **[回滚]** → 无数据/表结构变更，回滚 = 部署回旧 worker 版本（旧 worker 不读 `server_version`、与现库完全兼容）；桌面端旧版本不发 `base_version`、与新旧 worker 均兼容。两侧独立可退。

## 迁移计划

- **无 D1 迁移**（`server_version` 列阶段 1 已建）。
- 部署顺序无强耦合：worker 先上（新增 `/pull`、`/sync` 兼容缺 `base_version`）→ 桌面端随后发版。worker 上线后旧桌面端继续无条件覆盖照常工作。
- 回滚：worker 退版（数据兼容）；桌面端可独立回退。
- 验收：①两台（或脚本模拟两客户端）持同一 `base` 交替同步，先到 200、后到 409 且数据未被覆盖；②`force=true` 后到者成功覆盖；③`/pull` 拉回全量 + 版本，桌面端恢复后 `base_version` 对齐、下次同步直接 200。

## 待解决问题

- ~~`/sync` 200 无条件路径的「读回版本」微竞态~~ → 已决：读回并入 batch 末条 `SELECT`（决策 3），竞态消除，无需再议。
- 桌面端 409 的前端文案与交互（对话框 vs 行内提示）留待实现时按现有「数据管理」页风格定；spec 只约束行为（可下载/可强制覆盖），不约束像素。
- `clear_sync_config` 现状删整个 `sync.toml`（含 `client_id`），与 cloud-database-support 主规范「清除配置保留 client_id」措辞相矛盾——属阶段 1 既有不一致，本期 `accepted-degraded`（不扩范围去改清除语义，仅在本变更确保「清 base_version」不引入新矛盾），记为独立清理项。
