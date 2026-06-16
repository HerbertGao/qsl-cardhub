## 上下文

阶段 1（`add-tenant-foundation`）把 5 张业务表 + `sync_meta` 演进为 `(tenant_id, …)` 行级隔离，但显式把两张全局表推迟到阶段 4：`tenant-isolation` 规范的「openid 反查的租户不变量」与「顺丰呼号反查按服务端常量租户过滤」两需求声明 `callsign_openid_bindings`/`sf_route_log` 本期不加 `tenant_id`、route-push 注入硬编码常量 `bh2ro`，并钉死「阶段 4 上线第二个真实租户前必须先迁这两表」。

当前代码（`web_query_service/src/worker/index.js`）：
- route-push 呼号反查 join 注入 `const tenant_id = 'bh2ro'`（`:780/:783/:791`）；openid 反查 `WHERE callsign = ?`（`:799`，无租户维度）。
- 微信 auth-callback `state` = 纯 callsign（`:832/:836`）；绑定 INSERT `(callsign, openid, created_at)`（`:852-856`，无 tenant）。
- 创始租户 slug = `bh2ro`（部署落地值，非通用 `default`；见阶段 1 PR #45）。

本变更是 4-A，独立于 4-B（路径路由）/4-C（桌面端租户身份），是「上线第二个真实租户」的硬前置。

## 目标 / 非目标

**目标：**
- `callsign_openid_bindings` 加 `tenant_id`、主键 `(tenant_id, callsign, openid)`，存量回填 `bh2ro`，经一次性迁移 `0002` 交付 + `schema.sql` 双写。
- route-push 由「匹配到的 `sf_orders` 行」派生租户，openid 反查带派生租户维度，杜绝同呼号跨租户推送。
- 微信 auth-callback `state` 向前兼容解析 `tenant:callsign`、校验租户活跃、绑定写 `tenant_id`。

**非目标：**
- 不改前端（`state` 改发 `tenant:callsign` 属 4-B）。
- 不给 `sf_route_log` 加 `tenant_id`（保持全局去重，租户由 join 派生）。
- 不做路径路由 / `/api/config` 按租户 / tier 分级（4-B）。
- 不上线第二个真实租户（运维）。

## 决策

### D1：`sf_route_log` 保持全局、不加 `tenant_id`

设计蓝本 §6 与 `tenant-isolation` 规范均定 `sf_route_log` 全局去重——顺丰 waybill 全局唯一，去重维度不需按租户切分；推送目标租户由匹配的 `sf_orders` 派生。**替代方案**（给 `sf_route_log` 也加 `tenant_id`）被否：去重发生在「落库时」，此时尚未做订单匹配、无从知租户；强加 tenant 维度要么二次查询、要么破坏「同一 waybill 节点全局只处理一次」的去重语义。故迁移 `0002` 只动 `callsign_openid_bindings` 一张表。

### D2：route-push 租户「按匹配订单派生」，非注入常量

呼号反查由 `WHERE o.tenant_id = 'bh2ro' AND o.order_id = ?` 改为 `WHERE o.order_id = ?`（按全局唯一业务键匹配），`SELECT c.callsign, o.tenant_id`（同时取回派生租户），保留 `o.tenant_id = c.tenant_id`（同租户自洽）+ `o.card_id = c.id`（业务连接键）。再用派生 `tenant_id` 做 openid 反查 `WHERE tenant_id = ? AND callsign = ?`。

`order_id`（顺丰订单号）与 `waybill_no`（顺丰运单号）均**全局唯一**，故「全局匹配 + LIMIT 1 + 由匹配行取租户」无歧义。**替代方案**（要求 route-push 报文自带 tenant）被否：顺丰报文不含我方租户、且自报值不可信（违「禁止取自请求体自报值」）。派生租户唯一可信来源 = 服务端在 D1 匹配到的订单行。

### D3：auth-callback 租户来自 `state`，向前兼容 + 活跃校验

微信 OAuth 回调域名是公众号后台固定登记的，**租户无法靠 host 区分、必须靠 `state` 携带**（蓝本 §6）。`state` 解析以**首个**冒号分隔为 `tenant:callsign`；无冒号时 `callsign` = 整串、`tenant_id` 回退 `bh2ro`（本期前端仍发纯 callsign，4-B 才改发 `tenant:callsign`，故 worker 必须双向兼容）。解析出的 `tenant_id` 必须查 `tenants`（`status='active'`）校验，非活跃/不存在则拒绝写入——避免公开回调被构造任意 `state` 污染绑定表。

**替代方案**（不校验租户、直接写）被否：`callsign_openid_bindings.tenant_id` 无 FK 到 `tenants`（schema 一致性靠应用层），不校验则可写入任意租户字符串的垃圾行。校验成本 = 每次回调一次索引查询，可接受（回调本就低频 + 已有 authcb IP 限流）。

### D4：迁移沿用 0001 四步重建 + 索引后置

`callsign_openid_bindings` 经「建新表（含 `tenant_id NOT NULL` + 新 PK）→ `INSERT…SELECT` 回填 `'bh2ro'` → DROP 旧 → RENAME」。`idx_bindings_callsign` 必须在 RENAME **之后**重建（SQLite 索引名库级唯一，阶段 1 的 ship-blocker 教训：在 `*_new` 上建同名索引 → `already exists` 整文件失败）。整份文件无 `BEGIN/COMMIT`（D1 `--file` 整体原子，写显式事务报 `SQLITE_AUTH`）。`schema.sql` 同步双写终态。

回填无需单一所有者校验：绑定表无 `client_id`、回填值恒 `'bh2ro'`；存量 `(callsign, openid)` 唯一 → `('bh2ro', callsign, openid)` 仍唯一，无 PK 冲突。

## 风险 / 权衡

- **[迁移与 worker 强耦合]** 迁移后新表 `tenant_id NOT NULL`，旧 worker 的 `INSERT (callsign, openid, …)` 会因缺列失败 → 不可单独回退 worker。**缓解**：worker 与迁移配对部署；回滚 = 二者一起还原（退 worker 版本 + `wrangler d1 export` dump 还原表）；执行前强制 export 备份。
- **[order_id 跨租户理论碰撞]** 若两租户存在相同 `order_id`（顺丰订单号实际全局唯一，碰撞概率≈0），全局匹配 `LIMIT 1` 取首条、派生其租户。**缓解**：waybill_no 优先级路径同为全局唯一；实际不可达，接受残余风险（accepted out-of-scope）。
- **[本期前端未改、绑定仍落 bh2ro]** 4-A 上线后前端仍发纯 callsign → 所有新绑定 `tenant_id='bh2ro'`，与存量一致、无回归。第二租户的正确绑定要等 4-B 前端改发 `tenant:callsign`。**缓解**：这是分期预期；4-A 仅把服务端改成「能接住带租户的 state」+「按派生租户推送」，不依赖前端。
- **[公开回调租户校验绕过]** 攻击者构造 `state=<不存在租户>:CALL` → 被活跃校验拒，不写入。**缓解**：D3 强制 `tenants` 活跃校验；真正闸门仍是微信 OAuth `code` 必须有效（拿不到他人 openid）。

## 迁移计划

1. 实现 worker 改动（route-push 派生租户 + auth-callback state 解析/校验/写 tenant_id）+ 迁移 `0002` + `schema.sql` 双写。
2. 本地 verify：迁移离线断言（表结构/回填/索引/回填不撞 PK）+ worker smoke（带 tenant 的 state 解析、无冒号回退 bh2ro、非活跃租户拒绝、派生租户 openid 反查隔离）。
3. 运维执行（用户自跑，代理不碰生产）：`wrangler d1 export` 备份 → `wrangler d1 execute --file=migrations/0002_*.sql --remote` → 部署配对 worker。
4. 验收：现网订阅/推送仍正常（bh2ro 路径行为等价）；构造两租户同呼号绑定时推送不跨租户（可在临时库验证）。
5. 回滚：退 worker 版本 + 还原表 dump。

## 待解决问题

- 无阻塞性未决项。第二租户的实际 `tenant:callsign` 前端改动、tier 分级在 4-B 处理。
