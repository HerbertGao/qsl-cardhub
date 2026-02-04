# 变更：基于云端同步的 Cloudflare 后端与微信/顺丰推送

## 为什么

现有云端同步功能（见 `specs/cloud-database-support/spec.md`）已定义客户端向云端发送全量数据的接口规范（`GET /ping`、`POST /sync`），但未提供可部署的云端实现；用户也无法按呼号查询收卡信息。同时，顺丰路由推送（见 `docs/sf-route-push-service.md`）需要可接收推送的 URL，若能与微信服务号结合，可按呼号、运单号和 openid 向用户推送物流状态。

## 变更内容

- 在 **Cloudflare Pages + D1** 上实现云端后端：① 接收并存储客户端同步数据（兼容现有 `/ping`、`/sync` 规范）；② 提供按呼号查询收卡信息的接口。
- **探讨**微信服务号推送的可行性（服务号资质、模板消息/订阅消息、openid 与呼号绑定流程），并在可行前提下纳入本方案。
- 若集成微信推送：增加 **顺丰路由推送接收接口**（符合 `docs/sf-route-push-service.md`），接收顺丰 POST 的路由数据后，根据运单号/客户订单号解析出呼号，结合 D1 中「呼号 ↔ 微信 openid」绑定，向对应用户发送微信模板消息。
- 使用 **Vue 3 + Vite + TypeScript** 构建现代化查询页面（单页应用），以移动端适配为主，与主项目技术栈保持一致。
- 实现**自建轻量级验证码防护**：① IP 限流（20次/分钟）；② 请求签名校验（无感，过滤简单脚本）；③ 算术验证码（订阅时触发，Canvas 渲染 + 干扰线）。

## 影响

- **受影响规范**：无现有规范被修改；新增能力 `cloud-backend-api`、`sf-route-push-receiver`、`wechat-push`（后两者在微信可行时生效）、`captcha-protection`。
- **受影响代码**：在本项目 **`web_query_service`** 目录下使用 **Wrangler CLI** 新增 Cloudflare Pages 项目（Workers/Pages + D1），与现有 Tauri 桌面端无直接代码耦合；桌面端继续使用现有同步客户端与 API 规范。
