## 为什么

阶段 4-C「桌面端租户身份」要求客户端**显式声明**自己的租户、服务端**交叉校验**「声明的租户」与「写入 Key 解析出的租户」一致，挡住一类不可逆事故：

- `/sync` 是**全量覆盖**（`DELETE WHERE tenant_id=? + 重新 INSERT`）。若用户**误粘了别的租户的 Key**，其数据会静默覆盖到那个租户、抹掉对方全部数据。Key 是唯一归属真源，没有「人声明的意图」与之比对，这种粘贴错配无从拦截。
- 当前 `/ping` handler **不走** `resolveTenant`、只 `token === trim(env.API_KEY)` 直比——表驱动的「多 Key → 同租户」凭据在 `/ping` **根本不通**（只有 env.API_KEY 那一把能测通）。这是阶段 1 凭据表落地后遗留的实现缺口，且让「测试连接」无法回显用户的真实租户身份。

本变更是 4-C 的**第一子项（4-C1）**，纯服务端写入侧、**不依赖 4-B 路由**、**向后兼容**（缺 header 放行），可独立先行交付与部署。客户端发送 `X-Tenant-Id`、桌面端配置/UI 改造留 4-C2/4-C3。

## 变更内容

- **统一请求头 `X-Tenant-Id`**：`/sync`、`/pull`、`/ping` 三端接收客户端**声明的租户 slug**（GET 端点无 body，故用请求头而非 body 字段，三端同构）。`CORS_HEADERS` 的 `Access-Control-Allow-Headers` 加入 `X-Tenant-Id`（供浏览器侧 DIY 实现）。
- **worker 交叉校验 helper `crossCheckTenant(env, key, declared)`**：先 `resolveTenant(key)`，未命中 → 401；命中得 `tenant`；若 `declared` 非空且 `declared !== tenant` → **403 `{code:'tenant_mismatch'}`**；`declared` 缺失（旧客户端无 header）→ **向后兼容放行**。**红线（不可破）**：入库/读取的 `tenant_id` **恒**取 `resolveTenant` 的返回值，`X-Tenant-Id` **只用于校验+回显、绝不当写入/读取目标**——否则复活「信任客户端自报归属」（持 A 的有效 Key + 声明 B → 写进 B）。
- **`/sync`、`/pull`** 改用 `crossCheckTenant`：声明与解析不一致 → 403（零改动入库逻辑，仍用解析出的 `tenant`）。
- **`/ping` 升级**：从「直比 `env.API_KEY`」改为走 `resolveTenant`（**只读**——见下），回显 `{ success, message:'pong', server_time, tenant, fallback }`，让「测试连接」确认 Key 所属租户、并对 `X-Tenant-Id` 做同样交叉校验。
- **`resolveTenant` 加 `readonly` 参数**：`/ping` 传 `true` → **跳过 `service_counters('auth_fallback')` 兜底计数 UPDATE**。否则探活/测试连接会污染「撤兜底验收 `auth_fallback count===0`」（阶段 1 7.4）。`/sync`、`/pull` 仍计数（保持兜底可观测）。

不在本变更（属 4-C2/4-C3/4-C4）：桌面端 `SyncConfig.tenant` 字段 / 命令签名 / client 发 `X-Tenant-Id` / 四态枚举对接 / ts-rs；登录窗 + 标题栏徽章 + UI 重组；CLI mint 脚本 + `cloud-sync-api-spec` 文档。

## 功能 (Capabilities)

### 新增功能
<!-- 无新增能力。 -->

### 修改功能
- `cloud-backend-api`: `/ping` 从「直比 env.API_KEY」升级为走 `resolveTenant`（只读不计兜底）+ 回显 `{tenant, fallback}`；`/sync`、`/pull` 增加 `X-Tenant-Id` 交叉校验（声明 ≠ 解析 → 403 `tenant_mismatch`，缺 header 向后兼容放行）；`CORS` 放行 `X-Tenant-Id`。
- `tenant-isolation`: 新增「写入端点声明租户交叉校验」需求——`X-Tenant-Id` 必须与 `resolveTenant(Key)` 一致否则 403，声明值**禁止**当写入/读取目标（归属真源恒为 Key），缺 header 向后兼容放行；并钉死 `/ping` 的 `resolveTenant` 调用**必须**只读、不递增兜底计数。

## 影响

- **Worker**：`web_query_service/src/worker/index.js`——新增 `crossCheckTenant` helper；`resolveTenant` 加 `readonly` 参数（默认 false）、返回 `{tenant, viaFallback}`；`/ping` handler 重写；`/sync`、`/pull` handler 改用 helper + 读 `X-Tenant-Id`；`CORS_HEADERS` 的 `Access-Control-Allow-Headers` 加 `X-Tenant-Id`。（具体函数/handler 名见实现，避免引用易漂移的绝对行号。）
- **验证**：`web_query_service/verify/` 新增交叉校验单测（mismatch→403 / 缺 header 向后兼容 / /ping 回显 tenant+fallback / /ping 不计兜底计数 / 表驱动多 Key 在 /ping 通）；`run_worker_smoke.sh` 补端到端断言。
- **无 D1 迁移**；回滚 = 退 worker 版本。
- **向后兼容**：现有桌面端不发 `X-Tenant-Id` → 放行、行为等价（归属仍由 Key 决定）；`/ping` 对 env.API_KEY 那把 Key 仍通（现解析为 bh2ro），且表驱动凭据现在也能在 /ping 通（修复缺口）。客户端发送 `X-Tenant-Id` 与 403 态对接属 4-C2。
