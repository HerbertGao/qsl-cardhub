## 1. worker：默认租户配置化（消除硬编码 bh2ro）

- [x] 1.1 取默认租户统一经一个 helper `defaultTenant(env)` 返回 `(env.DEFAULT_TENANT || 'bh2ro')`——`'bh2ro'` 字面量**仅**保留在此 helper 的兜底默认一处（现网兼容）
- [x] 1.2 三处赋值改调 `defaultTenant(env)`：`index.js:710`（读取面查询）、`:287`（`resolveTenant` 的 `env.API_KEY` 写入面兜底默认）、`:896`（微信 `auth-callback` 无 state 兜底）；同步更新注释里的 `bh2ro` 表述为「默认租户/DEFAULT_TENANT」——注意 `:708` 注释含**带引号**的 `'bh2ro'`、`:873/:882/:908` 为无引号散文，均须改。验收：`grep "'bh2ro'" src/worker/`（含注释内引号）当前=4（287/710/896 + 708 引号注释），改后应**恰好 1 处**（`defaultTenant` helper 的兜底默认）
- [x] 1.3 无下游耦合假设：确认无代码把 `'bh2ro'` 当兜底租户做比较/断言（已核：287/710/896 均为赋值、无比较）；微信 896 改后仍经 line 910-914 活跃租户校验（误配 DEFAULT_TENANT → 400「无效租户」，安全拒绝）

## 2. worker：前缀解析与端点三分类

- [x] 2.1 `fetch` 顶部、分发前解析 `url.pathname`：段边界正则 `^/t/([a-z0-9-]{1,32})(/.*)?$`，命中得 `slug` + `routePath`（缺省 `/`，**独立局部变量**）；`/t`、`/t/`、或以 `/t/` 开头但 slug 段不匹配 → 404（`/t` 命名空间保留，判据 `^/t(/|$)`，**禁止** fall-through 当 bare）；**不命中 `^/t(/|$)`** → 无前缀 bare。**所有端点分发比较（`path === '/api/...'`、`startsWith` 等）改读 `routePath`**，原始 `url.pathname` 不就地改写
- [x] 2.2 **数据端点**（`/api/config` 显式前缀、`/api/query`、`/api/callsigns/:cs`）：`SELECT 1 FROM tenants WHERE tenant_id=? AND status='active'`，未命中 404；命中以 `tenant_id=slug` 作上下文
- [x] 2.3 **租户无关端点**（SPA 外壳、`/api/session/challenge`、`/api/session`）：接受任意文法合法前缀、剥离分发，**不**校验 slug、**不**读 tenants 表、**不**因未知 slug 404
- [x] 2.4 **非查询面端点**（`/sync`/`/pull`/`/ping`/`/api/sf/*`/`/api/wechat/*`）：带 `/t/` 前缀一律 404。**必须**置**前缀存在 gate**——「前缀存在 且 `routePath` ∈ 非查询面集合 → 立即 404」，**早于**这些 handler（否则 2.1 改读 routePath 后 `/t/x/sync` 的 `routePath==='/sync'` 会落入 sync handler）
- [x] 2.5 bare 路径（不带前缀）`tenantId = defaultTenant(env)`，**不读** tenants 表（热路径零新增 DB 读）

## 3. worker：读取面去硬编码 + 每租户 config

- [x] 3.1 `/api/query`、`/api/callsigns/:cs` 的 `tenant_id` 用 2.2/2.5 解析值（替换 `:710` 常量）；保留 `WHERE tenant_id = ? AND callsign = ? COLLATE NOCASE` 与 `projects` 的 `tenant_id` join
- [x] 3.2 `/api/config` 在既有 `{features, wechat_appid, filing}` 基础上加**嵌套** `tenant: { id, name }`：显式前缀 `SELECT name FROM tenants WHERE tenant_id=?`，bare 回显 `{ id: defaultTenant(env), name }`（name 可空）；既有字段语义不变，**禁止**下发查询签名密钥
- [x] 3.3 红线自审：路径 slug 只在「活跃校验通过后」用作读取面 `tenant_id`；**绝不**用作任何 INSERT/DELETE 写入目标

## 4. worker：会话签名与防爬不变量（双 path 变量）

- [x] 4.1 `verifySessionSig` 实参**保持原始 `url.pathname`（含前缀）**；剥离前缀只产 `routePath` 供分发，**禁止**就地改写 `url`/`url.pathname`、**禁止**把 `routePath` 传入签名校验（D4）
- [x] 4.2 确认 always-PoW：任意前缀（含 bare 默认）下查询都经「限流 → 会话校验 → 查询」管线，无按租户旁路
- [x] 4.3 确认限流/会话绑定键仍为 `clientBindingKey(getClientIP(...))`，前缀不进入键

## 5. worker：SPA 外壳前缀无关

- [x] 5.1 `/t/<slug>/`（及其下无扩展名路径）服务 `index.html` 外壳，不为外壳校验 slug；静态资源经绝对 `/assets/`、`/favicon.svg` 取用。**SPA fallback（`index.js:960-969`）的无扩展名守卫 `!path.includes('.')` 须改读 `routePath`**：`/t/<slug>/`→`routePath='/'` 无点→回退 index.html；`/t/<slug>/x.js`→`routePath='/x.js'` 有点→返 404（前缀路径非真实资产、`env.ASSETS.fetch` 必 404）

## 6. 前端：单点前缀前置

- [x] 6.1 新增 helper `tenantBase()`（如 `src/client/utils/tenant.ts`）：读 `location.pathname`，以**段边界**正则 `^/t/[a-z0-9-]{1,32}(?:/|$)` 匹配返回该前缀（无尾斜杠）否则 `''`——须与 worker parser 文法同构，禁用无边界正则（超长 slug 截断错误前缀）
- [x] 6.2 在调用入参处**单点前置** `tenantBase()`：`App.vue` 的 `/api/config`、`utils/session.ts` 的 `/api/session/challenge`、`/api/session`、`queryCallsign`→`requestQuery` 的 `/api/callsigns/:cs`
- [x] 6.3 `signQuery`/`sign.ts` **不改**（其 `new URL(path,origin).pathname` 已从带前缀入参派生）；**禁止**在 signQuery 内二次读 `location` 拼前缀（双前缀 → 401）
- [x] 6.4 前端对 `/api/config` 的 `tenant` 字段做展示（如标题显示当前租户）与「租户不存在」错误态（config 404）；静态资源引用保持绝对（前缀无关）
- [x] 6.5 **微信订阅租户**：`App.vue:89` 的 OAuth `state` 由 `encodeURIComponent(callsign)` 改为——`/t/<slug>/` 页 `encodeURIComponent(`${slug}:${callsign}`)`（`slug` 取自 **`tenantBase()`** 的 URL 派生、**非** config.tenant.id，避免 config 未就绪时空租户）；bare 页 `tenantBase()=''` → `encodeURIComponent(callsign)`（无冒号）。`redirect_uri` 保持 bare `/api/wechat/auth-callback`；确认 callback（`index.js:890-914`）已按**首个**冒号解析 `tenant:callsign` + 校验活跃租户、无冒号兜底取 `DEFAULT_TENANT`（随 1.2 配置化）

## 7. 配置与部署

- [x] 7.1 `wrangler.toml` 的 `[vars]` + `wrangler.toml.example` 增补 `DEFAULT_TENANT`（非密钥、可入版本控制示例）——「改配置文件即可换默认租户」的落点
- [x] 7.2 部署清单：CDN 为 `/t/*` 配回源至同一 worker（同域、无新增域名）；`/t/<slug>/api/*` 动态不可缓存，`/t/<slug>/`（外壳）缓存策略同现有 SPA 外壳
- [x] 7.3 **无数据库迁移**：三租户表 + 默认租户 seed 已由 0001 上线；新部署初始租户 seed 属部署期 DB 步骤，其 slug **必须** == `DEFAULT_TENANT`（否则 bare 面静默空结果）

## 8. 测试

- [x] 8.1 单测（`verify/`）：前缀文法解析——命中、不命中、`/t`、`/t/`、`/t//`、`/tfoo`(不命中命名空间)、大写 slug、超 32 长、`/t/bh2ro`(无尾斜杠→命中 slug+routePath=`/`)、`/t/x/`(裸外壳)；`/t` 命名空间非法前缀 → 404 而非 fall-through（判据 `^/t(/|$)`）
- [x] 8.2 单测：端点三分类——数据端点未知/停用 slug → 404；租户无关端点（外壳/session）未知 slug 不 404；**前缀存在 gate**：`/t/x/sync`/`/pull`/`/ping`/`/api/sf/*`/`/api/wechat/*` → 404（早于 handler）
- [x] 8.3 单测：显式活跃 slug → 解析；bare → `DEFAULT_TENANT` 且不读 tenants 表；`DEFAULT_TENANT` 未配置 → 缺省 `bh2ro`；`DEFAULT_TENANT=''`（空串）→ 经 `||` 缺省 `bh2ro`（钉死用 `||` 非 `??`，否则空串透传致全面静默空）
- [x] 8.4 单测（关键，D4）：带前缀路径 + 对带前缀路径的会话签名 → 通过；同一会话对**剥离前缀**路径的签名 → 401（断言剥离路径/双前缀未误入签名校验）
- [x] 8.5 单测：读取面 SQL 注入解析出的 `tenant_id`、不取前端参数；`/api/config` 含**嵌套** `tenant:{id,name}` 且不含签名密钥；bare config `tenant.name` 为 null（不读 tenants 表）
- [x] 8.6 单测/断言：微信订阅 OAuth `state` 为 `<URL派生slug>:<callsign>`（取自 `tenantBase()`、非 config.tenant.id）；callback 按**首个**冒号解析 `tenant:callsign` + 活跃校验、绑定写入该租户；无冒号兜底取 `DEFAULT_TENANT`；`:callsign`（空租户段）/`<未seed>:callsign` → callback 400 安全拒绝
- [x] 8.7 `node --test verify/*.test.js` 全绿（与 CI 一致，Node 版本无关 glob）

## 9. 验证与合并后

- [ ] 9.1 本地 worker 冒烟（`run_worker_smoke.sh` 或 `wrangler dev`）：`/t/bh2ro/api/config` 回显 `tenant.id=bh2ro`、`/t/bh2ro/api/callsigns/<已知呼号>` 经会话查询命中、`/t/nonexist/api/config` → 404、`/t//api/query` → 404、`/t` → 404、`/t/bh2ro`（无尾斜杠）→ 外壳、`/t/bh2ro/sync` → 404、bare `/api/config` 仍工作
- [ ] 9.1a **DEFAULT_TENANT 部署门（非重言式）**：冒烟**禁止**用「bare `/api/config` 的 `tenant.id == DEFAULT_TENANT`」（按构造恒真、假绿）；**必须**改打**显式** `GET /t/${DEFAULT_TENANT}/api/config` 断言 **200 非 404**（用 **shell 变量 `$DEFAULT_TENANT` 展开**、**禁止**写死 `/t/bh2ro/`——否则误配 `xyz` 时仍打 bh2ro 路径假绿；显式路径走 active-check，独立证 DEFAULT_TENANT 是 tenants 活跃行）；误配未 seed slug 时该断言失败、阻止部署
- [x] 9.2 对抗 review-loop 到 APPROVE（含提案本身正确性）
- [ ] 9.3【合并后】部署并线上冒烟：`/t/bh2ro/` 查询页可用、未知租户 404、现网 root `/` 与 bare `/api/*` 行为不变、`/sync` 等同步面不受影响
- [ ] 9.4 用户确认后 `openspec-cn archive route-query-by-tenant-path`（增量并入 `tenant-isolation`、`cloud-backend-api`、`wechat-push` 主规范 + 新增 `tenant-path-routing` 能力）
