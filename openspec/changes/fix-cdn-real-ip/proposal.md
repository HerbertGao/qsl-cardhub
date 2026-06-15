## 为什么

生产经**阿里云 CDN**（`qsl.herbert-dev.cn`，备案域名、大陆入口）回源到 **Cloudflare 源站**（`qsl.herbertgao.me`）。经 CDN 路径时，Worker 收到的 `CF-Connecting-IP` 是**阿里云 CDN 回源节点 IP、不是真实用户 IP**；真实用户 IP 由阿里云 CDN 在回源请求头里注入。

后果：现有按 `CF-Connecting-IP` 计数的 IP 限流（查询端点、`/api/captcha`、以及阶段 0 给 `/api/wechat/auth-callback` 加的独立桶限流）在 CDN 路径下把大量真实用户**归并到少数 CDN 回源 IP 桶** → 限流粒度失真：既可能误伤正常用户、也放过分布在不同 CDN 节点的攻击者。「`CF-Connecting-IP` 不可伪造」这一前提在前置 CDN 架构下**不再成立**。

这是一个独立可交付、可独立回滚的真实 bug 修复，同时也是阶段 3 防爬（per-IP 自适应难度、会话配额）能正确按用户计数的**前置**。

> **边界声明**：本变更产出的「可信真实客户端 IP」仅用作**限流/防爬计数键（抬高自动化批量调用成本）**，**不**用作访问控制/鉴权判据。鉴权由 Bearer Key（`resolveTenant`）与请求签名（`verifySignature`）承担，订阅绑定的真正闸门是微信 OAuth `code` 校验。IP 键被部分污染/坍缩，最坏是「成本抬升打折」，不构成鉴权绕过。

## 变更内容

- 建立**可信真实客户端 IP 解析**：`getClientIP` 改为双入口感知——
  - 当 `CF-Connecting-IP` 命中「阿里云 CDN 回源 IP 白名单」**且**受信真实 IP 头名已配置（运营者经证伪门证实覆写后显式启用）→ 采信**阿里云 CDN 写入的真实 IP 头**（推荐 `Ali-Cdn-Real-Ip`；其「单值、由 CDN 节点按 TCP 层对端写入并**覆盖**客户端同名头」是阿里云官方语义，**但该覆写须经部署期证伪门实证、证实前头名不配**）。**禁止采信 `X-Forwarded-For`（含首段）**——XFF 是 append 语义、首段是客户端可伪造值。采信前运行时校验该头为单值/合法 IP/不含逗号（多值=覆写假设失效信号 → `'unknown'`；但单值透传运行时无法识别，故覆写保证由证伪门承担）。
  - 命中白名单但**头名未配置**（未证实覆写=安全态）→ fail-safe 回退 `CF-Connecting-IP`，**不**默认采信任何头；
  - 命中白名单 + 头名已配但**本请求头缺失/多值/非法** → `'unknown'` 惩罚桶，**不**坍缩回 CDN 节点 IP；
  - 未命中白名单（直连，或来源不可信）→ 用 `CF-Connecting-IP`、忽略一切注入头；CF-IP 缺失 → `'unknown'`。**绝不无条件信任客户端可控头。**
- 阿里云 CDN 回源 IP 段走 **Worker 配置**（CIDR 列表，部署期注入、需定期维护），**禁止硬编码**；解析用**默认拒**口径——**仅接受全局可路由公网单播 IPv4**，私网/CGNAT(`100.64.0.0/10`)/loopback/link-local/组播/保留/`/0`/非法前缀一律丢弃（落进回源白名单几乎一定是误配，会把大片来源当成可信 CDN）。受信头名**无内置默认**。**未配置/配置为空 → fail-safe**：只信 `CF-Connecting-IP`，不信任何注入头。
- **信任前提（硬，且有顺序）**：把「源站 `qsl.herbertgao.me` 仅接受来自阿里云回源段 / Cloudflare 的回源」从可选加固**升格为本变更的部署前置**，且**必须先于**配置白名单/启用采信头——否则进入「白名单生效但可直连源站伪造头」的危险中间态（比都不配更糟）。写进迁移计划与文档。
- **回头修**阶段 0 的 `/api/wechat/auth-callback` 限流：从直接读 `CF-Connecting-IP` 改为统一调可信 IP 解析，使其在 CDN 路径下按真实用户 IP 计数。
- 查询端点与 `/api/captcha` 限流已走 `getClientIP`，统一改造后自动受益（仅需回归确认）。
- 更新 `wrangler.toml.example` 与部署文档，说明新配置项（回源白名单、可信真实 IP 头名）、回源 IP 段维护周期、源站回源限制前置。

**不做**（明确边界）：PoW / 短时会话 / 动态 `sign_key` / 删算术验证码（阶段 3 防爬主体，单独变更）；host/path → tenant 路由（阶段 4）；顺丰 route-push 鉴权（推迟到「接顺丰路由推送」变更）。本变更**不动** `verifySignature`。

## 功能 (Capabilities)

### 新增功能
- `trusted-client-ip`: 可信真实客户端 IP 解析——双入口（直连 Cloudflare vs 阿里云 CDN 回源）感知、CDN 回源 IP 白名单校验、采信 CDN 写入的不可伪造真实 IP 头（非 XFF 首段）、配置缺失 fail-safe、统一供所有 IP 计数类安全机制（限流、后续防爬）取用。

### 修改功能
- `cloud-backend-api`: 「订阅绑定端点的基础限流」需求中「计数键 IP 必须取自不可伪造的 `CF-Connecting-IP`、禁止用 `X-Forwarded-For`」的措辞，在前置 CDN 架构下不成立，须修正为「取自可信真实客户端 IP 解析」。
- `captcha-protection`: 「IP 限流」需求中「按 IP 计数」的 IP 来源须明确为「可信真实客户端 IP 解析」的产物（CDN 路径下按真实用户 IP，而非 CDN 回源节点 IP），与 `trusted-client-ip` 交叉引用。

## 影响

- 代码：`web_query_service/src/worker/index.js`（`getClientIP` 约 43-47 行；`/api/wechat/auth-callback` 限流约 736 行）。
- 配置：`web_query_service/wrangler.toml.example` 新增 CDN 回源白名单 + 可信真实 IP 头名配置项；部署文档新增维护说明与源站回源限制前置。
- 验证：`web_query_service/verify/`（worker smoke + CIDR/IP 解析纯函数单测）新增解析与限流键断言。
- 无 D1 迁移；回滚 = 退 Worker 版本 + 移除/还原配置项。纯服务端逻辑，桌面端/前端零改动。
