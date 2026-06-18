# 云端查询服务部署说明（Cloudflare Workers + D1）

本目录下的 **`web_query_service`** 为 QSL CardHub 云端同步与按呼号查询的可部署实现，使用 Wrangler CLI 部署到 Cloudflare Workers + D1。

## 快速开始

1. 进入服务目录并安装依赖：
   ```bash
   cd web_query_service
   npm install
   ```

2. 创建 D1 数据库并写入 `wrangler.toml`：
   ```bash
   npx wrangler d1 create qsl-sync
   ```
   将输出的 `database_id` 填入 `wrangler.toml` 中 `[[d1_databases]].database_id`。

3. 执行 D1 迁移（本地与远程）：
   ```bash
   npx wrangler d1 execute qsl-sync --local --file=./schema.sql
   npx wrangler d1 execute qsl-sync --remote --file=./schema.sql
   ```

4. 设默认租户并签发其写凭据（**新部署推荐路径**，详见下文「新增租户与自托管」节）：
   ```bash
   # 4a. 在 wrangler.toml 的 [vars] 写 DEFAULT_TENANT="default"（须与下面签发的 slug 一致；
   #     缺省时 worker 内置兜底是 bh2ro，见「默认 slug default 必须与 DEFAULT_TENANT 一致」节）
   # 4b. 生成高熵 Key 并离线签发「default 租户 + 写凭据」，管道直送 wrangler（Key 不进 history）
   KEY="$(openssl rand -hex 32)"   # 记进密码管理器——它就是桌面端要填的 API Key
   printf %s "$KEY" | node scripts/mint-credential.mjs default --key-stdin \
     | npx wrangler d1 execute qsl-sync --remote --file=-
   ```
   > **不要只配 `API_KEY` 当鉴权**：`API_KEY` 只是过渡期兜底路径，且其 `/sync` 兜底计数依赖
   > `service_counters` 里的 `auth_fallback` 行——该行**仅由迁移 `0001` seed、`schema.sql` 不含**，
   > 故「只跑 `schema.sql` + 配 `API_KEY`」的新部署 `/ping` 能过但 **`/sync` 会 500**。新部署请按上面
   > 签发租户凭据走表驱动路径（不碰兜底计数）。

5. 部署：
   ```bash
   npm run deploy
   ```

6. 在桌面端「数据管理 > 云端同步」中配置：
   - API 地址：`https://<你的 Workers 域名>`
   - 租户代码：`default`（与步骤 4 签发的 slug 一致）
   - API Key：步骤 4 生成的 `KEY`（即被签发为该租户写凭据的那把）

## 详细说明

- **环境变量与密钥**、**顺丰路由推送 URL 配置**、**按呼号查询页与订阅收卡**、**D1 表结构** 等见：  
  **[web_query_service/README.md](../web_query_service/README.md)**。

- **云端 API 规范**（GET /ping、POST /sync 请求/响应格式）见：  
  [cloud-sync-api-spec.md](cloud-sync-api-spec.md)。

- **顺丰路由推送**：服务端提供两条路径：正式 `POST /api/sf/route-push`、沙箱 `POST /api/sf/route-push/sandbox`；沙箱触发的用户推送内容带「【沙箱】」标记。请求/响应格式详见 OpenSpec 规范 `openspec/specs/sf-route-push-receiver/spec.md`。

## 阶段 4-A：全局表租户化迁移（0002）

把全局表 `callsign_openid_bindings` 加 `tenant_id` 行级隔离（主键 `(tenant_id, callsign, openid)`），存量回填创始租户 `bh2ro`；`sf_route_log` 保持全局不变（顺丰 waybill 全局唯一、tenant 由匹配的 `sf_orders` join 派生）。与配对 Worker 同批部署：Worker 的 route-push 按匹配订单派生租户后 `WHERE tenant_id=? AND callsign=? COLLATE NOCASE` 反查 openid（杜绝同呼号跨租户推送），auth-callback 由授权 `state`（`tenant:callsign`，无冒号回退 `bh2ro`）解析并校验活跃租户后写绑定。

### 部署顺序（迁移与 Worker 配对、不可单退 Worker）

1. **备份（回滚点）**：`npx wrangler d1 execute qsl-sync --remote --command "..."` 前，先全量导出
   ```bash
   npx wrangler d1 export qsl-sync --remote --output ~/qsl-d1-backup-before-0002.sql
   ```
2. **执行迁移**（运维在自己终端跑，单一 SQL 文件、整体原子，文件内无 `BEGIN/COMMIT`）
   ```bash
   npx wrangler d1 execute qsl-sync --remote --file=./migrations/0002_global_table_tenant.sql
   ```
3. **部署配对 Worker**：`pnpm run deploy`，记录新版本号与回滚目标版本。

### 回滚

迁移后新表 `tenant_id NOT NULL`，**旧 Worker 的 `INSERT (callsign, openid, …)` 会撞 NOT NULL → 不可单独回退 Worker**。回滚 = 退 Worker 版本 **+** 还原表（建新空 D1 import 备份 dump，或逐表 DROP 后 import；dump 不自动 DROP 已存在表）。未上线第二个真实租户前，全部绑定/订单仍归 `bh2ro`，迁移前后线上行为等价（无回归）。

## 阶段 4-B：查询面按租户路径路由（route-query-by-tenant-path）

公共查询面（Web/移动端按呼号查询）支持 `/t/<slug>/` 路径前缀显式选择租户：`<slug>` 即 `tenant_id`、经 `tenants.status='active'` 校验；bare 入口（root `/`、不带前缀的 `/api/*`）回退 `env.DEFAULT_TENANT`（默认 `bh2ro`）。写入面（`/sync`/`/pull`/`/ping`）不受影响——仍按 Key 解析租户 + `X-Tenant-Id` header 交叉校验，与公开 URL 解耦。

### 无数据库迁移

**本阶段零 D1 迁移**。三张租户表（`tenants` / `tenant_credentials` / `tenant_routes`）与默认租户 seed 已由迁移 0001 上线，纯 Worker 路由 + 前端 + 规范变更，**无需**任何新迁移、`schema.sql` 不变。

**新部署的初始租户 seed**（首次起新部署、tenants 表尚无默认行时）属**部署期 DB 步骤**：须在执行 schema/迁移后插入一条 active 的默认租户行。其 `tenant_id`（slug）**必须** == `env.DEFAULT_TENANT`（本部署 `bh2ro`）。

- **部署契约（红线）**：seed slug ≠ `DEFAULT_TENANT` → bare 查询面以未 seed 的 `tenant_id` 查询 → **静默空结果**（非报错），运营难察觉。务必让二者一致。
- 现网（已有 0001 的 `bh2ro` seed）无需任何动作——`DEFAULT_TENANT` 缺省即 `bh2ro`、与既有 seed 一致。

### 配置

- `env.DEFAULT_TENANT`（**非密钥**）：在 `wrangler.toml` 的 `[vars]` 配置（示例见 `wrangler.toml.example`）。「改配置文件即可换默认租户」的落点；缺省回退 `bh2ro`。
- **`[assets]` 须含 `binding = "ASSETS"`（必填，否则 `/t/<slug>/` 404）**：Worker 经 `env.ASSETS` 为 SPA 客户端路由（含 `/t/<slug>/` 租户入口、bare 无扩展名深链）返回 `index.html` 内容并**保留浏览器 URL**（不重定向，否则丢前缀、前端 `tenantBase()` 读不到租户而错当默认租户）。缺 `binding` 则 `env.ASSETS` 未定义、Worker 的 SPA 回退失效、`/t/<slug>/` 直接 404。现有部署若 `wrangler.toml` 的 `[assets]` 只有 `directory` 没有 `binding`，**部署 4-B 前须补上**（示例见 `wrangler.toml.example`）。

### CDN 回源（无新增域名）

`/t/*` 与现有 `/api/*`、`/assets/*` **同源、回源至同一 Worker、同一域名**——不新增任何域名。若生产经阿里云 CDN（`qsl.herbert-dev.cn`）回源 Cloudflare 源站（`qsl.herbertgao.me`），须确保 `/t/*` 命中既有「回源到源站」规则（与 `/api/*`、`/assets/*` 一致），不要为 `/t/*` 配置独立路由/拦截规则。

缓存策略按路径分两类：

- **`/t/<slug>/api/*`（含 `/t/<slug>/api/config`、`/api/query`、`/api/callsigns/:cs`）= 动态、不可缓存**：与 bare `/api/*` 同口径（查询结果/租户配置随数据变化，且查询走会话 + PoW 闸门，缓存会破坏防爬与正确性）。CDN 侧确保 `/t/*/api/*` 不被缓存（与现有 `/api/*` 不缓存策略一致）。
- **`/t/<slug>/`（SPA 外壳 HTML）= 缓存策略同现有 SPA 外壳**：外壳是与 bare root `/` 相同的静态 `index.html`（前缀无关、不为外壳做 slug 校验），缓存口径沿用现网 root `/` 外壳的策略，无需为 `/t/<slug>/` 单设规则。
- 静态资源（`/assets/*`、`/favicon.svg`）经**绝对路径**取用、**前缀无关**，缓存策略不变。

### lockstep 部署与回滚

- 前端（`src/client`→`public`）按 `location.pathname` 推导租户前缀，由 Worker 同部署（一次 `pnpm run deploy`）。
- 现网 bare 入口（root `/`、`/api/*`）行为不变；未知/停用 slug 的 `/t/<slug>/api/*` 返 404（不优雅降级到默认站，符合显式租户语义）。
- 回滚：退 Worker 版本 + 还原前端构建；**无 D1 迁移可退**。

### 部署后冒烟（DEFAULT_TENANT 部署门，非重言式）

冒烟**禁止**用「bare `/api/config` 的 `tenant.id == DEFAULT_TENANT`」断言——它按构造恒真（config 本就回显 `DEFAULT_TENANT`）= 假绿。**必须**改打**显式**路径（用 shell 变量 `$DEFAULT_TENANT` 展开、**禁止**写死 `/t/bh2ro/`）：

```bash
DEFAULT_TENANT=bh2ro   # 与 wrangler.toml [vars] 一致
# 显式路径走 active-check，独立证「DEFAULT_TENANT 是 tenants 表中的活跃行」；
# 误配未 seed slug 时此处返 404、阻止部署
curl -sf -o /dev/null -w '%{http_code}\n' \
  "https://<你的域名>/t/${DEFAULT_TENANT}/api/config"   # 期望 200，非 404
```

其余冒烟：`/t/<未知>/api/config` → 404、`/t//api/query` → 404、`/t` → 404、`/t/<DEFAULT_TENANT>`（无尾斜杠）→ 外壳、`/t/<DEFAULT_TENANT>/sync` → 404、bare `/api/config` 仍工作。

## 新增租户与自托管

> 同步面 API 契约真源是 OpenSpec 主规范 `openspec/specs/cloud-backend-api/spec.md`；面向自托管者的实现指引见 [cloud-sync-api-spec.md](cloud-sync-api-spec.md)。

### 新增租户（用 `mint-credential.mjs` 签发写凭据）

注册纯线下签发、无公开自助端点。新增一个租户的写凭据靠**离线**脚本 `web_query_service/scripts/mint-credential.mjs`：它**不连任何数据库**、**不落明文 Key**，只把「租户 slug + 写凭据 Key」算成 `key_hash = sha256(trim(key))` 后输出**可直接执行的 SQL**（`INSERT OR IGNORE INTO tenants …` + `INSERT INTO tenant_credentials …`），由运维自行用 `wrangler` 执行。

**推荐流程**（Key 不进 shell history、不进任何文件）：

```bash
cd web_query_service
# 1) 生成高熵 Key（先记到密码管理器；它即桌面端要填的 API Key，签发后无法从云端反查）
KEY="$(openssl rand -hex 32)"

# 2) 用 --key-stdin 让 Key 不出现在命令行参数（不进 history），管道直送 wrangler 执行
printf %s "$KEY" | node scripts/mint-credential.mjs <slug> --key-stdin \
  | npx wrangler d1 execute qsl-sync --remote --file=-
```

- **Key 的保密**：脚本输出**只含 `key_hash`、绝不含明文 Key**；Key 也不写进任何文件 / 迁移 SQL / history。务必在生成时就把它交给桌面端 / 存进密码管理器——服务端只存 hash，签发后无法从云端取回明文。
- **用 `openssl rand -hex 32` 生成 Key**：脚本的最小长度门（默认 **32**，对齐本建议）**仅是长度下限、不是熵保证**——`32 个 'a'` 仍能过门，但 unsalted `sha256` 低熵 Key 可离线爆破。真正的强度靠高熵生成（`openssl rand -hex 32` 给 256-bit 随机），而非凑够长度。
- **凭据 hash 无盐、无 pepper（`sha256(trim(key))`）**：把 D1 凭据表及其导出/备份视同**与明文 Key 等密级**保管——一旦表或备份泄露，低熵 Key 可被离线**全速**暴力破解（无 KDF 拉伸、无盐，hash 本身不安全可公开）。加 pepper/KDF 属后续 worker 加固项（多租户设计阶段 1 决策 D 把它推迟到出现第二租户、需防拖库撞库时；现已到该节点，作为已知残余风险跟踪），**不在本工具/本变更范围**。
- **slug 校验、拒绝不转换**：slug 须匹配 `^[a-z0-9-]{1,32}$`（与 `tenants.tenant_id` 的 CHECK 同语义）。含大写 / 空格 / 点号 / 超长 / 空 → 脚本非 0 退出、stderr 说明、stdout 无 SQL（**不**自动小写化或截断）。空 Key、弱 Key（trim 后 <32）同样非 0 退出且不输出 SQL。
- **凭据已存在 = `wrangler` 抛 SQLite 约束错误，不是脚本 bug**：凭据行用普通 `INSERT`（非 `OR IGNORE`）。若该 Key / 该租户已签发过，`wrangler d1 execute` 会抛 `UNIQUE constraint failed`（主键 `id` 或 active 唯一索引 `idx_tenant_credentials_active_key_hash`）。**这是预期的安全失败**——表示该 Key 已签发，绝不静默覆写；`tenants` 行因 `INSERT OR IGNORE` 不会重复创建。
- **一把 Key 不能跨租户复用、每租户签独立 Key**：`idx_tenant_credentials_active_key_hash` 是**全局** active 唯一索引（索引列只有 `key_hash`、不含 `tenant_id`），把同一把 Key 签给第二个租户会被它拒绝报错——一把 Key 不能解析到两个租户。为每个租户用 `openssl rand -hex 32` 单独生成 Key。

#### 密钥轮换（须生成全新 Key）

active 唯一索引**只覆盖 `status='active'` 的行**（partial index `WHERE status='active'`）。这意味着：

- 把旧凭据行 `status` 改成 `'revoked'`（轮换/吊销）后，它就退出唯一索引，不再占用该 `key_hash` 的 active 槽位。
- **用本脚本 re-mint 同一租户的同一 Key 不会复活它**：`id = '<slug>-' || key_hash` 是确定派生（同 slug + 同 Key → 同 id），且 `id` 是主键 → 重签 SQL 会**撞主键 `tenant_credentials.id` 失败报错**，不会插入新 active 行。
- 真正能「复活」只在旧 `revoked` 行的 `id` 与脚本派生值**不同**时：例如阶段 1 手工 seed 的 `bh2ro-key`（其 id 是 `bh2ro-key`、非 `slug-<hash>`），re-mint 后新 active 行 id 不撞主键、active 唯一索引又只覆盖 active 行 → 旧 Key 被重新激活；或运维**手工**把 revoked 行翻回 active。两者都不是本脚本的默认行为，但运维需知。
- **结论**：轮换时**务必生成一把全新 Key**（`openssl rand -hex 32`），绝不拿旧/已泄露 Key 重签或手工翻 active。

### 自托管（两类）

自托管分两类，所需安全设施不同：

**(a) 只实现同步 API 的自定义后端**

只想接住桌面端同步（`/ping`·`/sync`·`/pull`）、不提供公共查询面。**仅需 Bearer 鉴权**（按 `key_hash` 表驱动解析租户，可选 `X-Tenant-Id` 交叉校验），**无需** PoW / 会话 / `RATE_LIMIT` KV / `SESSION_SECRET`。契约照 [cloud-sync-api-spec.md](cloud-sync-api-spec.md) 实现即可。

**(b) 自部署本仓库 worker**

部署本仓库 `web_query_service` 完整 worker（含公共「按呼号查询」面）。公共查询面**恒走 PoW 防爬**，须配齐：

- **`RATE_LIMIT` KV 绑定**（会话存储 / PoW 防重放 / 配额，KV 未绑定时会话端点 fail-closed 503）。
- **`SESSION_SECRET`**（会话 token 的 HMAC 密钥）。

二者配置见下文「[防爬会话 / PoW](#防爬会话--powquery-antibot-session阶段-3-b)」节。

**计划中（尚未上线，本文档仅作前瞻说明）**：

- `REQUIRE_POW`：让「只实现同步 API」一类的自部署免公共查询面 PoW 的开关（计划中）。
- `REQUIRE_TENANT_HEADER`：全量客户端升级后把 `X-Tenant-Id` 从「可选交叉校验」收紧为「必填」（计划中）。

### 默认 slug `default` 必须与 `DEFAULT_TENANT` 一致

自托管最简部署可只用一个租户、slug 取 `default`，但**必须同时**：

1. 配 `env.DEFAULT_TENANT = "default"`（在 `wrangler.toml` 的 `[vars]`；worker 内置兜底是 `bh2ro`，不配则默认租户回退到 `bh2ro`）；
2. 让 seed 的租户行 slug 也是 `default`（用 `mint-credential.mjs default …` 签发，或在 seed 步骤插入 `tenant_id='default'` 的 active 行）。

二者**必须一致**——这正是「阶段 4-B」节**部署契约（红线）**「seed slug ≠ `DEFAULT_TENANT` → bare 查询面以未 seed 的 `tenant_id` 查询 → 静默空结果」的同一条铁律（详见该节，不再赘述）。换言之，把默认 slug 改成 `default` 时，`DEFAULT_TENANT` 与 seed 租户行须同步改成 `default`，否则裸查询面会静默返回空结果、运营难察觉。

> 官方云的创始租户仍是 `bh2ro`（`DEFAULT_TENANT` 缺省即 `bh2ro`、与既有 seed 一致），无需改动。本节的 `default` 只针对从零起步的自托管者。

## 防爬会话 / PoW（query-antibot-session，阶段 3-B）

公开「按呼号查询」不登录、面向公众。查询侧防爬用 **PoW 门票 + 短时会话（绑真实 IP+UA）+ 会话配额 + 会话专属动态签名**把全量爬库成本抬到不划算（爬全库成本 ≈ 会话数 × PoW）。**物理上限诚实声明**：只抬成本、不杜绝遍历；纯 SHA-256 hashcash 对 GPU/ASIC 有数量级摊薄（memory-hard PoW 本期不引入）。

握手流程：`GET /api/session/challenge` 下发 `{seed, difficulty}` → 前端 PoW（Web Worker 内同步 sha256 找 nonce 使 `sha256(seed+":"+nonce)` 前导零 ≥ difficulty）→ `POST /api/session {seed, nonce}` 验 PoW 签发 `{token, sk, exp, quota}` → 查询带 `token + _ts + _nonce + _sig`（`_sig = HMAC-SHA256(sk, canonicalPayload)`）。

### 硬部署前置

- **`RATE_LIMIT` KV 必须绑定**：会话存储 / PoW 防重放 / 配额 / 自适应难度计数依赖 KV。与纯 IP 限流（KV 缺失 fail-open）不同，会话相关端点在 KV 未绑定时 **fail-closed**（`/api/session*` 返 503、无有效会话的查询被拒），**绝不**因缺 KV 静默放行无会话查询。
- **`SESSION_SECRET`**：会话 token 的 HMAC-SHA256 密钥（机密）。
  ```bash
  openssl rand -hex 32 | npx wrangler secret put SESSION_SECRET
  ```
  未配置 → 会话功能 fail-closed（503）。泄漏可伪造 token，但仍受「KV 必命中 session + IP/UA 绑定 + 配额」三重约束；可轮换（轮换瞬间在途会话失效、客户端自动重走 PoW）。

### 参数（apply 时按实测调，须落在约束区间内）

- PoW 难度：`BASE`（基线，手机 ~0.1–0.3s）、`BASE_MIN`（正下限 >0，恒有真实 PoW）、`DIFF_MAX`（上限封顶，封顶仍手机可解，避免共享出口 IP 正常用户 DoS）；按真实 IP（IPv6 /64、IPv4 /32 归一）自适应升档、受 `DIFF_MAX` 封顶。
- 会话 TTL ~10min；常规配额 `QUOTA`≈50；`unknown` 来源配额 `QUOTA_unknown`≤3（不绑 IP、可搬移，压低使搬移价值趋零）。

### 退役

- `CLIENT_SIGN_KEY`（静态查询签名密钥，经 `/api/config` 明文下发=可公开值、对防爬零收益）已由会话专属 `sk` 取代；`/api/config` 不再下发 `sign_key`。
- `CAPTCHA_SECRET`（算术验证码）已移除（PoW 取代）。两者可清理残留 Secret。

### lockstep 部署与回滚

- 前端（`src/client`→`public`）由 Worker 同部署，BREAKING、**不留静态 `sign_key` 兼容期**（旧缓存页面刷新即恢复）。一次 `pnpm run deploy`（`vue-tsc + vite build` 前端 + `wrangler deploy` worker）同发。
- 回滚：退 Worker 版本 + 还原前端构建；无 D1 迁移。

## CDN 真实 IP 与限流

生产经**阿里云 CDN**（`qsl.herbert-dev.cn`，备案域名、大陆入口）回源到 **Cloudflare 源站**（`qsl.herbertgao.me`）。经 CDN 路径时 Worker 收到的 `CF-Connecting-IP` 是**阿里云 CDN 回源节点 IP、不是真实用户 IP**；真实用户 IP 由阿里云 CDN 在回源请求头里注入。若仍按 `CF-Connecting-IP` 计数，成千上万真实用户会被**归并到少数 CDN 回源 IP 桶**，限流粒度失真。

**信任信号 = 阿里云 CDN 注入的密钥回源头 `X-Origin-Auth`**，而非回源 IP 白名单。阿里云回源 IP **动态分配、官方明确不建议固定白名单**（"不建议在源站设置固定的回源 IP 列表，否则可能导致回源失败"），且查询回源 IP 的接口（`DescribeL2VipsByDomain`）有日峰值带宽 ≥1Gbps + 工单门槛——故本服务改用「CDN 注入、客户端伪造被覆盖、攻击者猜不到」的密钥头判定「确来自 CDN」。

> **边界**：本配置产物**仅**用作限流/防爬计数键（抬高自动化批量调用成本），**不**用作访问控制/鉴权判据。鉴权由 Bearer Key 与请求签名承担。IP 键被部分污染/坍缩，最坏是「成本抬升打折」，不构成鉴权绕过。

### 配置项含义

- **`CDN_ORIGIN_SECRET`** — 期望的 `X-Origin-Auth` 密钥值（机密）。请求带正确密钥（worker 常量时间比对）即视为「确来自阿里云 CDN 回源」，进而采信受信真实 IP 头。
  - 生成：`openssl rand -hex 32`。仅经 `wrangler secret put CDN_ORIGIN_SECRET` 注入；**禁写入仓库文件、禁经 `/api/config` 下发**（它与公开的 `CLIENT_SIGN_KEY` 是两个独立值：后者公开、前者机密）。
  - 未配置/为空 → fail-safe：只信 `CF-Connecting-IP`、忽略一切注入头。
- **`CDN_REAL_IP_HEADER`** — 受信真实 IP 头名，即阿里云 CDN 写入真实用户 IP 的请求头名。
  - **无内置默认**。未配置即 fail-safe 到 `CF-Connecting-IP`、不读任何注入头。
  - 推荐值 `Ali-Cdn-Real-Ip`：阿里云 CDN **默认携带**该头（无需配置），官方语义为「客户端与 CDN 节点建连时的真实 IP」、**正是为避免 `X-Forwarded-For` 被伪造**而设。但「覆写而非透传」须经下文「证伪式抓包门」实证后，才由运营者显式填入。
  - 采信时运行时还会校验该头为**单值、合法 IP 字面量（IPv4/IPv6）、不含逗号**；多值/含逗号/非法（覆写假设失效信号）→ 落 `'unknown'` 惩罚桶，不退回 CDN 节点 IP。
  - **绝不采信 `X-Forwarded-For`**：它是 append 语义，首段为客户端可伪造值。

### 阿里云 CDN 配置（注入密钥头 + 真实 IP 头）

1. **修改出站请求头**：CDN 控制台 → 域名管理 → `qsl.herbert-dev.cn` → 回源配置 → **修改出站请求头** → 新增，头名 `X-Origin-Auth`、值 `<CDN_ORIGIN_SECRET>`。操作类型**首选「替换」**（或「增加」+「是否允许重复」选**「否」**），使 CDN 的值**覆盖**客户端伪造的同名头。**严禁选「允许重复=是」**——那会变成追加（多值），worker 侧合并后 ≠ 密钥 → 静默 fail-safe 到 CDN 节点桶（安全但限流粒度失真）。该误配会被下文证伪门抓到。
2. **回源协议 = HTTPS**：CDN 控制台 → 回源配置 → 回源协议设为 **HTTPS**（或「跟随」+ 用户走 HTTPS），确保密钥头不在 CDN→Cloudflare 这一跳走明文。
3. `Ali-Cdn-Real-Ip` 默认自动携带，无需在阿里云额外配置。

### 部署顺序

配置顺序是依赖序、不是并列——**密钥头注入须先于 worker 启用采信**：

1. 阿里云配好上节的「修改出站请求头」（注入 `X-Origin-Auth`）+ 回源协议 HTTPS。
2. 部署 Worker（此时 `CDN_ORIGIN_SECRET`/`CDN_REAL_IP_HEADER` 可先不配，解析对所有路径 fail-safe，行为安全）。
3. `wrangler secret put CDN_ORIGIN_SECRET`（与阿里云注入值一致）。
4. **跑证伪式抓包门**（见下）证实 `Ali-Cdn-Real-Ip` 确被 CDN 覆写。
5. **仅在通过证伪门后**，才配 `CDN_REAL_IP_HEADER`（如 `Ali-Cdn-Real-Ip`）。

### 证伪式抓包门

临时给 worker 加一行 debug 日志，跑完即撤。**禁止明文打印密钥**：打印 `headers.get('X-Origin-Auth') === <期望密钥>` 的**布尔比对结果**（不打原值/不打期望值）、`headers.get('Ali-Cdn-Real-Ip')`、`resolveClientIP` 返回值。从**已知出口 IP 的测试机**经 `qsl.herbert-dev.cn` 发请求，故意带伪造头：

```
curl -s "https://qsl.herbert-dev.cn/api/config" \
  -H "X-Origin-Auth: forged-garbage" \
  -H "Ali-Cdn-Real-Ip: 8.8.8.8" \
  -H "ali-cdn-real-ip: 7.7.7.7" \
  -H "X-Forwarded-For: 9.9.9.9"
```

**唯一通过判据**：worker 收到的 `X-Origin-Auth` 布尔比对 = `true`（证 CDN 覆盖了客户端伪造的 `forged-garbage`），且 `Ali-Cdn-Real-Ip` = 该测试机真实出口 IP、≠ 任一伪造值、为单值（证 CDN 覆盖、客户端伪造无效）。任一不满足 = 门失败（头被透传/未覆写，须排查阿里云回源头/回源协议配置，保持 `CDN_REAL_IP_HEADER` 不配）。临时日志**禁止打印密钥原值**；可打印**本次测试机自身的出口 IP**（用于比对，非终端用户流量），但**禁止打印真实终端用户流量的 IP**；门通过后即移除（与 captcha-protection 主规范「不记录用户完整 IP」一致）。

### 维护与应急

- **密钥保密**：`CDN_ORIGIN_SECRET` 只存阿里云 CDN 配置 + Cloudflare secret 两处，收紧控制台访问权限。
- **应急轮换**（非定时；怀疑泄漏时）：阿里云改 `X-Origin-Auth` 新值 →（过渡期 worker 可临时同时接受新旧两值）→ `wrangler secret put CDN_ORIGIN_SECRET` 新值。
- CDN 侧配置（出站头/回源协议）变更后**须重跑证伪门**确认覆写仍成立。

### 未配影响

未配 `CDN_ORIGIN_SECRET`（或为空）→ fail-safe：只信 `CF-Connecting-IP`、忽略一切注入头。此时 **CDN 路径下限流仍按 CDN 回源节点 IP 计数（粒度失真但不被绕过）**。配齐密钥头 + 经证实的受信真实 IP 头名后，才恢复按真实用户 IP 的限流粒度。

### 回滚

Cloudflare dashboard 退回上一 Worker 版本 + 移除/还原配置项（尤其**清空 `CDN_ORIGIN_SECRET` 或 `CDN_REAL_IP_HEADER` → 立即 fail-safe**）。纯服务端逻辑，无 D1 迁移、桌面端/前端零改动。
