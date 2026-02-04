## 新增需求

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
- **并且** 按 `client_id` 隔离写入 D1（覆盖或按业务规则合并该 client 的同步数据）
- **并且** 返回 200 与 JSON：`{ "success": true, "message": "同步成功", "received_at": "...", "stats": { "projects", "cards", "sf_senders", "sf_orders" } }`
- **并且** 失败时返回非 200 或 `success: false` 及错误描述

---

### 需求：按呼号查询收卡信息接口

系统**必须**提供基于呼号查询收卡信息的接口，供「根据呼号查询收卡信息的单独页面」及外部调用使用；该单独页面在展示查询结果时提供「订阅收卡」按钮，用于微信绑定（见 wechat-push 规范）。

#### 场景：按呼号查询成功

- **当** 调用方发送 GET 请求到按呼号查询接口（如 `GET /api/callsigns/:callsign` 或 `GET /api/query?callsign=BG7XYZ`），且通过认证（若接口要求）
- **那么** 服务端必须从 D1 中查询该呼号对应的卡片数据（基于同步得到的 cards 及关联的 projects）
- **并且** 返回该呼号下的收卡信息：至少包含项目名称、数量、状态、分发备注（若有）等与收卡相关的字段
- **并且** 若无该呼号数据则返回空列表或 404，不泄露其他呼号数据

#### 场景：呼号查询未授权

- **当** 按呼号查询接口要求认证且未提供有效 API Key 或 Token
- **那么** 服务端必须返回 401 或 403
- **并且** 不返回任何卡片数据

#### 场景：查询接口供查询页与订阅入口使用

- **当** 部署「根据呼号查询收卡信息的单独页面」时
- **那么** 该页面必须调用本按呼号查询接口展示该呼号下的收卡信息
- **并且** 在展示结果时提供「订阅收卡」按钮与提示（订阅后将收到该呼号的卡片分发/物流信息），点击后进入微信授权并完成呼号–openid 绑定（见 wechat-push 规范）
