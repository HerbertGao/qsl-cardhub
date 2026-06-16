## 为什么

阶段 4-A/4-C1 已落地行级 `tenant_id` 隔离、Key 解析租户、声明租户交叉校验，并把三张租户表（`tenants` / `tenant_credentials` / `tenant_routes`）与 `bh2ro` seed 建好。但**公共查询面**（Web/移动端按呼号查询）的 `tenant_id` 仍**硬编码为常量 `bh2ro`**（`src/worker/index.js` line 710，注释明写「host/path 路由属阶段 4-B」），`tenant-isolation` 规范 line 83/89 亦标注此为 4-B 待办。

要让一套查询站点服务多个租户，需要一种**显式、服务端可校验**的方式选择「查询哪个租户的数据」。沿用既定决策——**租户必须显式定义在路径中**（`/t/<slug>/`）——本期把查询面的租户由「硬编码常量」升级为「路径前缀解析 + 活跃租户校验」，并让前端配置与会话按租户路由。**写入面（同步）不在本期**：它已由 Key 解析租户 + `X-Tenant-Id` header 交叉校验（4-C1），与公开 URL 解耦，本期保持不变。

## 变更内容

- **默认租户配置化（消除全部硬编码 `bh2ro`）**：worker 现有三处 `'bh2ro'` 字面量——`index.js:710`（读取面查询）、`:287`（写入面 `env.API_KEY` 兜底默认）、`:896`（微信 `auth-callback` 兜底）——**全部**改为读 `env.DEFAULT_TENANT`（缺省 `'bh2ro'` 保现网兼容）。改后**除 `defaultTenant(env)` helper 的兜底默认一处外、零 `'bh2ro'` 残留**（含注释），同一份代码经「改 `DEFAULT_TENANT` 配置 + seed 各自租户」即可被他人部署。
- **路径前缀路由（仅公共查询面）**：worker 在分发前解析可选前缀 `/t/<slug>/`，`<slug>` 即 `tenant_id`；端点按三类处理——**数据端点**（`/api/config` 显式、`/api/query`、`/api/callsigns`）校验 `tenants.status='active'`、未知 slug 404；**租户无关端点**（SPA 外壳、`/api/session*`）接受前缀但不校验 slug；**非查询面端点**（`/sync`/`/pull`/`/ping`/`/api/sf/*`/`/api/wechat/*`）带前缀一律 404。
- **`/t` 命名空间保留**：命中 `^/t(/|$)`（`/t`、`/t/…`）但 slug 段不匹配段边界文法 `^/t/[a-z0-9-]{1,32}(/.*)?$`（`/t`、`/t/`、`/t//`、大写/非法/超长 slug）→ 404，**禁止** fall-through 当 bare。仅**不命中 `^/t(/|$)`** 才走 bare 默认租户。
- **读取面去硬编码**：`/api/query`、`/api/callsigns/:callsign` 的 `WHERE tenant_id = ?` 改用路由解析出的租户（替换 `:710` 常量）；归属仍**服务端注入**——路径 slug 经活跃校验后才用，**禁止**取自查询参数/请求体自报、**禁止**未校验直用作 SQL 目标。
- **每租户 `/api/config`**：返回嵌套 `tenant: { id, name }`（取自 `tenants` 表）+ 既有全局配置（微信、备案——同域同备案，保持全局）；bare 回显默认租户 `tenant:{ id: DEFAULT_TENANT, name }`。
- **会话签名不变量（双 path 变量）**：剥离前缀仅产独立局部变量供分发；`verifySessionSig` 实参恒为**原始 `url.pathname`（含前缀）**，剥离路径禁入签名校验。
- **多租户订阅绑定**：查询页微信订阅的 OAuth `state` 由仅 `callsign` 改为 `<路由租户>:<callsign>`（路由租户取自 `tenantBase()` 的 URL slug、非 config 往返，避免 config 未就绪时空租户静默失败），使绑定写入当前查询页租户而非默认租户——堵 `/t/tenant-b/` 订阅错绑默认租户、route-push 推错/丢失的跨租户缺口（callback 已支持按首冒号解析+校验 `tenant:callsign`，仅前端 `App.vue:89` state 需改）。
- **always-PoW 不变量**：每个租户路径下查询**必须**走既有会话 + PoW 闸门，**无**按租户关闭开关；限流/会话绑定仍以客户端 IP 为键（反爬维度，与租户正交）。
- **前端单点前缀前置**：`src/client/` 按 `location.pathname` 推导租户前缀，**只**在 `fetch`/`requestQuery` 入参处前置一次；`signQuery` 不改（其 `url.pathname` 已继承入参前缀），**禁止**二次拼接致双前缀。SPA 外壳与静态资源前缀无关（资源绝对 `/assets/`、`/favicon.svg`）。
- **无数据库迁移**：三张租户表 + 默认租户 seed 已由 0001 上线，本期纯 worker 路由 + 前端 + 规范，**不需**任何 D1 迁移。新部署的初始租户 seed 属部署期 DB 步骤（其 slug 须 == `DEFAULT_TENANT`）。

## 功能 (Capabilities)

### 新增功能
- `tenant-path-routing`: 公共查询面按 `/t/<slug>/` 路径前缀路由——端点三分类（数据端点校验活跃租户/租户无关端点豁免/非查询面 404）、`/t` 命名空间保留、bare 回退 `DEFAULT_TENANT`、每租户 `/api/config`（嵌套 `tenant`）、会话签名双 path 变量不变量、always-PoW、SPA 前缀无关 + 前端单点前置。

### 修改功能
- `tenant-isolation`: ①「租户模型与默认租户」——默认租户 slug 由硬编码 `default`/`bh2ro` 改为 `env.DEFAULT_TENANT` 配置指定（本部署 `bh2ro`），消除规范与生产 seed 的 `default`↔`bh2ro` 漂移，并加部署契约「seed slug == DEFAULT_TENANT」；②「写入按 Key 解析租户」的 env.API_KEY 兜底租户由 `default` 改为 `DEFAULT_TENANT`（消除 `:287` 硬编码）；③「读取按服务端注入的租户过滤」——读取面 `tenant_id` 由「恒为常量 `bh2ro`」改为「`tenant-path-routing` 解析的活跃 slug 或 `DEFAULT_TENANT`」，读写口径区分不变。
- `cloud-backend-api`: 「云端接收同步数据接口」的 `/sync` 兜底租户描述配置化——`/sync` DB.batch 场景（兜底租户措辞）+「同步按租户全量替换」场景（line 41：本期→写入面 reframe + 删 stale `default` 占位注 + 加 `tenant-path-routing` 交叉引用），均由硬写「创始租户 `bh2ro`」改为「`env.DEFAULT_TENANT` 指定的默认租户（本部署 `bh2ro`）」，消除漂移（其余 `/ping`/client_id 形态/CAS/原子性/app_settings 场景逐字保留）。
- `cloud-backend-api`/`wechat-push` 之外补 `tenant-isolation` ① 的 `tenants` 模型描述追加一句「`tenants.status` 无 CHECK 枚举」的已知失败模式声明（与 design 风险一致、使其归档后存续，本期不改 schema）。
- `wechat-push`: 「按呼号查询结果页与订阅绑定流程」的「授权 state 向前兼容携带并校验租户」场景——无冒号 `state` 的兜底 `tenant_id` 由硬写「创始租户常量 `bh2ro`」改为「`env.DEFAULT_TENANT` 指定的默认租户」，消除与 896 配置化的漂移；并把「前端改发 `tenant:callsign` 属阶段 4-B」更新为「本阶段 `/t/<slug>/` 页发 `tenant:callsign`、bare 页发纯 callsign 走兜底」（其余 3 场景逐字保留）。

> `/api/config` 加租户身份字段（加性、不改既有字段）、会话签名 path 含 `/t/` 前缀（既有「签 `url.pathname`」的特例）均为**新关注点**，作为 `tenant-path-routing` 的 ADDED 场景纳入，**不** MODIFY `query-antibot-session`（`cloud-backend-api` 的 MODIFY 仅因兜底租户配置化、非因 /api/config 形状）。

## 影响

- 代码：`web_query_service/src/worker/index.js`（前缀解析 + 端点三分类分发 + 三处 `'bh2ro'` → `DEFAULT_TENANT` + 每租户 config + 双 path 变量签名）；`src/client/`（`App.vue` 的 API 调用入参单点前置 base + 微信 OAuth `state` 带路由租户、`utils/session.ts` 同）。
- 配置：新增 `env.DEFAULT_TENANT`（缺省 `bh2ro`，非密钥）；`wrangler.toml` 的 `[vars]` 与 `wrangler.toml.example` 增补——即「改配置文件即可换默认租户」的落点。
- 数据库：**无迁移**（表与 seed 已就位）。新增租户走线下签发（属 4-C4），非本期。
- 部署：CDN 路径 `/t/*` 需与现有 `/api/*`、`/assets/*` 同源回源（同一 worker、同一域），无新增域名。
- 兼容：现网 bare 入口（root `/`、`/api/*`）行为不变；桌面同步面（`/sync`/`/pull`/`/ping`）不受影响。
