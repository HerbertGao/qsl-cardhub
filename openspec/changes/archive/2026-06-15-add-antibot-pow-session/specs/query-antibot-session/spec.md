## 新增需求

### 需求：PoW 门票下发

系统**必须**提供 `GET /api/session/challenge`，下发工作量证明（PoW）题目 `{seed, difficulty}`。`seed` **必须**为高熵随机值、**一次性**（经 `RATE_LIMIT` KV 记录、防重放）；`difficulty`（hashcash 前导零 bit 数）**必须**按发起方真实 IP 自适应（见「自适应难度」）。题目 TTL **必须**有限（如 ~2min）。

`seed` 在 KV 记录的内容**必须**包含 `{difficulty, challenge_ip, challenge_ua_hash, exp}`：`challenge_ip` **必须**为下发题目时真实 IP 的归一键 `client_binding_key(getClientIP)`（见「自适应难度」对 `client_binding_key` 的定义），`challenge_ua_hash` **必须**为 `sha256(下发题目时的 User-Agent)`——以便兑换时校验「领题者 === 兑换者」，防「低难度 IP 领题→他处兑换」绕开自适应难度。

握手端点（`GET /api/session/challenge`、`POST /api/session`）**必须**有按 `getClientIP`（IPv6 按 /64 前缀，见「自适应难度」）的前置 IP 限流，且**必须**使用与查询端点限流分离的独立计数桶（如 `ratelimit:session:<ip>`），**禁止**与查询桶共用——使握手限流与会话查询配额是两层不同闸，避免握手挤占查询预算、或配额机制被 IP 限流先行挡死。

> 说明：本能力面向**不登录**的公开查询，目标是**抬高全量爬库成本**（爬全库成本 ≈ 会话数 × PoW），**不**声称杜绝全量遍历（物理上限）。

#### 场景：下发 PoW 题

- **当** 客户端请求 `GET /api/session/challenge`
- **那么** 服务端**必须**返回 `{seed, difficulty}`，并在 KV 记录该 `seed`（一次性、带 TTL）
- **并且** `difficulty` **必须**随发起方真实 IP 的近期建会话频率自适应升高

### 需求：PoW 验证与会话签发

系统**必须**提供 `POST /api/session`，校验 PoW 后签发短时会话。校验**必须**：① 该 `seed` 在 KV 存在（不存在=过期/重放→拒）；② **读后立即删**该 seed（一次性、防并发重放）；③ `sha256(seed + ":" + nonce)` 的二进制前导零位数 ≥ 该 seed 记录的 `difficulty`；④ 当前请求真实 IP 的归一键 `client_binding_key(getClientIP)` **必须**等于 `seed.challenge_ip`，且 `sha256(当前请求 UA)` **必须**等于 `seed.challenge_ua_hash`，不等则拒（防「低难度 IP 领题→他处兑换」绕开自适应难度；按归一键比对使同一 /64 内 IPv6 隐私地址轮换不误拒）。校验通过**必须**签发会话并返回 `{token, sk, exp, quota}`（`sk`=该会话专属签名密钥，经响应体 HTTPS 下发）。

#### 场景：PoW 正确则签发会话

- **当** `POST /api/session` 携带的 `{seed, nonce}` 满足 PoW 难度，且 `seed` 未被用过
- **那么** 服务端**必须**签发会话 token + 会话签名密钥 `sk` + 过期时间 + 配额，并将 `seed` 标记为已用（删除）

#### 场景：PoW 不足或 seed 重放被拒

- **当** `sha256(seed+":"+nonce)` 前导零位数 < `difficulty`，或 `seed` 已被用过/已过期
- **那么** 服务端**必须**拒绝签发（如 400/403），**禁止**发放会话
- **并且** challenge 与兑换的真实 IP/UA 不一致（当前 `client_binding_key(getClientIP)` ≠ `seed.challenge_ip`，或 `sha256(当前 UA)` ≠ `seed.challenge_ua_hash`）时也**必须**拒绝签发

### 需求：短时会话与绑定

会话**必须**短时（TTL ~10min）、存于 `RATE_LIMIT` KV（`session:<sid>` → `{binding_mode, ip|null, ua_hash, exp, sk}`，其中 `binding_mode ∈ {ip, none}`；`ip` 字段在 `none` 模式为 `null`）。会话 token **必须**用真 HMAC-SHA256（`crypto.subtle` HMAC），**禁止** `sha256(secret + msg)` 形式（防长度扩展攻击）。`sid` 字符集**必须**限定为 hex 或 base64url（**禁止**含 `.`），token 解析**必须**用 `lastIndexOf('.')` 切分 `sid` 与 HMAC 以避免歧义。

会话绑定**必须**按签发时 `getClientIP` 是否为 `unknown` 分为两种 `binding_mode`：

- **非 unknown 会话必须 IP 绑定**（`binding_mode = ip`）：会话**必须**绑定签发时的**真实客户端 IP 的归一键** `client_binding_key(getClientIP)`（`getClientIP` 取自 `trusted-client-ip`，非裸 `CF-Connecting-IP`；归一键定义见「自适应难度」）——即 `session:<sid>` 的 `ip` 字段存 `client_binding_key(getClientIP)` 归一键、校验时**必须** `client_binding_key(getClientIP) === session.ip`（均为归一键，**禁止**存/比完整 `getClientIP` 精确相等），使同一 /64 内 IPv6 隐私地址轮换不误杀会话、并与 `powrate` 计数口径一致。
- **unknown 会话**（签发时 `getClientIP == 'unknown'`）**必须**置 `binding_mode = none`（如 `session.ip = null` 并标记 `binding_mode = none`），其 `sessionValid` **必须**跳过 IP 比对，仅校验 token HMAC + KV 命中 `session:<sid>` + `exp` 未过期 + `ua_hash` + 会话签名 + 压低配额 `QUOTA_unknown`；unknown 会话因此可跨 IP 搬移，靠 `QUOTA_unknown`（≤ 3）+ 最高难度 PoW（≡ `DIFF_MAX`）兜底，使搬移价值趋零。

会话**必须**绑定 `sha256(User-Agent)`（两种 binding_mode 均绑），UA 缺失头时**必须**归一为空串后再 hash。

token 授权**必须**同时满足三者之与：① HMAC 有效 ∧ ② KV 命中 `session:<sid>` ∧ ③ 绑定项（`exp`/`ip`/`ua`）全部通过——**禁止**任何「HMAC 通过即放行」的 token-only 快路径。

#### 场景：会话校验绑定项

- **当** 携带会话 token 的请求到达
- **那么** 服务端**必须**校验：token HMAC 完整/未伪造、KV 命中 `session:<sid>`、会话未过期、请求真实 IP 的归一键 `client_binding_key(getClientIP)` === 会话绑定 `ip`（归一键）、`sha256(请求 UA)` === 会话绑定 ua_hash
- **并且** 当会话为 unknown 会话（`binding_mode = none`，签发时 `getClientIP == 'unknown'`）时，`sessionValid` **必须**跳过 IP 比对，其余项（HMAC、KV 命中、`exp`、`ua_hash`）照验
- **并且** 任一不满足**必须**判会话无效并拒绝（如 401），**禁止**放行
- **并且** **禁止**仅凭 HMAC 通过就放行（KV 命中与绑定项全过为硬约束）

#### 场景：UA 失配致会话失效

- **当** 请求 UA 与会话绑定 `ua_hash` 不符
- **那么** 该会话**必须**判失效并拒绝（与 IP 变更场景对称）
- **并且** 客户端**必须**自动重走 PoW 取新会话（给出客户端恢复路径）

#### 场景：换网致 IP 变更会话失效

- **当** 用户网络切换（蜂窝↔WiFi）致真实 IP 变化、其归一键 `client_binding_key(getClientIP)` 与会话绑定 `ip` 不符
- **那么** 该会话**必须**判失效，客户端**必须**重新走 PoW 取新会话（防会话搬移；PoW 成本低、可接受）

### 需求：查询侧会话校验与动态签名

`GET /api/query`、`GET /api/callsigns/:callsign` **必须**改为校验「有效会话 token + 会话签名 + 会话配额」，**取代**静态 `CLIENT_SIGN_KEY` 签名。查询签名**必须**用 `HMAC-SHA256(sk, canonicalPayload)`（`sk`=会话专属密钥，非任何静态/可公开密钥；**禁止** `sha256(payload + sk)` 形式）。

`canonicalPayload` **必须**无歧义，字段定义精确——**必须**按**固定顺序**串接以下五项：① 会话 `sid`（纳入 `sid` 而非完整 token，避免 client/worker 拼装漂移）；② 路径——**必须**为原始 `url.pathname`（**保留 URL 编码形态**，如 `%2F`/`%0A` 不解码），client 与 worker **必须**同口径（同属上文共用的单一 canonical 模块），避免 `:callsign` 段在一端解码、另一端不解码致签名不符（自我 DoS）；③ 本次请求**实际携带的业务查询参数**按 key 字母序排列后的串（**排除 `_sig`、`_ts`、`_nonce`**——`_sig` 是签名输出本身，`_ts`/`_nonce` 由字段④⑤专门承载，避免重复纳入）；④ `_ts`；⑤ `_nonce`。即 `_ts`、`_nonce` **必须**纳入签名输入（把新鲜度/防重放绑定进 HMAC，**禁止**换 ts/nonce 后复用旧 `_sig`），但**仅**经字段④⑤ 承载、**禁止**也出现在字段③；路径式端点（如 `/api/callsigns/:callsign`）无业务参数时字段③ 为空串（`_ts`/`_nonce` 仍由④⑤承载）。

序列化规则**必须**确定且无歧义：五项之间用换行符 `\n` 连接；查询参数串的每个 key 与 value **必须各自**用 `encodeURIComponent`（或等价、保证 `&`/`=`/`\n`/`%` 均被百分号编码的编码）后以 `=` 拼接 key 与 value、以 `&` 连接各对（按 key 字母序）。**必须**断言：编码后的参数串中除作为结构分隔符的 `&`/`=` 外，**禁止**出现裸 `&`/`=`/`\n`（使分隔符注入在规范层不可能）。client 与 worker **必须**采用同一实现（单一事实源，**禁止**两处各写一份易漂移的拼装逻辑）——`canonicalPayload` **必须**为 worker 与 client 共用的**单一事实源模块**（**禁止**两处各写易漂移实现）；落法**必须**保证 `pnpm run build`（`vue-tsc --noEmit`）仍绿——采用**共享 `.js` 模块 + 同名手写 sibling `.d.ts`**（worker `import` 该 `.js`、client `utils/sign.ts` re-export 该 `.js`，`.d.ts` 提供类型，**无需**改 tsconfig），或等价地启用 `allowJs` 并将该模块路径纳入 `tsconfig` `include`（二选一，apply 时择一）；**禁止**在 client 端另写一份独立 canonical 实现。

`_nonce` 字符集**必须**限定为 hex 或 base64url（与 `sid` 同），使其在 `canonicalPayload` 字段⑤ 与 `nonce:<nonce>` KV 键中天然无分隔符注入面。

服务端由 token→`sid`→KV 取 `sk` 验签，并校验 `_ts` 时窗 + `_nonce` 经 KV 防重放。`/api/config` **禁止**再下发任何查询签名密钥。

#### 场景：带有效会话与会话签名的查询

- **当** 查询请求携带有效会话 token、合法会话签名（用 `sk`）、`_ts` 在时窗内、`_nonce` 未用过、会话配额未尽
- **那么** 服务端**必须**放行并返回查询结果，并消耗一次配额

#### 场景：无会话/伪造签名的查询被拒

- **当** 查询请求无会话 token、token 无效/过期、或签名用非会话 `sk`（如旧静态 key）算出
- **那么** 服务端**必须**拒绝（401/403），**禁止**返回卡片数据
- **并且** `GET /api/query`、`GET /api/callsigns/:callsign` 的拒绝码优先级**必须**为：Layer0 纯 IP 限流查询桶 `ratelimit:<ip>`（429）**先于**会话校验——即「被限流」一律先返 429（无论有无会话），「未被限流但无有效会话/伪造签名」才返 401/403（此顺序是 smoke「未限流=401 / 已限流=429」哨兵的规范依据）

### 需求：会话查询配额

每个会话**必须**有查询次数上限（如 10min / 50 次），计数存 KV（`sessionq:<sid>`）。配额用尽**必须**拒绝后续查询（如 429），客户端**必须**重走 PoW 取新会话。

#### 场景：配额用尽触发重新 PoW

- **当** 某会话查询次数达上限
- **那么** 服务端**必须**对该会话后续查询返回配额用尽（429）
- **并且** 客户端**必须**重新走 challenge→PoW→新会话后才能继续查询

### 需求：自适应难度

PoW 难度**必须**按发起方**真实 IP**（`getClientIP`）的近期建会话频率自适应：正常用户低难度（手机 ~0.1–0.3s 无感）；同一真实 IP 短时大量建会话**必须**升高难度（使批量建会话成本快速上升）。频率计数 IP **必须**取自真实 IP（非 CDN 节点 IP）。

本能力**必须**定义并统一使用单一 IP 归一键函数 `client_binding_key(ip)`：IPv4 → 完整地址（/32，直通原值）；IPv6 → **/64 前缀**（取前 64 位归一）；`unknown` → `unknown`（直通）。`seed.challenge_ip`、`session:<sid>.ip`、`powrate` 计数桶、`ratelimit:session` 握手桶，以及兑换/查询时的 IP 比对，**全部**以 `client_binding_key(getClientIP)` 为键（存与比都用归一键，**禁止**用完整 `getClientIP` 精确相等），使会话绑定与频率计数口径同源、同一 /64 内 IPv6 隐私地址轮换不误杀。

频率计数键 `powrate`（key=`client_binding_key(getClientIP)`）**必须**对 IPv6 按 **/64 前缀**聚合、对 IPv4 按完整地址（/32）——否则攻击者可用同一 /64 内海量地址，每个地址只建一次会话即永远命中最低难度，使「同一真实 IP 短时大量建会话必须升难度」对 IPv6 不可满足。

`difficulty` **必须**有正下限 `BASE_MIN`（`BASE_MIN > 0`，保证始终存在真实 PoW，**禁止** `difficulty = 0` 致免 PoW）与上限封顶 `DIFF_MAX`；`DIFF_MAX` **必须**保证封顶难度下手机仍可在合理时间解出，避免对共享出口 IP 的正常用户造成不可解 DoS。

当 `getClientIP` 返回 `unknown`（连接 IP 缺失/受信头非法）时，会话签发**必须**对 `unknown` 来源采用最高难度档（该最高难度档**必须** ≡ `DIFF_MAX`，即取封顶值，消除「最高难度档 < `DIFF_MAX`」致 unknown 比顶格 IP 便宜的歧义）且**不**依赖 IP 绑定（仅靠 UA + 配额约束）；`unknown` **必须**使用独立计数桶，**禁止**污染任何具体 IP 桶。由于 `unknown` 会话不绑 IP（可跨 IP 搬移），其查询配额 `QUOTA_unknown` **必须**大幅压低（如 ≤ 3，远小于常规 `QUOTA`≈50），使「一次最高难度 PoW」只换极少次查询、搬移价值趋零。

> 说明：`unknown` 是平台异常兜底——连接层缺 `CF-Connecting-IP` 或受信头非法时才出现；按 `trusted-client-ip`/`client-ip.js` 既有行为，直连且无密钥时**回退** `CF-Connecting-IP`（非 `unknown`），故 `unknown` **非攻击者可单方面诱发**。压低配额作为纵深，即使偶发也无搬移收益。

#### 场景：高频建会话升难度

- **当** 同一真实 IP 在窗口内建会话频率超过阈值
- **那么** 该 IP 后续 `challenge` 的 `difficulty` **必须**升高
- **并且** 正常低频用户的 `difficulty` **必须**保持低位（移动端 PoW 无感）
- **并且** `difficulty` **必须**始终 ≥ `BASE_MIN`（>0，恒有真实 PoW）且 ≤ `DIFF_MAX`（封顶，手机可解）

#### 场景：IPv6 按 /64 聚合频率计数

- **当** 来自同一 IPv6 /64 前缀内的不同地址在窗口内大量建会话
- **那么** 频率计数**必须**按 /64 前缀聚合归入同一桶，使该前缀整体命中升难度
- **并且** **禁止**因每个 /128 地址各自只建一次会话而永远停留在最低难度

#### 场景：真实 IP 为 unknown 的兜底

- **当** `getClientIP` 返回 `unknown`（连接 IP 缺失或受信头非法）
- **那么** 会话签发**必须**对该来源采用最高难度档（**必须** ≡ `DIFF_MAX`），且**不**依赖 IP 绑定（仅靠 UA + 配额约束）
- **并且** `unknown` **必须**计入独立计数桶，**禁止**污染任何具体 IP 桶
- **并且** 该会话查询配额**必须**取大幅压低的 `QUOTA_unknown`（如 ≤ 3，远小于常规 `QUOTA`），使一次最高难度 PoW 仅换极少次查询、跨 IP 搬移无收益

### 需求：防爬范围与物理上限声明

本能力**必须**明确：在「不登录、公开查询」前提下，PoW+会话+配额+动态签名**只抬高全量爬库成本、不杜绝全量遍历**（物理上限）。本能力产物（会话/IP 计数）仅用于**抬成本**，**禁止**作为访问控制/鉴权的唯一闸门误用。`RATE_LIMIT` KV 是本能力的**硬部署前置**（会话存储/PoW 防重放/配额依赖 KV）；KV 未绑定时会话相关端点**必须** fail-closed（如 503），**禁止**因 KV 缺失而静默放行无会话查询。

fail 策略矩阵**必须**两套语义并存且各自明确：

- **反滥用关键键 fail-closed / fail-secure**：当 `RATE_LIMIT` KV **未绑定**，或对反滥用键的运行时读写**失败/超时**时，**必须**按各键 fail-secure，**禁止** fail-open 放行：
  - `seed` / `session` / `quota`（`sessionq`）/ `nonce`：会话签发与校验、配额计数、查询签名 nonce 防重放对该请求**必须** fail-closed（会话端点 503、查询 401/503）。
  - `powrate`（自适应计数）读写**失败/超时** → `difficulty` **必须**取 `DIFF_MAX`（最高档，fail-secure），**禁止**跌回 `BASE_MIN`（防攻击者诱发读失败降难度）。
  - `ratelimit:session`（握手桶）读写**失败/超时** → 握手端点（`GET /api/session/challenge`、`POST /api/session`）**必须** fail-closed（503），**禁止**随纯 IP 限流的 fail-open 语义放行（它是反滥用闸）。
- **纯 IP 限流 fail-open**：与之相对，**仅**纯 IP 限流查询桶 `ratelimit:<ip>`（`checkRateLimit`）保持 fail-open（可用性优先），其退化语义不变；其余反滥用键（seed/session/quota/nonce/powrate/ratelimit:session）一律 fail-closed/fail-secure。

**实现约束（限流原语 fail 方向由调用方按桶传入）**：限流原语的 fail 方向**必须**由调用方按桶传入策略决定——握手桶（`ratelimit:session`）调用**必须**显式 fail-closed（KV 未绑 / 读写失败/超时 → 503），查询桶（`ratelimit:<ip>`）保持 fail-open。**禁止**让握手桶复用查询桶 fail-open 的默认返回值（否则攻击者诱发握手桶读失败即继承 fail-open、绕开握手限流），**禁止**在限流原语内硬编码单一 fail 方向。

**顺序不变量（会话校验主导）**：查询桶 IP 限流 fail-open **禁止**在会话校验之前短路放行——即便查询桶 `ratelimit:<ip>` fail-open，会话校验（fail-closed）**仍必须**执行并主导最终放行决定。**禁止**实现者为性能把 fail-open 短路提前到会话校验之前致绕过。

#### 场景：KV 未绑定时 fail-closed

- **当** `RATE_LIMIT` KV 未绑定
- **那么** 会话签发/校验**必须** fail-closed（会话端点 503 / 无有效会话的查询被拒），**禁止**静默放行

#### 场景：反滥用关键键运行时读写失败时 fail-closed

- **当** 对 seed / session / quota / nonce 键的 KV 读写在运行时失败或超时
- **那么** 会话签发/校验、配额计数、nonce 防重放对该请求**必须** fail-closed（会话端点 503、查询 401/503），**禁止** fail-open 放行
- **并且** `powrate`（自适应计数）读写失败/超时时 `difficulty` **必须**取 `DIFF_MAX`（fail-secure，**禁止**跌回 `BASE_MIN`）
- **并且** `ratelimit:session`（握手桶）读写失败/超时时握手端点**必须** fail-closed（503）
- **并且** 纯 IP 限流查询桶 `ratelimit:<ip>`（`checkRateLimit`）**必须**仍保持 fail-open（可用性优先），两套语义互不混淆

#### 场景：两桶共用限流原语 fail 语义不串味

- **当** `ratelimit:session`（握手桶）的 KV 读写失败/超时
- **那么** 握手端点（`GET /api/session/challenge`、`POST /api/session`）**必须** fail-closed 返回 503
- **并且** 当 `ratelimit:<ip>`（查询桶）同时读写失败/超时时，查询端点**必须**仍 fail-open 放行（不因共用限流原语而继承握手桶 fail-closed，亦不令握手桶继承查询桶 fail-open）

### 需求：客户端会话状态机（并发与重试约束）

客户端的握手与查询逻辑**必须**满足以下并发与重试约束：客户端**必须** singleflight 进行中的握手——并发查询**必须**共享同一握手 promise，**禁止**重复 PoW 或互相覆盖全局凭据。每次查询**必须**读取一份不可变的 `{token, sk, exp}` 快照（**禁止**查询过程中读到被其他流程改写的半态凭据）。遇 401/429 **必须**仅使**匹配的旧会话**失效、重走握手后**至多重试一次**。无有效会话时**必须**先完成握手、**禁止**裸发查询。重试耗尽仍失败**必须**给用户可感知的降级提示。

#### 场景：并发查询共享单次握手

- **当** 多个查询在无有效会话时并发发起
- **那么** 客户端**必须** singleflight 握手（共享同一握手 promise），**禁止**重复 PoW 或互相覆盖全局凭据
- **并且** 每个查询**必须**基于握手完成后不可变的 `{token, sk, exp}` 快照发起

#### 场景：会话失效/配额用尽的有限重试

- **当** 某查询收到 401（会话失效）或 429（配额用尽）
- **那么** 客户端**必须**仅使匹配的旧会话失效、重走握手后**至多重试一次**
- **并且** 重试耗尽仍失败**必须**向用户给出可感知的降级提示

#### 场景：无会话时禁止裸发查询

- **当** 当前无有效会话快照
- **那么** 客户端**必须**先完成握手再发查询，**禁止**裸发查询
