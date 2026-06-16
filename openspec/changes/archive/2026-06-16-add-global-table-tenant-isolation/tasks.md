## 1. 数据库迁移与 schema 双写

- [x] 1.1 新建 `web_query_service/migrations/0002_global_table_tenant.sql`：仅重建 `callsign_openid_bindings`——建 `callsign_openid_bindings_new`（含 `tenant_id TEXT NOT NULL`、主键 `(tenant_id, callsign, openid)`、`created_at`）→ `INSERT…SELECT 'bh2ro', callsign, openid, created_at FROM callsign_openid_bindings` 回填 → `DROP TABLE callsign_openid_bindings` → `ALTER TABLE callsign_openid_bindings_new RENAME TO callsign_openid_bindings`
- [x] 1.2 在 DROP+RENAME **之后**重建 `idx_bindings_callsign`（SQLite 索引名库级唯一，沿用 0001 索引后置教训，避免 `already exists`）；review-loop 升级为 `(tenant_id, callsign COLLATE NOCASE)`（带租户维度 + 大小写不敏感，服务 route-push 反查、与全系统 NOCASE 约定一致）
- [x] 1.3 迁移文件内**禁止** `BEGIN/COMMIT/SAVEPOINT`（D1 `--file` 整体原子）；`sf_route_log` 不出现在迁移中（保持全局不变）
- [x] 1.4 `web_query_service/schema.sql` 双写：`callsign_openid_bindings` 改为含 `tenant_id`、PK `(tenant_id, callsign, openid)`、索引 `(tenant_id, callsign COLLATE NOCASE)`；更正其与 `sf_route_log` 的注释

## 2. Worker：route-push 按匹配订单派生租户

- [x] 2.1 移除 `index.js` route-push 中硬编码 `const tenant_id = 'bh2ro'`
- [x] 2.2 呼号反查按 `order_id`/`waybill_no` 匹配 `sf_orders`、`SELECT c.callsign, o.tenant_id` 取回派生租户。review-loop 升级为**合并两键候选行 + distinct 租户决策**（==1 推 / ==0 不推 / >1 fail-closed 跳过+日志），覆盖单键跨租户 + 两键互相矛盾
- [x] 2.3 openid 反查改为 `SELECT DISTINCT openid FROM callsign_openid_bindings WHERE tenant_id = ? AND callsign = ? COLLATE NOCASE`（带派生租户维度 + NOCASE + DISTINCT 防重复推送）
- [x] 2.4 保留 `o.tenant_id = c.tenant_id` 同租户自洽 + `o.card_id = c.id` 业务连接键（二者缺一不可）

## 3. Worker：auth-callback state 携带并校验租户

- [x] 3.1 解析 `state`：以**首个**冒号分隔 `tenant:callsign`；无冒号回退 `'bh2ro'`。review-loop 加：入参长度上限（state>256/code>512→400）、`decodeURIComponent` try/catch（畸形→400）、`encodeURIComponent(code)`
- [x] 3.2 解析出的 `tenant_id` 查 `tenants`（`status='active'`）校验，非活跃/不存在则 400「无效租户」（不写绑定）
- [x] 3.3 绑定 INSERT 改为 `INSERT OR IGNORE INTO callsign_openid_bindings (tenant_id, callsign, openid, created_at) VALUES (?,?,?,?)`
- [x] 3.4 前端不改（`App.vue:89` 仍发纯 callsign；`state` 改发 `tenant:callsign` 留待 4-B）。review-loop 加：callsign 字符白名单 `^[A-Z0-9/]{1,16}$`（堵成功页反射 XSS 源）+ 成功页 `safeCallsign` HTML 转义

## 4. 本地验证

- [x] 4.1 新建 `verify/run_0002_migration.sh`（sqlite3 离线）：A 段迁移正确性（无 `already exists`、新表结构/回填 bh2ro/索引 NOCASE/行数/无 PK 冲突/sf_route_log 未变/schema 双写列+PK+索引一致）；B 段 route-push 跨租户隔离不变量（派生/openid 隔离/NOCASE 与 BINARY 证伪/单键歧义/两键矛盾 fail-closed/同租户自洽 join）。**28 断言全 PASS**
- [x] 4.2 扩展 `verify/run_worker_smoke.sh` 4.6 段：state 解析/无冒号回退/不存在·非活跃租户拒绝/空呼号/畸形%25/超长/XSS 构造 共 9 条 HTTP 断言；并修 start_cdn/start_nokv 的 `--persist-to`（防 D1 持久化分叉打空库）
- [x] 4.3 跑通 `pnpm run build` 绿、迁移离线 28/28、worker smoke **91/0**、`test:unit` 81/81、`node --check` OK、shellcheck OK
- [x] 4.4 对抗 review-loop 到 clean APPROVE（Codex+Code Reviewer+Reality Checker[general-purpose 兜底]+Database Optimizer+Security Engineer；4 轮收敛，修掉 callsign 大小写漏推/跨租户单号歧义 fail-closed 不完整/反射 XSS 等）

## 5. 文档与运维交付

- [x] 5.1 在 `docs/web-query-service-deploy.md` 增补「阶段 4-A 全局表租户化迁移（0002）」节（export 备份 → `wrangler d1 execute --file --remote` → 配对部署 worker → 回滚剧本：退 worker 版本 + 还原表 dump，强调不可单退 worker）
- [x] 5.2【用户自跑】`wrangler d1 export qsl-sync --remote` 全量备份 → `~/qsl-d1-backup-before-0002.sql`
- [x] 5.3【用户自跑】`wrangler d1 execute --remote --file=…/0002_*.sql` 执行迁移成功（5 queries、changed_db；远端确认新表 tenant_id+PK+NOCASE 索引、sf_route_log 未碰、绑定表 0 行）
- [x] 5.4【代理代部署】`pnpm run deploy` → worker 版本 **facccf8a-07d9-4a87-9ccd-a8b57a54f022**（回滚目标 **bf733673**）
- [x] 5.5【验收】生产冒烟全绿：/ping 401 / /api/config 200(无 sign_key) / /api/query 401；auth-callback 差分 nope→400 无效租户 / bh2ro:&lt;script&gt;→400 无效呼号 / %25→400 无效 state / bh2ro:CALL→503（过租户校验，止于微信未配=线上未配微信服务号、订阅推送本未启用）。bh2ro 路径行为等价、零回归

## 6. 归档

- [ ] 6.1 用户确认验收通过后 `openspec-cn archive add-global-table-tenant-isolation`（增量并入 `tenant-isolation`/`wechat-push`/`sf-route-push-receiver` 主规范）
