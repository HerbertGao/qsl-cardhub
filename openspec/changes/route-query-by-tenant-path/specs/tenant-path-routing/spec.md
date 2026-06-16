## ADDED Requirements

### 需求：公共查询面按 `/t/<slug>/` 路径前缀路由租户

公共查询面**必须**支持可选路径前缀 `/t/<slug>/` 显式选择租户，`<slug>` 即 `tenant_id`（文法 `[a-z0-9-]{1,32}`，与 `tenants.tenant_id` 的 CHECK 一致）。worker **必须**在路由分发前解析该前缀，剥离前缀后按原端点分发。前缀端点按是否依赖租户数据分**三类**处理：

- **数据端点**（`/api/config` 的显式前缀形态、`/api/query`、`/api/callsigns/:callsign`）：**必须**校验 `<slug>` 命中 `tenants` 表 `status='active'`，未命中 → **404**；命中则以 `tenant_id=<slug>` 作服务端租户上下文。
- **租户无关端点**（SPA 外壳、`/api/session/challenge`、`/api/session`）：**接受**任意文法合法前缀、剥离后分发，**不**校验 slug 活跃性（这些端点不读不写租户数据：外壳是静态壳、会话是 IP/PoW 凭证跨租户可复用），**不**因未知 slug 而 404，且**不**为此读 `tenants` 表（省 DB 读）。
- **非查询面端点**（`/sync`、`/pull`、`/ping`、`/api/sf/*`、`/api/wechat/*`）：带 `/t/` 前缀一律 **404**（其租户绝不取自路径——写入面由 Key 解析、推送由匹配订单派生）。

#### 场景：数据端点显式前缀解析并校验活跃租户

- **当** 请求路径形如 `/t/<slug>/<数据端点>` 且 `<slug>` 命中 `tenants` 表 `status='active'` 的行
- **那么** worker **必须**以 `tenant_id=<slug>` 作为本请求查询面的服务端租户上下文，并剥离 `/t/<slug>` 前缀后按 `<数据端点>` 分发
- **并且** 该 `tenant_id` 经活跃校验后方可使用，**禁止**未校验直接用作 SQL 过滤目标

#### 场景：`/t` 命名空间保留——非法前缀一律 404

- **当** 请求路径为 `/t`、`/t/`，或以 `/t/` 开头但 slug 段**不**匹配段边界文法 `^/t/[a-z0-9-]{1,32}(/.*)?$`（如 `/t//...`、含大写/非法字符的 slug、超 32 字符的 slug）
- **那么** **必须**返回 404，**禁止** fall-through 当作无前缀 bare 路径处理（避免 `/t/<非法>/api/query` 被误当 bare 而服务默认租户数据，或 `/t`/`/t/` 落入 SPA 外壳）
- **并且** 「是否属 `/t` 命名空间」的判据**必须**统一为正则 `^/t(/|$)`（`/t`、`/t/...` 命中、`/tfoo` 不命中）；命中命名空间但 slug 段不合文法 → 404，仅**不命中** `^/t(/|$)` 的路径才按 bare（默认租户）处理

#### 场景：数据端点未知或停用 slug 返回 404

- **当** 请求形如 `/t/<slug>/<数据端点>` 但 `<slug>` 文法合法却在 `tenants` 表不存在或 `status!='active'`
- **那么** 数据端点**必须**返回 404
- **并且** **禁止**回退到默认租户、**禁止**静默服务其他租户的数据

#### 场景：租户无关端点接受前缀但不校验 slug

- **当** 请求形如 `/t/<slug>/`（外壳）或 `/t/<slug>/api/session/challenge`、`/t/<slug>/api/session`，`<slug>` 文法合法
- **那么** **必须**剥离前缀后正常服务（外壳返回 `index.html`、会话端点走既有 PoW/会话流程），**不**校验 `<slug>` 是否活跃、**不**为此读 `tenants` 表、**不**因未知 slug 而 404
- **并且** 非法/停用 `<slug>` 的错误态由数据端点 `/t/<slug>/api/config` 的 404 驱动前端展示（外壳与会话不承担租户存在性判定）

#### 场景：bare 路径解析为默认租户且热路径零新增 DB 读

- **当** 请求**不命中 `^/t(/|$)`**（含现网 root `/`、`/api/query`、`/api/config` 等；即非 `/t` 命名空间路径）
- **那么** worker **必须**以 `tenant_id = env.DEFAULT_TENANT`（未配置时缺省 `bh2ro`）作为服务端租户上下文
- **并且** bare 路径**禁止**为解析默认租户而读 `tenants` 表（运营配置可信、保持现网热路径零新增 DB 读）；仅**显式**数据端点前缀才触发一次活跃校验查询

#### 场景：非查询面端点带前缀一律 404

- **当** 请求路径形如 `/t/<slug>/<非查询面端点>`（如 `/t/x/sync`、`/t/x/pull`、`/t/x/ping`、`/t/x/api/sf/route-push`、`/t/x/api/wechat/auth-callback`）
- **那么** **必须**返回 404
- **并且** 因分发比较改读剥离后的 `routePath`，**必须**在分发**前置**一道守卫——「前缀存在 且 `routePath` ∈ 非查询面集合 → 立即 404」，**早于** `/sync`/`/pull`/`/ping`/`/api/sf/*`/`/api/wechat/*` 各 handler（否则 `/t/x/sync` 的 `routePath==='/sync'` 会落入 sync handler）；等价地这些 handler 只在**无前缀**时可达
- **并且** 同步面与推送端点的租户**禁止**取自路径前缀（写入面租户恒由 Key 解析、推送由匹配订单派生，见 `tenant-isolation`）

### 需求：每租户前端配置端点

`/api/config`（含 `/t/<slug>/api/config`）**必须**在既有全局配置基础上回显**路由解析出的租户身份**，供前端确认当前查询的租户。全局配置项（功能开关、`wechat_appid`、备案 `filing`）因同域同备案、单一公众号，本期**仍**为全局共享、按 worker env 下发。

#### 场景：配置端点回显路由解析的租户身份（嵌套对象形状）

- **当** 客户端请求 `GET /t/<slug>/api/config`（`<slug>` 为活跃租户）
- **那么** 响应**必须**包含嵌套租户身份对象 `tenant: { id, name }`（`id` 为 `tenant_id`、`name` 为展示名，均取自 `tenants` 表），并保留既有全局字段（`features`、`wechat_appid`、`filing`）
- **并且** 该响应**禁止**下发任何查询签名密钥（沿用 `cloud-backend-api` 的配置卫生：`CLIENT_SIGN_KEY` 已退役）
- **并且** 显式前缀的 `/api/config` 属数据端点，未知/停用 `<slug>` **必须** 404（驱动前端「租户不存在」错误态）

#### 场景：bare 配置端点回显默认租户

- **当** 客户端请求 `GET /api/config`（无前缀）
- **那么** 响应**必须**回显默认租户身份 `tenant: { id: env.DEFAULT_TENANT, name }`，既有全局字段保持与本期前一致（加性变更，不改既有字段语义）
- **并且** bare `/api/config` 的 `name` 恒为 `null`/省略——**不**为取展示名而读 `tenants` 表（与「bare 路径热路径零新增 DB 读」一致）；显式 `/t/<slug>/api/config` 才读 `tenants.name`。二者的 `tenant.id` 在默认租户下相等，`name` 可不同（bare 为 null、显式为表中值），属有意差异

### 需求：订阅绑定按路由租户传递

查询页的微信订阅**必须**把当前路由租户随 OAuth `state` 传递，使订阅绑定（`callsign_openid_bindings`）写入**当前查询页的租户**而非硬编码默认租户；否则 `/t/<tenant-b>/` 页面的订阅会错绑到默认租户，致 route-push 在 tenant-b 名下反查不到绑定、推送丢失或推错租户（这是 4-B 引入多租户查询面后必须同步堵的跨租户缺口）。

#### 场景：订阅 OAuth state 携带路由租户

- **当** 用户在 `/t/<slug>/` 查询页订阅某呼号
- **那么** 前端构造微信 OAuth 授权 URL 时 `state` **必须**为 `<当前路由租户>:<callsign>`，其中路由租户取自 **`tenantBase()` 派生的 URL slug**（页面租户已权威地在自身 URL 中、本地可得），**禁止**仅传 `callsign`、**禁止**改取 `/api/config` 的 `tenant.id`（config 加载失败/未就绪会令 `state` 退化为 `:callsign` 空租户 → callback 400 → 订阅静默失败）
- **并且** `auth-callback`（属非查询面端点、保持 bare `/api/wechat/auth-callback`，**不**走 `/t/` 前缀）**必须**解析 `state` 的 `tenant:callsign`（按**首个**冒号拆分）、校验该 `tenant` 为 `tenants` 表活跃租户（现状已落地），并把绑定写入该 `tenant_id`
- **并且** `redirect_uri` 保持 bare（微信面租户经 `state` 传递、不经路径前缀；`/t/` 对 `/api/wechat/*` 一律 404，见前述）
- **并且** 微信订阅是查询面**唯一**「租户经 `state` 携带、不在路径」的通道（有意例外）；后续多租户化其它面时**禁止**假设「查询面租户必在路径」

#### 场景：默认租户页订阅回退取 DEFAULT_TENANT

- **当** bare 查询页（`tenantBase()` 为空）订阅、或 `state` 无冒号
- **那么** 前端 `state` 为无冒号 `callsign`；callback 无冒号 `state` 的兜底租户**必须**取 `env.DEFAULT_TENANT`（**禁止**硬编码 `'bh2ro'` 字面量，与本变更默认租户配置化一致），绑定写入该默认租户
- **并且** `state` 为 `:callsign`（**含冒号但租户段空**）或 `<未seed>:callsign` 时，callback 活跃校验失败 → **400「无效租户」**（安全拒绝，不落垃圾绑定）；前端按本场景的 URL 派生不会产生空租户段

### 需求：会话签名与防爬维度对前缀的处理

引入 `/t/<slug>/` 前缀**禁止**破坏既有会话动态签名与 PoW 闸门。前缀剥离**仅**用于路由分发与租户解析，**禁止**就地改写原始请求 URL/`url.pathname`。

#### 场景：剥离前缀仅供分发，签名校验用原始完整 pathname

- **当** 客户端在 `/t/<slug>/api/callsigns/:callsign` 发起带会话签名的查询
- **那么** 客户端**必须**对其实际请求的**含前缀** pathname 签名；服务端的 `verifySessionSig` 实参**必须**为**原始 `url.pathname`（含同一前缀）**
- **并且** 前缀剥离**必须**产出**独立的局部变量**（如 `routePath`）仅供端点分发判定使用，**禁止**把剥离后的路径传入 `verifySessionSig`、**禁止**就地改写 `url`/`url.pathname`——否则前端签带前缀路径、服务端按无前缀校验将致所有前缀查询 401（须有断言：带前缀路径配对带前缀签名通过、配对剥离路径签名失败）

#### 场景：always-PoW —— 任意前缀下查询都走会话+PoW 闸门

- **当** 任意租户前缀（含默认租户 bare 路径）下发起按呼号查询
- **那么** **必须**经既有「限流 → 会话校验（token+签名+配额）→ 查询」管线（见 `query-antibot-session`）
- **并且** **禁止**提供任何按租户关闭会话/PoW 的开关

#### 场景：限流与会话绑定维度与租户正交

- **当** 计算限流桶键与会话绑定键
- **那么** **必须**仍以客户端真实 IP 的归一绑定键（`clientBindingKey(getClientIP(...))`）为键
- **并且** 租户前缀**禁止**进入限流/会话绑定键（反爬针对客户端、不按租户切分，跨租户共用同一 IP 维度）

### 需求：SPA 外壳前缀无关与前端单点前缀前置

`/t/<slug>/` 下的 SPA 外壳与静态资源**必须**前缀无关：外壳服务同一 `index.html`，静态资源经绝对路径 `/assets/` 取用；前端**必须**按当前 URL 推导租户前缀，并在**调用入参处单点前置**到所有 API 路径，**禁止**在签名函数内部二次拼接前缀。

#### 场景：外壳服务不为外壳做 slug 校验

- **当** 请求 `/t/<slug>/`（或其下无扩展名路径）且 `<slug>` 文法合法
- **那么** **必须**返回 SPA 外壳 `index.html`，**不**为外壳服务读 `tenants` 表（省 DB 读）
- **并且** SPA fallback 分支的无扩展名守卫（现状 `!path.includes('.')`）**必须**改读 `routePath`：`/t/<slug>/`→`routePath='/'` 无点→回退 `index.html`；`/t/<slug>/x.js`→`routePath='/x.js'` 有点→**不**回退、返 404（前缀路径非真实资产，`env.ASSETS.fetch` 对其必 404）
- **并且** 静态资源引用**必须**保持绝对 `/assets/`、`/favicon.svg`（Vite base `/`，前缀无关），使外壳在任意前缀下均能加载（浏览器不会在前缀下请求资产）

#### 场景：前端按 URL 推导前缀并在调用入参处单点前置

- **当** 前端发起 `/api/config`、`/api/session/challenge`、`/api/session`、`/api/callsigns/:callsign` 调用
- **那么** 前端**必须**经 helper（读 `location.pathname`，以**段边界**正则 `^/t/[a-z0-9-]{1,32}(?:/|$)` 匹配得前缀、无匹配则空——须与服务端 parser 文法同构，**禁止**用无边界的 `^/t/[a-z0-9-]{1,32}` 对超长 slug 截断出错误前缀）在**调用入参处**前置该前缀**一次**
- **并且** 签名函数（`signQuery`/`sign.ts`）**禁止**再读 `location` 二次拼接前缀——其 `url.pathname` 已从带前缀的入参派生，二次拼接将致双前缀（`/t/x/t/x/...`）签名错配 401
