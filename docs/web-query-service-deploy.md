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

- **顺丰路由推送** 请求/响应格式见 [sf-route-push-service.md](sf-route-push-service.md)。服务端提供两条路径：正式 `POST /api/sf/route-push`、沙箱 `POST /api/sf/route-push/sandbox`；沙箱触发的用户推送内容带「【沙箱】」标记。
