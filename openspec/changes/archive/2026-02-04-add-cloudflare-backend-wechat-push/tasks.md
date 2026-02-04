## 1. Cloudflare 项目与 D1

- [x] 1.1 在本项目 **`web_query_service`** 目录下使用 **Wrangler CLI** 创建 Cloudflare Pages 项目，配置 Wrangler 与 D1 绑定
- [x] 1.2 设计并执行 D1 初始化脚本：sync 相关表（按 client_id 隔离的 projects、cards、sf_senders、sf_orders 及元数据）、呼号查询所需索引
- [x] 1.3 若启用微信+顺丰：增加路由/运单记录表、呼号–openid 绑定表及索引

## 2. 接收数据与呼号查询 API

- [x] 2.1 实现 GET /ping：Bearer API Key 校验，返回与 `specs/cloud-database-support` 一致的 JSON
- [x] 2.2 实现 POST /sync：校验、解析请求体，按 client_id 写入 D1，返回 success/stats 与现有规范一致
- [x] 2.3 实现按呼号查询收卡接口：GET /api/callsigns/:callsign 或 GET /api/query?callsign=，从 D1 返回该呼号下的收卡信息（项目、数量、状态、分发备注等）
- [x] 2.4 为上述接口配置 CORS、限流（可选）与错误响应格式

## 3. 顺丰路由推送接收（在集成微信时必做）

- [x] 3.1 实现 POST /api/sf/route-push：接收 JSON  Body.WaybillRoute[]，解析 mailno、orderid、acceptTime、remark、opCode 等
- [x] 3.2 在约定时间内返回顺丰要求的 JSON：return_code（0000/1000）、return_msg
- [x] 3.3 路由数据落库（可选去重），并解析出呼号（通过 orderid 或 D1 中订单/卡片映射）
- [x] 3.4 若启用微信推送：根据呼号查 openid，调用微信模板消息接口发送物流状态

## 4. 按呼号查询页与微信订阅收卡绑定

- [x] 4.1 实现「根据呼号查询收卡信息的单独页面」：调用按呼号查询接口展示结果，在结果区域提供「订阅收卡」按钮，并提示「订阅后将收到该呼号的卡片分发/物流信息」
- [x] 4.2 订阅收卡流程：用户点击「订阅收卡」后跳转微信公众平台网页授权；配置授权回调 URL，回调时用 code 换取 openid，将当前查询的呼号与该 openid 写入 D1 绑定表
- [x] 4.3 文档：整理微信服务号推送可行性（资质、模板消息/订阅消息、网页授权与 openid）
- [x] 4.4 若可行：配置服务号模板、环境变量（appid、secret、template_id），实现「根据 openid 发送模板消息」的后端逻辑；顺丰推送时根据呼号查绑定表，向对应用户推送
- [x] 4.5 顺丰推送链路联调：顺丰回调 → 解析呼号 → 查绑定表得 openid → 发微信
- [x] 4.6 使用 Vue 3 + Vite + TypeScript 重构查询页面为现代化单页应用
- [x] 4.7 合并项目结构：移除 pnpm-workspace，Worker 与前端统一在 `web_query_service` 下管理
- [x] 4.8 移动端适配优化：触摸目标 ≥44px、垂直布局优先、安全区域适配
- [x] 4.9 新增 GET /api/config 接口：返回前端配置（WECHAT_APPID）
- [x] 4.10 配置管理：wrangler.toml 加入 .gitignore，创建 wrangler.toml.example 模板

## 5. 文档与部署

- [x] 5.1 更新或新增「云端 API 实现说明」：从 `web_query_service` 目录使用 Wrangler 部署到 Cloudflare Pages + D1 的步骤、环境变量、D1 表结构摘要
- [x] 5.2 顺丰路由推送：在文档中说明回调 URL 配置、请求/响应格式及与呼号/微信的关联方式
- [x] 5.3 从 `web_query_service` 目录执行 Wrangler 部署到 Cloudflare Pages，配置生产/预览环境与 D1 绑定

## 6. 验证码防护

### 6.1 基础设施

- [x] 6.1.1 创建 Cloudflare KV 命名空间 `RATE_LIMIT`，用于存储限流计数和 nonce 去重
- [x] 6.1.2 在 wrangler.toml.example 中添加 KV 绑定配置和新增环境变量（`CLIENT_SIGN_KEY`、`CAPTCHA_SECRET`）
- [x] 6.1.3 在 GET /api/config 接口中返回 `CLIENT_SIGN_KEY`（前端签名用）

### 6.2 Layer 0: IP 限流

- [x] 6.2.1 实现限流中间件：按 IP 统计请求频率，默认 20 次/分钟
- [x] 6.2.2 使用 KV 存储计数，key 为 `ratelimit:{ip}`，设置 1 分钟 TTL
- [x] 6.2.3 超限请求返回 HTTP 429 + JSON 错误响应（含友好提示和重试时间）

### 6.3 Layer 1: 请求签名校验

- [x] 6.3.1 前端实现签名生成：`sha256(path:params:timestamp:nonce + CLIENT_SIGN_KEY)`
- [x] 6.3.2 修改 SearchBox.vue，查询请求携带签名参数（`_ts`、`_nonce`、`_sig`）
- [x] 6.3.3 Worker 实现签名校验中间件：验证时间窗口（5分钟）、nonce 唯一性（KV 去重）、签名正确性
- [x] 6.3.4 签名无效返回 HTTP 403 + JSON 错误响应

### 6.4 Layer 2: 算术验证码

- [x] 6.4.1 实现 GET /api/captcha 接口：生成随机算术题（加减法）、返回题目文本 + 加密 token（含答案、过期时间、HMAC 签名）
- [x] 6.4.2 前端实现 MathCaptcha.vue 组件：Canvas 渲染题目 + 干扰线、输入框、刷新按钮
- [x] 6.4.3 修改 SubscribeCard.vue，订阅前弹出验证码组件，验证通过后才跳转微信授权
- [x] 6.4.4 Worker 实现验证码校验：在订阅相关接口校验 `captcha_token` 和 `captcha_answer`

### 6.5 集成与测试

- [x] 6.5.1 为所有公开 API（/api/query、/api/captcha）应用 Layer 0 + Layer 1
- [x] 6.5.2 为订阅流程应用 Layer 2
- [x] 6.5.3 更新 wrangler.toml.example 文档，说明新增配置项
- [x] 6.5.4 本地测试：验证限流、签名校验、验证码流程正常工作
- [x] 6.5.5 部署到 Cloudflare，验证生产环境防护生效

## 7. 微信功能开关

- [x] 7.1 修改 GET /api/config 接口：返回 `features` 对象，包含 `wechat_subscribe` 和 `wechat_push` 布尔值
  - `wechat_subscribe`: `WECHAT_APPID` 和 `WECHAT_SECRET` 均存在时为 `true`
  - `wechat_push`: `WECHAT_APPID`、`WECHAT_SECRET`、`WECHAT_TEMPLATE_ID` 均存在时为 `true`
  - `wechat_appid`: 仅在 `wechat_subscribe` 为 `true` 时返回实际值，否则为 `null`
- [x] 7.2 修改前端 App.vue：根据 `features.wechat_subscribe` 决定是否显示「订阅收卡」按钮（SubscribeCard 组件）
- [x] 7.3 修改 Worker 顺丰推送逻辑：仅在 `WECHAT_APPID`、`WECHAT_SECRET`、`WECHAT_TEMPLATE_ID` 均存在时才尝试发送微信消息
- [x] 7.4 更新 wrangler.toml.example 文档：说明微信相关环境变量为可选配置，未配置时订阅功能自动禁用
- [x] 7.5 测试：验证未配置微信参数时查询功能正常、订阅按钮隐藏；配置后订阅功能正常启用
