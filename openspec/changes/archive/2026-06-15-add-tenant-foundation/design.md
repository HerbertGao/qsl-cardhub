## 上下文

云端 `web_query_service`（Cloudflare Worker + D1）当前是单一数据集：桌面端用全局 `API_KEY` 全量覆盖式 `/sync` 写入，公开移动端按呼号读出。业务表虽有 `client_id` 列但 `/sync` 无 `WHERE` 全表清空再重写，多 `client_id` 物理上无法共存；`/api/query` 零隔离。本变更是多租户演进的**阶段 1（地基与写入隔离）**，铺「行级 `tenant_id` + 按 Key 解析租户」的地基，对外可见行为不变（`tenant_id` 恒为 `default`）。

完整分期蓝本见 `docs/multi-tenant-design.md`（阶段 0 已归档）。本期落地前已就 4 个关键决策（迁移交付、凭据校验、`server_version` 前置、seed 时机）由后端架构评审拍板，记录于下。

约束：
- 生产前置阿里云 CDN（qsl.herbert-dev.cn 回源 Cloudflare），故 `CF-Connecting-IP` 在 CDN 路径下是回源节点 IP——本期**不**触碰 IP 来源（属阶段 3）。
- 全局工作规则：**不让自动化代理跑生产迁移 / 接触 secret 明文**，迁移命令与 hash 计算均展示给用户在自己终端执行。
- D1 无独立迁移 runner；schema 现状由 `schema.sql` + worker 内联 `CREATE TABLE IF NOT EXISTS` 双写维护。

## 目标 / 非目标

**目标：**
- 建租户模型（`tenants`/`tenant_credentials`/`tenant_routes`）+ 业务表行级 `tenant_id` 隔离 + 主键演进 `(tenant_id, id)` + 删 `client_id`。
- `/sync` 按 Key 解析租户 + 单事务按租户全量替换（顺带修 DELETE/INSERT 非原子缺陷）。
- `/api/query` 与顺丰 route-push join 注入/改用 `tenant_id`。
- 现有桌面端、移动端**零改动**继续工作（全部落 `default`）。
- 一次到位钉死「将来要二次迁表」的结构点。

**非目标：**
- OCC 版本护栏的 compare-and-swap 逻辑、`/pull`（阶段 2，本期仅占 `server_version` 列）。
- PoW 会话、动态签名、CDN 真实 IP 来源修正（阶段 3）。
- host/path → 租户路由、`/api/config` 按租户下发、桌面端租户身份配置、上线第二个真实租户（阶段 4）。
- PII 字段加密（独立可选项）。

## 决策

### 决策 A：迁移以单一 SQL 文件交付，由用户 `wrangler d1 execute --remote`

不在 worker 内加 `/admin/migrate` 端点。表重建是离线一次性动作；让 worker 持 DDL 权限会引入一个需自保护、用完即死的长期攻击面，违反最小权限。也符合「不让代理跑生产迁移」的工作规则。
- **替代（否决）**：worker 内受保护迁移端点——背一个长期攻击面换一次性便利，不划算。
- **D1 事务语义（Cloudflare 文档证实）**：Cloudflare D1 **不支持**在 SQL 文件里写显式 `BEGIN TRANSACTION` / `COMMIT` / `SAVEPOINT`——这些语句经 `wrangler d1 execute --file` 会以 `not authorized: SQLITE_AUTH` 失败（D1 自管事务边界、禁用用户侧事务控制）。原子性来自 Cloudflare 文档明载的保证：**整份迁移 SQL 文件由 `wrangler d1 execute --file --remote` 单次执行，「若执行未能完成，DB 返回原状、可安全重试」**（全文件回滚，强于「每表一个事务单元」）。因此迁移文件内**禁止**写任何 `BEGIN/COMMIT/SAVEPOINT`，靠 `--file` 的全文件回滚保证「不留半迁移」。表重建步骤 = 建新表 → `INSERT…SELECT` → DROP 旧 → RENAME；本项目业务表间**无真实外键**（projects↔cards 是逻辑关联）、**无触发器、无视图**（已核对 schema），故 DROP+RENAME 无级联副作用、无需切换 `PRAGMA foreign_keys`。
- **文档证实仍以实测确认（非矛盾）**：上面是 Cloudflare 文档层面的保证；tasks 6.2 仍作为**上生产前的硬门**在一次性 `--remote` 临时库实测「中段语句失败整文件回滚」，确认当前 wrangler 版本行为与文档一致——这是确认性验证，不是「原子性尚未确定」。
- **本地验证不等价**：`wrangler d1 execute --local` 在部分版本对 DDL/RENAME 报 `SQLITE_AUTH`，本地结果不能判定 `--remote` 可行性——验证须用一次性 `--remote` 临时/preview D1（见 tasks 6 与待解决问题）。

### 决策 B：凭据校验「表驱动为主 + `env.API_KEY` 直比兜底一期」

终态是纯表驱动（查 `tenant_credentials`），但本期硬验收是「现有桌面端零改动」，而它用的就是 `env.API_KEY`。若只走表驱动，桌面端能否同步完全押在迁移 SQL 里那行 seed 的 `key_hash` 算得对不对——一字符错即生产 401 且难定位。兜底：表未命中再比 `token === trim(env.API_KEY)` → 解析为 `default`。
- **替代（否决）**：立刻纯表驱动——把「桌面端可用性」押在 seed 正确性上，风险不对称。
- **兜底必须客观可检测（修订，且计数器本身不得假绿）**：仅「打日志」不够——seed 算错（trim/编码不一致是高概率诱因）会让表驱动**永久 miss**、每次 `/sync` 静默落兜底并返回 200，外观与表驱动成功**完全一致**，靠人肉翻日志确认极脆弱。故钉死：① 计数器**必须落 D1 计数行**（具名表 `service_counters`，与业务库同实例，**禁止**用 KV——KV 未绑定时 fail-open 会吞掉递增致假绿），**迁移须建 `service_counters` 表并 seed 一条 `count=0` 初始行**；② 兜底命中时递增该行，判写失败用 `.run()` 返回的 `result.meta.changes === 0`（行缺失）或写抛错（表缺失）→ `/sync` 返非 200（不静默吞；**禁用** SQLite `SELECT changes()` 写法）；③ 验收（tasks 7.3）断言**「该计数行存在 且 `count === 0`」（严格 `===` + 存在性双检）**——注意宽松写法 `(row?.count ?? 0)===0` / `count>=0` / `!count` / `""==0` 会把「行缺失」误当 0 而假绿（注：`null==0` 本身为 `false`，真正假绿来自 `?? 0`、`>=0`、`!`、空串==0 这些路径），故强制存在性 + 严格相等；计数行缺失/不可读则判 inconclusive、**不得**判 pass；④ 撤兜底有**量化判据**（连续 N 次/T 时间窗表驱动命中且兜底计数恒 0 方可撤），否则「`token===env.API_KEY` 全局放行」这条长期信任路径会无限期挂着，与决策 A「不留长期攻击面」自相矛盾。
- **`env.API_KEY` 缺失语义（修订）**：现状 worker `if (env.API_KEY && token !== …)` 在 `env.API_KEY` 为空时**跳过校验放行**。新 `resolveTenant` **禁止**沿用此「空则放行」——`env.API_KEY` 未配置时兜底分支一律视为不命中、返回 401，避免「表空 + key 空 → 裸 /sync 无鉴权写入」。

### 决策 C：重建 `sync_meta` 时顺手加 `server_version`（只占列，不实现逻辑）

PK 从 `client_id` 改 `tenant_id` 必然重建 `sync_meta`，顺手加 `last_client_id` 与 `server_version INTEGER NOT NULL DEFAULT 0`。多一个未读写的列零运行时影响、零兼容风险，省阶段 2 再为一列动表。**边界纪律**：只加列、不加阶段 2 的 OCC compare-and-swap 逻辑——带列不带逻辑不算越界。

### 决策 D：`default` 凭据 hash 离线算、写进迁移 SQL

`API_KEY` 是 wrangler secret，worker 运行时虽能读，但 D2（worker 首次自算 upsert）引入「表空→首请求触发 seed 写」的冷启动竞态且把一次性逻辑变常驻。D1：离线在终端算 `sha256(trim(API_KEY))`，作字面量写进迁移 SQL 的 `INSERT`，与表重建同一文件一次过。
- **trim 必须两侧逐字符等价（修订，纠正初稿的算法不一致）**：worker 侧 `getBearerToken` 用 JS `String.prototype.trim()`（**只去首尾**空白），`resolveTenant` 算 `sha256(trim(key))`。初稿离线命令用 `tr -d '[:space:]'`（去**全部**空白，含中间）与 JS `trim()` **不是同一算法**——若 Key 含中间空白二者 hash 不同。故离线命令**必须**用与 JS `trim()` 等价的「仅去首尾」实现，最稳是直接用 Node 同源计算：`node -e 'process.stdout.write(require("crypto").createHash("sha256").update((process.env.API_KEY||"").trim()).digest("hex"))'`（与 worker 逐字符同语义，输出 64 位小写 hex 无前缀；本项目本就是 Node 工具链 `.nvmrc`/pnpm，node 可用）。**禁止** `tr -d '[:space:]'`。若执行环境确无 node，退路命令**必须**与 JS `trim()` 逐字符等价（仅去首尾空白）、小写 hex 无前缀，并与 worker 实算一次比对确认一致。并在部署清单声明 `API_KEY` 为高熵随机值、无中间空白。两侧不一致的唯一外在症状就是「表驱动永久 miss、全程兜底」——由「兜底计数==0」主动断言兜住。
- **不加 pepper 的残余风险（诚实标注，非中性权衡）**：`key_hash = sha256(key)` 无 pepper、无慢哈希。若 D1 被拖库且 `API_KEY` **低熵**，攻击者可离线还原 Key 并伪造 `/sync`（覆盖式全量替换 = 可改写/清空 `default` 生产数据）。故本期前提：**`API_KEY` 必须是高熵随机值**（部署清单须声明）；「本期不加 pepper」**≠ 该风险已闭合**，而是「在高熵 Key 前提下可接受，阶段 4 引第二租户前补 pepper」。
- 算 hash 命令给用户自跑（代理不碰 secret 明文）。

### 决策：能力切分

- 新能力 `tenant-isolation`：租户模型、Key→租户解析、业务表行级隔离与主键、读取注入、迁移。
- 修改能力 `cloud-backend-api`：`/sync` 从「删所有 client_id」改为「按租户单事务替换」、`app_settings` 结构 `client_id`→`tenant_id`。`/ping` 正常路径不变，仅边角收紧（`env.API_KEY` 补 trim + 空时 401，与 `/sync` 一致，见 spec），非完全不变。查询展示逻辑不变，故其租户注入归入 `tenant-isolation` 的「读取注入」需求、不重抄 `cloud-backend-api` 的巨型展示需求块。

## 风险 / 权衡

- **[内联 `CREATE TABLE` 与新表打架]** `/sync`（`index.js:259-297`）每次同步内联建带旧 `client_id` PK 的表。迁移后对已存在新表是 no-op 不破坏数据，但若某表被手工 DROP 会按旧结构重建、与新 `INSERT`（绑 `tenant_id`）冲突。→ **缓解**：本期 worker 改动**首选整段删除**这批内联建表（迁移后库结构由迁移文件保证；缺表时让 `/sync` 显式报错，而非静默重建一个无索引空表掩盖问题），别只改迁移 SQL。
- **[单 batch 全量替换的真实上限（已核实，纠正维度）]** 真正硬约束**不是**「绑定参数数」（那是**每条语句 ≤100 参数**，单行 INSERT 远不及），而是 **D1 每次 Worker 调用的查询数上限（Paid 1000 / Free 50 子请求）**。`DELETE×5 + 上万 INSERT` 远超 1000 → 整 batch 失败。→ **缓解**：tasks 1.2 **先确认该 D1 是 Free 还是 Paid 计划**（阈值取 50 还是 1000，"~900" 仅对 Paid 成立；若 Free 则即便小数据量也会撞 50，"单 batch 可容纳" 假设不成立须分块），并把**单次 `/sync` 调用的全部子请求**计入预算：`resolveTenant` 的凭据查询 + 兜底计数器写 + `DELETE×5` + `N×INSERT` + `sync_meta` upsert（阈值留余量，**Paid 下总语句 ≤~850** 才走真单 batch；`~1000` 是理论硬上限，取 `~850` 留头寸，二者非两个阈值）；超阈则分块为多个 `DB.batch`，且**第一块必须含 DELETE×5 + 首批 INSERT 同 batch**以保留「删写同事务边界」。注意**分块后跨 batch 不再原子**——若需分块，spec 的 `/sync` 原子性声明须如实降级，或改影子表/版本切换（本期数据量小、且为 Paid，预期单 batch 可容纳，走真单 batch 并在 1.2 设阈值「超 N 行停下重设计」）。
- **[迁移撞主键]** 业务表在「全量覆盖」模型下任一时刻只该有一个 `client_id`（每次 `/sync` 先删所有 client_id 再写），故单一所有者基本由构造保证；但 `sync_meta` **从不被 DELETE**（`index.js` 只 upsert，全局 DELETE 块不含 sync_meta），历史换机/重装（桌面端 `client_id = Uuid::new_v4()` 每安装随机）会累积多行 → 重建为 PK `tenant_id` 时 `INSERT…SELECT` 撞键。→ **缓解**：单一所有者校验**必须把 `sync_meta` 也纳入**；`sync_meta` 回填取最新一行 `client_id` 作 `last_client_id`（或置 NULL 交首个 `/sync` 写），不照搬多行。「取最新」**必须确定性**：`ORDER BY received_at DESC, client_id DESC LIMIT 1`（并列时由 `client_id` 兜底 tiebreaker，避免任意取值）。`received_at` 是 TEXT，其可字典序==时序由 `serverTime()`（`index.js`，`new Date().toISOString().replace('Z','+00:00')`，定宽零填充毫秒 ISO）保证、历史行同由它写入，已核实——若将来改 `serverTime()` 格式须同步校验此 tiebreaker。多 `client_id` 时中止并按「确认仅一台机器同步过 / 择最新一条、余删」人工处置（迁移文档须给步骤，非笼统「人工处置」）。
- **[seed 一致性]** `sha256(API_KEY)` 与 worker 比对值因尾随空白/编码不一致 → 表驱动永久 miss、全程兜底。→ **缓解**：两侧统一对 trim 后值算同格式 hash（见决策 D）；由「兜底计数==0」主动断言兜住；决策 B 的兜底使其不锁死桌面端但**不掩盖**（计数暴露）。
- **[占位符残留致表驱动永久 miss]** 迁移 SQL 的 `default` 凭据 seed 用占位 `'<占位:…>'`，若未替换会作为一条 active 凭据残留。**风险更正**：占位串**不可直接当 Bearer 用**——worker 比对 `sha256(trim(key))`，而占位行 `key_hash` 存的是占位串本身（非其 sha256），故拿占位串当 Bearer 会 hash 后 miss；真实风险是占位行占用 `idx_tenant_credentials_active_key_hash` 的 active 唯一槽位 → 表驱动对真实 Key 永久 miss → 全程走兜底（由部署后验收 `auth_fallback count===0` 兜住）。→ **缓解**：迁移文件**执行后自检** `SELECT key_hash FROM tenant_credentials WHERE key_hash LIKE '<占位%'` 为**强制门**，返回任何行即视为迁移失败须中止回滚（D1 顶层无 RAISE，靠运维核对此输出 + 部署后 `count===0` 双重兜住；spec 钉死）。
- **[回滚剧本（修订，原表述是错的）]** 迁移后 schema 与 worker **强耦合**：旧 worker 全是 `client_id` 列 SQL（迁后表是 `tenant_id` → `no such column` 整站 500）；而新 worker 依赖 `tenant_credentials` 等新表（恢复旧 dump 后新表消失 → resolveTenant 查无表整站 500）。故**既不能单独退 worker、也不能单独退数据**——回滚是**配对动作**，且**「同时」运维上不可真原子**，须按 fail-closed 五步顺序做避免可用性窗口（编号与迁移计划步骤 8 一致）：**①先把新 worker 下线**（具体手段：部署一个对所有路由返 503 的临时 worker 版本，或在 Cloudflare dashboard 解绑生产路由——非抽象「维护页」）→ **②清空迁移后 D1 的全部表**（**推荐走「建新空 D1 + 改 `wrangler.toml` 的 `database_id` 绑定」分支**，天然规避「逐表 DROP 漏表」；若走逐表 DROP，须 DROP **全部 12 张表**——三租户表 + `service_counters` + **业务 5 表**（projects/cards/sf_senders/sf_orders/app_settings）+ `sync_meta` + **两张本期未迁的全局表 `callsign_openid_bindings`/`sf_route_log`**；漏清这两张全局表 → 旧 dump 的 `INSERT` 撞其 PK 致 import 中止）→ **③import 步骤 1 的旧 dump**（dump 不自动 DROP 已存在表，故须先清）→ **④部署回迁移前旧 worker**（旧 dump 含 `client_id` 列、与旧 worker 匹配）→ **⑤撤下线/恢复路由**。任一中间态都是拒绝服务（500/503）而非裸写——因强耦合：新 worker 缺旧表→500、旧 worker 读新表也→500，无「无隔离静默写入」窗口。→ **缓解**：迁移计划给出此有序 fail-closed 序列；更稳的实践是**只前滚**（迁前用步骤 1 的 dump + 行数核对确保备份完整可重建），把回滚当最后手段。
- **[CDN 下查询限流仍失真]** 本期动了 `/api/query`（注入租户）但**不**修 IP 来源；经 CDN 的 `CF-Connecting-IP`=回源节点 IP，限流粒度粗放——这是既有现状、非本期回归，读取侧真正访问控制等阶段 3。→ **缓解**：不在本期承诺防爬提升，spec 不声称由租户过滤获得防爬能力。
- **[全局表无租户=阶段4 跨租户串号点（结构债）]** `callsign_openid_bindings`、`sf_route_log` 本期**不加** `tenant_id`（callsign 为全局键）。route-push 反查链为两段：`order/waybill → cards.callsign`（本期注入 tenant）与 `callsign → openid`（`index.js:575` 无 tenant、表本身无列）。本期恒 default 无暴露，但与「一次到位钉死二次迁表点」目标冲突，阶段 4 上第二租户、两租户同呼号订阅即跨租户推送（openid+物流轨迹泄漏）。→ **缓解**：本期**不改表**，但在 spec 把「openid 反查须带 tenant 维度」写成隔离不变量、并在此**显式声明这两表的租户化推迟到阶段 4 且为已知二次迁表点、阶段 4 上线第二租户前必须先迁**（放宽「一次到位」措辞为「业务 5 表 + sync_meta 一次到位；两张全局订阅/日志表显式延后」）。

## 迁移计划

> **冻结写入**：迁移执行窗口内应停桌面端同步（避免「校验后、迁移前又同步引入异常行」竞态）；迁移文件头注释提示此点。

1. **备份**：`wrangler d1 export`（远端，默认含 schema+data）全量导出作回滚点。
2. **前置校验**：对**每张业务表 + `sync_meta`** 跑 `SELECT client_id, COUNT(*) … GROUP BY client_id`，确认单一所有者；非单一则中止并按既定步骤（确认仅一台机器同步过 / 择 `received_at` 最新一条、余删）人工处置后重来。
3. **离线算 seed**：用户在终端算 `node -e 'process.stdout.write(require("crypto").createHash("sha256").update((process.env.API_KEY||"").trim()).digest("hex"))'`（与 worker JS `trim()` 逐字符同语义、64 位小写 hex；**禁用** `tr -d '[:space:]'`，它去全部空白与 JS `trim()` 仅去首尾不等价），结果替换迁移 SQL 里 `default` 凭据 `INSERT` 的占位符。
4. **执行迁移 SQL**（单文件，`wrangler d1 execute --file --remote`，**文件内禁写 `BEGIN/COMMIT`**，靠 `--file` 全文件回滚保原子）：建 `tenants`/`tenant_credentials`/`tenant_routes`/`service_counters`；逐业务表「建新表(含 `tenant_id`,新 PK,**完整重建全部业务索引**) → `INSERT…SELECT` 回填 `default` → DROP 旧 → RENAME」；重建 `sync_meta` 为终态（多行取最新）；seed `default` 租户 + `default` 写凭据 + `service_counters('auth_fallback', 0)`。
5. **执行后自检**：`SELECT key_hash FROM tenant_credentials WHERE key_hash LIKE '<占位%'` 须 0 行（占位符已替换），否则视为失败。
6. **部署 worker**：含 `/sync` 按 Key 解析（**删除旧 `token!==env.API_KEY` 前置 401 门**，鉴权统一由 resolveTenant 命中决定）+ 单 batch（或分块）替换、`/ping` 与 `/sync` 在 `env.API_KEY` 空时**均**401（消除 fail-open）且仅在 `env.API_KEY` 侧补 trim（token 侧 `getBearerToken` 已 trim）、`/api/query`（保留 LEFT JOIN）与 route-push join 注入服务端常量 `default`、整段删内联建表、D1 兜底计数器。
7. **验收**：现有桌面端 `/sync` 200 且数据落 `default`、**兜底计数行存在且 `count===0`**（表驱动真命中；缺失/不可读判 inconclusive 非 pass）；桌面端「测试连接」(`/ping`) 200；移动端裸域查询照常字段集不变。
8. **回滚（有序 fail-closed，前滚优先）**：既不能单独退 worker、也不能单独退数据（见风险段）。按序：①新 worker 下线（部署返 503 临时 worker 版本 或 dashboard 解绑路由）→ ②清空迁移后 D1 全部表（**推荐建新空 D1 + 改 `wrangler.toml` 绑定**；若逐表 DROP 须含全部 12 张：三租户表 + `service_counters` + 业务 5 表 + `sync_meta` + 全局 `callsign_openid_bindings`/`sf_route_log`，漏清全局表会让 import 撞 PK）→ ③`wrangler d1 execute --file <步骤1的dump> --remote` 导入旧 dump → ④部署回迁移前旧 worker → ⑤恢复路由。任一中间态皆 fail-closed（500/503，非裸写）。优先只前滚、把回滚当最后手段。

## 待解决问题

- ~~`tenant_id` slug 约束落地形式~~ **已定**：落可执行 CHECK（见 tasks 2.1）。⚠️ 注意 GLOB 语义坑：`GLOB '[a-z0-9-]*'` 中 `*` 是「任意后缀」（shell glob 非 regex），**只约束首字符**，`abc!` 能通过——**禁用**。正确写法为否定类：`tenant_id TEXT NOT NULL PRIMARY KEY CHECK (length(tenant_id) BETWEEN 1 AND 32 AND tenant_id NOT GLOB '*[^a-z0-9-]*')`（`TEXT PRIMARY KEY` 在 rowid 表不隐含 NOT NULL，须显式加）。
- `default` 真实业务表行数量级 + **D1 计划档（Free 50 / Paid 1000）**（共同决定单 `DB.batch` 是否需分块，阈值见风险段）——实现前 `wrangler d1 execute "SELECT COUNT(*) FROM cards" --remote` 等量一下、并查计划档。
- 用 `--remote` 一次性临时/preview D1 跑确认性实测「中段语句失败是否整文件回滚」，确认当前 wrangler 版本与 Cloudflare 文档保证一致（本地 `--local` 可能 `SQLITE_AUTH`，不算数）——作上生产前硬门，非「原子性尚未确定」。
