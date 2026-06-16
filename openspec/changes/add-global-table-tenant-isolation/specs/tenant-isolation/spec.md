## 修改需求

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

## 新增需求

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
