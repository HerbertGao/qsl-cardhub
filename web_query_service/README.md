# QSL CardHub 云端查询与同步服务

基于 Cloudflare Workers + D1 + KV 的云端后端：接收桌面端全量同步、按呼号查询收卡、顺丰路由推送接收；可选微信服务号推送与验证码防护。

## 功能

| 能力           | 方法/路径              | 说明 |
|----------------|------------------------|------|
| 连接测试       | GET /ping              | Bearer API Key 校验，返回 pong |
| 全量同步       | POST /sync             | 接收客户端 projects/cards/sf_senders/sf_orders，按 client_id 写入 D1 |
| 按呼号查询收卡 | GET /api/callsigns/:callsign 或 GET /api/query?callsign= | 返回该呼号下的收卡信息（项目名、数量、状态、备注等） |
| 验证码生成     | GET /api/captcha       | 生成算术验证码，返回题目与加密 token |
| 顺丰路由推送（正式） | POST /api/sf/route-push     | 丰桥生产环境回调，JSON 入参，返回 0000/1000；用户推送不标环境 |
| 顺丰路由推送（沙箱） | POST /api/sf/route-push/sandbox | 丰桥沙箱环境回调，同上；发给用户的微信推送内容带「【沙箱】」标记 |
| 按呼号查询页   | GET /                  | Vue 3 单页应用：输入呼号查询，展示结果与「订阅收卡」入口 |
| 前端配置       | GET /api/config        | 返回功能开关（features）、签名密钥（sign_key）、WECHAT_APPID |

## 功能开关

系统支持按需启用功能，未配置相关环境变量时功能自动禁用：

| 功能 | 所需配置 | 说明 |
|------|----------|------|
| 订阅收卡（微信授权绑定） | `WECHAT_APPID` + `WECHAT_SECRET` | 未配置时「订阅收卡」按钮隐藏 |
| 微信推送 | `WECHAT_APPID` + `WECHAT_SECRET` + `WECHAT_TEMPLATE_ID` | 未配置时顺丰推送仅落库不发微信 |
| 验证码防护 | `CLIENT_SIGN_KEY` + `CAPTCHA_SECRET` + KV 绑定 | 未配置时跳过限流和签名校验 |

前端通过 `GET /api/config` 获取功能开关状态：

```json
{
  "features": {
    "wechat_subscribe": true,
    "wechat_push": true,
    "captcha": true
  },
  "wechat_appid": "wx...",
  "sign_key": "your-sign-key"
}
```

## 前置条件

- Node.js 18+
- pnpm 8+
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/install-and-update/)

## 安装依赖

```bash
cd web_query_service
pnpm install
```

## 创建 D1 数据库

```bash
cd web_query_service
pnpm exec wrangler d1 create qsl-sync
```

将输出中的 `database_id` 填入 `wrangler.toml` 中 `[[d1_databases]].database_id`。

## 创建 KV 命名空间（可选，用于验证码防护）

```bash
cd web_query_service
pnpm exec wrangler kv namespace create "RATE_LIMIT"
```

将输出中的 `id` 填入 `wrangler.toml` 中 `[[kv_namespaces]].id`。

## 执行 D1 迁移

```bash
# 本地开发库
pnpm run db:migrate:local

# 远程（生产）
pnpm run db:migrate
```

## 环境变量与密钥

在 `wrangler.toml` 的 `[vars]` 中可配置非敏感变量。敏感信息请用 Secret：

```bash
# /ping、/sync 的 Bearer 校验（必设，否则不校验）
pnpm exec wrangler secret put API_KEY

# 可选：微信服务号（顺丰推送后按呼号查绑定并发模板消息）
pnpm exec wrangler secret put WECHAT_APPID
pnpm exec wrangler secret put WECHAT_SECRET
pnpm exec wrangler secret put WECHAT_TEMPLATE_ID

# 可选：验证码防护
# CLIENT_SIGN_KEY 可在 [vars] 中配置（非敏感）
# CAPTCHA_SECRET 需用 secret（敏感）
pnpm exec wrangler secret put CAPTCHA_SECRET
```

**推荐：使用以下命令生成随机密钥**

```bash
# 生成 API_KEY（32 字节 hex，64 字符）
openssl rand -hex 32

# 生成 CAPTCHA_SECRET（32 字节 hex，64 字符）
openssl rand -hex 32

# 生成 CLIENT_SIGN_KEY（16 字节 hex，32 字符，非敏感可短一些）
openssl rand -hex 16
```

本地开发时在项目根目录创建 `web_query_service/.dev.vars`（不要提交）：

```
API_KEY=your-api-key
CLIENT_SIGN_KEY=your-sign-key
CAPTCHA_SECRET=your-captcha-secret
```

## 本地开发

```bash
cd web_query_service
pnpm install
pnpm run db:migrate:local   # 首次：初始化本地 D1
pnpm run dev                # 同时启动 Worker (8787) 和前端开发服务器 (5173)
```

- 前端开发：访问 `http://localhost:5173/`（带热更新）
- 直接访问 Worker：`http://localhost:8787/`

## 部署

```bash
cd web_query_service
pnpm run db:migrate         # 首次：对远程 D1 执行 schema.sql
pnpm run deploy             # 构建前端并部署到 Cloudflare
```

部署后记下 Workers 域名（如 `qsl-web-query-service.xxx.workers.dev`），在桌面端「数据管理 > 云端同步」中配置：

- API 地址：`https://qsl-web-query-service.xxx.workers.dev`
- API Key：与 `wrangler secret put API_KEY` 设置的一致

## 验证码防护

系统提供自建的轻量级防刷机制，分三层：

### Layer 0: IP 限流

- 按 IP 限制请求频率，默认 20 次/分钟
- 使用 Cloudflare KV 存储计数
- 超限返回 HTTP 429

### Layer 1: 请求签名校验

- 前端 JS 生成签名：`sha256(path:params:timestamp:nonce + CLIENT_SIGN_KEY)`
- 服务端校验时间窗口（5分钟）、nonce 唯一性、签名正确性
- 无效请求返回 HTTP 403
- 对用户完全无感，可过滤不执行 JS 的简单脚本

### Layer 2: 算术验证码

- 仅「订阅收卡」操作触发
- 简单加减法运算，Canvas 渲染 + 干扰线
- 服务端生成加密 token（含答案、过期时间、HMAC 签名）

## 顺丰路由推送配置（沙箱与正式双 URL）

顺丰丰桥需分别配置**沙箱**与**正式**两个回调地址，服务端提供两条路径：

1. **正式环境**：在丰桥**生产环境**中配置「路由推送」接收地址：
   `https://<你的 Workers 域名>/api/sf/route-push`
2. **沙箱环境**：在丰桥**沙箱环境**中配置「路由推送」接收地址：
   `https://<你的 Workers 域名>/api/sf/route-push/sandbox`
3. 两处请求方法均选择 **JSON**，Content-Type：`application/json; charset=UTF-8`。
4. 服务端会先返回 `{ "return_code": "0000", "return_msg": "成功" }`，再异步落库并根据 `orderid`/`mailno` 在 D1 中解析呼号；若配置了微信且该呼号有订阅用户，则发送模板消息。**沙箱**路径触发的推送在发给用户的内容中会带「【沙箱】」标记，正式路径不添加该标记。

## 按呼号查询与「订阅收卡」

- 打开 `https://<你的 Workers 域名>/`，输入呼号查询，可看到该呼号下的收卡记录。
- 结果页提供「订阅收卡」按钮（需配置 WECHAT_APPID + WECHAT_SECRET，否则按钮不显示）。
- 若启用验证码防护，点击订阅前需完成算术验证码。
- 授权完成后将呼号与当前用户 openid 写入 `callsign_openid_bindings` 表。
- 顺丰推送解析到呼号后，会查该表并向对应用户发送微信模板消息（需配置 WECHAT_TEMPLATE_ID）。

## 微信服务号推送可行性

- **资质**：需已认证的**服务号**（非订阅号）。
- **模板消息**：在微信公众平台「模板消息」中申请模板，审核通过后获得 `template_id`；物流类可申请「物流状态通知」等模板。
- **网页授权与 openid**：用户「订阅收卡」时跳转微信网页授权（snsapi_userinfo 或 snsapi_base），授权后回调带 `code`，服务端用 `code` 换取 `access_token` 与用户信息（含 `openid`），再将当前呼号与该 `openid` 写入 `callsign_openid_bindings`。
- **订阅消息**：若使用订阅消息替代模板消息，需用户一次订阅授权，流程见微信文档。
- **限制**：模板消息有频率与内容规范；需在公众平台配置授权回调域名（如 Workers 域名）。

## D1 表结构摘要

- **sync_meta**：client_id, sync_time, received_at（各客户端最近同步时间）
- **projects / cards / sf_senders / sf_orders**：与桌面端一致，每表增加 `client_id` 做多端隔离
- **callsign_openid_bindings**：callsign, openid（订阅收卡绑定）
- **sf_route_log**：顺丰路由去重与记录（id, mailno, orderid, op_code, accept_time, remark）

详见 `schema.sql`。

## KV 命名空间

- **RATE_LIMIT**：存储 IP 限流计数和 nonce 去重（验证码防护功能需要）
