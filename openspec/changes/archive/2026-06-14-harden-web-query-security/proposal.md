## 为什么

`web_query_service`（Cloudflare Worker，对外 `qsl.herbertgao.me`）存在数处既有安全缺陷，与多租户/防爬大改造解耦、可独立修复，作为演进路线阶段 0 先行处理（详见 `docs/multi-tenant-design.md`）：

- 微信订阅绑定回调 `/api/wechat/auth-callback` **无任何限流**，可被自动化批量绑定虚假呼号；当前前端验证码是纯客户端剧场、服务端零校验（`verifyCaptcha` 从未被调用）。
- `CLIENT_SIGN_KEY` 经 `/api/config` **明文公开下发**（按设计视为可公开值）；其真实值存于 gitignored 的本地配置、**未进入 Git 历史**，但 `wrangler.toml.example`/README 存在「按非敏感 `[vars]` 配置」的过时指引。
- 服务端异常将原始信息回显客户端（顶层 catch 回显 `e.message`；微信回调分支回显上游 errmsg/原始结构）。

> **范围说明**：顺丰路由推送端点的来源鉴权原列入本期，但经核对顺丰 RoutePushService **不支持配置自定义请求头**（只让客户填接收 URL + 选 json/form）、且生产**尚未接入**该推送 → 整体推迟到「接顺丰路由推送」变更时用 **query token** 方案一并实现（鉴权 + 完整字段去重 + json 校验），**本期不含**。

## 变更内容

- **删 `verifyCaptcha` 死代码** + 给 `/api/wechat/auth-callback` 复用 `checkRateLimit`、以**独立计数桶**补 **IP 限流**（用不可伪造的 `CF-Connecting-IP`，不与查询端点抢预算）。前端验证码 UI 与 `/api/captcha` 保留至阶段 3 统一处理。
- **CLIENT_SIGN_KEY 配置卫生**：真实值不写入任何纳入版本控制的文件，修正过时 `[vars]` 指引为 Secret。**注意**：本期不改 `/api/config` 下发机制，故对「查询签名被绕过」零防护收益；因未进 git 历史，轮换为可选，面向公网暴露面在阶段 3 闭合。
- **错误响应脱敏**：顶层 catch 与微信回调等业务分支均不回显内部异常/上游原始错误。

## 功能 (Capabilities)

### 新增功能

（无 —— 本期均为对现有能力的行为修正）

### 修改功能

- `cloud-backend-api`: 错误响应脱敏（含上游错误分支）；签名密钥与服务凭据的配置卫生；订阅绑定回调纳入 IP 限流。

> 说明：删除 `verifyCaptcha` 为纯实现层死代码清理。captcha-protection 主规范 FR-3.4/FR-3.5（服务端校验验证码）本就**无承载实现**（前端为纯客户端剧场、`verifyCaptcha` 从未接线），删它不改变任何**已生效**行为，但使该条款暂无实现代码——此规范-实现缺口**非本期新增**，FR-3 的真正落地随阶段 3 防爬体系（PoW）统一处理。故本期不产 captcha-protection 增量规范，仅作实现任务与影响记录。

## 影响

- 代码：`web_query_service/src/worker/index.js`（删 `verifyCaptcha`、auth-callback 限流、错误脱敏含微信回调分支）。
- 配置/文档：`web_query_service/wrangler.toml.example` 与 `web_query_service/README.md`（修正 `CLIENT_SIGN_KEY` 过时配置指引为 Secret）。
- 部署：确认 `CLIENT_SIGN_KEY` 以 Secret 管理。
- 兼容：现有桌面端同步、移动端查询、微信订阅流程行为不变（订阅新增 IP 限流，正常单次不受影响）。
