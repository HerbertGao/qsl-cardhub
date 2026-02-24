## MODIFIED Requirements

### 需求：云端接收同步数据接口

系统**必须**在 Cloudflare Pages + D1 上提供与现有云端 API 规范兼容的接口，用于接收客户端全量同步数据。

#### 场景：连接测试 GET /ping

- **当** 客户端发送 GET 请求到 `{api_url}/ping`，且请求头携带 `Authorization: Bearer {api_key}`
- **那么** 服务端必须校验 API Key 有效性
- **并且** 校验通过时返回 200 与 JSON：`{ "success": true, "message": "pong", "server_time": "..." }`（与 `specs/cloud-database-support` 一致）
- **并且** 校验失败时返回 401 或相应错误信息

#### 场景：全量同步 POST /sync

- **当** 客户端发送 POST 请求到 `{api_url}/sync`，请求体包含 `client_id`、`sync_time`、`data`（projects、cards、sf_senders、sf_orders、app_settings）
- **那么** 服务端必须校验 Bearer API Key
- **并且** 必须先删除该 `client_id` 下的所有现有数据（包括 app_settings 表）
- **并且** 写入本次同步的全量数据（包括 app_settings 表）
- **并且** 返回 200 与 JSON：`{ "success": true, "message": "同步成功", "received_at": "...", "stats": { "projects", "cards", "sf_senders", "sf_orders" } }`
- **并且** 失败时返回非 200 或 `success: false` 及错误描述

#### 场景：云端 D1 app_settings 表结构

- **当** 云端 D1 数据库初始化时
- **那么** 必须创建 `app_settings` 表，包含 `client_id`（TEXT）、`key`（TEXT）、`value`（TEXT）三列
- **并且** 主键为 `(client_id, key)` 组合键
- **并且** `/sync` 端点 DELETE 阶段必须清除该 `client_id` 下所有 `app_settings` 记录
- **并且** INSERT 阶段必须写入请求体 `data.app_settings` 中的所有键值对（如果该字段存在）
