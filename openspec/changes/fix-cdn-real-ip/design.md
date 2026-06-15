## 上下文

`web_query_service`（Cloudflare Worker + D1）有**两条入口**：

1. **直连 Cloudflare 源站** `qsl.herbertgao.me` —— Cloudflare 注入的 `CF-Connecting-IP` 即真实用户 IP（Cloudflare 边缘按 TCP 层对端写入、覆盖客户端发来的同名头，此路径下不可伪造）。
2. **经阿里云 CDN** `qsl.herbert-dev.cn`（备案域名、大陆入口）回源到源站 —— Cloudflare 看到的 `CF-Connecting-IP` 是**阿里云 CDN 回源节点 IP**；真实用户 IP 由阿里云 CDN 在回源请求头中注入。

现状 `getClientIP`（`web_query_service/src/worker/index.js:43-47`）：`CF-Connecting-IP || X-Forwarded-For 首段 || 'unknown'`。问题有二：

- 经 CDN 时优先返回 `CF-Connecting-IP`（CDN 节点 IP），把成千上万真实用户**归并到少数桶** → 限流粒度失真。
- 若改成优先 XFF 首段，则**更糟**（见决策 1 的安全分析）：XFF 是 append 语义，首段是客户端可伪造值。

需要一个**按来源决定是否采信、且采信不可伪造头**的可信解析。该解析是阶段 3 防爬（per-IP 自适应难度、会话配额）正确按用户计数的前置。

约束：Worker 运行时无内置 CIDR 库；阿里云回源 IP 段会变；本变更不动 `verifySignature`、不动 D1。本解析产物只作限流计数键（抬成本），非鉴权。

## 目标 / 非目标

**目标：**
- 统一的可信真实客户端 IP 解析：双入口感知、白名单校验、采信 CDN 写入的不可伪造真实 IP 头、配置缺失 fail-safe。
- 把现有所有 IP 计数点（查询、`/api/captcha`、`/api/wechat/auth-callback`）切到该解析。
- 配置化 CDN 回源白名单 + 可信真实 IP 头名 + 文档维护说明 + 源站回源限制前置。

**非目标：**
- PoW / 短时会话 / 动态 `sign_key` / 删算术验证码（阶段 3 防爬主体，单独变更）。
- host/path → tenant 路由（阶段 4）。
- 顺丰 route-push 鉴权（推迟到「接顺丰路由推送」变更）。
- 不改签名校验、不改 nonce 防重放、不改限流阈值/窗口本身、无 D1 迁移。
- 不在 Worker 内运行时调用阿里云 API 拉回源 IP（见决策 2）。
- 不把 IP 用作访问控制——它只是限流计数键（见信任根与边界声明）。

## 信任根（整个模型的根，先声明）

- **`CF-Connecting-IP` 可信的充要条件**：请求必经且仅经 Cloudflare 边缘，`CF-Connecting-IP` 由 CF 边缘按 TCP 层对端写入并**覆盖**客户端发送的任何同名头；当前架构 Cloudflare 之前无其他代理层。本模型一切判定以此为根。
- **采信注入头的充要条件**：仅当 `CF-Connecting-IP` ∈ 受信 CDN 回源段（证明这一跳来自真正的阿里云 CDN 回源），**且**该头是 **CDN 节点按 TCP 层对端写入、覆盖（非透传）客户端发来的同名头**（推荐 `Ali-Cdn-Real-Ip`），其值才等价于真实用户 IP。「覆写而非透传」是单点信任根——若 CDN 透传客户端的该头，攻击者每请求自填即造无限自选桶（=F1 换头名）。故**头名无内置默认、未经抓包证实覆写前保持未配置**（解析 fail-safe 到 CF-IP），并加运行时校验（单值/合法 IP/不含逗号，否则落 unknown）。
- **源站回源限制前置（硬，且有顺序）**：源站 `qsl.herbertgao.me` 必须仅接受来自阿里云回源段 / Cloudflare 的回源（Cloudflare WAF/Firewall Rules）。否则攻击者可从白名单段内直连源站 + 伪造头被采信——白名单本身保证不了「请求真来自 CDN」。**中间态更危险**：「配了白名单但没配源站回源限制」比两者都没配更糟（前者让伪造头被采信，后者 fail-safe 到 CF-IP），故**必须先使源站回源限制生效、再配置白名单/启用采信注入头**。

## 决策

### 决策 1：采信阿里云 CDN 写入的真实 IP 头（头名配置化、无内置默认，推荐 `Ali-Cdn-Real-Ip`），绝不取 XFF 首段

`getClientIP(request, env)` 逻辑：
```
cf = CF-Connecting-IP
if cf 且 cf ∈ CDN回源白名单:              # 这一跳确来自受信 CDN 回源
    if 受信真实IP头名 未配置:             # 未证实覆写 = 安全态
        return cf || 'unknown'           # fail-safe 到 CF-IP，不采信任何头
    real = trim(header[受信真实IP头名])
    if real 为空 / 含逗号 / 多值 / 非合法IP字面量:
        return 'unknown'                 # 缺失或覆写假设失效 → 惩罚桶，不退回 cf（节点桶）
    return real
else:                                    # 直连 CF，或来源不可信
    return cf || 'unknown'               # 只信 CF-Connecting-IP，忽略一切注入头
```
注意「头名未配 → fail-safe 到 CF-IP」（配置未完成的安全降级）与「头名已配但本请求缺失/多值 → 'unknown'」（运行时异常/覆写失效信号）是两类回退，不可混用。

**为什么不是 XFF 首段（核心安全分析，否决原方案）**：`X-Forwarded-For` 是标准 append-only 头——最左段是链路最远端、**最不可信**的一方（通常是原始客户端自报值），每过一跳代理把它看到的上游 IP append 到**右侧**。阿里云 CDN 遵循此语义：回源时**保留**客户端发来的 XFF、向右追加它实际看到的客户端 IP。故经 CDN 路径时真实 XFF 形如 `<攻击者任填值>, <CDN看到的真实IP>, ...`，**首段完全由攻击者控制**。若采信首段，攻击者经 `qsl.herbert-dev.cn` 每请求填不同伪造首段（`CF-Connecting-IP` 命中白名单）即可落进无限自选桶 → 限流被单请求绕过，比修复前更糟。

`Ali-Cdn-Real-Ip`（阿里云官方为防 XFF 伪造而提供的头）是**单值、CDN 节点按 TCP 层对端写入**，客户端发同名头会被 CDN 覆盖、无法注入。边界：若用户在 CDN 前还套了一层代理，它会是那层代理 IP——对限流仍是合理计数单位，且不可被 HTTP 头伪造，远优于 XFF 首段。

**头名配置化、无内置默认**：受信真实 IP 头名做成配置，**无内置默认值**——未配置即视为「未证实覆写」，解析 fail-safe 到 CF-IP（不读任何注入头）；推荐值 `Ali-Cdn-Real-Ip`，经迁移步骤 4 证伪门证实覆写后由运营者**显式设置**。实现**禁止**写成 `env.受信头名 ?? 'Ali-Cdn-Real-Ip'`（默认值会让部署即采信某头、复活伪造态）。

**被否方案：**
- **XFF 首段**：见上，首段可伪造。否。
- **XFF rightmost-untrusted（从右往左跳过可信代理链路后第一个 IP）**：理论可行但要枚举并信任整条 CDN→CF 链路每一跳、CDN 内部跳数不固定，复杂脆弱。`Ali-Cdn-Real-Ip` 一步到位。否。
- 按 `Host`/域名判定信 XFF：`Host` 直连 Worker 时可任意设，攻击者直连 + 伪造 Host + 伪造头即绕过。否。
- mTLS/Authenticated Origin Pulls：阿里云 CDN→Cloudflare 不是这种关系。否。

### 决策 2：CDN 回源白名单走部署期配置（CIDR 列表），拒私网/保留/`/0`，不硬编码、不运行时拉取

新增 Worker 配置项 `CDN_ORIGIN_CIDRS`（逗号/空白分隔 CIDR 串，经 `wrangler.toml` `[vars]` 或 Secret 注入）+ 受信真实 IP 头名配置（**无内置默认**，推荐 `Ali-Cdn-Real-Ip`，经证实覆写后由运营者显式设置）。解析时切分缓存。白名单准入用**默认拒**口径：**仅接受全局可路由公网单播 IPv4 CIDR**，其余一律丢弃——私网（`10/8`、`172.16/12`、`192.168/16`）、CGNAT（`100.64.0.0/10`）、loopback（`127/8`）、link-local（`169.254/16`）、组播（`224.0.0.0/4`）、保留（`240.0.0.0/4`、`0.0.0.0/8`）、`/0`、非法前缀（非整数/>32/负/前导零）或非法 IP。黑名单枚举易漏（如 CGNAT），故用「只放行公网单播」而非「拒几个已知段」。它们落进回源白名单几乎一定是误配，会把大片来源当成可信 CDN（见风险段）。

**被否方案：**
- 硬编码回源 IP 段：阿里云会调整，硬编码即埋雷、需改码重部署。否。
- Worker 运行时调阿里云 `DescribeCdnBackSourceIp` 拉取：需在 Worker 持阿里云 AK、加网络往返与失败处理、放大攻击面。否。改为「用户用该 API/控制台导出 IP 段、填进配置、定期维护」，文档写明。

### 决策 3：在 Worker 内自实现 CIDR 成员判定（IPv4 优先，IPv6 安全降级），钉死 JS 位运算陷阱

无外部库。IPv4：把 IP 与 CIDR 网络地址各转 32 位无符号整数，按前缀位数掩码比较。**必须钉死的 JS 位运算陷阱（验收点）：**
- **`>>> 0` 无符号化**：`<<` 产生有符号 int32，首字节 ≥128 的 IP（如 `200.x.x.x`）会变负，比较前所有值必须 `>>> 0` 转无符号，否则符号位致误判。
- **`/0` 的 `<<32` no-op 陷阱**：JS 移位计数按 mod 32，`0xFFFFFFFF << 32` === `<< 0` === 原值（不是 0），朴素掩码公式对 `/0` 算反。但按决策 2，`/0` 在回源白名单里**直接拒绝**，不进入匹配——故实现上 `/0` 被配置解析阶段挡掉；CIDR 匹配函数仍须对 `prefix===0` 安全（特判或 `prefix===0 ⇒ mask=0`）防御性正确。
- 边界：`/32`（精确）、非法 CIDR/IP（跳过该条、**绝不**误判命中）。
- **IPv6**：若 `CF-Connecting-IP` 为 IPv6 或白名单含 IPv6 段——本期默认**安全降级为「不命中」**（IPv6 的 `CF-Connecting-IP` 一律视为不可信来源 → 走 fail-safe 用 `CF-Connecting-IP`、忽略注入头），**绝不**误判命中。是否实现 IPv6 前缀匹配按阿里云实际是否启用 IPv6 回源在 apply 时定；未实现即默认不命中（写死 spec）。

**被否方案：** 引入 CIDR npm 库——增包体、为一个小函数不值当。否。

### 决策 4：配置缺失/为空 ⇒ fail-safe（只信 CF-Connecting-IP）

白名单未配置或解析为空集时，决策 1 的 `cf ∈ 白名单` 恒为 false ⇒ 自动退化为「只信 `CF-Connecting-IP`、忽略一切注入头」。即默认行为安全（粗粒度但不被伪造，依赖信任根「CF-IP 由 CF 边缘写入覆盖客户端头」），不会因漏配而开伪造口子。文档标注：**未配白名单 = CDN 路径下限流仍按 CDN 节点 IP（失真但不被绕过）**，配置白名单 + 源站回源限制才恢复真实粒度。

### 决策 5：`getClientIP` 签名加 `env`，统一所有调用点

`getClientIP(request)` → `getClientIP(request, env)`（读 `env.CDN_ORIGIN_CIDRS` 与受信头名）。改所有调用点：查询端点（约 572）、`/api/captcha`（约 542）、`/api/wechat/auth-callback`（约 736，从直接读 `CF-Connecting-IP` 改为调 `getClientIP`）。`auth-callback` 原注释「IP 取自不可伪造的 CF-Connecting-IP」一并更正。改造后**禁止** worker 内残留 `getClientIP` 之外的 `CF-Connecting-IP`/`X-Forwarded-For` 直读（回归 grep 断言）。

### 决策 6：可测性——CIDR/IP 解析抽为纯函数 + worker smoke 双轨验证

`getClientIP` 的取值与 CIDR 匹配抽为**纯函数**（输入 headers + 配置 → IP 字符串），用 node 单测覆盖黑盒打不到的边界（`/0` 拒绝、`/32`、非法条目跳过、高位 IP 符号位、IPv6 不命中、伪造 XFF/伪造头不被采信）。worker smoke（`run_worker_smoke.sh`）补端到端断言（RC 实测已证：miniflare 透传 `CF-Connecting-IP`、`wrangler dev --local` 下 `RATE_LIMIT` KV 真实计数到 429）——需在 `start()` 注入 `CDN_ORIGIN_CIDRS`、段间清 KV、按真实/伪造头断言限流键归桶。

## 风险 / 权衡

- [`Ali-Cdn-Real-Ip` 覆写是单点信任根] → 若 CDN 透传而非覆写客户端的该头，则经 CDN 路径攻击者自填即被采信（=F1 换头名），且 fail-safe **不**兜底此根错误。缓解：头名无内置默认、未经抓包证实覆写前保持未配置（=安全态）；运行时校验单值/合法 IP/不含逗号（多值=覆写失效信号→unknown）；抓包门用证伪式判据（见迁移步骤 4）；CDN 配置变更后须重跑证实。
- [阿里云回源 IP 段变更致白名单陈旧——**两类**] →（a）**缺失新段**：降级为按 CDN 节点 IP 计数（失真但不被绕过，fail-safe 兜底，非安全事故）；（b）**保留废弃/过宽段**：=安全洞，任意命中该段的请求都被当可信 CDN、采信其注入头。缓解：决策 2 默认拒（只放行公网单播）；文档强制「宁缺勿宽 + 最小化 + 定期校验 + 变更需人工 sign-off」；来源用阿里云官方回源 IP 列表/`DescribeCdnBackSourceIp`；信任根 R3「源站仅允许 CDN/CF 回源」前置消灭「从白名单段直连源站」。
- [配置中间态比都不配更危险] → 「配了白名单但没配源站回源限制」会让伪造头被采信（而都不配时 fail-safe 到 CF-IP）。缓解：强制顺序——先使源站回源限制生效、再配白名单/启用采信注入头（迁移步骤有依赖序，非并列）。
- [直连源站旁路致限流预算叠加] → 攻击者可同时走「直连（按 CF-IP 计数）」+「经 CDN（按真实 IP 计数）」拿两份预算。这是限流=抬成本（非访问控制）的已知接受范围内弱化；信任根 R3「源站仅允许 CDN 回源」前置可消灭直连旁路、单一计数来源。文档化为 accepted。
- [命中白名单但真实 IP 头缺失/多值] → 决策 1 回退 `'unknown'` 惩罚桶（不退回 CDN 节点 IP），避免坍缩进节点大桶。`Ali-Cdn-Real-Ip` 正常一定由 CDN 注入单值，缺失/多值属异常，进惩罚桶合理。
- [`'unknown'` 为全局共享桶] → 大量「头缺失」回退请求挤同一惩罚桶，攻击者无法借此扩大自身预算，但可打满该桶致合法的「头缺失」请求被拒。属可接受 fail-safe 副作用（DoS 面仅限本就异常的请求子集）。
- [IPv6 回源未覆盖] → 安全降级为「IPv6 不命中 ⇒ 用 CF-Connecting-IP」，最坏 IPv6 经 CDN 计数失真，绝不被伪造绕过。apply 时确认阿里云是否启用 IPv6 回源再定实现深度。
- [限流键变更影响既有计数] → 仅改 key 取值，KV 结构/TTL/阈值不变；切换瞬间旧 key 计数作废、按新 key 重新计窗，无数据迁移、无可用性风险。
- [`checkRateLimit` 在 `RATE_LIMIT` KV 未绑定时 fail-open] → 既有行为（本变更不动），accepted：在「限流=抬成本非鉴权」边界下可接受，且本变更不改阈值/窗口/可用性取向（KV 为部署前置，未绑则不限流而非拒服务）。smoke 的「真计数到 429」断言在 KV 已绑时隐式守护此点。

## 迁移计划

1. 合并 + 部署 Worker（`pnpm run deploy`）——此时**受信头名未配**，解析对所有路径 fail-safe（CDN 路径按 CDN 节点 IP，失真但不被绕过；行为安全）。
2. **源站回源限制先行（硬前置）**：在 Cloudflare 配 `qsl.herbertgao.me` 仅接受阿里云回源段 / CF 的回源（WAF/Firewall Rules）。**必须在配白名单之前完成**（否则进入「白名单生效但任意人可直连源站落进白名单段」的危险中间态）。
3. 配 `CDN_ORIGIN_CIDRS`（阿里云回源公网 IP 段）。此时经 CDN 路径命中白名单但受信头名仍未配 → 仍 fail-safe 到 CF-IP（安全），尚未采信任何注入头。
4. **上线前阻断门（启用采信注入头的前置，证伪式，针对最终要配置的那个头名，必做）**：从已知出口 IP 的机器经 `qsl.herbert-dev.cn` 发请求，故意带**伪造的目标头（含大小写变体，如 `Ali-Cdn-Real-Ip: 8.8.8.8` + `ali-cdn-real-ip: 7.7.7.7`）**与伪造 `X-Forwarded-For: 9.9.9.9`，在 Worker 临时日志打印**两者**：(a) `request.headers.get('<配置头名>')` 原始值；(b) `resolveClientIP` 的最终返回值。**门唯一通过判据**：(b) = 该机器真实出口 IP、**不等于** 8.8.8.8/7.7.7.7 任一伪造值、**且为单值**（仅此证该头名被 CDN 覆写）。**其余结果皆门失败**（该头名被透传或透传+追加、不可用）：若 (a) 含逗号/多值（CDN 把伪造值追加进来、未覆写）→ (b)=`'unknown'`，**这是失败信号、不是「覆写成立」**；若 (b)=任一伪造值（CDN 透传单值未覆写）→ 失败。注意：**运行时无法区分「CDN 覆写的单值」与「客户端透传的同名单值」**——故覆写只能靠此门的「(b)≠伪造值」实证，运行时多值校验只是纵深防御。另用一个 CDN 不注入的头名跑同样请求作对照，观察 (b) 取到攻击者值/落 unknown，实证「未被覆写的头名不可用」。**仅在该头名通过上述唯一判据后**，才把受信真实 IP 头名显式设为该证实可信的头（如 `Ali-Cdn-Real-Ip`）。
5. 验收：构造经 CDN（CF-IP ∈ 白名单 + 伪造 XFF + 伪造/真实 `Ali-Cdn-Real-Ip`）与直连（伪造头）两类请求，断言限流键取值正确（见 tasks）。
6. **回滚**：Cloudflare dashboard 退回上一 Worker 版本 + 移除/还原配置项（尤其清空受信头名 → 立即 fail-safe）。纯服务端逻辑，无 D1 迁移、桌面端/前端零改动。

## 开放问题

- 受信真实 IP 头的最终确认由迁移计划步骤 4 的证伪式抓包门给出（推荐 `Ali-Cdn-Real-Ip`，以实测为准，未证实前头名不配）；AI 不碰生产配置。
- 确认阿里云是否启用 IPv6 回源（决定决策 3 的 IPv6 实现深度）。
- 回源 IP 段的具体 CIDR 清单由用户从阿里云获取后填入。
