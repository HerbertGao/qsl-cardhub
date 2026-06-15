## 上下文

`web_query_service`（Cloudflare Worker + D1）有**两条入口**：

1. **直连 Cloudflare 源站** `qsl.herbertgao.me` —— `CF-Connecting-IP` 即真实用户 IP（CF 边缘按 TCP 层对端写入、覆盖客户端同名头，此路径不可伪造）。
2. **经阿里云 CDN** `qsl.herbert-dev.cn`（备案域名、大陆入口）回源到源站 —— `CF-Connecting-IP` 是 CDN 回源节点 IP；真实用户 IP 由阿里云 CDN 注入回源头。

现状 `getClientIP`：`CF-Connecting-IP || X-Forwarded-For 首段 || 'unknown'`。问题：经 CDN 时把成千上万真实用户归并到少数 CDN 节点 IP 桶（限流粒度失真）；而优先 XFF 首段更糟（append 语义、首段客户端可伪造）。需要一个**可信判定「请求是否来自 CDN」**的解析，是阶段 3 防爬按用户计数的前置。

约束：本解析产物仅作限流计数键（抬成本），非鉴权；不动 `verifySignature`、不动 D1。

## 目标 / 非目标

**目标**：统一可信真实客户端 IP 解析（双入口感知、以密钥头判来源、采信 CDN 写入的不可伪造真实 IP 头、配置缺失 fail-safe）；切换现有 IP 计数点；配置 + 文档 + 验证。

**非目标**：PoW/会话/动态 sign_key/删验证码（阶段 3 主体）；host/path 路由（阶段 4）；route-push 鉴权（推迟）；不改签名/nonce/阈值；无 D1 迁移；IP 不作访问控制。

## 决策

### 决策 1：信任信号 = CDN 注入的密钥回源头 `X-Origin-Auth`，不用回源 IP 白名单

阿里云官方文档证实：**回源 IP 动态分配、不建议在源站设固定回源 IP 白名单**（"否则可能导致回源失败"），查询回源 IP 的接口（`DescribeL2VipsByDomain`）还有日峰值带宽 ≥1Gbps + 工单门槛。故 IP 白名单方案对本项目**落不了地**。改用密钥头：

- 阿里云 CDN 用「修改出站请求头」以**覆盖语义**（「增加 + 不允许重复」或「替换」）注入固定密钥 `X-Origin-Auth: <CDN_ORIGIN_SECRET>`——客户端伪造的同名头在回源时被覆盖，攻击者也猜不到密钥。
- worker 持期望密钥 `env.CDN_ORIGIN_SECRET`，对请求的 `X-Origin-Auth` 做**常量时间**比对。等于 → 确来自 CDN → 采信真实 IP 头；不等于/缺失 → 直连或伪造 → 只用 `CF-Connecting-IP`、忽略注入头。

`resolveClientIP(headers, originSecret, realIpHeaderName)` 分支：
```
cf = CF-Connecting-IP
if !cf: return 'unknown'
if originSecret 空: return cf                       # 未启用采信 = 安全态
if !timingSafeEqual(headers['X-Origin-Auth'], secret): return cf   # 无/错密钥 → 只信 cf，忽略注入头
if realIpHeaderName 未配: return cf                 # 头名待证伪门确认
real = headers[realIpHeaderName]
return isTrustedHeaderValueValid(real) ? real.trim() : 'unknown'   # 缺失/多值/非法 → 惩罚桶，不退 cf 节点桶
# 绝不读 X-Forwarded-For
```

**被否方案**：①回源 IP 白名单（CDN_ORIGIN_CIDRS + Cloudflare WAF IP List）——阿里云回源 IP 动态、不可稳定白名单，否。②XFF 首段——append 语义、首段可伪造，否。③Cloudflare WAF 单独把关（不改代码）——worker 仍需自身判据，且 WAF 单点；让 worker 直接验密钥更稳、且省掉 WAF/IP List 整套。

### 决策 2：真实 IP 头采信 `Ali-Cdn-Real-Ip`（无内置默认 + 运行时校验）

阿里云 CDN **默认携带** `Ali-Cdn-Real-Ip` = 「客户端与 CDN 节点建连时的真实 IP」，官方明确**为避免 X-Forwarded-For 伪造**而设。`CDN_REAL_IP_HEADER` 配为它。

- **无内置默认**：未配 → fail-safe 到 CF-IP；实现禁 `env.CDN_REAL_IP_HEADER ?? 'Ali-Cdn-Real-Ip'`（默认值会让部署即采信某头）。证伪门确认覆写后才显式配。
- **运行时校验**（纵深防御，非充分）：采信前校验单值/合法 IP 字面量（IPv4/IPv6）/不含逗号；多值/含逗号/非法 → `'unknown'`。**禁 split 取段**（否则透传 `8.8.8.8,<真实>` 首段过校验=伪造复现）。**但单值透传运行时无法识别**——覆写保证由部署期证伪门承担。

### 决策 3：密钥常量时间比对 + 机密管理

`timingSafeEqualStr`（长度不同直接 false，逐字符 XOR 累积）避免计时侧信道。`CDN_ORIGIN_SECRET` 经 `wrangler secret` 注入、禁进仓库、禁经 `/api/config` 下发（与公开的 `CLIENT_SIGN_KEY` 区分）。回源协议 HTTPS 使密钥不走明文。

### 决策 4：`getClientIP(request, env)` 统一所有调用点

签名 `getClientIP(request, env)` 读 `env.CDN_ORIGIN_SECRET`/`CDN_REAL_IP_HEADER`。改查询（约 535）、`/api/captcha`（约 565）、`/api/wechat/auth-callback`（约 729，原直读 CF-IP）；回归 grep 确认无 `getClientIP` 之外的 CF-IP/XFF 直读。

### 决策 5：可测性——纯函数单测 + worker smoke

`resolveClientIP`/校验函数抽纯函数，node:test 覆盖黑盒打不到的边界。worker smoke 4.4 端到端断言归桶（有效密钥/无密钥/错误密钥三类），经测试 `[vars]` toml 注入密钥 + 段间清 KV。

## 风险 / 权衡

- [密钥泄漏] → 同时存阿里云 CDN 配置 + Cloudflare secret 两处；收紧控制台权限；应急轮换路径（阿里云改值 → worker secret 改值，过渡期可临时同时接受新旧两值）。泄漏单独不足以利用（仍需把伪造头送达且 worker 比对——但密钥本就是唯一门，故仍须保密 + 可轮换）。blast radius = 攻击者可自选限流桶（绕限流，非鉴权绕过）。
- [单值透传无法运行时识别] → 覆写保证靠部署期证伪门；运行时多值校验是纵深防御。
- [回源协议非 HTTPS 致密钥明文] → 文档要求回源协议 HTTPS。
- [`'unknown'` 为共享惩罚桶] → 大量「密钥通过但真实头缺失」请求挤同一桶，攻击者无法借此扩大预算；属可接受 fail-safe 副作用。
- [限流键变更影响既有计数] → 仅改 key 取值，KV 结构/TTL/阈值不变；切换瞬间旧 key 作废、按新 key 重计窗，无迁移、无可用性风险。

## 迁移计划

1. 阿里云 CDN：「修改出站请求头」以覆盖语义注入 `X-Origin-Auth: <密钥>` + 回源协议 HTTPS。
2. 部署 worker（`CDN_ORIGIN_SECRET`/`CDN_REAL_IP_HEADER` 可先不配 → fail-safe）。
3. `wrangler secret put CDN_ORIGIN_SECRET`（与阿里云注入值一致）。
4. **证伪式抓包门**：带伪造密钥头（含大小写变体）+ 伪造真实 IP 头 + 伪造 XFF，断言 worker 收到的密钥头=真实密钥（覆写成立）、解析返回值=真实出口 IP 且≠伪造值且单值。通过后才 `wrangler secret put CDN_REAL_IP_HEADER`（=Ali-Cdn-Real-Ip）；撤临时 log。
5. 验收：经 CDN 不同真实用户独立计数到 429、伪造 XFF/伪造头/错误密钥不绕过、直连按 CF-IP。
6. **回滚**：退 Worker 版本 + 清空 `CDN_ORIGIN_SECRET`/`CDN_REAL_IP_HEADER`（即 fail-safe）。无 D1 迁移、桌面端/前端零改动。

## 开放问题

- 受信真实 IP 头最终由步骤 4 证伪门给出（推荐 `Ali-Cdn-Real-Ip`，以实测为准）。
- 阿里云回源是否启用 IPv6：密钥头模型下与 IP 白名单无关，IPv6 回源同样靠密钥判来源、真实 IP 头按 IPv6 字面量采信（已支持）。
