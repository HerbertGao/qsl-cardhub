## MODIFIED Requirements

### 需求：租户模型与默认租户

云端**必须**提供行级多租户隔离的数据模型，由 `tenants`、`tenant_credentials`、`tenant_routes` 三张表承载，并保证既有数据归属于**由 `DEFAULT_TENANT` 配置指定的内置默认租户**（部署落地为 `bh2ro`）。worker 代码**禁止散落**硬编码的默认租户字面量（如 `'bh2ro'`）——默认租户身份**必须**统一经单一 helper 取自 `env.DEFAULT_TENANT`，**仅**该 helper 的兜底默认（未配置时缺省 `bh2ro`，保持现网兼容）允许保留一处 `'bh2ro'` 字面量，使同一份代码经改配置 + seed 各自租户即可被他人部署。

- `tenants`：租户主表，`tenant_id` 为人类可读 slug，**必须** `NOT NULL`（`TEXT PRIMARY KEY` 在 rowid 表不隐含 NOT NULL）。字符集约束（小写字母、数字、连字符，长度上限）**必须**以可执行约束落地，且**必须**用否定字符类：`CHECK (length(tenant_id) BETWEEN 1 AND 32 AND tenant_id NOT GLOB '*[^a-z0-9-]*')`——**禁用** `GLOB '[a-z0-9-]*'`（其 `*` 是 shell-glob「任意后缀」、只约束首字符，`abc!` 会通过），更**禁止**仅以注释表达「必须」。**已知缺口**：`tenants.status` 现**无** `CHECK` 枚举约束（不同于 `tenant_credentials.status`），活跃校验为 `status='active'` 精确等值，线下签发误写 `'Active'`/`' active'` 会令该租户整站静默 404；补 `CHECK(status IN ('active',...))` 属后续阶段（4-C4 线下签发工具）。
- `tenant_credentials`：写入凭据表，存 `key_hash = sha256(trim(key))`（**禁止**存明文 Key；本期**不加 pepper**，前提是 `API_KEY` 为高熵随机值——见 design 决策 D 残余风险）；命中得 `tenant_id`；**必须**支持「多 Key → 同一租户」；`status` **必须** `NOT NULL DEFAULT 'active'` 且 `CHECK` 取值域（如 `'active'`/`'revoked'`），同一 `key_hash` 在 `status='active'` 下**必须**唯一（部分唯一索引），**禁止**一把 Key 解析到两个租户。
- `tenant_routes`：host/path → 租户路由表（本期建表即可，路由解析逻辑属后续阶段）。

#### 场景：内置默认租户与默认写凭据

- **当** 多租户地基初始化（迁移）完成
- **那么** `tenants` 中**必须**存在 `tenant_id` 等于该部署 `DEFAULT_TENANT` 配置值（本部署 `bh2ro`）的活跃租户
- **并且** `tenant_credentials` 中**必须**存在一条 `tenant_id` 为该默认租户、`key_hash = sha256(trim(现有全局 API_KEY))` 的活跃写凭据（`trim` 与 worker 解析侧逐字符一致）
- **并且** 现有桌面端用原全局 API_KEY 同步时**必须**被解析为该默认租户
- **并且** 部署契约：seed 的默认租户 slug **必须**与 `env.DEFAULT_TENANT` 配置值一致（二者不一致 → bare 面以未 seed 租户查询、静默空结果）

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
- **那么** 服务端**必须**将其解析为 `env.DEFAULT_TENANT` 指定的默认租户并放行（兜底，避免 seed 异常锁死现有桌面端）；该默认租户身份**禁止**硬编码字面量，**必须**取自 `DEFAULT_TENANT`（缺省 `bh2ro`）
- **并且** 走兜底路径时**必须**递增一个 **D1 计数行**（具名表 `service_counters(name TEXT PRIMARY KEY, count INTEGER NOT NULL DEFAULT 0)` 的 `name='auth_fallback'` 行；**禁止**用 KV——KV 未绑定时 fail-open 会吞掉递增致假绿）；该表与初始 `count=0` 行**必须**由迁移**自建（`CREATE TABLE`）并 seed**（迁移文件须含建表语句、排在 seed 之前，否则生产 D1 无此表致 seed `no such table`）；**计数器写失败**（用 `.run()` 返回的 `result.meta.changes === 0` 判行缺失，或写抛错判表缺失）**必须使 `/sync` 返非 200**，不得静默吞
- **并且** 迁移后验收**必须**断言「该计数行**存在 且 `count === 0`**」（**严格 `===` + 存在性双检**——因 `(row?.count ?? 0)===0`、`count>=0`、`!count`、`""==0` 等宽松写法会把「行缺失」误当 0 而假绿（注：`null==0` 本身为 `false`，假绿来自 `?? 0`/`>=0`/`!`/空串==0 这些路径），故强制存在性 + 严格相等）；计数行缺失/不可读**必须**判 inconclusive，**禁止**判 pass 或当作可接受降级而误判表驱动已生效
- **并且** 撤销兜底**必须**有量化判据（连续 N 次/T 时间窗表驱动命中且兜底计数恒 0）

#### 场景：env.API_KEY 未配置时拒绝（不沿用「空则放行」）

- **当** `env.API_KEY` 未配置（空/undefined）
- **那么** 兜底分支**必须**视为不命中、返回 401，**禁止**沿用现状 worker「`env.API_KEY` 为空即跳过校验放行」的语义（否则表空 + key 空 → 裸 `/sync` 无鉴权写入）

#### 场景：无效 Key 拒绝

- **当** Key 既未命中 `tenant_credentials` 也不等于 `trim(env.API_KEY)`
- **那么** 服务端**必须**返回 401，**禁止**写入任何数据，且错误响应**禁止**回显内部结构（与已归档「错误响应脱敏」需求一致）

### 需求：读取按服务端注入的租户过滤

按呼号查询（`/api/query`、`/api/callsigns/:callsign`）与顺丰 route-push 的呼号反查 join、及 route-push 的 openid 反查 **必须**按服务端确定的 `tenant_id` 过滤，`tenant_id` **禁止**取自前端参数或请求体自报值。其中：

- **按呼号查询**侧的 `tenant_id` **必须**由 `tenant-path-routing` 解析：显式 `/t/<slug>/` 前缀经 `tenants` 表 `status='active'` 校验后取 `<slug>`，bare 路径取 `env.DEFAULT_TENANT`（缺省默认租户 `bh2ro`）；**禁止**取自前端查询参数或请求体自报值，**禁止**未经活跃校验直接把路径 slug 用作 SQL 过滤目标。读取面是公开匿名读，URL 即「选择查询哪个活跃租户的公开数据」，与**写入面**租户恒由 Key 解析（密钥、绝不取路径）的口径**必须**区分。
- **route-push** 是无凭据公开端点、无路由/凭据上下文，其 `tenant_id` **必须**由「本次推送匹配到的 `sf_orders` 行」**派生**（按全局唯一的 `order_id`/`waybill_no` 匹配订单，由该订单行返回其 `tenant_id`），**禁止**再注入硬编码常量；openid 反查**必须**带该派生 `tenant_id` 维度。

#### 场景：查询注入租户过滤

- **当** 调用方按呼号查询
- **那么** 服务端**必须**在 SQL 中注入 `WHERE tenant_id = ? AND callsign = ?`，其中 `tenant_id` 来自 `tenant-path-routing` 解析（显式 `/t/<slug>/` 前缀的活跃租户 slug，或 bare 路径的 `env.DEFAULT_TENANT`），经服务端校验后注入
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
