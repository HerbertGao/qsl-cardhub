## 为什么

多租户地基（阶段 1）已把 5 张业务表 + `sync_meta` 演进为 `(tenant_id, …)` 行级隔离，但两张全局表 `callsign_openid_bindings`（呼号–微信 openid 绑定）与顺丰路由推送链路被显式**推迟到阶段 4**（见 `tenant-isolation` 规范「openid 反查的租户不变量」「顺丰呼号反查按服务端常量租户过滤」两需求）。当前后果：

- `callsign_openid_bindings` 无 `tenant_id`、PK 仅 `(callsign, openid)`；route-push 反查 openid 是 `WHERE callsign = ?`（无租户维度，`index.js:799`）。一旦上线第二个真实租户、两租户出现**同一呼号**的订阅者，顺丰物流推送会**跨租户**发给对方订阅者——openid 与物流轨迹（运单号/节点/时间）泄漏。
- route-push 的呼号反查 join 当前注入硬编码常量 `tenant_id = 'bh2ro'`（`index.js:780`），不是「按匹配到的订单派生租户」，第二个租户的订单永远反查不到呼号、推送静默丢失。
- 微信授权 `state` 仅携带 callsign（`App.vue:89`），回调无从得知归属租户。

这是「上线第二个真实租户」的**硬前置**：阶段 1 已钉死「第二个真实租户上线前必须先迁这两表」。本变更独立、含一次性 D1 迁移、无路由依赖，先于阶段 4 的路径路由（4-B）与桌面端租户身份（4-C）交付。

## 变更内容

- **`callsign_openid_bindings` 加 `tenant_id` 列**，主键演进为 `(tenant_id, callsign, openid)`；存量行回填创始租户 `bh2ro`。以一次性迁移 `migrations/0002_*.sql`（建新表 → `INSERT…SELECT` 回填 → DROP → RENAME → 重建 `idx_bindings_callsign`）交付，并在 `schema.sql` 双写终态结构。
- **`sf_route_log` 保持全局、不加 `tenant_id`**（顺丰 waybill 全局唯一，去重维度全局；租户由匹配的 `sf_orders` join 派生）。本变更仅更正其 schema 注释，不改表结构。
- **route-push 按订单派生租户过滤推送**（`index.js:772-809`）：呼号反查改为按 `order_id`/`waybill_no` 全局匹配 `sf_orders`、由匹配行返回其 `tenant_id`；openid 反查改为 `WHERE tenant_id = ? AND callsign = ?`（注入派生租户）。移除硬编码 `'bh2ro'`。
- **微信授权 `state` 向前兼容携带租户**（`index.js:820-858`）：回调解析 `state` 为 `tenant:callsign`（无冒号 → 租户回退 `bh2ro`、callsign=整串，兼容本期仍发纯 callsign 的前端）；解析出的租户**必须**校验为 `tenants` 中的活跃租户，否则拒绝；绑定 INSERT 写入 `tenant_id`。**前端本期零改动**（`state` 改发 `tenant:callsign` 留待 4-B 路径路由落地）。

不在本变更：路径路由解析、`/api/config` 按租户下发、tier 分级（4-B）；桌面端租户身份（4-C）；上线第二个真实租户（运维）。

## 功能 (Capabilities)

### 新增功能
<!-- 无新增能力；本变更兑现既有能力中显式推迟到阶段 4 的需求。 -->

### 修改功能
- `tenant-isolation`: 兑现「openid 反查的租户不变量」——`callsign_openid_bindings` 加 `tenant_id`、PK `(tenant_id, callsign, openid)`、迁移回填创始租户；把「顺丰呼号反查按服务端常量租户过滤」从「本期恒常量、按 order 派生属阶段 4」升级为「按匹配订单派生租户、openid 反查注入派生租户」；`sf_route_log` 显式定为全局、不加 `tenant_id`。
- `wechat-push`: 呼号–openid 绑定升级为**租户维度**——授权 `state` 携带（向前兼容）并校验租户、绑定写入 `tenant_id`；顺丰推送时按「呼号 + 派生租户」查绑定，杜绝同呼号跨租户推送。
- `sf-route-push-receiver`: 路由数据与呼号关联时，由匹配的 `sf_orders` 派生 `tenant_id`，并据此过滤 openid 反查与推送目标。

## 影响

- **数据库**：新增迁移 `web_query_service/migrations/0002_*.sql`（仅 `callsign_openid_bindings` 重建）；`web_query_service/schema.sql` 双写（`callsign_openid_bindings` 新结构 + `sf_route_log` 注释更正）。
- **Worker**：`web_query_service/src/worker/index.js` 的 route-push 反查（`:772-809`）与微信 auth-callback（`:820-858`）。
- **前端**：本变更不改（`App.vue` 的 `state` 改动属 4-B）。
- **迁移交付**：单一 SQL 文件，运维在自己终端 `wrangler d1 execute --file --remote`，执行前 `wrangler d1 export` 备份；与 worker 配对部署（迁移后 worker 读 `tenant_id` 列，不可单独回退 worker）。
- **回滚**：退 worker 版本 + 还原表（dump import 或 DROP 后重建旧结构）。
- **现网安全态不回退**：未上线第二个真实租户前，全部绑定/订单仍归 `bh2ro`，行为与现状等价。
