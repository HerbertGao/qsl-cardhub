# tenant-isolation 规范

## 目的
待定 - 由归档变更 add-tenant-foundation 创建。归档后请更新目的。
## 需求
### 需求：租户模型与默认租户

云端**必须**提供行级多租户隔离的数据模型，由 `tenants`、`tenant_credentials`、`tenant_routes` 三张表承载，并保证既有数据归属于内置的 `default` 租户。

- `tenants`：租户主表，`tenant_id` 为人类可读 slug，**必须** `NOT NULL`（`TEXT PRIMARY KEY` 在 rowid 表不隐含 NOT NULL）。字符集约束（小写字母、数字、连字符，长度上限）**必须**以可执行约束落地，且**必须**用否定字符类：`CHECK (length(tenant_id) BETWEEN 1 AND 32 AND tenant_id NOT GLOB '*[^a-z0-9-]*')`——**禁用** `GLOB '[a-z0-9-]*'`（其 `*` 是 shell-glob「任意后缀」、只约束首字符，`abc!` 会通过），更**禁止**仅以注释表达「必须」。
- `tenant_credentials`：写入凭据表，存 `key_hash = sha256(trim(key))`（**禁止**存明文 Key；本期**不加 pepper**，前提是 `API_KEY` 为高熵随机值——见 design 决策 D 残余风险）；命中得 `tenant_id`；**必须**支持「多 Key → 同一租户」；`status` **必须** `NOT NULL DEFAULT 'active'` 且 `CHECK` 取值域（如 `'active'`/`'revoked'`），同一 `key_hash` 在 `status='active'` 下**必须**唯一（部分唯一索引），**禁止**一把 Key 解析到两个租户。
- `tenant_routes`：host/path → 租户路由表（本期建表即可，路由解析逻辑属后续阶段）。

#### 场景：内置默认租户与默认写凭据

- **当** 多租户地基初始化（迁移）完成
- **那么** `tenants` 中**必须**存在 `tenant_id = 'default'` 的活跃租户
- **并且** `tenant_credentials` 中**必须**存在一条 `tenant_id='default'`、`key_hash = sha256(trim(现有全局 API_KEY))` 的活跃写凭据（`trim` 与 worker 解析侧逐字符一致）
- **并且** 现有桌面端用原全局 API_KEY 同步时**必须**被解析为 `default` 租户

#### 场景：同一 Key 不得登记到两个租户

- **当** 尝试为同一把活跃 Key 的 `key_hash` 登记第二个 `tenant_id`
- **那么** 唯一约束**必须**拒绝该写入（解析无歧义）

### 需求：写入按 Key 解析租户

`/sync` 等写入端点的 `tenant_id` **必须**由服务端从写入 Key 解析，**禁止**取自请求体自报的任何 id（含 `client_id`）。

#### 场景：表驱动解析命中租户

- **当** 客户端以 Bearer Key 发起 `/sync`
- **那么** 服务端**必须**以 `sha256(trim(Key))` 查 `tenant_credentials`（`status='active'`）得到 `tenant_id`，其中 `trim` 语义与离线 seed 计算两侧完全一致（64 位小写 hex，无前缀）
- **并且** 全部写入**必须**落到该解析出的 `tenant_id`
- **并且** 请求体携带的 `client_id` **禁止**用于决定数据归属，仅可作溯源记录（写入 `sync_meta.last_client_id`）

#### 场景：env.API_KEY 直比兜底（过渡期）且可客观检测

- **当** `sha256(trim(Key))` 未命中 `tenant_credentials`，但 `trim(Key)` 等于 `trim(env.API_KEY)` 且 `env.API_KEY` 非空
- **那么** 服务端**必须**将其解析为 `default` 租户并放行（兜底，避免 seed 异常锁死现有桌面端）
- **并且** 走兜底路径时**必须**递增一个 **D1 计数行**（具名表 `service_counters(name TEXT PRIMARY KEY, count INTEGER NOT NULL DEFAULT 0)` 的 `name='auth_fallback'` 行；**禁止**用 KV——KV 未绑定时 fail-open 会吞掉递增致假绿）；该表与初始 `count=0` 行**必须**由迁移**自建（`CREATE TABLE`）并 seed**（迁移文件须含建表语句、排在 seed 之前，否则生产 D1 无此表致 seed `no such table`）；**计数器写失败**（用 `.run()` 返回的 `result.meta.changes === 0` 判行缺失，或写抛错判表缺失）**必须使 `/sync` 返非 200**，不得静默吞
- **并且** 迁移后验收**必须**断言「该计数行**存在 且 `count === 0`**」（**严格 `===` + 存在性双检**——因 `(row?.count ?? 0)===0`、`count>=0`、`!count`、`""==0` 等宽松写法会把「行缺失」误当 0 而假绿（注：`null==0` 本身为 `false`，假绿来自 `?? 0`/`>=0`/`!`/空串==0 这些路径），故强制存在性 + 严格相等）；计数行缺失/不可读**必须**判 inconclusive，**禁止**判 pass 或当作可接受降级而误判表驱动已生效
- **并且** 撤销兜底**必须**有量化判据（连续 N 次/T 时间窗表驱动命中且兜底计数恒 0）

#### 场景：env.API_KEY 未配置时拒绝（不沿用「空则放行」）

- **当** `env.API_KEY` 未配置（空/undefined）
- **那么** 兜底分支**必须**视为不命中、返回 401，**禁止**沿用现状 worker「`env.API_KEY` 为空即跳过校验放行」的语义（否则表空 + key 空 → 裸 `/sync` 无鉴权写入）

#### 场景：无效 Key 拒绝

- **当** Key 既未命中 `tenant_credentials` 也不等于 `trim(env.API_KEY)`
- **那么** 服务端**必须**返回 401，**禁止**写入任何数据，且错误响应**禁止**回显内部结构（与已归档「错误响应脱敏」需求一致）

### 需求：业务表行级租户隔离与主键演进

业务表 `projects`、`cards`、`sf_senders`、`sf_orders`、`app_settings` **必须**以 `tenant_id` 作行级隔离键，主键演进为 `(tenant_id, id)`（`app_settings` 为 `(tenant_id, key)`），**必须**删除 `client_id` 列。`sync_meta` 主键**必须**为 `tenant_id`。

#### 场景：业务表结构含 tenant_id

- **当** 多租户地基初始化完成
- **那么** 上述业务表**必须**含 `tenant_id` 列且主键以 `tenant_id` 为前缀
- **并且** 业务表中**禁止**保留 `client_id` 列
- **并且** `cards` **必须**建有 `(tenant_id, callsign COLLATE NOCASE)` 复合索引以支撑按租户的呼号查询

#### 场景：表重建后所有现有业务索引完整重建

- **当** 业务表经「建新表 → DROP 旧表 → RENAME」重建（旧表索引随 DROP 一并消失）
- **那么** 新表**必须**重建全部现有读路径依赖的索引，至少包括：`sf_orders` 的 `order_id`、`waybill_no`、`card_id` 三个索引（route-push 反查 `index.js:559/:567` 的 `WHERE o.order_id=?`/`o.waybill_no=?` 依赖），`cards` 的 `project_id`、`created_at`，`projects` 的 `created_at`
- **并且** 索引应在「建新表」步骤一并建好（避免 RENAME 后漏建）
- **并且** 查询侧比较**必须**与索引列的 collation 一致（`cards` 的呼号查询保留 `callsign = ? COLLATE NOCASE` 以命中 NOCASE 索引），可用 `EXPLAIN QUERY PLAN` 断言命中 `idx_cards_tenant_callsign` / `idx_sf_orders_order_id`

#### 场景：sync_meta 一次性建成终态结构

- **当** 多租户地基初始化完成
- **那么** `sync_meta` 主键**必须**为 `tenant_id`
- **并且** **必须**含 `last_client_id`（溯源）与 `server_version INTEGER NOT NULL DEFAULT 0`（为后续乐观并发护栏预留列；本期不实现 compare-and-swap 逻辑）

### 需求：读取按服务端注入的租户过滤

按呼号查询（`/api/query`、`/api/callsigns/:callsign`）与顺丰 route-push 的呼号反查 join、及 route-push 的 openid 反查 **必须**按服务端确定的 `tenant_id` 过滤，`tenant_id` **禁止**取自前端参数或请求体自报值。其中：

- **按呼号查询**侧的路由解析尚未引入（host/path → 租户路由属阶段 4-B），故其 `tenant_id` **必须**恒为创始租户常量（部署落地为 `bh2ro`）。
- **route-push** 是无凭据公开端点、无路由/凭据上下文，其 `tenant_id` **必须**由「本次推送匹配到的 `sf_orders` 行」**派生**（按全局唯一的 `order_id`/`waybill_no` 匹配订单，由该订单行返回其 `tenant_id`），**禁止**再注入硬编码常量；openid 反查**必须**带该派生 `tenant_id` 维度。

#### 场景：查询注入租户过滤

- **当** 调用方按呼号查询
- **那么** 服务端**必须**在 SQL 中注入 `WHERE tenant_id = ? AND callsign = ?`，其中 `tenant_id` 来自服务端上下文（本期为创始租户常量 `bh2ro`；按路由解析属阶段 4-B）
- **并且** 即便前端传入任何租户参数，也**禁止**据此跨租户读取
- **并且** 关联 `projects` 的 join **必须**同时按 `tenant_id` 匹配

#### 场景：顺丰呼号反查按匹配订单派生租户

- **当** route-push 通过 `sf_orders` join `cards` 反查呼号
- **那么** 查询**必须**按全局唯一业务键匹配订单（`WHERE o.order_id = ?` 或 `WHERE o.waybill_no = ?`，**不再**附加 `WHERE o.tenant_id = 常量`），并由匹配行**返回其 `tenant_id`**（`SELECT c.callsign, o.tenant_id … LIMIT 1`）供后续 openid 反查使用
- **并且** **必须**保留同租户自洽连接键 `o.tenant_id = c.tenant_id` 与业务连接键 `o.card_id = c.id`（二者缺一不可：丢 `card_id` 退化为笛卡尔积错号；丢 `o.tenant_id = c.tenant_id` 会让订单跨租户错配卡片）
- **并且** 派生租户**禁止**取自请求体或顺丰推送报文中的任何自报字段，**必须**来自服务端在 D1 中匹配到的 `sf_orders` 行

#### 场景：openid 反查按派生租户过滤（阶段 4-A 落地）

- **当** route-push 反查到 callsign 后查 `callsign_openid_bindings` 选 openid 发推送
- **那么** 该 `callsign → openid` 反查**必须**带 tenant 维度：`SELECT openid FROM callsign_openid_bindings WHERE tenant_id = ? AND callsign = ?`，其中 `tenant_id` 为上一场景由匹配订单派生的租户
- **并且** **禁止**退回 `WHERE callsign = ?` 的无租户全局反查——否则两租户同一呼号的订阅者会跨租户收到对方物流推送（openid + 运单号/节点/时间轨迹泄漏）
- **并且** 因 `callsign_openid_bindings` 已加 `tenant_id`（见「全局绑定表的租户化迁移」需求），同呼号在不同租户下的绑定行物理隔离，反查只命中派生租户名下的 openid

### 需求：生产数据迁移到默认租户

多租户地基的表结构演进**必须**以可回滚、可前置校验的一次性迁移交付，保全既有数据并全部回填到 `default` 租户。

#### 场景：迁移前校验单一所有者（含 sync_meta）

- **当** 准备执行业务表重建迁移
- **那么** **必须**先对**每张业务表 + `sync_meta`** 校验 `client_id` 为单一所有者（`SELECT client_id, COUNT(*) … GROUP BY client_id`）；`sync_meta` 因现状 worker 从不 DELETE 旧行、换机/重装（桌面端 `client_id` 每安装随机）会累积多行，**必须**一并纳入校验
- **并且** 若存在多个 `client_id`，**必须**中止迁移，并按既定步骤人工处置（确认仅一台机器同步过 / 择 `received_at` 最新一条、余删）后重来——**禁止**仅笼统写「人工处置」而不给步骤
- **并且** 迁移执行窗口内**应**冻结桌面端写入（避免「校验后、迁移前又同步引入异常行」竞态）

#### 场景：表重建保全数据并回填 default（全文件原子，禁显式事务）

- **当** 执行迁移
- **那么** 每张业务表**必须**经「建新表（含 `tenant_id`、新主键、**完整重建全部业务索引**）→ `INSERT…SELECT` 回填（`client_id` 既有行 → `tenant_id='default'`）→ DROP 旧表 → RENAME」
- **并且** 原子性**必须**依赖「整份迁移 SQL 文件由 `wrangler d1 execute --file --remote` 单次执行、任一语句失败即回滚到原状」，迁移文件内**禁止**写 `BEGIN`/`COMMIT`/`SAVEPOINT`（D1 不支持用户侧事务控制，写了会 `SQLITE_AUTH` 失败）
- **并且** `sync_meta` 重建时 `last_client_id` **必须**取最新一行的 `client_id`（或置 NULL 交首个 `/sync` 写），**禁止**照搬多行致 PK 冲突；「取最新」**必须确定性**（`ORDER BY received_at DESC, client_id DESC LIMIT 1`，并列时由 `client_id` 兜底，且依赖 `received_at` 为可字典序的零填充 ISO 文本）
- **并且** 迁移完成后既有数据**必须**全部归属 `default` 且原 `id` 不变

#### 场景：迁移执行后自检占位符已替换

- **当** 迁移 SQL 执行完成
- **那么** **必须**自检 `SELECT key_hash FROM tenant_credentials WHERE key_hash LIKE '<占位%'` 返回 0 行
- **并且** 该自检是**强制门**：返回任何行即视为迁移失败，**必须**回滚（D1 顶层无 `RAISE`，靠运维核对此输出 + 部署后 `count===0` 双重兜住）
- **并且** 风险更正：占位串**不可直接当 Bearer 用**（worker 比对 `sha256(trim(key))`，而占位行 `key_hash` 存的是占位串本身、非其 `sha256`）；真实风险是占位行占用 `idx_tenant_credentials_active_key_hash` 的 active 唯一槽位 → 表驱动对真实 Key 永久 miss → 全程走兜底，由部署后验收「`auth_fallback count===0`」兜住

#### 场景：迁移交付与回滚（前滚优先）

- **当** 对生产 D1 执行迁移
- **那么** 迁移**必须**以单一 SQL 文件交付、由运维在自己终端执行（`wrangler d1 execute --file --remote`），**禁止**由自动化代理代跑生产迁移
- **并且** 执行前**必须**以 `wrangler d1 export`（默认含 schema+data）全量备份作为回滚点
- **并且** `default` 写凭据的 `key_hash` **必须**为离线计算的 `sha256(trim(API_KEY))` 字面量（64 位小写 hex，计算过程不暴露 secret 明文给自动化代理）
- **并且** 回滚剧本**必须**如实写明：迁移后 schema 与 worker 强耦合，**不可单独回退 worker**（旧 worker 读 `client_id` 列会整站 500）；数据回滚 = 先 DROP 全部表（或建新空 D1）再 import 旧 dump（dump 不自动 DROP 已存在表）

### 需求：全局绑定表的租户化迁移

`callsign_openid_bindings`（呼号–微信 openid 绑定）**必须**经一次性、可回滚、可前置备份的迁移加入 `tenant_id` 行级隔离键，主键演进为 `(tenant_id, callsign, openid)`，存量行全部回填创始租户（部署落地为 `bh2ro`）。`sf_route_log`（顺丰路由去重/日志）**必须**保持全局、**不加** `tenant_id`（顺丰 waybill 全局唯一，去重维度全局；其 tenant 由匹配的 `sf_orders` 派生，不单独隔离去重维度）。

#### 场景：绑定表重建并回填创始租户

- **当** 执行本期迁移（`migrations/0002_*.sql`）
- **那么** `callsign_openid_bindings` **必须**经「建新表（含 `tenant_id TEXT NOT NULL`、主键 `(tenant_id, callsign, openid)`）→ `INSERT…SELECT` 回填（存量行 `tenant_id` 置创始租户字面量 `'bh2ro'`）→ DROP 旧表 → RENAME」
- **并且** 读路径依赖的索引 `idx_bindings_callsign` **必须**在 DROP+RENAME **之后**重建（SQLite 索引名库级唯一，若在「建新表」阶段用同名索引会 `already exists` 致整文件失败——沿用 0001 教训）
- **并且** 原子性**必须**依赖「整份迁移 SQL 由 `wrangler d1 execute --file --remote` 单次执行、任一语句失败即回滚」，迁移文件内**禁止**写 `BEGIN`/`COMMIT`/`SAVEPOINT`
- **并且** 终态结构**必须**在 `web_query_service/schema.sql` 双写（schema.sql 源与迁移文件一致）

#### 场景：回填不撞主键

- **当** 存量 `callsign_openid_bindings` 行（原主键 `(callsign, openid)` 唯一）回填统一 `tenant_id = 'bh2ro'`
- **那么** 新主键 `(tenant_id, callsign, openid)` = `('bh2ro', callsign, openid)` **必须**仍唯一（不引入 PK 冲突），迁移不需单一所有者校验（绑定表无 `client_id`、回填值恒一）

#### 场景：sf_route_log 保持全局不迁

- **当** 本期迁移执行
- **那么** `sf_route_log` 表结构**必须**不变（不加 `tenant_id` 列、不重建）
- **并且** spec **必须**声明：route-push 处理时其租户由匹配的 `sf_orders` join 派生（见「读取按服务端注入的租户过滤」需求），`sf_route_log` 仅作全局去重/审计、无隔离维度

#### 场景：迁移交付与配对回滚

- **当** 对生产 D1 执行本期迁移
- **那么** 迁移**必须**以单一 SQL 文件交付、由运维在自己终端执行（`wrangler d1 execute --file --remote`），**禁止**由自动化代理代跑生产迁移
- **并且** 执行前**必须**以 `wrangler d1 export` 全量备份作为回滚点
- **并且** 回滚剧本**必须**写明：迁移后 worker 读 `callsign_openid_bindings.tenant_id` 列（auth-callback 写、route-push 反查），**不可单独回退 worker**（旧 worker 的 `INSERT (callsign, openid, …)` 撞新表 NOT NULL `tenant_id`、`WHERE callsign=?` 反查仍可用但写入会失败）；回滚 = worker 与表配对还原

