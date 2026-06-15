## 修改需求

### 需求：云端接收同步数据接口

系统**必须**在 Cloudflare Pages + D1 上提供与现有云端 API 规范兼容的接口，用于接收客户端全量同步数据。同步数据**必须**按服务端从写入 Key 解析出的 `tenant_id` 隔离写入（见 `tenant-isolation` 规范），**禁止**取请求体自报的 `client_id` 决定数据归属。

#### 场景：连接测试 GET /ping

- **当** 客户端发送 GET 请求到 `{api_url}/ping`，且请求头携带 `Authorization: Bearer {api_key}`
- **那么** 服务端必须校验 API Key 有效性；比对**必须**对 `env.API_KEY` 侧补 `trim`（token 侧 `getBearerToken` 已 `trim`），使 `trim` 语义与 `/sync` 一致——否则 `env.API_KEY` 含尾随空白时桌面端「测试连接」会 401，与本场景「行为不变」矛盾
- **并且** `env.API_KEY` 未配置（空/undefined）时 `/ping` **必须**返回 401，与 `/sync` 一致，**禁止**沿用现状「`env.API_KEY` 为空即跳过校验放行」（否则同一 secret 缺失下 `/sync` 401 而 `/ping` 200 的语义分裂会误导运维判「鉴权正常」）
- **并且** 校验通过时返回 200 与 JSON：`{ "success": true, "message": "pong", "server_time": "..." }`（与 `specs/cloud-database-support` 一致）
- **并且** 校验失败时返回 401 或相应错误信息

#### 场景：全量同步 POST /sync

- **当** 客户端发送 POST 请求到 `{api_url}/sync`，请求体包含 `client_id`、`sync_time`、`data`（projects、cards、sf_senders、sf_orders、app_settings）
- **那么** 服务端必须校验 Bearer API Key，并由该 Key 解析出 `tenant_id`（表驱动为主 + `env.API_KEY` 直比兜底，见 `tenant-isolation`）
- **并且** `client_id` **仍为请求体字段**（桌面端零改动继续发送，是设备实例 ID）；服务端**可保留**其存在性校验（缺失 → 400）作为**请求形态契约**，但**禁止**用它决定数据归属——租户身份**仅**由 Key 解析；`client_id` 仅记入 `sync_meta.last_client_id` 作溯源（即「请求体形态」与「身份解析来源」是两件分开钉死的事，实现**禁止**因「不信任 client_id」而连带删除形态校验、导致溯源丢失）
- **并且** `client_id` 是客户端可控字段，落 `sync_meta.last_client_id` 前**必须**长度归一 **≤128**（超长截断，不因超长拒绝合法桌面端），防超长串污染溯源列
- **并且** 必须仅删除该 `tenant_id` 名下各业务表的全部数据（`DELETE … WHERE tenant_id = ?`），**禁止**删除其他租户的数据
- **并且** 必须将本次同步的全量数据（含 app_settings 表）以解析出的 `tenant_id` 写入
- **并且** 上述「删除该租户全量 + 重新写入 + `sync_meta` upsert」**必须**置于单个 `DB.batch` 事务，中途失败时**禁止**留下已删未写的空表状态；当 `default` 行数在 D1 单次调用查询上限（Paid 1000 / Free 50 子请求）内时为真单 batch 原子（本期数据量预期可容纳）
- **并且** 若行数超 D1 单次上限须分块，则 `DELETE×5 + 首块 INSERT` **必须**至少同一 batch，且此时**必须**如实声明「跨块非原子」（或改影子表/版本切换实现真原子），**禁止**在 spec 声称分块仍全量原子
- **并且** 返回 200 与 JSON：`{ "success": true, "message": "同步成功", "received_at": "...", "stats": { "projects", "cards", "sf_senders", "sf_orders" } }`
- **并且** 失败时返回非 200 或 `success: false` 及错误描述，且错误响应**禁止**回显内部结构（与「错误响应脱敏」需求一致）

#### 场景：同步按租户全量替换且不影响其他租户

- **当** 解析出 `tenant_id` 为 "T1" 的客户端发起同步
- **并且** 云端数据库中存在 `tenant_id` 为 "T1" 与 "T2" 的历史记录
- **那么** 服务端在写入 "T1" 的数据前，必须仅删除各业务表（projects、cards、sf_senders、sf_orders、app_settings）中 `tenant_id = 'T1'` 的全部记录
- **并且** `tenant_id = 'T2'` 的数据**必须**原样保留、不受影响
- **并且** 最终 "T1" 名下仅存在本次同步写入的数据
- **并且** 在本期（路由未引入、写入恒解析为 `default`）下，等价于仅替换 `default` 租户数据

#### 场景：云端 D1 app_settings 表结构

- **当** 云端 D1 数据库初始化时
- **那么** 必须创建 `app_settings` 表，包含 `tenant_id`（TEXT）、`key`（TEXT）、`value`（TEXT）三列
- **并且** 主键为 `(tenant_id, key)` 组合键，且**禁止**保留 `client_id` 列
- **并且** `/sync` 端点 DELETE 阶段必须按 `WHERE tenant_id = ?` 清除该租户的 `app_settings` 记录
- **并且** INSERT 阶段必须以解析出的 `tenant_id` 写入请求体 `data.app_settings` 中的所有键值对（如果该字段存在）
