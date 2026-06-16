## 上下文

阶段 4-C 给桌面端引入「显式租户身份 + 服务端交叉校验」。本变更是其**第一子项 4-C1**，只做**服务端写入侧**：worker 接受客户端声明的租户头、与写入 Key 解析出的租户交叉校验。客户端配置/UI/发头属 4-C2/4-C3。

现状（`web_query_service/src/worker/index.js`）：
- `/sync`、`/pull` handler 已 `resolveTenant(env, token)`→null 则 401，入库/读取用解析出的 `tenant_id`（归属真源=Key）。
- `/ping` handler **不走** `resolveTenant`，仅 `token === trim(env.API_KEY)` 直比 → 表驱动「多 Key→同租户」凭据（阶段1 已建 `tenant_credentials`）在 /ping 通不过。
- `resolveTenant` 兜底命中时会 `UPDATE service_counters('auth_fallback')`。
（以函数/handler 名定位，不引用易漂移的绝对行号。）

需求收敛（见 `docs/multi-tenant-design.md` §5 阶段4 修订）：官方云不收游客、key=认证码、交叉校验「申报 tenant 与 key→tenant 一致否则 403」、归属真源恒为 Key。

## 目标 / 非目标

**目标：**
- 三端（/sync、/pull、/ping）接受 `X-Tenant-Id`，与 `resolveTenant(Key)` 交叉校验，不一致 403。
- 入库/读取恒用 `resolveTenant` 返回值，声明值绝不当目标。
- /ping 升级走 resolveTenant（只读、不计兜底）+ 回显 `{tenant, fallback}`。
- 向后兼容：缺 header 放行。

**非目标：**
- 桌面端 `SyncConfig.tenant`、命令签名、client 发 `X-Tenant-Id`、四态枚举对接、ts-rs（→ 4-C2）。
- 登录窗 / 标题栏徽章 / UI 重组（→ 4-C3）。
- CLI mint 脚本 / `cloud-sync-api-spec` 文档 / 自托管文档（→ 4-C4）。
- 撤兜底 7.4（独立项）；`REQUIRE_POW` / `REQUIRE_TENANT_HEADER` 强制开关（未来按需）。

## 决策

### D1：传输用统一请求头 `X-Tenant-Id`，非 body 字段

`/pull`、`/ping` 是 GET、无 body，body 方案对其不可用、会逼出「三端三种传法」。请求头三端同构、与 `Authorization` 同层、worker 一处 `request.headers.get('X-Tenant-Id')` 统一读。**替代方案**（/sync 用 body.tenant + GET 用 ?tenant=）被否：分裂、易漏校验。需把 `X-Tenant-Id` 加进 CORS `Access-Control-Allow-Headers`（桌面端 reqwest 不受 CORS 约束，此项为浏览器侧 DIY 实现者）。

### D2：单一 helper `crossCheckTenant`，声明值绝不当目标

```
async function crossCheckTenant(env, key, declared, opts?):
  tenant = await resolveTenant(env, key, opts)   // opts.readonly 透传
  if (tenant == null) return { ok:false, status:401, code:'auth_failed' }
  if (declared != null && declared !== tenant)
      return { ok:false, status:403, code:'tenant_mismatch' }
  return { ok:true, tenant }     // tenant 永远是 resolveTenant 的值
```
- `/sync`、`/pull`、`/ping` 调用点把 `resolveTenant` 替换为 `crossCheckTenant`，读 `request.headers.get('X-Tenant-Id')` 传入；403 时返 `{success:false, code:'tenant_mismatch'}`。
- **后续入库/读取逻辑零改动**，仍用返回的 `tenant`（即 resolveTenant 的值）。
- **缺 header（declared==null）→ 放行**：向后兼容灰度期旧客户端不发头。
- **红线**：declared 仅参与比较，从不进 SQL 的 `tenant_id`。这是守住「不复活信任客户端自报归属」的关键——`crossCheckTenant` 返回的 `tenant` 恒为 resolveTenant 值，调用方不得改用 declared。

### D3：/ping 走 resolveTenant + 只读参数（不计兜底）+ 回显

`resolveTenant` 加可选 `readonly` 参数（默认 false）：为 true 时跳过 `service_counters('auth_fallback')` 的 UPDATE。`/ping` 传 `readonly:true`——探活/测试连接高频，若计入兜底会污染「撤兜底验收 count===0」（阶段1 7.4）；`/sync`、`/pull` 仍计数（兜底可观测）。`/ping` 200 **必须**回显 `{ success, message:'pong', server_time, tenant, fallback }`（`tenant`、`fallback` 两字段为 SHALL、非可选——4-C2 模式徽章依赖其存在），让桌面端测试连接确认租户、显示模式徽章。`resolveTenant` 返回 `{tenant, viaFallback}`，`fallback = viaFallback`（本次是否经 env.API_KEY 兜底命中）。

### D4：交叉校验用 ADDED、/ping 升级用 MODIFIED（两者分开判定）

两段改动的 OpenSpec 归类**不同**，必须分开：

- **`/sync`·`/pull`·`/ping` 的 X-Tenant-Id 交叉校验 = ADDED 新需求**：纯新增关注点（X-Tenant-Id 此前不存在、不改既有写/读成功行为），按 OpenSpec「只加新关注点用 ADDED」正确。
- **`/ping` 从「直比 env.API_KEY」升级为「走 resolveTenant + 回显 + 只读」= MODIFIED 既有「云端接收同步数据接口」需求的「连接测试 GET /ping」场景**：起初考虑用 ADDED + 「旧断言仍成立」声明避免复制巨块需求，但 review 指出这会致**归档矛盾**——旧场景措辞「比对必须对 `env.API_KEY` 侧补 trim」隐含「只比 env.API_KEY、表驱动不通」，与新「走 resolveTenant 表驱动可通」并入主规范后并列即自相矛盾（声明在 delta 文档里，不会去改主规范里旧场景的原始措辞）。故改为 MODIFIED：逐字复制该需求、**改写 /ping 场景**为「走 resolveTenant（表驱动+env.API_KEY 兜底，只读不计兜底）+ 回显 {tenant,fallback} + X-Tenant-Id 交叉校验」，trim/空→401 作为 resolveTenant 下的派生断言保留；其余场景（/sync、app_settings 等）原样复制。归档后 /ping 只有一条自洽契约。

### D5：本变更只产 403+code，桌面端枚举态属 4-C2

worker 产出 `403 {code:'tenant_mismatch'}`。桌面端把它解析成 `SyncCmdResult::TenantMismatch` 四态（决策 #4：仅 /sync 建枚举、pull/ping 走 Err 文案）属 4-C2。本期 verify 用 HTTP/单测断言 403+code 即可。

## 风险 / 权衡

- **[声明头可伪造]** declared 是客户端可控。**缓解**：D2 红线——恒用 resolveTenant 值入库，declared 只比较+回显；单测断言「声明 B + A 的 Key → 403 且不写 B」。
- **[灰度期旧客户端不发头]** 4-C1 先部署、4-C2 后部署期间旧客户端无 `X-Tenant-Id`。**缓解**：declared==null 放行，归属仍由 Key、安全不降级。未来 `REQUIRE_TENANT_HEADER` env 在全量升级后收紧（非本期）。
- **[/ping 计兜底污染撤兜底验收]** **缓解**：D3 readonly 参数，/ping 不计数；单测断言「/ping 经兜底命中后 auth_fallback 计数不变」。
- **[/ping 归档契约一致性]** 已由 D4 的 MODIFIED 消解：cloud-backend-api delta **逐字复制**「云端接收同步数据接口」需求、**改写** /ping 场景为「走 resolveTenant + 只读 + 回显 + 交叉校验」。归档（增量并入主规范）时 MODIFIED 整体替换该需求，旧「比对 env.API_KEY」措辞被取代，/ping 仅剩**一条自洽契约**，不再有新旧场景并列矛盾。
- **[CORS]** 仅浏览器 DIY 实现需要；桌面端 reqwest 无 CORS。低风险，文档（4-C4）会说明。

## 迁移计划

1. 实现 worker：`crossCheckTenant` helper + `resolveTenant` readonly 参数 + /ping 重写 + /sync·/pull 改用 helper + CORS 加头。
2. verify：单测覆盖（红线「声明值绝不当目标」/ 缺·空 header 向后兼容 / 表驱动 Key 在 /ping 通 / /ping 回显 tenant+fallback / /ping 只读不计兜底）+ worker smoke 端到端断言（逐条用例清单见 tasks）。
3. 部署：`pnpm run deploy`，记录版本 + 回滚目标。无 D1 迁移。
4. 验收：现有桌面端（不发头）行为等价；带正确头 200、错头 403；/ping 回显租户。
5. 回滚：退 worker 版本（无迁移、无配对）。

## 待解决问题

- `fallback` 字段实现形态（resolveTenant 如何把「是否走兜底」告知 /ping 调用方）——任务中定，不阻塞。
- 是否本期就预留 `REQUIRE_TENANT_HEADER` env（默认放行）——倾向不做，留全量升级后。
