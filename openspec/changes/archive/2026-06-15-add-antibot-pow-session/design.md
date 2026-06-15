## 上下文

公开按呼号查询不登录、面向公众。现状防护：IP 限流（阶段 3-A 已按真实 IP）+ 静态 `CLIENT_SIGN_KEY` 签名（经 `/api/config` 明文下发=可公开值，对防爬零收益）+ 算术验证码剧场（订阅入口才弹、不挡查询）。阶段 3-A 的 `getClientIP`（经 `X-Origin-Auth` 密钥头判 CDN 来源 → 采信 `Ali-Cdn-Real-Ip`）已激活，按真实用户 IP 计数可用。

本阶段用 PoW + 短时会话 + 配额 + 会话动态签名抬高全量爬库成本。`RATE_LIMIT` KV 已绑（限流/nonce 用），本阶段复用它存 seed/session/quota。worker 单文件 `src/worker/index.js`（ES module）；公开前端 `src/client/`（Vue3，`App.vue` 取 `/api/config` 的 `sign_key`、`utils/sign.ts` 签名、`MathCaptcha.vue` 验证码剧场）。

约束：不登录、不引入账号；只抬成本不杜绝遍历（物理上限）；本解析/会话仅护查询读取面，不动 Bearer 写鉴权/verifySignature 之外的同步路径。

## 目标 / 非目标

**目标**：PoW 门票 + 短时会话（绑真实 IP+UA）+ 会话配额 + 会话动态签名 + 自适应难度；删算术验证码；前端 PoW/会话管理；`/api/config` 停发静态 sign_key。

**非目标**：账号登录/mTLS；host/path tenant 路由（阶段4）；route-push 鉴权（推迟）；PII 加密（正交）；订阅回调升级（仍只 IP 限流）；杜绝全量遍历（只抬成本）。

## 决策

### 决策 1：会话存 KV（sid→状态），token = 高熵不可猜 sid + 真 HMAC-SHA256

- `POST /api/session` 验 PoW 通过后：生成高熵 `sid`（crypto.randomUUID + 随机）、会话签名密钥 `sk`（随机 32B hex）；写 KV `session:<sid>` = JSON `{binding_mode, ip|null, ua_hash, exp, sk}`（`binding_mode ∈ {ip, none}`；`ip` 在 `none` 模式为 `null`，见决策 7），TTL ~10min。
- **token = `sid.HMAC(SESSION_SECRET, sid)`**，HMAC **必须**为真 HMAC-SHA256（`crypto.subtle` HMAC），**禁止** `sha256(secret + msg)` 形式（防长度扩展攻击）。`sid` 字符集**必须**限定为 hex 或 base64url（**禁止**含 `.`），解析**必须**用 `lastIndexOf('.')` 切分 `sid` 与 HMAC 避免歧义。token 返回客户端（响应体，HTTPS）。
- 授权**必须**三要素之与：① HMAC 有效 ∧ ② KV 命中 `session:<sid>` ∧ ③ 绑定项（exp/ip/ua）全过；**禁止**任何「HMAC 通过即放行」的 token-only 快路径。
- 查询校验：拆 token 取 sid + 验 HMAC → KV 读 `session:<sid>` → 校验 `exp` 未过期、`client_binding_key(getClientIP(request))` === `session.ip`（均为归一键；unknown 会话 binding_mode=none 时跳过此项，见决策 7）、`ua_hash === sha256(UA)` → 取 `sk` 验查询签名 → 校验+递增配额。任一不过 → 401（会话无效/过期）。
- **被否**：JWT（需库/更重，KV 已是状态源）；把 `sk` 放进 token（暴露，改为存 KV + 响应体下发）。

### 决策 2：PoW = hashcash（sha256 前导零位），seed 一次性 KV 防重放 + 绑 IP/UA

- `GET /api/session/challenge`：生成随机 `seed`（hex）、按自适应算 `difficulty`（前导零 bit 数）；写 KV `powseed:<seed>` = `{difficulty, challenge_ip, challenge_ua_hash, exp}`（`challenge_ip`=`client_binding_key(getClientIP)`（归一键，与兑换比对同口径）、`challenge_ua_hash`=`sha256(UA)`），TTL ~2min（一次性）。返回 `{seed, difficulty}`。
- 前端找 `nonce` 使 `sha256(seed + ":" + nonce)` 的二进制前导零 ≥ `difficulty`。
- `POST /api/session {seed, nonce}`：读 KV `powseed:<seed>`（不存在→过期/重放，拒）→ **立即删**（一次性、防并发重放）→ 校验 `sha256(seed+":"+nonce)` 前导零 ≥ 该 seed 的 difficulty → **校验当前 `client_binding_key(getClientIP)` === `seed.challenge_ip` 且 `sha256(当前 UA)` === `challenge_ua_hash`**（均为归一键；不等则拒，防低难度 IP 领题→他处兑换绕开自适应难度）→ 通过则签发会话。
- 难度→耗时：18–20 bit ≈ 手机 0.1–0.3s（2^18≈26 万次 sha256）。base 取可调常量。
- **写后读一致性**：Cloudflare KV「写后立即读」最终一致，可致正常用户算完 PoW 提交时 challenge 写入的 seed 尚不可见 → 偶发「seed 不存在」误拒。缓解=客户端遇 seed-not-found 短退避后重试 `challenge` 一次。
- **本期契约路径＝KV `powseed`**：本变更采用上文 spec/tasks 已写的 KV `powseed:<seed>` 记录 `{difficulty, challenge_ip, challenge_ua_hash, exp}` 方案，作为唯一契约路径。
- **无状态签名 seed（本期不实现、留待后续的可选注记）**：可选形态 `seed = HMAC(SESSION_SECRET, randomNonce‖difficulty‖exp‖challenge_ip‖challenge_ua_hash)`（把这些字段编入 seed 自身），verify 时验 HMAC + 解码取回字段、**仅在兑换时写一次性消费标记**，可同时消除 challenge 端 KV 写放大与写后读窗口。**本期不实现**（与 KV `powseed` 契约互斥，避免双路径歧义）。若将来采纳**必须**：(a) 为 seed 与 token 的 HMAC 做**域分隔**（固定不同 label 前缀如 `"qsl.seed.v1"`/`"qsl.token.v1"`，或 HKDF 派生独立子密钥，避免共用 `SESSION_SECRET` 跨用途混淆，即 N11）；(b) 兑换时比对的是**当前请求的真实 IP/UA** 对 seed 内经 HMAC 完整性保护的 challenge 值（seed 内值仅作完整性锚点、不可被客户端篡改）。

### 决策 3：自适应难度（按真实 IP）

- **单一 IP 归一键 `client_binding_key(ip)`**（统一全套按-IP 键的归桶口径）：IPv4 → 完整地址（/32，直通）；IPv6 → /64 前缀（取前 64 位归一）；`unknown` → `unknown`（直通）。`seed.challenge_ip`、`session:<sid>.ip`、`powrate` 桶、`ratelimit:session` 握手桶、兑换/查询时的 IP 比对**全部**用 `client_binding_key(getClientIP)`（存与比都用归一键，**而非**完整 IP 精确 ===），使会话绑定与频率计数同源、同一 /64 内 IPv6 隐私地址轮换不误杀。
- KV `powrate:<client_binding_key>` 计数（固定窗口，TTL ~5min）。归桶口径即 `client_binding_key`：**IPv6 按 /64 前缀聚合、IPv4 按完整地址（/32）**——否则攻击者用同一 /64 内海量地址每个只建一次会话即永远停在最低难度，使升难度对 IPv6 不可满足。`unknown` 来源（`client_binding_key=='unknown'`）用独立计数桶，**不**污染具体 IP 桶。
- `challenge` 时 `difficulty = BASE + tier(powrate)`：rate 越高 tier 越高（每超阈值 +2~4 bit，单调递增），但**受 `DIFF_MAX` 封顶**。正常用户低难度无感；同一真实 IP 短时狂建会话→指数级变贵直至封顶。
- **难度约束区间**：下限 `BASE_MIN > 0`（恒有真实 PoW，禁止 difficulty=0 致免 PoW）；上限 `DIFF_MAX`（手机仍可在合理时间解出，避免对共享出口 IP 的正常用户造成不可解 DoS）；tier 单调递增但封顶于 `DIFF_MAX`。`unknown` 来源取最高难度档（**≡ `DIFF_MAX`**，取封顶值，避免 unknown 比顶格 IP 便宜；不依赖 IP 绑定，仅靠 UA+配额）。
- 自适应输入 IP **必须** getClientIP（真实 IP）——否则 CDN 节点 IP 失真（与阶段 3-A 一致）。

### 决策 4：会话动态签名（取代静态 CLIENT_SIGN_KEY）

- 查询签名 **必须** `_sig = HMAC-SHA256(sk, canonicalPayload)`（`sk`=会话专属密钥；**禁止** `sha256(payload + sk)` 形式）。查询带 `token` + `_ts` + `_nonce` + `_sig`。
- `canonicalPayload` **必须**无歧义，按**固定顺序**串接五项（用 `\n` 连接）：① `sid`（纳入 sid 而非完整 token，避免拼装漂移）；② 路径（**原始 `url.pathname`，保留 URL 编码形态、`%2F`/`%0A` 不解码**，client/worker 同口径，避免 `:callsign` 段一端解码一端不解码致签名不符）；③ 本次请求实际携带**业务**查询参数按 key 字母序的已 URL 编码串 `key=value&...`（**排除 `_sig`、`_ts`、`_nonce`**——`_ts`/`_nonce` 由字段④⑤专门承载，避免与③重复纳入）；④ `_ts`；⑤ `_nonce`。即 `_ts`/`_nonce` **必须**纳入 HMAC 输入（绑定新鲜度/防重放，**禁止**换 ts/nonce 复用旧 `_sig`），但仅经④⑤承载、不出现在③；路径式端点（`/api/callsigns/:callsign`）字段③为空串（`_ts`/`_nonce` 仍由④⑤承载）。字段③ 序列化：每个 key 与 value 各自 `encodeURIComponent`（保证 `&`/`=`/`\n`/`%` 均百分号编码）后以 `=` 拼对、`&` 连接，断言编码后除结构分隔符 `&`/`=` 外无裸 `&`/`=`/`\n`（分隔符注入规范层不可能）。client 与 worker **必须**同一实现：`canonicalPayload` **必须**为 worker 与 client 共用的**单一事实源模块**（**禁止**两处各写易漂移实现）；落法**必须**保证 `pnpm run build`（`vue-tsc --noEmit`）仍绿——采用**共享 `.js` 模块 + 同名手写 sibling `.d.ts`**（worker `import` 该 `.js`、client `utils/sign.ts` re-export 该 `.js`，`.d.ts` 提供类型，**无需**改 tsconfig），或等价地启用 `allowJs` 并将该模块路径纳入 `tsconfig` `include`（二选一，apply 时择一）。
- worker：由 token→sid→KV `sk` 验签；`_ts` 时窗 + `_nonce` 经 KV 防重放（复用现有 `nonce:` 机制）。
- `/api/config` **删 `sign_key` 字段**（不再静态下发）；前端改从 `POST /api/session` 响应取 `sk`。`CLIENT_SIGN_KEY` 退役（worker 不再读、不再 /api/config 暴露）。
- **会话动态签名的防护定位**：主防护是 PoW + 会话 + 配额 + 绑定；动态签名主要承担**请求完整性 / 重放防护**。相对静态 key 的增益在于「每会话 + 配额绑定」，**而非密钥保密**（`sk` 经响应体下发给客户端，本就不是公网明文，但其价值不在保密而在与会话/配额绑死）。
- **BREAKING + lockstep**：前端 `src/client`→`public` 由 worker 同部署，旧缓存页面刷新即恢复；不留静态兼容期（用户已定）。

### 决策 5：会话配额

- KV `sessionq:<sid>` 计数，TTL=会话 TTL；每次查询 +1，超 `QUOTA`（如 50）→ 429（配额用尽）。客户端遇 429→重走 challenge→PoW→新会话。
- **`unknown` 来源会话压低配额**：`unknown` 会话不绑 IP（可跨 IP 搬移），其配额**必须**取 `QUOTA_unknown`（如 ≤ 3，远小于常规 50），使一次最高难度 PoW 仅换极少次查询、搬移价值趋零（见决策 7 / 自适应难度 unknown 兜底）。
- KV 最终一致 → 配额近似（可能轻微超）；anti-abuse 可接受。

### 决策 6：KV 键设计 + 部署前置 + fail 矩阵

- `powseed:<seed>`（`{difficulty,challenge_ip,challenge_ua_hash,exp}`, TTL~2min, 一次性）｜`session:<sid>`（{binding_mode,ip|null,ua_hash,exp,sk}, TTL~10min）｜`sessionq:<sid>`（count, TTL~10min）｜`powrate:<client_binding_key>`（count, TTL~5min）｜`ratelimit:session:<client_binding_key>`（握手端点前置限流, **独立于查询桶**；此处 `<ip>`/`<client_binding_key>` 均指 `client_binding_key(getClientIP)` 归一键，见决策 3）。复用既有 `ratelimit:<ip>`（查询桶）、`nonce:<nonce>`。
- **握手桶隔离**：`GET /api/session/challenge`、`POST /api/session` 用独立桶 `ratelimit:session:<ip>`（IPv6 /64），与查询桶 `ratelimit:<ip>` 分离——握手限流与会话查询配额是两层不同闸，避免握手挤占查询预算、或配额被 IP 限流先行挡死。
- **`RATE_LIMIT` KV 是硬部署前置**：会话存储/PoW 防重放/配额**依赖 KV**。
- **KV 退化 fail 矩阵**（两套语义并存）：
  - **反滥用关键键 fail-closed / fail-secure**：KV **未绑** 或对反滥用键的运行时读写**失败/超时**时按各键 fail-secure：
    - seed/session/quota/nonce → 会话签发/校验、配额计数、nonce 防重放**必须** fail-closed（session 端点 503、查询 401/503），**禁止** fail-open。
    - `powrate`（自适应计数）读写失败/超时 → `difficulty` **必须**取 `DIFF_MAX`（最高档 fail-secure），**禁止**跌回 `BASE_MIN`（防攻击者诱发读失败降难度）。
    - `ratelimit:session`（握手桶）读写失败/超时 → 握手端点 fail-closed（503），**禁止**随纯 IP 限流 fail-open（它是反滥用闸）。
  - **纯 IP 限流 fail-open**：**仅**查询桶 `ratelimit:<ip>`（`checkRateLimit`）保持 fail-open（可用性优先），退化语义不变；其余反滥用键（seed/session/quota/nonce/powrate/ratelimit:session）一律 fail-closed/fail-secure。
- **实现约束（限流原语 fail 方向调用方传入，不复用查询桶 fail-open 短路）**：握手桶 `ratelimit:session` 与查询桶 `ratelimit:<ip>` 若共用同一限流原语（如 `checkRateLimit`），fail 方向**必须**由调用方按桶传入——握手桶调用显式 fail-closed（KV 未绑/读写失败→503），查询桶保持 fail-open。**禁止**握手桶复用查询桶 fail-open 默认返回值（否则诱发握手桶读失败即继承 fail-open 绕开握手限流），**禁止**原语内硬编码单一 fail 方向。
- KV 未绑 → session 端点返 503（功能不可用），查询无有效会话即拒。文档标注 KV 必绑。
- **查询拒绝码顺序不变量**：`GET /api/query`、`/api/callsigns/:callsign` 的 Layer0 纯 IP 限流查询桶 `ratelimit:<ip>`（429）**必须**先于会话校验执行——「被限流」一律先返 429（无论有无会话），「未被限流但无有效会话/伪造签名」才返 401/403（也是 smoke 4.4 哨兵「未限流=401 / 已限流=429」依据）。但该 Layer0 限流为查询桶（fail-open）：即便查询桶 fail-open **禁止**在会话校验之前短路放行——会话校验（fail-closed）**仍必须**执行并主导最终放行决定（详见决策 7 顺序不变量）。

### 决策 7：会话绑定 IP+UA 与容忍

- **`binding_mode` 显式区分**：非 unknown 会话 `binding_mode = ip`——`session.ip` 绑 `client_binding_key(getClientIP)` 归一键（非完整 IP 精确值），请求归一键 ≠ 会话 `ip` → 会话失效（防会话搬移）；unknown 会话（签发时 `getClientIP == 'unknown'`）`binding_mode = none`——`session.ip = null`、`sessionValid` 跳过 IP 比对（可跨 IP 搬移，靠 `QUOTA_unknown` + 最高难度 PoW≡`DIFF_MAX` 兜底，见决策 3/5）。
- 移动端切网（蜂窝↔WiFi）会变 IP → ip 绑定会话失效 → 客户端重走 PoW（~0.3s，可接受）。文档说明此权衡。**IPv6 按 /64 绑定**（而非 /128，即 `client_binding_key` 对 IPv6 取 /64），缓解 IPv6 隐私地址轮换对正常用户的误伤、并与 `powrate` 计数口径一致。
- `ua_hash` 绑 sha256(UA) 精确；UA 一般会话内不变。UA 绑定仅作「被动 token 泄漏后异客户端复用」的**弱信号**，**不**计入反自动化防护强度。UA 缺失头时归一为空串后再 hash。
- **UA 失配恢复路径**：请求 UA 与会话 `ua_hash` 不符 → 会话失效 → 客户端**自动重走握手取新会话**（与 IP 变更场景对称）。
- **`unknown` 来源边界**：`unknown` 是平台异常兜底（连接层缺 `CF-Connecting-IP` 或受信头非法才出现；按 `client-ip.js` 既有行为，直连无密钥时回退 `CF-Connecting-IP`、并非 `unknown`），**非攻击者可单方面诱发**。`unknown` 会话不绑 IP（`binding_mode=none`）、取最高难度档（**≡ `DIFF_MAX`**）+ 压低配额 `QUOTA_unknown`（决策 5）作为纵深，即使偶发也无搬移收益。

### 决策 8：删算术验证码清单

- worker：删 `/api/captcha` handler、`generateCaptcha`、`CAPTCHA_SECRET` 引用、`/api/config` 的 `captchaEnabled`/`features.captcha`。
- 前端：删 `components/MathCaptcha.vue`、`App.vue` 的 import/`captchaEnabled`/订阅前验证码 gate。
- 配置：`wrangler.toml.example` 删 `CAPTCHA_SECRET`。
- spec：captcha-protection 移除「算术验证码/验证码生成接口/验证码校验参数」。

### 决策 9：前端 PoW + 会话状态机（src/client）

- 启动：`GET /api/config`（无 sign_key，仅 filing/wechat/features/title）。
- 首次查询前（或页面加载）：`challenge` → 算 PoW（**Web Worker** 跑，避免阻塞 UI；降级主线程）→ `POST /api/session` → 存 {token, sk, exp} 于内存。
  - PoW 紧循环（找 nonce）用**共享同步 sha256**（`src/worker/sha256.js`，对 NIST 向量验证、与服务端 `crypto.subtle` 逐字节一致）——Web Crypto `crypto.subtle.digest` 每次 Promise 调度开销无法满足 ~0.1–0.3s 目标；协议哈希仍是 `sha256(seed + ":" + nonce)`，服务端 `verifyPow` 用 `crypto.subtle` 校验，二者结果一致。
- 查询：用 sk+token 签名发。401（过期/失效）或 429（配额尽）→ 自动重走 challenge→PoW→session 后重试一次。
- 删 MathCaptcha；`utils/sign.ts` 改用会话 sk。

### 决策 10：可测性

- PoW 验证（前导零位计数）、会话 token 签发/校验、配额、自适应 tier、查询签名（会话 sk）抽**纯函数**（如 `src/worker/session.js`），node:test 覆盖边界（难度边界、seed 重放、过期、IP/UA 不匹配、配额超限、签名错）。worker smoke 端到端（challenge→PoW→session→query 走通；无会话查询拒；配额用尽 429）。

## 风险 / 权衡

- [物理上限] PoW 只抬成本、挡不住有算力的全量爬；诚实声明（design §80），不声称杜绝。
- [移动端切网致会话失效] IP 绑定权衡——重走 PoW（cheap）；不放松 IP 绑定（防会话搬移）。
- [KV 最终一致致配额/防重放近似] Cloudflare KV 无原子 read-then-write（CAS），故 seed 一次性删、查询 `_nonce` 防重放、会话配额计数在**高并发同值请求**下均为**近似**（窄重放窗 / 配额可能轻微超）。本能力的契约即「近似 anti-abuse」而非「严格一次性」：seed 用「读后立即删」尽量收紧（重放仍各需一次 PoW，价值有限）；`_nonce`/配额并发窗内至多多放行少量同会话请求（攻击者得到的仍是其自有会话+配额内的数据，无越权）。**严格原子需 Durable Object / D1 事务**——属新子系统、本期**非目标**（only-raise-cost 模型下近似已足够）；如未来需硬上限再引入。
- [KV 未绑 = 功能不可用] 硬前置，文档标注；fail-closed（session 503）优于 fail-open（防爬形同虚设）。
- [powrate fail-secure 对称代价] `powrate` 读失败 fail-secure→`DIFF_MAX` 的对称代价＝KV 部分故障期正常用户难度短暂顶格（体验降级）；此代价换取「不可诱发降难度」。因 `powrate` 读失败**不可被攻击者定向施加于特定受害者**、且 seed/session 同时 fail-closed 会先令会话签发不可用（503），故该降级**不构成可武器化的定向 DoS**。
- [PoW 阻塞低端机 UI] Web Worker 跑 + 难度自适应保正常用户 <0.3s；难度参数可调。
- [BREAKING 前端切换] lockstep 同部署、不留静态 sign_key 兼容期（用户已定）；回滚=退 worker 版本+还原前端。
- [SESSION_SECRET 泄漏] 可伪造 token → 绕 PoW（但仍受配额+IP/UA 绑定限制）；机密管理 + 可轮换（同阶段 3-A 密钥纪律）。
- [信任根爆炸半径] `getClientIP`（经 `X-Origin-Auth` / `CDN_ORIGIN_SECRET` 判 CDN 来源）是整套**按-IP 防护**（自适应难度 + 配额归桶 + 会话 IP 绑定）的**信任根**。`CDN_ORIGIN_SECRET` 泄漏 = 攻击者直连源站自报任意真实 IP → 全体按-IP 防护失效；其与 `SESSION_SECRET` **同级**密钥纪律（Secret 管理 + 可轮换）。明确：因阿里云回源 IP 动态分配，沿用阶段 3-A 决策**不**采用 CIDR 来源网段白名单；残余风险靠密钥不可猜 + 直连无密钥时回退连接层 IP 兜底（见 `client-ip.js` 既有行为）。
- [SESSION_SECRET 爆炸半径与轮换] 伪造 token 仍受「KV **必须**命中 `session:<sid>`」硬约束（爆炸半径小于单看 HMAC 的直觉——无对应 KV 会话仍被拒）。轮换语义 = 单密钥即时失效 → 全量在途会话需重握手（反爬场景可接受），或可选双密钥灰度。KV 内明文存 `sk` + IP：KV 内容泄漏 = 全量在线会话的 `sk` + IP 暴露，列入风险面（IP 仅计数用、不入完整日志的约束不变）。
- [成本模型量化 + 硬件摊薄] 量化「抬成本」：爬 N 个呼号 ≈ `(N / 单会话配额)` 次 PoW；BASE 难度下单会话 PoW 在手机约 ~0.1–0.3s、服务器更快（量级参考）。**诚实声明**：纯 SHA-256 hashcash 对 GPU/ASIC 有数量级摊薄（与「物理上限：只抬成本不杜绝遍历」一致）。memory-hard PoW（Argon2/scrypt）属**未来可选增强、本期不引入**（避免新依赖 / 构建子系统，属非目标）。
- [_ts 时窗 vs 会话 TTL] 会话 TTL（10min）> 签名时窗（5min）：时钟漂移 > 窗 → 签名 403（客户端重签重试）；会话失效 → 401（客户端重握手）；两类客户端重试语义须区分。
- [物理上限的具体数字] 稳态抓取速率上界 ≈ `握手桶阈值（次/窗） × 单会话配额 QUOTA / 会话 TTL`（每 client_binding_key）；同一 client_binding_key 顶到 `DIFF_MAX` 后单位查询成本恒定 = `DIFF_MAX 难度 PoW / QUOTA` 次查询。apply 时按此式校准 握手桶阈值 / `QUOTA` / `DIFF_MAX`，使稳态速率落在可接受区间（具体数值见开放问题，待 apply 实测）。
- [IPv6 大前缀放大面] `client_binding_key` 仅按 /64 聚合；单户持有的超大前缀（/48、/56）分别可拆出 256、65536 个独立 /64 桶 = 同倍数的 DIFF_MAX 配额放大面（/64 聚合**未覆盖**更上层前缀）。本期**诚实声明**此放大面；若不可接受属后续加固（如按更上层前缀二级聚合），不在本期范围。
- [IPv6 /64 绑定搬移面] `client_binding_key` 把 IPv6 会话 IP 绑定从 /128 放宽到 /64，残余搬移面＝同一 /64 内不同 /128 地址可复用同一会话（绑定按 /64 视为同 IP）。该残余面被「token 不可猜 + UA 绑定 + 会话配额」兜住，且依赖运营商**不**把不同用户混租同一 /64（IPv6 标准每户独占 ≥/64，RFC 6177 / RIPE-690 模型）。本期**诚实声明**此假设，以便审计：若运营商违反「每户 ≥/64 独占」混租同一 /64，同 /64 内不同用户的会话将互可搬移（残余面随之放大）。

## 迁移计划

1. 部署前置：确认 `RATE_LIMIT` KV 已绑；`wrangler secret put SESSION_SECRET`。
2. lockstep 部署（worker 新逻辑 + 前端新 src/client 一起 `pnpm run deploy`）；`CLIENT_SIGN_KEY`/`CAPTCHA_SECRET` 退役（可留 secret 不读，后续清）。
3. 验收：正常用户查询走通（PoW 无感）、无会话/伪造签名被拒、配额用尽触发重 PoW、删 captcha 无回归。
4. 回滚：退 worker 版本 + 还原前端构建；无 D1 迁移。

## 开放问题

- PoW 难度 BASE 与自适应 tier 阈值的具体数值（apply 时按实测手机耗时定，design 给区间）。数值待 apply 实测，但**必须**落在决策 3 给出的约束区间（下限 `BASE_MIN>0` / 上限 `DIFF_MAX` / 单调递增封顶）内。
- 会话 TTL / 配额上限具体值（10min / 50 次为初值，可调）。数值待 apply 实测，但 TTL 与签名时窗的关系**必须**满足决策（会话 TTL > 签名时窗，见风险节 `_ts` 时窗）。
- token 是否需 HMAC：**已决——必须真 HMAC-SHA256**（见决策 1；token-only 快路径禁止，授权三要素与）。
