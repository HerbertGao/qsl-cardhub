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

为此服务支持两项配置，让限流/防爬按真实用户 IP 计数。

> **边界**：这两项配置的产物**仅**用作限流/防爬计数键（抬高自动化批量调用成本），**不**用作访问控制/鉴权判据。鉴权由 Bearer Key 与请求签名承担。IP 键被部分污染/坍缩，最坏是「成本抬升打折」，不构成鉴权绕过。

### 配置项含义

- **`CDN_ORIGIN_CIDRS`** — 阿里云 CDN 回源 IP 段白名单，逗号/空白分隔的 CIDR 串。只有当请求的 `CF-Connecting-IP` 落在该白名单内，才认为「这一跳来自受信 CDN 回源」，进而进入采信注入头的判定。
  - **仅接受全局可路由的公网单播 IPv4** CIDR；私网（`10/8`、`172.16/12`、`192.168/16`）、CGNAT（`100.64.0.0/10`）、loopback（`127/8`）、link-local（`169.254/16`）、组播（`224.0.0.0/4`）、保留（`240.0.0.0/4`、`0.0.0.0/8`）、`/0`、非法前缀/IP 会被解析阶段**一律丢弃**。这些落进回源白名单几乎一定是误配，会把大片来源当成可信 CDN。
  - **禁止硬编码进代码**，由部署期经 `[vars]` 或 Secret 注入。
- **`CDN_REAL_IP_HEADER`** — 受信真实 IP 头名，即阿里云 CDN 写入真实用户 IP 的请求头名。
  - **无内置默认**。未配置即视为「未证实覆写」（安全态），解析 fail-safe 到 `CF-Connecting-IP`、不读任何注入头。
  - 推荐值 `Ali-Cdn-Real-Ip`（阿里云官方语义：单值、由 CDN 节点按 TCP 层对端写入并**覆盖**客户端同名头）。但该「覆写而非透传」是单点信任根，**必须**经下文「证伪式抓包门」实证该头名确被 CDN 覆写之后，才由运营者显式填入。
  - 采信时运行时还会校验该头为**单值、合法 IP 字面量、不含逗号**；多值/含逗号/非法（覆写假设失效信号）→ 落 `'unknown'` 惩罚桶，不退回 CDN 节点 IP。

### 从阿里云获取回源 IP 段

阿里云回源 IP 段会随阿里云调整而变化，须从官方来源导出、不可猜测或硬编码：

- **OpenAPI**：调用 `DescribeCdnBackSourceIp` 获取当前回源节点 IP/IP 段。
- **控制台**：CDN 控制台的「回源」/「回源 IP」相关页面也可查询导出。

把导出的公网单播 IPv4 CIDR 填入 `CDN_ORIGIN_CIDRS`。

### 维护周期与纪律

- **定期校验并更新**白名单：阿里云回源段变更后及时同步。
  - **缺失新段**：降级为按 CDN 节点 IP 计数（失真但不被绕过，fail-safe 兜底，非安全事故）。
  - **保留废弃/过宽段**：= 安全洞，任意命中该段的请求都会被当作可信 CDN、采信其注入头。故须**宁缺勿宽 + 最小化**，及时移除废弃/过宽段。
- **白名单与受信头名的任何变更需人工 sign-off**；`CDN_REAL_IP_HEADER` 的启用或 CDN 侧配置变更后**必须重跑下文证伪式抓包门**证实覆写仍成立。

### 部署顺序硬约束（信任前置）

源站 `qsl.herbertgao.me` **必须仅接受来自阿里云回源段 / Cloudflare 的回源**（Cloudflare WAF/Firewall Rules）。否则攻击者只要其真实源 IP 落在白名单段内（如在阿里云回源段内开 ECS，或白名单过宽/过期），即可直连源站——`CF-Connecting-IP` 命中白名单——再伪造注入头被采信。**白名单本身保证不了「请求真来自 CDN」。**

这条限制**必须先于**配置 `CDN_ORIGIN_CIDRS` / 启用采信头生效。**中间态比都不配更危险**：「配了白名单但没配源站回源限制」会让伪造头被采信，而「都不配」时 fail-safe 到 `CF-Connecting-IP`（不被伪造）。因此配置顺序是依赖序、不是并列：

1. 部署 Worker（此时 `CDN_REAL_IP_HEADER` **不配**，解析对所有路径 fail-safe，行为安全）。
2. **先**在 Cloudflare 配好「源站仅接受阿里云回源段 / CF 回源」（WAF/Firewall Rules）并使其**生效**。
3. **再**配 `CDN_ORIGIN_CIDRS`（此时命中白名单但头名仍未配 → 仍 fail-safe 到 CF-IP，尚未采信任何注入头）。
4. **跑证伪式抓包门**：从已知出口 IP 的测试机经 `qsl.herbert-dev.cn` 发请求，故意带伪造的目标头（含大小写变体，如 `Ali-Cdn-Real-Ip: 8.8.8.8` + `ali-cdn-real-ip: 7.7.7.7`）与伪造 `X-Forwarded-For: 9.9.9.9`，临时日志打印 (a) 该头原始值与 (b) 解析最终返回值。**唯一通过判据**：(b) = 该机器真实出口 IP、≠ 任一伪造值、且为单值（证实 CDN 对该头名做了覆写）。其余结果（(a) 含逗号/多值，或 (b) = 伪造值）皆为门失败 = 该头名被透传/未覆写、不可用，须排查 CDN 配置并保持头名未配。临时日志须用已知出口 IP 的测试机、禁止打印真实终端用户 IP、门通过后即移除。
5. **仅在该头名通过证伪门后**，才显式配置 `CDN_REAL_IP_HEADER`（如 `Ali-Cdn-Real-Ip`）。

### 未配影响

未配 `CDN_ORIGIN_CIDRS`（或解析为空）→ fail-safe：只信 `CF-Connecting-IP`、忽略一切注入头。此时 **CDN 路径下限流仍按 CDN 回源节点 IP 计数（粒度失真但不被绕过）**。配齐白名单 + 源站回源限制 + 经证实的受信头名后，才恢复按真实用户 IP 的限流粒度。

### 回滚

Cloudflare dashboard 退回上一 Worker 版本 + 移除/还原配置项（尤其**清空 `CDN_REAL_IP_HEADER` → 立即 fail-safe**）。纯服务端逻辑，无 D1 迁移、桌面端/前端零改动。
