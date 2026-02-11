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

#### 场景：查询结果页按分发方式展示已分发记录

- **当** 查询结果中的某条记录 `status = distributed`
- **并且** 返回的 `distribution.method = 自取`
- **那么** 结果页必须在该记录中显示“自取”
- **并且** 该“自取”展示禁止出现复制按钮
- **并且** 当 `distribution.remarks` 存在时，结果页仍必须显示备注并提供复制按钮

- **当** 查询结果中的某条记录 `status = distributed`
- **并且** 返回的 `distribution.method = 代领`
- **那么** 结果页必须显示“代领”
- **并且** 当 `distribution.proxy_callsign` 有值时必须同时显示“代领人：<呼号>”
- **并且** 分发方式展示禁止出现复制按钮

- **当** 查询结果中的某条记录 `status = returned`
- **那么** 结果页必须在分发方式位置显示退卡处理方式（如"查无此人"、"呼号无效"、"拒收"、"其他"）
- **并且** 当 `return.remarks` 存在时，结果页必须显示退卡备注并提供复制按钮
- **并且** 结果页禁止显示"自取/代领/代领人"分发方式扩展文案

- **当** 查询结果中的某条记录 `status` 既非 `distributed` 也非 `returned`
- **那么** 结果页禁止显示分发方式、退卡方式等扩展文案

#### 场景：按呼号查询成功

- **当** 调用方发送 GET 请求到按呼号查询接口（如 `GET /api/callsigns/:callsign` 或 `GET /api/query?callsign=BG7XYZ`），且通过认证（若接口要求）
- **那么** 服务端必须从 D1 中查询该呼号对应的卡片数据（基于同步得到的 cards 及关联的 projects）
- **并且** 响应体必须只包含前端展示所需的最小字段集，每个卡片项目必须包含且仅包含：
  - `id`: 卡片唯一标识
  - `project_name`: 项目名称
  - `status`: 卡片状态（pending / distributed / returned）
  - `distribution`: 分发信息对象（已分发或已退卡时存在，退卡后保留原分发记录），包含：
    - `method`: 分发方式（如自取、代领、邮寄等）
    - `proxy_callsign`: 代领呼号（仅代领时存在）
    - `remarks`: 分发备注
  - `return`: 退卡信息对象（仅当已退卡时存在），包含：
    - `method`: 退卡处理方式（NOT FOUND / CALLSIGN INVALID / REFUSED / OTHER）
    - `remarks`: 退卡备注
- **并且** 禁止返回以下字段：`project_id`、`callsign`（冗余）、`qty`、`serial`、`created_at`、`updated_at`、`metadata`（完整对象）、分发地址、退卡时间
- **并且** 若无该呼号数据则返回空列表，不泄露其他呼号数据

#### 场景：呼号查询未授权

- **当** 按呼号查询接口要求认证且未提供有效 API Key 或 Token
- **那么** 服务端必须返回 401 或 403
- **并且** 不返回任何卡片数据

#### 场景：查询接口供查询页与订阅入口使用

- **当** 部署「根据呼号查询收卡信息的单独页面」时
- **那么** 该页面必须调用本按呼号查询接口展示该呼号下的收卡信息
- **并且** 在展示结果时提供「订阅收卡」按钮与提示（订阅后将收到该呼号的卡片分发/物流信息），点击后进入微信授权并完成呼号–openid 绑定（见 wechat-push 规范）
