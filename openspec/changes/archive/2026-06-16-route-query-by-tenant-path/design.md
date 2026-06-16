## 上下文

公共查询面（`/api/query`、`/api/callsigns/:callsign`）当前把 `tenant_id` 硬编码为常量 `'bh2ro'`（`src/worker/index.js:710`），`tenant-isolation` 规范 line 83/89 标注其「按路由解析属阶段 4-B」。租户三表（`tenants`/`tenant_credentials`/`tenant_routes`）与 `bh2ro` 活跃 seed 已由迁移 0001 上线（`tenants.tenant_id` 即人类可读 slug，CHECK `length 1-32 且 [a-z0-9-]`）。前端 `src/client/` 现以**绝对路径** `/api/config`、`/api/session/challenge`、`/api/session`、`/api/callsigns/:cs` 调用（始终落原点根）。会话签名 `verifySessionSig({ path: url.pathname, ... })`（`index.js:216`）对**原始 `url.pathname`** 签名/校验，而路由分发用另一变量 `path = url.pathname.replace(/\/$/,'')`（`index.js:319`）。

## 目标 / 非目标

**目标：**
- 公共查询面支持 `/t/<slug>/` 路径前缀显式选择租户；`<slug>`=`tenant_id`，校验 `tenants.status='active'`。
- bare 路径（无前缀，含现网 root 与 `/api/*`）= `env.DEFAULT_TENANT`（默认 `bh2ro`），现网零中断、且**不引入新的每请求 DB 读**（热路径与今日一致）。
- 读取面去硬编码：查询 SQL 的 `WHERE tenant_id=?` 用路由解析值，归属仍服务端注入。
- 每租户 `/api/config`（回显租户身份 + 全局配置）；前端按 URL 推导前缀。
- always-PoW 不变量：任意租户前缀下查询都走既有会话+PoW 闸门，无按租户旁路。

**非目标：**
- host 路由 / 自定义域名（`tenant_routes` 解析）——留后续阶段。
- 同步面（`/sync`/`/pull`/`/ping`）路径路由——维持 Key 解析 + `X-Tenant-Id` header（4-C1），本期不动。
- 新增租户的线下签发工具与文档（属 4-C4）；桌面端租户身份（4-C2/4-C3）。
- 每租户独立限流/配额——限流与会话绑定仍以客户端 IP 为键（反爬维度，租户正交）。
- 数据库迁移——表与 seed 已就位，本期零迁移。

## 决策

### D1. 前缀文法与解析位置
在 `fetch` 顶部、路由分发前解析 `url.pathname`：段边界正则 `^/t/([a-z0-9-]{1,32})(/.*)?$`。命中 → 候选 `slug` + 余下 `rest`（缺省 `/`，即 `routePath`）；命中 `^/t(/|$)` 但 slug 段不合文法 → `/t` 命名空间内非法前缀，**404**（见 D2，**禁止** fall-through）；**不命中 `^/t(/|$)`** → 无前缀 bare。端点按 D2 三分类（数据端点校验 slug / 租户无关端点豁免 / 非查询面端点 404）处理。

### D2. 端点三分类与 slug 校验（热路径零 DB 读 + `/t` 命名空间保留）
前缀端点按是否依赖租户数据分三类：
- **数据端点**（`/api/config` 显式前缀形态、`/api/query`、`/api/callsigns/:cs`）：`SELECT 1 FROM tenants WHERE tenant_id=? AND status='active'`（PK 命中，廉价）。文法合法但未命中/停用 → **404**（不回退默认、不静默服务他租户数据）。
- **租户无关端点**（SPA 外壳、`/api/session/challenge`、`/api/session`）：接受任意文法合法前缀、剥离分发，**不**校验 slug 活跃性、**不**读 tenants 表、**不**因未知 slug 而 404（外壳是静态壳、会话是 IP/PoW 凭证跨租户可复用）。解决「外壳/会话被未知 slug 404 卡死」与「外壳又不校验」的张力——错误态只由数据端点 `/api/config` 404 驱动。
- **非查询面端点**（`/sync`/`/pull`/`/ping`/`/api/sf/*`/`/api/wechat/*`）：带前缀一律 404。**前缀存在 gate（关键）**：因 task 把分发比较改读 `routePath`，若不加守卫，`/t/x/sync` 的 `routePath==='/sync'` 会落入 sync handler。故**必须**在分发前置一道守卫——「前缀存在 且 `routePath` ∈ 非查询面集合 → 立即 404」，**早于**任何 `/sync`/`/pull`/`/ping`/`/api/sf/*`/`/api/wechat/*` handler。等价地：这些 handler 只在**无前缀**时可达。
- **`/t` 命名空间保留**：判据统一为正则 `^/t(/|$)`——路径为 `/t`、`/t/`、或以 `/t/` 开头但 slug 段不匹配 `^/t/[a-z0-9-]{1,32}(/.*)?$`（`/t//`、大写/非法字符、超 32 长）→ **404**，**禁止** fall-through 当 bare（否则 `/t/<非法>/api/query` 被误当默认租户、`/t`/`/t/` 落外壳）。仅**不匹配 `^/t(/|$)`** 的路径才走 bare。
- 取舍：显式数据端点每请求 +1 PK 查；bare/现网/外壳/会话路径 0 新增查。

### D3. 默认租户配置化（消除全部硬编码 bh2ro）+ 读写口径区分（红线）
- **配置化**：worker 现有**三处** `'bh2ro'` 字面量——① `index.js:710`（读取面查询）；② `index.js:287`（`resolveTenant` 的 `env.API_KEY` 写入面兜底默认）；③ `index.js:896`（微信 `auth-callback` 租户兜底）——**全部**改为读 `env.DEFAULT_TENANT`（缺省 `'bh2ro'` 保现网兼容）。改后 worker 代码**除 `defaultTenant(env)` helper 的兜底默认一处外、零 `'bh2ro'` 残留**（含注释；grep 验收恰好 1 处），同一份代码经「改 `DEFAULT_TENANT` 配置 + seed 各自租户」即可被他人部署。三处语义统一为「本部署的默认租户身份」，非外溢：读取面 bare 默认、写入面 legacy Key 归属、微信无 state 回退，本就都指「这个部署的默认租户」。
- **读写口径区分（红线不变）**：读取面 `tenant_id` = 经活跃校验后的**路径 slug**（或 `DEFAULT_TENANT`）；写入面 `tenant_id` = **Key 解析值**（密钥，绝不取路径）。`DEFAULT_TENANT` 只是各面「无显式来源时」的默认，**不**改变「路径 slug 绝不当写入目标」这条红线。
- **部署契约**：seed 的默认租户 slug **必须** == `env.DEFAULT_TENANT`（不一致 → bare 面以未 seed 租户查询、静默空结果，见风险）。

### D4. 会话签名对前缀的处理（关键不变量）+ 双 path 变量
剥离前缀**仅**产出一个**独立局部变量**（如 `routePath`）供端点分发判定使用；现状全套分发比较（`path === '/api/config'`、`/api/query`、`startsWith('/api/callsigns/')` 等）**必须**改读 `routePath`（否则 `/t/<slug>/api/query` 不匹配 `path === '/api/query'` 会误落入 SPA 外壳而非数据端点校验）——**含 SPA fallback 分支的无扩展名守卫** `!routePath.includes('.')`（现状 `index.js:966` 读全局 `path`，须改 `routePath`）：`/t/<slug>/`→`routePath='/'` 无点→回退 `index.html`；`/t/<slug>/x.js`→`routePath='/x.js'` 有点→不回退、返 404（前缀非真实资产路径，`env.ASSETS.fetch` 对其必 404；真实资产经绝对 `/assets/` 取用、不带前缀）。原始 `url`/`url.pathname` 对象**不可就地改写**。`verifySessionSig` 实参**必须**保持**原始 `url.pathname`（含 `/t/<slug>/` 前缀）**，**禁止**把 `routePath`（剥离后）喂给签名校验。前端签名所用 path 是其实际请求的带前缀 pathname → 二者含同一前缀、签名一致。**反模式**：若实现就地把全局 `path`（index.js:319）改成剥离值并贯穿、或把 `routePath` 传入 `verifySessionSig` → 前端签带前缀、服务端按无前缀校验 → 全部前缀查询 401。须有断言：带前缀路径配对带前缀签名通过、配对剥离路径签名失败。

### D5. 每租户 `/api/config`（嵌套形状）
解析租户后，显式前缀时 `SELECT name FROM tenants WHERE tenant_id=?` 取展示名；返回 `{ tenant: { id, name }, features, wechat_appid, filing }`（**嵌套** `tenant` 对象，proposal/spec/tasks/测试统一此形状）。`features`/`wechat_appid`/`filing` 仍取自 worker env（同域同备案、单一公众号 → 全局共享，本期不做每租户化）。bare `/api/config` 回显 `tenant:{ id: DEFAULT_TENANT, name }`。

### D6. SPA 外壳前缀无关 + 前端单点前缀前置
- 外壳：`/t/<slug>/`（及其下无扩展名路径）→ 服务 `index.html`（同一 SPA 外壳），**不**为外壳做 slug 校验（省 DB 读）；非法 slug 由随后 `/t/<slug>/api/config` 返回 404 驱动前端显示「租户不存在」。静态资源用绝对 `/assets/`、`/favicon.svg`（Vite base `/`），前缀无关。
- 前端**单点前置**：helper `tenantBase()` 读 `location.pathname`，以**段边界**正则 `^/t/[a-z0-9-]{1,32}(?:/|$)` 匹配 → 返回该前缀（无尾斜杠）否则 `''`（**须与服务端 parser 文法同构**，禁用无边界正则以免超长 slug 截断出错误前缀）；前缀**只**在调用 `fetch`/`requestQuery` 的**入参处前置一次**。`signQuery`/`sign.ts` **不改**——其 `new URL(path,origin).pathname` 已从带前缀入参派生，**禁止**在 `signQuery` 内再读 `location` 二次拼前缀（否则双前缀 `/t/x/t/x/...` 签名错配 401）。

### D9. 多租户订阅绑定按路由租户（堵跨租户泄漏）
4-B 引入多租户查询面后，查询页的微信订阅**必须**把当前路由租户写入绑定，否则 `/t/<tenant-b>/` 页订阅会错绑默认租户、route-push 在 tenant-b 名下反查不到 → 推送丢失/推错租户。现状 `App.vue:89` 的 OAuth `state = encodeURIComponent(callsign)`（仅呼号），callback（`index.js:890-914`）已支持解析 `tenant:callsign` + 校验活跃租户。
- **state 租户源 = `tenantBase()` 派生的 URL slug**（**非** `/api/config` 的 `tenant.id`）：页面租户**已权威地在自身 URL 里**，本地可得、不依赖 config 往返；用 config.tenant.id 则 config 加载失败/未就绪时 `state` 退化为 `:callsign`（空租户）→ callback 活跃校验 400 → 订阅静默失败（正是要防的「推送丢失」）。修法：`/t/<slug>/` 页 `state = encodeURIComponent(`${slug}:${callsign}`)`（slug 来自 `tenantBase()`）；bare 页 `tenantBase()=''` → `state = encodeURIComponent(callsign)`（无冒号）→ callback 兜底取 `DEFAULT_TENANT`。到达订阅即说明 `/api/config` 已成功（非法 slug 早已 404 挡在错误态），故 URL slug 必为活跃租户。
- `redirect_uri` 保持 bare（微信只认固定回调域、且 `/t/api/wechat/*` → 404）；callback 无冒号兜底取 `DEFAULT_TENANT`（随 D3 配置化，非硬编码）。
- **不变量（防后续面误用）**：微信订阅是查询面**唯一**「租户不在路径、改由 `state` 携带」的通道；后续把其它面（route-push/site-verification 等）多租户化时，**勿**假设「查询面租户必在路径」——订阅是有意例外。属 4-B 真范围（多租户查询页的订阅动作必须租户自洽）。

### D7. 限流/会话绑定维度不变
`clientBindingKey(getClientIP(...))` 仍为反爬绑定键，跨租户共用同一 IP 维度的握手桶/查询桶/会话。租户前缀不进入限流键（反爬针对客户端，不按租户切分）。

## 风险 / 权衡

- **签名前缀错配（D4）**：最易踩——剥离路径误入签名校验或就地改写 `path` 致前缀查询全 401。缓解：D4 双 path 变量不变量 + 双向断言测试（带前缀签名过、剥离签名挂）。
- **前端双前缀（D6）**：第二易踩——入参前置 + signQuery 内再拼 = `/t/x/t/x/...` 签名错配 401。缓解：D6 单点前置 + signQuery 不改 + 断言。
- **`DEFAULT_TENANT` 误配（D3 部署契约）**：bare 面为省 DB 读不校验 `DEFAULT_TENANT` 活跃性；若运维把它配成未 seed/非 active 的 slug（拼错、指向已删租户），三面失败模式**不同**：① **读面** bare `/api/query` 以不存在 `tenant_id` 查 → **静默空结果**（非报错），`/api/config` 回显坏 id；② **写面** `env.API_KEY` 兜底(287) 解析为幽灵租户、**无 active-check**（表无 FK）→ `/sync` **静默写进幽灵 tenant_id**（数据落在不存在的租户名下）；③ **微信面** callback(896) 兜底经 line 910-914 活跃校验 → **400 安全拒绝**。①②是静默错、③安全。缓解（**关键，防验证门假绿**）：部署冒烟**禁止**用「bare `/api/config` 的 `tenant.id == DEFAULT_TENANT`」断言——它按构造恒真（config 本就回显 DEFAULT_TENANT）= 重言式假绿；**必须**改为打**显式** `GET /t/<DEFAULT_TENANT>/api/config`（显式路径走 active-check）断言 **200 非 404**，即独立证「DEFAULT_TENANT 是 tenants 表中的活跃行」（tasks 9.1a）。属运维契约 + 部署门，非运行时护栏（取舍：换热路径零 DB 读）。
- **`tenants.status` 无 CHECK 枚举（现状 schema）**：`tenant_credentials.status` 有 `CHECK(IN('active','revoked'))`，但 `tenants.status` 无（migration 0001:124）。D2 活跃校验是 `status='active'` 精确等值，4-C4 线下签发误写 `'Active'`/`' active'` 会令该租户整站静默 404。本期不改 schema（无迁移目标），留 4-C4 自律或补 CHECK；此处声明失败模式。
- **读取面 slug 客户端可控（D3）**：设计即如此（公开匿名读，URL 选活跃租户）。隔离保证：只能命中 active 租户、且只读该租户（`WHERE tenant_id=校验后slug`）。红线是「校验后才用、绝不当写入目标」。
- **显式数据端点每请求 +1 DB 读（D2）**：可接受（PK 命中）；bare/现网/外壳/会话路径零新增读。缓存留后续。
- **CDN 回源**：`/t/*` 须与 `/api/*`、`/assets/*` 同源回同一 worker（同域、无新增域名）。若 CDN 有路径级缓存/路由规则，需为 `/t/*` 配回源（部署清单覆盖）。`/t/<slug>/api/*` 属动态、不可缓存；`/t/<slug>/`（外壳 HTML）缓存策略同现有 SPA 外壳。
- **未知 slug 404 而非默认**：避免拼错租户静默服务他租户数据；代价是错误 URL 不再「优雅降级」到默认站（符合显式租户语义）。
