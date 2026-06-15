# 云端查询服务部署说明（Cloudflare Workers + D1）

本目录下的 **`web_query_service`** 为 QSL CardHub 云端同步与按呼号查询的可部署实现，使用 Wrangler CLI 部署到 Cloudflare Workers + D1。

## 快速开始

1. 进入服务目录并安装依赖：
   ```bash
   cd web_query_service
   npm install
   ```

2. 创建 D1 数据库并写入 `wrangler.toml`：
   ```bash
   npx wrangler d1 create qsl-sync
   ```
   将输出的 `database_id` 填入 `wrangler.toml` 中 `[[d1_databases]].database_id`。

3. 执行 D1 迁移（本地与远程）：
   ```bash
   npx wrangler d1 execute qsl-sync --local --file=./schema.sql
   npx wrangler d1 execute qsl-sync --remote --file=./schema.sql
   ```

4. 配置 API Key（用于 /ping、/sync 的 Bearer 校验）：
   ```bash
   npx wrangler secret put API_KEY
   ```

5. 部署：
   ```bash
   npm run deploy
   ```

6. 在桌面端「数据管理 > 云端同步」中配置：
   - API 地址：`https://<你的 Workers 域名>`
   - API Key：与上一步设置一致

## 详细说明

- **环境变量与密钥**、**顺丰路由推送 URL 配置**、**按呼号查询页与订阅收卡**、**D1 表结构** 等见：  
  **[web_query_service/README.md](../web_query_service/README.md)**。

- **云端 API 规范**（GET /ping、POST /sync 请求/响应格式）见：  
  [cloud-sync-api-spec.md](cloud-sync-api-spec.md)。

- **顺丰路由推送**：服务端提供两条路径：正式 `POST /api/sf/route-push`、沙箱 `POST /api/sf/route-push/sandbox`；沙箱触发的用户推送内容带「【沙箱】」标记。请求/响应格式详见 OpenSpec 规范 `openspec/specs/sf-route-push-receiver/spec.md`。

## CDN 真实 IP 与限流

生产经**阿里云 CDN**（`qsl.herbert-dev.cn`，备案域名、大陆入口）回源到 **Cloudflare 源站**（`qsl.herbertgao.me`）。经 CDN 路径时 Worker 收到的 `CF-Connecting-IP` 是**阿里云 CDN 回源节点 IP、不是真实用户 IP**；真实用户 IP 由阿里云 CDN 在回源请求头里注入。若仍按 `CF-Connecting-IP` 计数，成千上万真实用户会被**归并到少数 CDN 回源 IP 桶**，限流粒度失真。

**信任信号 = 阿里云 CDN 注入的密钥回源头 `X-Origin-Auth`**，而非回源 IP 白名单。阿里云回源 IP **动态分配、官方明确不建议固定白名单**（"不建议在源站设置固定的回源 IP 列表，否则可能导致回源失败"），且查询回源 IP 的接口（`DescribeL2VipsByDomain`）有日峰值带宽 ≥1Gbps + 工单门槛——故本服务改用「CDN 注入、客户端伪造被覆盖、攻击者猜不到」的密钥头判定「确来自 CDN」。

> **边界**：本配置产物**仅**用作限流/防爬计数键（抬高自动化批量调用成本），**不**用作访问控制/鉴权判据。鉴权由 Bearer Key 与请求签名承担。IP 键被部分污染/坍缩，最坏是「成本抬升打折」，不构成鉴权绕过。

### 配置项含义

- **`CDN_ORIGIN_SECRET`** — 期望的 `X-Origin-Auth` 密钥值（机密）。请求带正确密钥（worker 常量时间比对）即视为「确来自阿里云 CDN 回源」，进而采信受信真实 IP 头。
  - 生成：`openssl rand -hex 32`。仅经 `wrangler secret put CDN_ORIGIN_SECRET` 注入；**禁写入仓库文件、禁经 `/api/config` 下发**（它与公开的 `CLIENT_SIGN_KEY` 是两个独立值：后者公开、前者机密）。
  - 未配置/为空 → fail-safe：只信 `CF-Connecting-IP`、忽略一切注入头。
- **`CDN_REAL_IP_HEADER`** — 受信真实 IP 头名，即阿里云 CDN 写入真实用户 IP 的请求头名。
  - **无内置默认**。未配置即 fail-safe 到 `CF-Connecting-IP`、不读任何注入头。
  - 推荐值 `Ali-Cdn-Real-Ip`：阿里云 CDN **默认携带**该头（无需配置），官方语义为「客户端与 CDN 节点建连时的真实 IP」、**正是为避免 `X-Forwarded-For` 被伪造**而设。但「覆写而非透传」须经下文「证伪式抓包门」实证后，才由运营者显式填入。
  - 采信时运行时还会校验该头为**单值、合法 IP 字面量（IPv4/IPv6）、不含逗号**；多值/含逗号/非法（覆写假设失效信号）→ 落 `'unknown'` 惩罚桶，不退回 CDN 节点 IP。
  - **绝不采信 `X-Forwarded-For`**：它是 append 语义，首段为客户端可伪造值。

### 阿里云 CDN 配置（注入密钥头 + 真实 IP 头）

1. **修改出站请求头**：CDN 控制台 → 域名管理 → `qsl.herbert-dev.cn` → 回源配置 → **修改出站请求头** → 新增，头名 `X-Origin-Auth`、值 `<CDN_ORIGIN_SECRET>`。操作类型**首选「替换」**（或「增加」+「是否允许重复」选**「否」**），使 CDN 的值**覆盖**客户端伪造的同名头。**严禁选「允许重复=是」**——那会变成追加（多值），worker 侧合并后 ≠ 密钥 → 静默 fail-safe 到 CDN 节点桶（安全但限流粒度失真）。该误配会被下文证伪门抓到。
2. **回源协议 = HTTPS**：CDN 控制台 → 回源配置 → 回源协议设为 **HTTPS**（或「跟随」+ 用户走 HTTPS），确保密钥头不在 CDN→Cloudflare 这一跳走明文。
3. `Ali-Cdn-Real-Ip` 默认自动携带，无需在阿里云额外配置。

### 部署顺序

配置顺序是依赖序、不是并列——**密钥头注入须先于 worker 启用采信**：

1. 阿里云配好上节的「修改出站请求头」（注入 `X-Origin-Auth`）+ 回源协议 HTTPS。
2. 部署 Worker（此时 `CDN_ORIGIN_SECRET`/`CDN_REAL_IP_HEADER` 可先不配，解析对所有路径 fail-safe，行为安全）。
3. `wrangler secret put CDN_ORIGIN_SECRET`（与阿里云注入值一致）。
4. **跑证伪式抓包门**（见下）证实 `Ali-Cdn-Real-Ip` 确被 CDN 覆写。
5. **仅在通过证伪门后**，才配 `CDN_REAL_IP_HEADER`（如 `Ali-Cdn-Real-Ip`）。

### 证伪式抓包门

临时给 worker 加一行 debug 日志，跑完即撤。**禁止明文打印密钥**：打印 `headers.get('X-Origin-Auth') === <期望密钥>` 的**布尔比对结果**（不打原值/不打期望值）、`headers.get('Ali-Cdn-Real-Ip')`、`resolveClientIP` 返回值。从**已知出口 IP 的测试机**经 `qsl.herbert-dev.cn` 发请求，故意带伪造头：

```
curl -s "https://qsl.herbert-dev.cn/api/config" \
  -H "X-Origin-Auth: forged-garbage" \
  -H "Ali-Cdn-Real-Ip: 8.8.8.8" \
  -H "ali-cdn-real-ip: 7.7.7.7" \
  -H "X-Forwarded-For: 9.9.9.9"
```

**唯一通过判据**：worker 收到的 `X-Origin-Auth` 布尔比对 = `true`（证 CDN 覆盖了客户端伪造的 `forged-garbage`），且 `Ali-Cdn-Real-Ip` = 该测试机真实出口 IP、≠ 任一伪造值、为单值（证 CDN 覆盖、客户端伪造无效）。任一不满足 = 门失败（头被透传/未覆写，须排查阿里云回源头/回源协议配置，保持 `CDN_REAL_IP_HEADER` 不配）。临时日志**禁止打印密钥原值**；可打印**本次测试机自身的出口 IP**（用于比对，非终端用户流量），但**禁止打印真实终端用户流量的 IP**；门通过后即移除（与 captcha-protection 主规范「不记录用户完整 IP」一致）。

### 维护与应急

- **密钥保密**：`CDN_ORIGIN_SECRET` 只存阿里云 CDN 配置 + Cloudflare secret 两处，收紧控制台访问权限。
- **应急轮换**（非定时；怀疑泄漏时）：阿里云改 `X-Origin-Auth` 新值 →（过渡期 worker 可临时同时接受新旧两值）→ `wrangler secret put CDN_ORIGIN_SECRET` 新值。
- CDN 侧配置（出站头/回源协议）变更后**须重跑证伪门**确认覆写仍成立。

### 未配影响

未配 `CDN_ORIGIN_SECRET`（或为空）→ fail-safe：只信 `CF-Connecting-IP`、忽略一切注入头。此时 **CDN 路径下限流仍按 CDN 回源节点 IP 计数（粒度失真但不被绕过）**。配齐密钥头 + 经证实的受信真实 IP 头名后，才恢复按真实用户 IP 的限流粒度。

### 回滚

Cloudflare dashboard 退回上一 Worker 版本 + 移除/还原配置项（尤其**清空 `CDN_ORIGIN_SECRET` 或 `CDN_REAL_IP_HEADER` → 立即 fail-safe**）。纯服务端逻辑，无 D1 迁移、桌面端/前端零改动。
