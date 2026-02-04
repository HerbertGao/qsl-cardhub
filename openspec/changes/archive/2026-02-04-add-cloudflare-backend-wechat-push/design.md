# 设计：Cloudflare 云端后端与微信/顺丰推送

## 上下文

- **现有能力**：桌面端（Tauri）已支持云端同步：配置 API 地址与 API Key，向 `GET /ping`、`POST /sync` 发送全量数据（projects、cards、sf_senders、sf_orders），规范见 `specs/cloud-database-support/spec.md`。
- **顺丰路由推送**：顺丰丰桥提供 RoutePushService，向客户配置的 URL 以 POST（JSON 或 form）推送运单路由；报文含 `mailno`（运单号）、`orderid`（客户订单号）、`acceptTime`、`remark`、`opCode` 等；客户需在约定时间内返回 `return_code`（0000/1000）与 `return_msg`，见 `docs/sf-route-push-service.md`。
- **约束**：云端需可公网访问、支持 HTTPS、可接收 POST；若集成微信，需服务号与模板消息/订阅消息能力；顺丰推送需稳定响应避免重复推送。

## 目标 / 非目标

- **目标**：
  - 在 Cloudflare Pages（Functions） + D1 上实现与现有规范兼容的接收接口（`/ping`、`/sync`）及按呼号查询收卡信息接口。
  - 探讨微信服务号推送的可行性与集成方式。
  - 若集成微信：实现顺丰路由推送接收接口，并根据呼号、顺丰单号与微信 openid 向用户发送推送。
- **非目标**：不修改桌面端同步协议；不替代现有本地 SQLite。

## 决策

### 1. 云端技术栈：Cloudflare Pages + D1

- **选择**：使用 Cloudflare Pages 的 Functions（基于 Workers）提供 API，使用 D1（SQLite）存储同步数据与绑定关系；**使用 Wrangler CLI 创建项目，目标目录为本项目下的 `web_query_service`**。
- **理由**：免费额度充足、全球边缘、HTTPS 自带、与顺丰/微信回调所需的公网 URL 匹配；D1 满足按 client_id 隔离、按呼号查询、存储「呼号–openid–运单」等需求；与主项目同仓便于维护与 CI。
- **替代**：自建 VPS、Vercel + 数据库、阿里云函数等；未选因希望降低运维与成本，且 CF 国内可访问性可接受时足够使用。

### 2. 数据模型（D1）

- **sync 数据**：按 `client_id` 隔离；至少包含与现有 sync 请求体一致的表：projects、cards、sf_senders、sf_orders（可冗余或按需拆表），并记录 `sync_time`、`received_at`。
- **呼号查询**：基于 cards 表（或同步后的卡片视图）按 `callsign` 查询收卡信息（可返回项目名、数量、状态、分发备注等）。
- **顺丰 + 微信**（若启用）：  
  - 呼号解析：顺丰推送中的 `orderid` 对应 `sf_orders.order_id`、`mailno` 对应 `sf_orders.waybill_no`；用其一在 D1 中查 `sf_orders`，再通过 `sf_orders.card_id` 关联 `cards` 表得到 `cards.callsign`（同步接口已同步 sf_orders 与 cards）。  
  - 运单/路由：可存路由节点与时间供去重或查询。  
  - 绑定表：`callsign` ↔ `wechat_openid`，用于收到顺丰推送后查找要通知的 openid。

### 3. 顺丰路由推送接收

- **选择**：单独提供 POST 接口，请求方法支持 JSON（`application/json`）；解析 `Body.WaybillRoute[]`，按 `docs/sf-route-push-service.md` 在约定时间内返回 `{ "return_code": "0000", "return_msg": "成功" }` 或失败报文。
- **沙箱与正式双接口**：顺丰丰桥需分别配置**沙箱**与**正式**两个回调 URL，因此服务端提供两条路径，便于在丰桥后台各填一个地址：
  - **沙箱**：`POST /api/sf/route-push/sandbox` — 在丰桥沙箱环境中配置此 URL。
  - **正式**：`POST /api/sf/route-push` — 在丰桥生产环境中配置此 URL。
- **用户推送中的环境标记**：向用户（如微信模板消息）发送物流推送时，**沙箱**来源的推送必须在内容中标记「【沙箱】」，以便用户区分测试数据；**正式**来源不添加任何环境标记。
- **与呼号关联**：现有同步接口会同步顺丰订单表 `sf_orders` 及卡片表 `cards`（与导出/导入一致，同步范围已包含数据库全部业务表：projects、cards、sf_senders、sf_orders，无需修改同步规则）。服务端根据顺丰推送中的 `orderid`（对应 `sf_orders.order_id`）或 `mailno`（对应 `sf_orders.waybill_no`）在 D1 中查询 `sf_orders`，再根据 `sf_orders.card_id` 关联 `cards` 表得到 `cards.callsign`。
- **幂等**：同一运单同一路由节点不重复推送；可对 `id` 或 `(mailno, opCode, id)` 做去重，避免重复写入与重复推送用户。

### 4. 微信服务号推送与绑定方式

- **可行性**：
  - 需已认证的**服务号**；模板消息需事先在公众平台配置模板并获取 template_id。
  - 用户 openid 通过服务号网页授权获得；绑定流程见下。
  - 订阅消息/模板消息需符合微信规范（频率、内容合规）；物流类通知通常可落在「物流状态通知」类模板。
- **绑定方式（本期采用）**：
  - 提供**根据呼号查询收卡信息的单独页面**（使用按呼号查询接口展示结果）。
  - 在**查询结果页**提供「**订阅收卡**」按钮，并提示：订阅后将收到该呼号相关的卡片分发/物流信息。
  - 用户点击「订阅收卡」后，按微信公众平台要求进行**网页授权**（用户授权后获取用户信息与 openid）。
  - 授权完成后，服务端将该**呼号与当前用户（openid）**建立绑定并写入 D1。
  - **后续**：收到顺丰路由推送并解析出呼号后，根据 D1 中该绑定关系找到对应呼号下的用户（openid），向这些用户发送微信模板消息。
- **未决**：微信模板 ID 与文案的最终确定（依赖公众平台配置）。

### 5. API 形态摘要

| 能力           | 方法/路径            | 说明 |
|----------------|----------------------|------|
| 连接测试       | GET /ping            | 与现有规范一致，Bearer 校验 |
| 接收同步数据   | POST /sync           | 与现有规范一致，写入 D1 |
| 按呼号查询收卡 | GET /api/callsigns/:callsign 或 GET /api/query?callsign= | 查询该呼号下的收卡信息（项目、数量、状态等） |
| 前端配置       | GET /api/config      | 返回前端所需配置（如 WECHAT_APPID） |
| 顺丰路由推送（正式） | POST /api/sf/route-push     | 丰桥生产环境回调，JSON 入参，返回 0000/1000；用户推送不标环境 |
| 顺丰路由推送（沙箱） | POST /api/sf/route-push/sandbox | 丰桥沙箱环境回调，同上；用户推送内容带「【沙箱】」标记 |
| 按呼号查询页   | GET /                | Vue 3 单页应用，展示查询结果 +「订阅收卡」按钮，跳转微信授权 |
| 微信授权回调   | GET /api/wechat/auth-callback | 微信网页授权后带回 code，换取 openid，写入呼号–openid 绑定 |
| 微信推送       | 后端调用微信 API                   | 顺丰推送后按绑定关系向对应用户发送模板消息 |

### 6. 前端技术栈与项目结构

- **选择**：使用 Vue 3 + Vite + TypeScript 构建现代化单页应用，以移动端适配为主。
- **理由**：与主项目（Tauri 桌面端）技术栈一致，便于维护；Vue 3 组件化开发效率高；Vite 构建快速。
- **项目结构**：
  ```
  web_query_service/
  ├── package.json          # 统一管理所有依赖（pnpm）
  ├── wrangler.toml         # Cloudflare 配置（已加入 .gitignore）
  ├── wrangler.toml.example # 配置模板
  ├── vite.config.ts        # Vite 构建配置
  ├── index.html            # Vite 入口
  ├── static/               # 静态资源（favicon 等）
  ├── src/
  │   ├── worker/           # Cloudflare Worker 代码
  │   │   └── index.js
  │   └── client/           # Vue 前端代码
  │       ├── main.ts
  │       ├── App.vue
  │       ├── style.css
  │       └── components/
  │           ├── SearchBox.vue
  │           ├── ResultList.vue
  │           └── SubscribeCard.vue
  └── public/               # 构建输出（.gitignore）
  ```
- **移动端适配**：
  - 触摸目标最小 44px（iOS 推荐）
  - 垂直布局优先，全宽按钮
  - 安全区域适配（刘海屏）
  - 防止 iOS 输入框缩放

### 7. 验证码防护（自建轻量方案）

- **选择**：自建三层防护机制，不依赖第三方验证码服务。
- **理由**：
  - QSL 查询服务攻击价值较低，不需要应对专业验证码破解
  - 自建方案无外部依赖，更稳定、无成本、无隐私顾虑
  - 与 Cloudflare 架构天然契合（KV 存储限流计数和 nonce）
- **替代**：腾讯天御、极验等第三方服务；未选因增加外部依赖和成本，且对当前威胁模型过度设计。

#### 7.1 Layer 0: IP 限流

- 所有 API 请求按 IP 限制频率，默认 20 次/分钟
- 使用 Cloudflare KV 存储计数，key 为 `ratelimit:{ip}`，设置 1 分钟 TTL
- 超限返回 HTTP 429 + 友好提示

#### 7.2 Layer 1: 请求签名校验（无感）

- 前端 JS 生成签名：`sha256(payload + CLIENT_SIGN_KEY)`
- payload 包含：请求路径、关键参数、时间戳、随机 nonce
- 服务端校验：
  1. 时间窗口检查（5 分钟内）
  2. nonce 唯一性检查（KV 存储，5 分钟 TTL）
  3. 签名正确性校验
- 无效请求返回 HTTP 403
- 此层对用户完全无感，可过滤所有不执行 JS 的简单脚本

#### 7.3 Layer 2: 算术验证码（订阅时触发）

- 仅「订阅收卡」操作需要完成算术验证码
- 题目为简单加减法（如 "12 + 7 = ?"）
- 使用 Canvas 渲染题目，添加轻微干扰线，增加 OCR 难度
- 服务端生成题目时返回加密 token（含正确答案 + 过期时间 + HMAC 签名）
- 用户提交答案时校验 token 有效性和答案正确性
- token 有效期 5 分钟

#### 7.4 新增环境变量

| 变量名 | 说明 |
|--------|------|
| `CLIENT_SIGN_KEY` | 前端签名密钥（可公开，用于生成请求签名） |
| `CAPTCHA_SECRET` | 验证码 token 签名密钥（仅服务端） |

#### 7.5 新增 KV 命名空间

| 命名空间 | 用途 |
|----------|------|
| `RATE_LIMIT` | 存储 IP 限流计数和 nonce 去重 |

## 风险 / 权衡

- **Cloudflare 在国内访问**：部分网络下延迟或不可达，影响桌面端同步体验；可接受时再上，或同步保留「自建 API 地址」的灵活性。
- **顺丰推送超时**：微信/下游处理过慢可能导致顺丰重试；需快速响应（先 200 + return_code 0000，再异步写库与发微信）。
- **呼号解析依赖同步**：呼号由 sf_orders（order_id/waybill_no）+ cards（card_id → callsign）解析，故需桌面端先完成同步，顺丰推送时 D1 中已有对应订单与卡片数据。
- **微信模板审核**：模板需审核通过方可使用；物流类一般可通过，但文案需符合规范。

## 迁移计划

- 无迁移：云端后端为新增部署，桌面端无需改版即可指向新 API 地址；若后续更换域名或路径，仅需在客户端更新「云端 API 地址」配置。
- 回滚：关闭 Pages 部署或切回旧 API 地址即可。

## 待决问题

- 微信模板 ID 与文案的最终确定（依赖公众平台配置）。
