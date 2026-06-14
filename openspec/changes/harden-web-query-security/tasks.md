## 1. 密钥与配置卫生

- [x] 1.1 修正 `web_query_service/wrangler.toml.example` 与 `README.md` 中「`CLIENT_SIGN_KEY` 按非敏感 `[vars]` 配置」的过时指引为 Secret；确认这些文件仅含占位符
- [x] 1.2 注明顺丰 checkword 与 `CLIENT_SIGN_KEY` 同属「gitignored + CI/secret 注入、未进 git 历史」，统一风险口径

## 2. 删验证码死代码 + 订阅回调限流

- [x] 2.1 删除 `index.js` 从未被调用的 `verifyCaptcha` 函数
- [x] 2.2 给 `/api/wechat/auth-callback` 加 IP 限流，**独立计数键** `ratelimit:authcb:${ip}`（不与查询共桶）；计数 IP 用不可伪造的 `CF-Connecting-IP`（非 `X-Forwarded-For` 回退）；`checkRateLimit` 增桶参；注明 KV 未配时 fail-open、KV 为部署前置
- [x] 2.3 保留 `/api/captcha` 与前端 `MathCaptcha`（阶段 3 处理）；将 `features.captcha` 误导性记入阶段 3 待办
- [ ] 2.4 验证（前提：`RATE_LIMIT` KV 已绑定，否则 `checkRateLimit` fail-open 不限流）：Worker 构建无未定义引用；同 IP 超额打 auth-callback 被限流，正常单次订阅不受影响；确认计数键源为 `CF-Connecting-IP`、伪造 `X-Forwarded-For` 不改变限流桶

## 3. 错误响应脱敏

- [x] 3.1 顶层 catch 返回通用 `{success:false,message:'服务器错误'}`+500，移除 `e.message` 回显
- [x] 3.2 微信回调分支改返回通用「微信授权失败」，移除 `tokenData.errmsg || JSON.stringify(tokenData)` 回显
- [x] 3.3 保留 `console.error`，详细异常/上游错误仅落服务端日志
- [ ] 3.4 验证：构造内部异常与微信失败时，响应体均不含原始异常/上游原始结构

## 4. 部署与回归

- [ ] 4.1 部署：确认 `CLIENT_SIGN_KEY` 以 Secret 管理 → 部署 Worker
- [ ] 4.2 回归：桌面端 `/sync`、移动端查询、微信订阅流程行为不变
- [x] 4.3 确认仓库内无明文签名密钥/凭据（仅占位符出现在 `.example`/文档）
- [x] 4.4 OpenSpec 归档预演：`openspec validate harden-web-query-security` 通过；`cloud-backend-api` 增量能无错误合并进主规范
