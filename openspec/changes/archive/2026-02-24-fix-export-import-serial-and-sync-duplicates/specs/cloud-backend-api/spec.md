## MODIFIED Requirements

### 需求：云端接收同步数据接口

系统**必须**在 Cloudflare Pages + D1 上提供与现有云端 API 规范兼容的接口，用于接收客户端全量同步数据。

#### 场景：连接测试 GET /ping

- **当** 客户端发送 GET 请求到 `{api_url}/ping`，且请求头携带 `Authorization: Bearer {api_key}`
- **那么** 服务端必须校验 API Key 有效性
- **并且** 校验通过时返回 200 与 JSON：`{ "success": true, "message": "pong", "server_time": "..." }`（与 `specs/cloud-database-support` 一致）
- **并且** 校验失败时返回 401 或相应错误信息

#### 场景：全量同步 POST /sync

- **当** 客户端发送 POST 请求到 `{api_url}/sync`，请求体包含 `client_id`、`sync_time`、`data`（projects、cards、sf_senders、sf_orders）
- **那么** 服务端必须校验 Bearer API Key
- **并且** 必须先删除该 `client_id` 下的所有现有数据
- **并且** 必须额外删除其他 `client_id` 下与本次同步数据 UUID（`id` 字段）重复的记录，防止跨客户端重复数据
- **并且** 写入本次同步的全量数据
- **并且** 返回 200 与 JSON：`{ "success": true, "message": "同步成功", "received_at": "...", "stats": { "projects", "cards", "sf_senders", "sf_orders" } }`
- **并且** 失败时返回非 200 或 `success: false` 及错误描述

#### 场景：同步清除跨客户端重复数据

- **当** 云端数据库中 client_id 为 "AAA" 的记录包含卡片 id 为 "card-001"
- **并且** 客户端使用 client_id "BBB" 同步的数据中也包含卡片 id 为 "card-001"
- **那么** 服务端在写入 "BBB" 的数据前，必须删除 "AAA" 下的 "card-001" 记录
- **并且** 最终数据库中 "card-001" 仅存在于 client_id "BBB" 下
