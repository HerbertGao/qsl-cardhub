## 上下文

阶段 0 针对 `web_query_service`（Cloudflare Worker，单文件 `src/worker/index.js`）的既有安全缺陷做最小修复，集中在 Worker 后端与配置/文档，**不涉及多租户/数据模型变更**，不改桌面端同步与查询前端的现有行为。整体路线见 `docs/multi-tenant-design.md`。

**事实校正**：`CLIENT_SIGN_KEY` 的真实值存于 `web_query_service/wrangler.toml`，该文件被 `.gitignore` 忽略、**从未进入 Git 历史**（`git log --all -S` 零命中）；进入版本控制的只有 `wrangler.toml.example`（占位符）。它经 `/api/config` 明文下发，按设计属可公开值。

> **范围说明**：route-push 来源鉴权原属本期，但顺丰 RoutePushService **不支持配置自定义请求头**（只填接收 URL + 选 json/form），且生产**尚未接入**该推送 → 整体推迟到「接顺丰路由推送」变更，届时用 **query token** 承载鉴权凭证（URL 参数，从 `searchParams` 读）+ 完整字段去重 + json 请求方法校验一并实现。本期 route-push handler 维持改造前状态、不含鉴权。

## 目标 / 非目标

**目标：**
- 删 `verifyCaptcha` 死代码 + 订阅回调最小限流。
- `CLIENT_SIGN_KEY` 配置卫生。
- 错误响应脱敏（顶层 + 业务分支）。

**非目标：**
- 不改 `/api/config` 下发 `sign_key` 的机制（动态化属阶段 3）。**故阶段 0 对 sign_key 不提供任何查询签名绕过防护——已知、deferred，由阶段 3 闭合。**
- 不删 `/api/captcha` 端点与前端验证码 UI（阶段 3 随防爬统一处理）。
- 不引入多租户 / `tenant_id`。
- **不含 route-push 来源鉴权**（推迟到「接顺丰推送」变更，见上）。

## 决策

> capability 归属判据（正面二分规则）：**单端点行为加固/横切修正**——复用共享设施、**不引入新的对外契约**（错误脱敏、配置卫生、订阅限流复用）——归入聚合 capability `cloud-backend-api`；**引入独立对外来源契约**的（如将来 route-push 来源鉴权）独立成 capability。

### 1. CLIENT_SIGN_KEY 配置卫生（非泄漏急修）

- 事实：未进 Git 历史，按设计可公开 → **不是**泄漏急修；轮换为可选。
- 做：真实值不写入任何版本控制文件；修正 `wrangler.toml.example` 与 `README.md` 中「`[vars]` 配置（非敏感）」过时指引为 Secret。
- 不做：不改 `/api/config` 下发 → 对查询签名绕过**零收益**（显式披露，避免「已加固」错觉）；真正闭合在阶段 3。
- 同口径：顺丰 checkword（`config/sf_express_default.toml`，与 wrangler.toml 处境对称：gitignored、未进 git、CI 注入）一并纳入「配置卫生」口径，不单独定性为泄漏。

### 2. 删 verifyCaptcha + 订阅回调最小限流

- 删 `index.js` 从未被调用的 `verifyCaptcha`（端到端死代码：前端 `handleCaptchaSuccess` 亦丢弃答案）。
- **诚实定性**：删它**不**消除「虚假安全表象」——前端验证码剧场仍在（保留至阶段 3）。本期真正补的防护是给 `/api/wechat/auth-callback` 加 IP 限流，复用 `checkRateLimit`/`getClientIP`（零新依赖/KV），堵住「自动化批量绑定」这一当前裸奔的已声明威胁。
- **独立计数桶 + 不可伪造 IP**：现有 `checkRateLimit` 用全局 `ratelimit:${ip}`；auth-callback **必须**用独立键 `ratelimit:authcb:${ip}`（否则订阅与查询共用 20/min 预算、互相饿死），计数 IP **必须**用不可伪造的 `CF-Connecting-IP`（非 `X-Forwarded-For` 回退）。`checkRateLimit` 加可选 bucket 参（默认保持查询端点原行为）；其在 `RATE_LIMIT` KV 未配时 fail-open（可用性优先）、KV 为部署前置。
- **carry-over**：阶段 4 将改 `/api/wechat/auth-callback`（state 带 tenant），届时**须保留**本期注入的独立桶限流。
- `/api/config` 的 `features.captcha`（= `!!(CLIENT_SIGN_KEY && CAPTCHA_SECRET)`，两 secret 均配置时为 true）删 verifyCaptcha 后仍下发、前端仍弹剧场验证码 → 误导仍在，记入阶段 3 待办。

### 3. 错误脱敏

- 顶层 catch 返回通用 `{success:false,message:'服务器错误'}`+500；原始异常仅 `console.error`。
- 微信回调分支不再回显 `tokenData.errmsg || JSON.stringify(tokenData)`，改通用「微信授权失败」，原始仅日志。

## 风险 / 权衡

- [删 `verifyCaptcha`] → 已确认端到端零引用，删除无行为影响。
- [轮换 `CLIENT_SIGN_KEY` 致已加载旧前端的签名失效] → 影响极小，刷新即取新值；选低峰部署。
- [auth-callback 限流共桶/可伪造 IP] → 已用独立桶 `ratelimit:authcb` + `CF-Connecting-IP` 规避。

## 迁移与回滚

- 部署：`wrangler secret put CLIENT_SIGN_KEY`（如尚未以 secret 管理）→ 部署 Worker。
- 回滚：Worker 回滚到上一版本即可；secret 保留无副作用。

## 待解决问题

（无阻塞项。route-push 鉴权的承载方式（query token）、json/form 校验、完整字段去重、顺丰源 IP 段等问题随其推迟，移至「接顺丰路由推送」变更处理。）
