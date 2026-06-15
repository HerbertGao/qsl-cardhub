## 为什么

公开「按呼号查询」面向公众、不登录。当前读取侧防护形同虚设：① IP 限流（已按真实用户 IP，阶段 3-A）只挡同 IP 高频，挡不住分布式/低频全量爬；② 请求签名密钥 `CLIENT_SIGN_KEY` 经 `/api/config` **明文下发**，是设计上的可公开值——攻击者直接拿它签名，签名校验对防爬零收益；③ 算术验证码（`/api/captcha` + `MathCaptcha.vue` + `CAPTCHA_SECRET`）是「剧场」，订阅入口才弹、查询本身不挡，且可绕。

阶段 3-A 已就位「可信真实客户端 IP 解析」（`getClientIP`），使 per-IP 防护能按真实用户计数——这是本阶段自适应难度/会话配额的前置。本阶段（3-B）落地防爬主体：**不引入账号登录，用 PoW + 短时会话 + 会话配额 + 会话动态签名把全量爬库成本抬到不划算**（爬全库成本 ≈ 会话数 × PoW）。接受「物理上无法杜绝全量遍历、只能抬高成本」的上限。

## 变更内容

- **PoW 门票**：新增 `GET /api/session/challenge` 下发 `{seed, difficulty}`（seed 一次性、经 `RATE_LIMIT` KV 防重放）；前端算 hashcash（找 `nonce` 使 `sha256(seed+":"+nonce)` 满足前导零位数）。
- **短时会话**：新增 `POST /api/session` 验 PoW → 签发**会话 token**（HMAC，绑真实 IP + UA，TTL ~10min）+ **该会话专属短时签名密钥**。
- **查询改验会话**：`GET /api/query`、`/api/callsigns/:callsign` 改验「会话 token + 会话签名 + 会话配额」（单会话查询次数封顶，如 10min/50 次，用尽重算 PoW）；IP 一律取自 `getClientIP`（阶段 3-A 真实 IP），并经 `clientBindingKey` 归一（IPv4/32、IPv6/64、unknown 直通），与 design/spec 口径一致。
- **会话动态签名（BREAKING）**：`/api/config` **停止下发** `CLIENT_SIGN_KEY`；签名密钥改为会话建立时动态下发、短时有效（每会话专属 `sk`）；查询签名口径为 `HMAC-SHA256(sk, canonicalPayload)`（与 design 决策 4 一致，取代静态 `sha256(payload+CLIENT_SIGN_KEY)`）。前端不再持静态 key。
- **自适应难度**：正常低难度（手机 ~0.1–0.3s 无感）；同一真实 IP 短时大量建会话 → 升难度。
- **删算术验证码剧场**：移除 `/api/captcha` 端点 + `generateCaptcha`/`CAPTCHA_SECRET` + 前端 `MathCaptcha.vue`（PoW 体系统一替换）。
- **前端**（`web_query_service/src/client/`）：PoW 计算（Web Crypto）+ 会话状态机（取 challenge → 算 PoW → 建会话 → 带 token+会话签名查询 → 配额用尽/过期重建）。
- 复用 `RATE_LIMIT` KV 存 seed 防重放、会话状态/配额计数。

**前端/worker 同步切换、不留静态 `sign_key` 兼容期**（前端由 worker 服务 `src/client`→`public`，lockstep 同部署；旧缓存页面刷新后即恢复）——已与用户确认。

**不做**（明确边界）：host/path → tenant 路由 + 多租户前端 + 桌面端租户身份（阶段 4）；顺丰 route-push 鉴权（推迟「接顺丰路由推送」变更）；PII 字段加密（可选增强，正交）；订阅绑定回调（`/api/wechat/auth-callback`）作为 OAuth 被动跳转**不**升级为 PoW+会话（仍只 IP 限流）。

## 功能 (Capabilities)

### 新增功能
- `query-antibot-session`: 查询侧防爬——PoW 门票（challenge/verify）、短时会话 token（绑真实 IP+UA、TTL）、会话专属动态签名密钥、查询侧会话校验、单会话查询配额、按真实 IP 的自适应难度。物理上限诚实声明（只抬成本、不杜绝遍历）。

### 修改功能
- `captcha-protection`: **移除**「算术验证码」「验证码生成接口」「验证码校验参数」需求（被 PoW 取代）；**移除**「请求签名校验」「请求签名格式」（迁移到 `query-antibot-session` 的会话动态签名）；更新「用户体验」「性能」「安全」「范围限制」以对齐 PoW+会话机制（IP 限流计数键继续取自 `trusted-client-ip`）。
- `cloud-backend-api`: 新增 `GET /api/session/challenge`、`POST /api/session`；`GET /api/query`、`/api/callsigns/:callsign` 改验会话 token+会话签名+配额；`GET /api/config` 停止下发 `CLIENT_SIGN_KEY`。

## 影响

- 代码：`web_query_service/src/worker/index.js`（新增 session 端点 + PoW 验证 + 会话签名/配额校验 + 改 query/config、删 captcha）；可能新增 `src/worker/session.js`（PoW/会话纯函数，便于单测）。
- 前端：`src/client/`（新增 PoW + 会话管理；删 `MathCaptcha.vue`；改 `App.vue`/`utils/sign.ts`）。
- 配置：`CAPTCHA_SECRET` 移除；新增会话 HMAC 密钥（`SESSION_SECRET`，wrangler secret）；`CLIENT_SIGN_KEY` 退役（不再 /api/config 下发）。`wrangler.toml.example` + 部署文档更新。
- 存储：`RATE_LIMIT` KV 增 seed/session/quota 键（部署前置：会话/PoW/防重放/配额/握手桶（`ratelimit:session`）**fail-closed**（会话端点 503），`powrate` 读写失败取 `DIFF_MAX`（**fail-secure**）；**仅既有查询桶 `ratelimit:<ip>`（checkRateLimit）保持 fail-open**；见 design 决策 6 / `query-antibot-session` 防爬范围需求）。
- 验证：worker smoke + 纯函数单测（PoW 验证、会话签发/校验、配额、自适应难度）。
- 回滚：纯服务端 + 前端 lockstep；无 D1 迁移；退 worker 版本 + 还原前端。
