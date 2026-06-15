## 为什么

生产经**阿里云 CDN**（`qsl.herbert-dev.cn`，备案域名、大陆入口）回源到 **Cloudflare 源站**（`qsl.herbertgao.me`）。经 CDN 路径时，Worker 收到的 `CF-Connecting-IP` 是**阿里云 CDN 回源节点 IP、不是真实用户 IP**；真实用户 IP 由阿里云 CDN 在回源请求头里注入。

后果：现有按 `CF-Connecting-IP` 计数的 IP 限流（查询端点、`/api/captcha`、`/api/wechat/auth-callback`）在 CDN 路径下把大量真实用户**归并到少数 CDN 回源 IP 桶** → 限流粒度失真。「`CF-Connecting-IP` 不可伪造」在前置 CDN 架构下**不再成立**。

这是一个独立可交付、可独立回滚的真实 bug 修复，同时也是阶段 3 防爬（per-IP 自适应难度、会话配额）能正确按用户计数的**前置**。

> **边界声明**：本变更产出的「可信真实客户端 IP」仅用作**限流/防爬计数键（抬高自动化批量调用成本）**，**不**用作访问控制/鉴权判据。鉴权由 Bearer Key（`resolveTenant`）与请求签名（`verifySignature`）承担。IP 键被部分污染/坍缩，最坏是「成本抬升打折」，不构成鉴权绕过。

## 变更内容

- 建立**可信真实客户端 IP 解析**（`web_query_service/src/worker/client-ip.js`），信任信号 = **阿里云 CDN 注入的密钥回源头 `X-Origin-Auth`**：
  - 请求带有效 `X-Origin-Auth`（worker 常量时间比对等于 `CDN_ORIGIN_SECRET`）= 确来自 CDN → 采信 CDN 写入的真实 IP 头（推荐 `Ali-Cdn-Real-Ip`，阿里云默认携带、为防 XFF 伪造而设；运行时校验单值/合法 IP/不含逗号，多值/非法 → `'unknown'`）；
  - 无密钥/错误密钥（直连，或伪造密钥）→ 只用 `CF-Connecting-IP`、**忽略一切注入头**；
  - 密钥未配 / 真实头名未配 → fail-safe 到 `CF-Connecting-IP`；CF-IP 缺失 → `'unknown'`。
  - **绝不读 `X-Forwarded-For`**（append 语义、首段客户端可伪造）。
- **不采用回源 IP 白名单**：阿里云回源 IP 动态分配、官方明确不建议固定白名单、查询接口还有带宽门槛——故以密钥头判定「来自 CDN」，绕开动态 IP 死结。
- `getClientIP(request, env)` 统一供查询端点（约 535）、`/api/captcha`（约 565）、`/api/wechat/auth-callback`（约 729）取用；改造后 worker 内无 `getClientIP` 之外的 `CF-Connecting-IP`/`X-Forwarded-For` 直读。
- 配置：`CDN_ORIGIN_SECRET`（机密，`wrangler secret`，= X-Origin-Auth 期望值；未配 → fail-safe）+ `CDN_REAL_IP_HEADER`（受信真实 IP 头名，**无内置默认**，经证伪门证实覆写后显式配 `Ali-Cdn-Real-Ip`）。
- 阿里云侧：「修改出站请求头」以覆盖语义注入 `X-Origin-Auth: <密钥>` + 回源协议 HTTPS（`Ali-Cdn-Real-Ip` 默认携带、无需配置）。
- 验证：`verify/client-ip.test.js`（node:test 纯函数单测）+ `run_worker_smoke.sh` 加 CDN 双入口（有效密钥/无密钥/错误密钥）限流归桶端到端断言；`wrangler.toml.example` + `docs/web-query-service-deploy.md` 配置说明与部署顺序/证伪门。

**不做**（明确边界）：PoW / 短时会话 / 动态 `sign_key` / 删算术验证码（阶段 3 防爬主体，单独变更）；host/path → tenant 路由（阶段 4）；顺丰 route-push 鉴权（推迟到「接顺丰路由推送」变更）。本变更**不动** `verifySignature`、无 D1 迁移。

## 功能 (Capabilities)

### 新增功能
- `trusted-client-ip`: 可信真实客户端 IP 解析——双入口（直连 Cloudflare vs 阿里云 CDN 回源）感知、以 CDN 注入的不可伪造密钥头（`X-Origin-Auth`）判定来源、采信 CDN 写入的真实 IP 头（非 XFF 首段）、配置缺失 fail-safe、统一供所有 IP 计数类安全机制（限流、后续防爬）取用。

### 修改功能
- `cloud-backend-api`: 「订阅绑定端点的基础限流」需求中限流计数键 IP 来源，须取自「可信真实客户端 IP 解析」。
- `captcha-protection`: 「IP 限流」需求中「按 IP 计数」的 IP 来源须明确为「可信真实客户端 IP 解析」的产物（CDN 路径下按真实用户 IP，而非 CDN 回源节点 IP）。

## 影响

- 代码：`web_query_service/src/worker/client-ip.js`（新增）、`web_query_service/src/worker/index.js`（`getClientIP` import + 3 调用点）。
- 配置：`web_query_service/wrangler.toml.example` 新增 `CDN_ORIGIN_SECRET` / `CDN_REAL_IP_HEADER` 占位说明；部署文档新增阿里云配置 + 部署顺序 + 证伪门 + 维护。
- 验证：`web_query_service/verify/client-ip.test.js`（新增）+ `run_worker_smoke.sh`（4.4 段）+ `package.json`（test:unit）。
- 无 D1 迁移；回滚 = 退 Worker 版本 + 清空配置项（即 fail-safe）。纯服务端逻辑，桌面端/前端零改动。
