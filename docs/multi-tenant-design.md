# 多租户与防爬架构设计

本文档描述 QSL-CardHub 云端服务（`web_query_service`，部署于 Cloudflare Workers + D1，对外域名 `qsl.herbertgao.me`）从「单一数据集」演进为「多租户」的目标架构，以及在「公开前端 + 不登录」前提下抬高暴力拉取成本的防爬设计。作为后续实现的蓝本。

## 背景与目标

当前云端是**单一数据集**：一个桌面端用全局 API Key 全量覆盖式 `/sync` 写入，一个公开前端用公开签名密钥按呼号读出。目标：

- **多租户隔离**：不同组织各自的数据互不可见，共用同一个 Worker + 同一个 D1。
- **前端各自独立**：受认证租户使用我们提供的前端，非认证租户自建前端。
- **向后兼容**：现存数据归入默认租户 `default`，现有桌面端与移动端零改动继续工作。
- **防暴力拉取**：在不引入账号登录的前提下，让全量爬库不划算。

## 现状与问题

| 问题 | 现状 | 影响 |
|------|------|------|
| 伪多租户 | 业务表有 `client_id` 列，但 `/sync` 用无 `WHERE` 的 `DELETE FROM` 全表清空后重写 | 多 `client_id` 物理上无法共存，实质单租户 |
| `client_id` 无业务语义 | 桌面端安装时随机 `Uuid::new_v4()`，是设备实例 ID | 不能承载「组织」概念 |
| 查询零隔离 | `/api/query` 仅 `WHERE callsign=?` | 所有数据混查 |
| 签名密钥公开 | `CLIENT_SIGN_KEY` 经 `/api/config` 明文下发（真实值存于 gitignored 配置、未进 git 历史；按设计视为可公开值）| 读取侧无真实访问控制 |
| 顺丰回调无鉴权 | `/api/sf/route-push` 无任何来源校验 | 可被伪造，触发对任意呼号的微信推送 |
| PII 明文 | `sf_senders`/`sf_orders` 含姓名/电话/地址，明文存 D1 | 拖库即泄漏敏感个人信息 |

## 设计决策

### 1. 租户隔离模型：行级 `tenant_id`

- 共享 schema + 行级 `tenant_id` 隔离（单 Worker / 单 D1 约束下唯一现实解；D1-per-tenant 受 Worker↔D1 绑定的部署期静态声明限制，无法运行时加租户）。
- `tenant_id` 为人类可读 slug（`default`、`bjradio` …），进路由、进凭据绑定、便于运维对账。
- **业务表唯一隔离键为 `tenant_id`，主键演进为 `(tenant_id, id)`；删除业务表的 `client_id` 列**。设备身份仅保留在 `sync_meta` 作租户级溯源（`last_client_id`）。
- 隔离边界由服务端强制：所有读写的 `tenant_id` 来自「写入 Key 解析」或「请求路由解析」，**绝不信任请求体自报的 id**。

### 2. 写入鉴权与同步

#### 2.1 每租户独立写入凭据

- 写入 Key 不再是单一全局值。D1 存凭据表，`sha256(key)` 比对，命中得 `tenant_id`。支持「多 Key → 同一租户」（便于按设备签发/吊销）。
- 现有全局 API Key 的哈希登记为 `default` 的写凭据，现有桌面端零改动。
- **「一个租户默认一个用户」是使用约束、非结构变化**：保留「多 Key → 同一租户」超集（一个用户的多台设备各持一把可独立吊销的 Key）；「一个用户」语义落在 UI/业务层（当前只开一个登录席位），不把模型收紧成「一租户一 Key」（否则用户加第二台设备就要扩表回去）。未来「一租户多席位」只需正交新增 `tenant_users` 表，不回迁、不触发已有表重建。
- 桌面端「租户身份 + 专属写凭据」的配置改造落**阶段 4**（与「第二个真实租户上线」同属写入侧）。阶段 1 服务端先把凭据表与 Key→tenant 解析建好；第二个真实租户上线前，桌面端继续用全局 API Key 落 `default`。

#### 2.2 同步模型 A：上传为主 + 按需下载 + 版本护栏

云端定位为「桌面端的备份镜像 + 移动端查询源」，不做多端实时合并（与单人/小组织现实匹配，避免 CRDT 过度工程）。

- **`/sync`（上传）**：按 Key 解析 `tenant_id` → `DELETE … WHERE tenant_id=?` + 重新 INSERT，**全部置于单个 `DB.batch` 事务**（修复当前 DELETE 与 INSERT 不在同一事务、中途失败即清空的缺陷）。
- **乐观并发版本护栏**：`sync_meta` 维护单调 `server_version`。桌面端持有 `base_version` 并在上传时回传；服务端用条件写做 compare-and-swap：

  ```sql
  UPDATE sync_meta
     SET server_version = server_version + 1, last_client_id = ?, last_sync_at = ?
     WHERE tenant_id = ? AND server_version = ?;   -- ? = 桌面端回传的 base_version
  -- changes == 0 → 基线陈旧或并发抢先 → 409，且未删除任何数据
  -- changes == 1 → 继续执行 DELETE + INSERT
  ```

  两台设备并发只有一台命中，另一台 409。`force=true` 可人工跳过比较（用于「确实要用旧备份覆盖」）。
- **`/pull`（下载，新增）**：按 Key 解析 `tenant_id`，返回该租户全量快照 + 当前 `server_version`。用途：换机/新设备初始化、被护栏挡住后先下载再续、主动从云端恢复。
- 下载不是每次同步的常规步骤；单设备用户永远只走上传，体验不变。

### 3. 读取侧与移动端

#### 3.1 租户路由

- 由访问的 host / path 前缀解析 `tenant_id`：子域 `bjradio.qsl.herbertgao.me`、自建域、或路径前缀 `/t/bjradio/`，查 `tenant_routes` 表。
- 裸 `qsl.herbertgao.me`（无子域、无前缀）回退 `default`。现有移动端因此零改动落到 `default`。

#### 3.2 查询隔离

- `/api/query`、`/api/callsigns/:callsign` 服务端强制注入 `WHERE tenant_id=? AND callsign=?`，`tenant_id` 来自路由解析，不来自前端参数。即使某租户的查询凭据泄漏，攻击者也只能查其所属租户。

#### 3.3 前端多实例

- 一份前端代码 + 多实例运行：前端不内嵌 tenant，启动时 `GET /api/config`（服务端按 host 给出该租户的 `sign_key`/`features`/备案/微信配置/标题），UI 数据驱动。
- 加新租户 = 插一行 `tenants` + 一条凭据 + 一条路由，零重新构建/部署前端。

### 4. 防爬（方向 A）

受认证租户的查询面向公众、不登录。**接受「无法杜绝全量遍历、只能抬高成本」的物理上限**，用以下组合让批量爬库不划算：

- **PoW 门票**：进入页面时 `POST /api/session/challenge` 取 `{ seed, difficulty }`，前端算 hashcash（找 `nonce` 使 `sha256(seed+nonce)` 满足前导零位数），`POST /api/session` 验 PoW（`seed` 经 KV 防重放）后签发会话。
- **短时会话凭据**：会话 token（HMAC/JWT，绑 IP+UA，TTL ~10min）+ **该会话专属、短时的 `sign_key`**。后续查询带 token 与请求签名。
- **会话配额**：单会话查询次数封顶（如 10min / 50 次），用尽则重新算 PoW。爬全库成本 ≈ 会话数 × PoW。
- **自适应难度**：正常低难度（手机 ~0.1–0.3s 无感），同一 IP 短时大量建会话则升难度。
- 复用现有 `RATE_LIMIT` KV 存 `seed` 防重放与会话配额计数。

> 前提：`sign_key` 必须从「`/api/config` 静态明文下发」改为「会话建立时动态下发、短时有效」，否则攻击者直接持公开 key 签名查询，绕过整套 PoW。

> **前提（CDN 架构下的真实 IP）**：生产经**阿里云 CDN**（qsl.herbert-dev.cn）回源 Cloudflare（qsl.herbertgao.me），故 worker 的 `CF-Connecting-IP` 在 CDN 路径下是**阿里云 CDN 回源节点 IP、非真实用户**（实测确认：响应头含 kunlun 节点 / Tengine / Ali-Swift / eagleid + cf-ray 透传）。因此一切「同一 IP 限流 / 升难度」「按 IP 计 nonce/会话」**必须**改从**阿里云 CDN 注入的真实 IP 头**（如 `X-Forwarded-For` 首段）取，**并校验请求确来自阿里云 CDN 回源**（白名单 CDN 回源 IP 段，否则该头客户端可伪造）——「`CF-Connecting-IP` 不可伪造」在前置 CDN 下**不再成立**。此约束同样回头影响阶段 0 已加的 `/api/wechat/auth-callback` 限流（当前按 `CF-Connecting-IP`，CDN 路径下粒度失真，待此阶段一并修正）。

### 5. 租户分级

| | ① 受认证租户 | ② 非认证租户 |
|---|---|---|
| 前端 | 我们提供（现有移动端形态） | 租户自建 |
| 查询防护 | PoW + 短时会话 + 会话配额 + 动态签名 | 仅 IP 限流 |
| 数据存储 | 同 D1（可叠加 PII 字段加密） | 明文 |
| 隔离 | 服务端注入 `tenant_id` | 服务端注入 `tenant_id`，且与①物理共表但逻辑隔离 |
| 责任 | 平台提供 best-effort 防爬 | 条款声明「数据视为公开」，平台不托底 |

②的明文自助须在条款写明：数据视为公开、平台不担保密性；②租户作为其上传他人 PII 的处理者自负告知义务；禁止上传敏感个人信息（手机号+详址）到明文存储。

> **阶段 4 修订（2026-06-16，用户拍板）——本表 ①/② 分级在官方云上退役**：
>
> **官方云只收认证租户（写入），不收游客。** 故官方云上不存在「②非认证租户」住户——每个官方租户都有运维线下签发的注册凭据 `(tenant, key)`，`key` 即「认证码」。后果：
> - **写入（sync/pull）**：仅认证租户（worker 交叉校验申报 `tenant` 与 `key→tenant` 一致，不一致 403）。无游客自动建行、无防 spam、无撞码问题。
> - **读取（query）**：仍公开、按租户路由，**恒走 PoW**（官方租户全是认证档、无可切换的 open 档）。
> - **`tier` 退役**：`tenants.tier` 列保留为预留字段，官方云**不接 per-tenant tier 分流逻辑**。原②档（轻量明文、不托底）整体**搬到自托管**——自托管者若想免 PoW，做成 worker 部署期 env 开关（如 `REQUIRE_POW`），而非 per-tenant tier。
> - **客户端两模式**（非三模式）：① **云同步**（官方或自托管同一套配置 `(api_url, tenant, key)`，仅 `api_url` 不同）；② **纯本地**（无云、导出/导入、零配置，现有本地用户不被挡）。租户身份常驻标题栏、点击导航到「租户 & 云端同步」设置。
> - **自托管**：我方只提供 **API 规范**（`docs/cloud-sync-api-spec.md`，默认 `default` 单租户）+ 开源 worker 代码可自部署；不承诺 turnkey 单租户产品。
> - **注册**：纯线下运维签发 `(tenant, key)`，**无公开自助注册端点**（无滥用面）；加分项给管理员一个 **CLI 脚本** mint 凭据。
> - **存量 `bh2ro` 零凭据迁移**：阶段 1 已把 `sha256(trim(API_KEY))` seed 为 `bh2ro-key` 凭据，现有全局 API_KEY 本就解析为 `bh2ro`——存量用户升级后只需在租户框填 `bh2ro`、key 不变。

### 6. 全局表处理

- **`callsign_openid_bindings`**：加 `tenant_id`，主键 `(tenant_id, callsign, openid)`。微信授权 `state` 改带 `tenant:callsign`，回调解析后按 tenant 写入；顺丰推送选 openid 时按 tenant 过滤，避免「A 租户订阅者收到 B 租户物流推送」。
- **`sf_route_log`**：保持全局去重（顺丰 waybill 全局唯一），`tenant_id` 通过匹配的 `sf_orders` join 派生，不单独隔离去重维度。

## 数据模型（DDL 草图）

```sql
-- 租户与凭据
CREATE TABLE tenants (
  tenant_id  TEXT PRIMARY KEY,           -- slug
  name       TEXT NOT NULL,
  tier       TEXT NOT NULL DEFAULT 'authenticated',  -- authenticated | open
  status     TEXT NOT NULL DEFAULT 'active',
  created_at TEXT NOT NULL
);

CREATE TABLE tenant_credentials (
  id          TEXT PRIMARY KEY,
  tenant_id   TEXT NOT NULL REFERENCES tenants(tenant_id),
  scope       TEXT NOT NULL,             -- 'write'
  key_hash    TEXT NOT NULL,             -- sha256(key)(+pepper)，不存明文
  status      TEXT NOT NULL DEFAULT 'active',
  created_at  TEXT NOT NULL,
  last_used_at TEXT
);
CREATE UNIQUE INDEX idx_cred_hash ON tenant_credentials(key_hash) WHERE status='active';  -- 唯一：防一把 Key 登记到两个租户的解析歧义

CREATE TABLE tenant_routes (
  route_key TEXT PRIMARY KEY,            -- host 或 path 前缀
  tenant_id TEXT NOT NULL REFERENCES tenants(tenant_id)
);

-- 同步元数据（增加版本护栏）
-- sync_meta(tenant_id PK, server_version INTEGER, last_client_id, last_sync_at, received_at)

-- 业务表演进示例：主键 (tenant_id, id)，无 client_id
-- PRIMARY KEY (tenant_id, id); INDEX (tenant_id, callsign COLLATE NOCASE)
```

## API 端点总览（改动）

| 端点 | 变化 |
|------|------|
| `POST /sync` | 按 Key 解析 tenant；OCC 版本护栏；单事务全量替换 |
| `GET /pull` | **新增**：按 Key 拉回该租户全量快照 + server_version |
| `GET /api/session/challenge` | **新增**：下发 PoW 题 |
| `POST /api/session` | **新增**：验 PoW，签发会话 token + 会话签名密钥 |
| `GET /api/query`、`/api/callsigns/:callsign` | 注入 `tenant_id` 过滤；改验会话 token + 会话签名 + 会话配额 |
| `GET /api/config` | 按路由 tenant 下发该租户配置 |
| `GET /api/wechat/auth-callback` | `state` 带 tenant，按 tenant 写绑定 |
| `POST /api/sf/route-push` | **加来源鉴权**；按 order join 派生 tenant 过滤推送 |

## 待修复的既有安全问题（独立于多租户）

1. `/api/sf/route-push` 无鉴权 → 加共享 secret（**query token**，URL 参数；顺丰不支持自定义请求头、回调亦不带签名）。**留待「接顺丰路由推送」变更**（生产未接入推送前低优先）。
2. `CLIENT_SIGN_KEY` 配置卫生 → 真实值（存于 gitignored 配置、未进 git 历史）不进版本控制、修正文档过时指引；其经 `/api/config` 面向公网的暴露面属设计「可公开」，真正闭合在阶段 3 动态下发。
3. captcha 死代码（`verifyCaptcha` 从不被调用）→ 阶段 0 删除后端死代码（前端验证码剧场与 `/api/captcha` 留待阶段 3 防爬体系统一替换，不在阶段 0 接入）。
4. 错误响应直接回显 `e.message` → 脱敏。
5. `/api/query` 零租户隔离 → 注入 `tenant_id` 过滤（随多租户地基一并修）。

## 实施路线（分期）

每一期均设计为可独立交付、可独立回滚；前期为后期的前置。

### 阶段 0：安全急修（不依赖多租户）
- `CLIENT_SIGN_KEY` 配置卫生（真实值不进版本控制、修正过时指引；面向公网暴露面 deferred 阶段 3）；删 verifyCaptcha 死代码并给订阅回调补最小 IP 限流（独立桶 + `CF-Connecting-IP`）；错误响应脱敏（含上游错误分支）。**route-push 来源鉴权不在此期**——顺丰 RoutePushService 不支持自定义请求头、且生产未接入推送，推迟到「接顺丰路由推送」变更用 **query token** 方案（URL 参数）+ 完整字段去重 + json 校验一并做。
- 验收（前提：`RATE_LIMIT` KV 已绑定，否则 `checkRateLimit` fail-open 不限流）：超额打订阅回调被独立桶限流且伪造 `X-Forwarded-For` 不绕过、内部异常/微信失败响应不含原始结构、仓库无明文密钥。回滚：各项独立，可单独还原。（「伪造顺丰回调被拒」随 route-push 鉴权移到「接顺丰路由推送」变更。）

### 阶段 1：多租户地基与写入隔离
- 建 `tenants` / `tenant_credentials` / `tenant_routes`；业务表加 `tenant_id`（`DEFAULT 'default'`）→ 主键迁移 `(tenant_id, id)`、删 `client_id`（建新表 + `INSERT…SELECT` 回填 default + DROP + RENAME，每表四步置于同一 batch 事务单元；本项目业务表间无真实外键，无需切换 FK）。
- `/sync` 按 Key 解析 tenant + 单事务按租户全量替换；`/api/query` 注入 `tenant_id` 过滤；现有 API Key 哈希登记为 default 写凭据。
- **迁移交付**：单一 SQL 文件走 `wrangler d1 execute --file --remote`，命令展示给用户在自己终端跑（不让 AI 跑生产迁移）。迁前 `wrangler d1 export` 全量备份 + `SELECT client_id,COUNT(*) … GROUP BY client_id` 校验单一所有者（历史若混进第二个 client_id，无脑回填 default 会撞 `(tenant_id,id)` 主键、迁移失败）。
- **凭据校验**：表驱动为主（查 `tenant_credentials`）+ `env.API_KEY` 直比兜底一期；兜底路径打标记/日志，确认表驱动路径真命中后再撤兜底（避免「以为切表了其实一直走兜底」）。
- **default 凭据 seed**：离线算 `sha256(API_KEY)` 写进迁移 SQL（AI 不碰 secret 明文，算 hash 命令也给用户自跑）；阶段 1 不加 pepper（留到有第二租户、需防拖库撞库时再加）；secret 尾随空白两侧对齐（worker 比对与算 hash 均对 `.trim()` 后的值）。
- **现在就钉死、防二次迁表**：`(tenant_id, id)` 主键 + `(tenant_id, callsign COLLATE NOCASE)` 复合索引；`tenant_credentials.key_hash` 部分唯一索引（`UNIQUE … WHERE status='active'`）；`tenant_id` 字符集/长度约定（slug：小写字母数字 + 连字符 + 长度上限）；`sync_meta` 一次性建终态（PK=tenant_id + `last_client_id` + `server_version INTEGER NOT NULL DEFAULT 0`，本期只占列、OCC 逻辑留阶段 2）。
- **worker 同步改**：删/改 `/sync` 内联 `CREATE TABLE`（旧 `client_id` schema 与迁移后新表打架，某表被手工 DROP 时会按旧结构重建）；DELETE + INSERT 合入单 `DB.batch` 按租户全量替换（实测 default 真实行数确认不撞 D1 单次语句数/绑定参数上限）。
- **不在本期**：IP 来源修正（CDN 真实 IP 头 + 回源白名单，阶段 3）；桌面端租户身份配置（阶段 4）。
- 验收：现有桌面端、现有移动端零改动继续工作（落 default）。回滚点：DROP+RENAME 前以 `wrangler d1 export` 兜底。

### 阶段 2：同步健壮性
- `sync_meta` 加 `server_version`；`/sync` 加 OCC 版本护栏 + `force`；新增 `GET /pull`。
- 桌面端：持久化 `base_version`、处理 409（引导先下载或强制覆盖）、`/pull` 恢复入口。
- 验收：两设备交替同步不再静默互相覆盖；换机可从云端拉回。

### 阶段 3：防爬与读取侧动态化
- 新增 `GET /api/session/challenge`、`POST /api/session`（PoW + 签发会话 token 与会话签名密钥）；`/api/query` 改验会话 token + 会话签名 + 会话配额；自适应难度。
- 前端：PoW 计算与会话管理。
- 验收：无会话/无有效签名的直接查询被拒；全量遍历需大量 PoW。

### 阶段 4：多租户前端与桌面端身份（拆 4-A/4-B/4-C；见 §5 阶段 4 修订）

- **4-A 全局表租户化（已闭环 2026-06-16）**：`callsign_openid_bindings` 加 `tenant_id`（迁移 0002）+ 微信 `state` 向前兼容 `tenant:callsign` + route-push 按匹配订单派生租户过滤；`sf_route_log` 保持全局。已部署生产 worker `facccf8a`、归档。
- **4-B 路径路由 + 按租户配置（待做）**：路径前缀 `/t/<slug>/` → `tenant_routes`/`tenants` 解析；`/api/config` 按租户下发（filing/标题/功能）；查询/会话按租户隔离（KV 键加租户命名空间）。**官方云恒 PoW、不接 tier 分流**（见 §5 修订）。前端从 URL 派生 tenant、`state` 改发 `tenant:callsign`。上线第二个真实租户（建 tenant + 凭据 + 路由 + 前端实例）。
- **4-C 桌面端租户身份与登录态（待做，需求已收敛 2026-06-16）**：客户端两模式（云同步 `(api_url, tenant, key)` / 纯本地）；首启登录窗；标题栏常驻租户、点击导航到「租户 & 云端同步」设置；`SyncConfig` 加 `tenant`（硬性必填走云同步时）；`/sync`·`/pull` 带申报 tenant，worker 交叉校验 `resolveTenant(key)===申报tenant` 不一致 403（**归属真源仍是 key、申报 tenant 只做校验+展示，绝不当写入目标**）；`/ping` 回显认证态。线下签发 + CLI mint 脚本（4-C4）；`cloud-sync-api-spec.md` 补 tenant 契约 + 自托管文档（4-C4，本期未改）。存量 `bh2ro` 零凭据迁移。**拆 4 子变更**（依赖序，4-C 全程不依赖 4-B、4-C1 可独立先行）：4-C1 worker 交叉校验（统一请求头 `X-Tenant-Id` + helper `crossCheckTenant`：`resolveTenant(key)` 命中后比对申报值、不一致 403、缺 header 向后兼容放行；**入库恒用 resolveTenant 返回值、申报值只校验+回显**；/ping 升级走 resolveTenant + 回显认证态 + readonly 不计兜底）→ 4-C2 桌面端 `SyncConfig.tenant`(Option+serde default)+命令+client header+四态枚举（sync 类型**纳入 ts-rs**）→ 4-C3 应用内首屏登录遮罩 + 自绘标题栏租户徽章 + DataTransferView 重组；4-C4 CLI `mint-credential.mjs`（离线 hash 出幂等 SQL）+ 文档，与 4-C1 并行。
- 验收：新租户与 bh2ro 数据互不可见；现有移动端/本地用户不受影响。

### 可选增强（独立项，按需排期）
- **PII 字段加密**：`sf_senders`/`sf_orders` 的姓名/电话/地址用对称 AES-GCM 字段加密，DEK 在 Cloudflare secret 并离线备份（防拖库、对齐个人信息保护合规）。与多租户、防爬正交，可独立实施。

## 明确不做（避免过度工程）

- 给公开前端分配公私钥防爬：公开前端藏不住密钥，复杂度换零收益。
- 端到端/零知识加密：微信推送与顺丰下单要求云端可读 PII，技术上跑不通。
- callsign 盲索引：呼号低熵、索引密钥泄漏即被反查表击穿，且防的是拖库非前端枚举，ROI 低。
- 租户准入 token（B 方案）：已选 A 方案（面向公众查询），不引入准入令牌。
- 多设备双向合并 / CRDT：与「按租户全量覆盖」模型相悖，单人/小组织无此需求。
- mTLS / 账号登录给公众查询：与「不登录」前提冲突。
