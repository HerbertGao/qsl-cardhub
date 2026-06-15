## 修改需求

### 需求：云端接收同步数据接口

系统**必须**在 Cloudflare Pages + D1 上提供与现有云端 API 规范兼容的接口，用于接收客户端全量同步数据。同步数据**必须**按服务端从写入 Key 解析出的 `tenant_id` 隔离写入（见 `tenant-isolation` 规范），**禁止**取请求体自报的 `client_id` 决定数据归属。`/sync` 的写入**必须**受乐观并发版本护栏约束（见 `cloud-sync-versioning` 规范）：携带有效 `base_version` 且非 `force` 时做 compare-and-swap，陈旧则 409 且零改动。

#### 场景：连接测试 GET /ping

- **当** 客户端发送 GET 请求到 `{api_url}/ping`，且请求头携带 `Authorization: Bearer {api_key}`
- **那么** 服务端必须校验 API Key 有效性；比对**必须**对 `env.API_KEY` 侧补 `trim`（token 侧 `getBearerToken` 已 `trim`），使 `trim` 语义与 `/sync` 一致——否则 `env.API_KEY` 含尾随空白时桌面端「测试连接」会 401，与本场景「行为不变」矛盾
- **并且** `env.API_KEY` 未配置（空/undefined）时 `/ping` **必须**返回 401，与 `/sync` 一致，**禁止**沿用现状「`env.API_KEY` 为空即跳过校验放行」（否则同一 secret 缺失下 `/sync` 401 而 `/ping` 200 的语义分裂会误导运维判「鉴权正常」）
- **并且** 校验通过时返回 200 与 JSON：`{ "success": true, "message": "pong", "server_time": "..." }`（与 `specs/cloud-database-support` 一致）
- **并且** 校验失败时返回 401 或相应错误信息

#### 场景：全量同步 POST /sync

- **当** 客户端发送 POST 请求到 `{api_url}/sync`，请求体包含 `client_id`、`sync_time`、`data`（projects、cards、sf_senders、sf_orders、app_settings），并**可选**携带 `base_version`（整数，所持云端基线）与 `force`（布尔）
- **那么** 服务端必须校验 Bearer API Key，并由该 Key 解析出 `tenant_id`（表驱动为主 + `env.API_KEY` 直比兜底，见 `tenant-isolation`）
- **并且** `client_id` **仍为请求体字段**（桌面端零改动继续发送，是设备实例 ID）；服务端**可保留**其存在性校验（缺失 → 400）作为**请求形态契约**，但**禁止**用它决定数据归属——租户身份**仅**由 Key 解析；`client_id` 仅记入 `sync_meta.last_client_id` 作溯源（即「请求体形态」与「身份解析来源」是两件分开钉死的事，实现**禁止**因「不信任 client_id」而连带删除形态校验、导致溯源丢失）
- **并且** `client_id` 是客户端可控字段，落 `sync_meta.last_client_id` 前**必须**长度归一 **≤128**（超长截断，不因超长拒绝合法桌面端），防超长串污染溯源列
- **并且** 当请求携带 `base_version` 且未带 `force=true` 时，写入**必须**走乐观并发护栏（见 `cloud-sync-versioning`）：仅当云端 `server_version == base_version` 才替换并 +1，否则返回 **409** 且**零数据改动**；当 `force=true` 或**未携带** `base_version`（旧桌面端）时走无条件覆盖 + 版本单调 +1（兼容降级）
- **并且** 必须仅删除该 `tenant_id` 名下各业务表的全部数据（`DELETE … WHERE tenant_id = ?`），**禁止**删除其他租户的数据
- **并且** 必须将本次同步的全量数据（含 app_settings 表）以解析出的 `tenant_id` 写入
- **并且** 上述「删除该租户全量 + 重新写入 + `sync_meta` 版本写」**必须**置于单个 `DB.batch` 事务，中途失败时**禁止**留下已删未写的空表状态；护栏路径下递增版本的 CAS **必须**为 batch 末条、且版本守卫覆盖全部 `DELETE/INSERT`（见 `cloud-sync-versioning` 原子性约束）；当兜底创始租户 `bh2ro`（当前 `env.API_KEY` 兜底解析出的租户）行数在 D1 单次调用查询上限（Paid 1000 / Free 50，按 batch 内语句条数计、与单语句行数无关）内时为真单 batch 原子（本期数据量预期可容纳）
- **并且** 若行数超 D1 单次上限须分块，则 `DELETE×5 + 首块 INSERT` **必须**至少同一 batch，且此时**必须**如实声明「跨块非原子」（或改影子表/版本切换实现真原子），**禁止**在 spec 声称分块仍全量原子
- **并且** 返回 200 与 JSON：`{ "success": true, "message": "同步成功", "received_at": "...", "server_version": <新版本号>, "stats": { "projects", "cards", "sf_senders", "sf_orders" } }`；其中 `server_version` **必须**为本次写入后的新版本，供客户端刷新基线
- **并且** 因基线陈旧拒绝时**必须**返回 **409** 与 JSON：`{ "success": false, "message": "...", "server_version": <云端当前版本号> }`，且**禁止**已对数据产生任何改动；当 `sync_meta` 行不存在导致 409 时 `server_version` **必须**为 `null`（或省略），**禁止** `undefined`/`NaN`（见 `cloud-sync-versioning`）
- **并且** 其他失败时返回非 200 或 `success: false` 及错误描述（含顶层未捕获异常 → 500），且错误响应**禁止**回显内部结构（与「错误响应脱敏」需求一致）

#### 场景：同步按租户全量替换且不影响其他租户

- **当** 解析出 `tenant_id` 为 "T1" 的客户端发起同步
- **并且** 云端数据库中存在 `tenant_id` 为 "T1" 与 "T2" 的历史记录
- **那么** 服务端在写入 "T1" 的数据前，必须仅删除各业务表（projects、cards、sf_senders、sf_orders、app_settings）中 `tenant_id = 'T1'` 的全部记录
- **并且** `tenant_id = 'T2'` 的数据**必须**原样保留、不受影响
- **并且** 最终 "T1" 名下仅存在本次同步写入的数据
- **并且** 在本期（路由未引入、写入由 Key 解析；当前 `env.API_KEY` 兜底恒解析为创始租户 `bh2ro`）下，等价于仅替换 `bh2ro` 租户数据（注：阶段 1 设计文档用通用占位 `default` 表述，落地创始租户 slug 实为 `bh2ro`）

#### 场景：云端 D1 app_settings 表结构

- **当** 云端 D1 数据库初始化时
- **那么** 必须创建 `app_settings` 表，包含 `tenant_id`（TEXT）、`key`（TEXT）、`value`（TEXT）三列
- **并且** 主键为 `(tenant_id, key)` 组合键，且**禁止**保留 `client_id` 列
- **并且** `/sync` 端点 DELETE 阶段必须按 `WHERE tenant_id = ?` 清除该租户的 `app_settings` 记录
- **并且** INSERT 阶段必须以解析出的 `tenant_id` 写入请求体 `data.app_settings` 中的所有键值对（如果该字段存在）

## 新增需求

### 需求：云端拉取同步数据接口

系统**必须**提供 `GET /pull` 端点，供持写入凭据的客户端拉回所属租户的全量数据快照与当前版本，用于换机初始化、被 409 挡住后先下载再续、或主动从云端恢复。鉴权与租户解析**必须**复用 `/sync` 的写入 Key 路径（`resolveTenant`），**禁止**用面向公众的查询签名 key（见 `cloud-sync-versioning`）。

#### 场景：拉取全量快照 GET /pull

- **当** 客户端发送 GET 请求到 `{api_url}/pull`，请求头携带 `Authorization: Bearer {写入 api_key}`
- **那么** 服务端**必须**按 Key 解析 `tenant_id`，未命中（或 `env.API_KEY` 缺失兜底也不命中）**必须**返回 401
- **并且** 命中时返回 200 与 JSON：`{ "success": true, "server_version": <整数>, "data": { "projects": [...], "cards": [...], "sf_senders": [...], "sf_orders": [...], "app_settings": [...] }, "last_client_id": "...", "sync_time": "..." }`
- **并且** `data` 各表的字段形态**必须**与 `/sync` 入参/桌面端 `export_database()` 的**对象/布尔**形态一致（而非 D1 原始存储形态）：`cards.metadata`、`sf_orders.sender_info`、`sf_orders.recipient_info` 在 D1 存为 JSON 字符串，`/pull` **必须** `JSON.parse` 还原为对象（NULL/空串容错）；`sf_senders.is_default` 存为整数，**必须**还原为布尔。否则桌面端反序列化失败、恢复不可用（见 `cloud-sync-versioning`）
- **并且** 若某行的 `metadata`/`sender_info`/`recipient_info` 存的非合法 JSON（数据损坏，正常不应发生——worker 入库经 `JSON.stringify`），`/pull` **必须** fail-closed 返回 500（脱敏），**禁止**静默跳过该行或返回半快照（半快照会让桌面端重建出残缺库）
- **并且** 读取**必须**注入服务端解析出的 `tenant_id`（`WHERE tenant_id = ?`），**禁止**跨租户返回数据
- **并且** 失败响应**禁止**回显内部结构（与「错误响应脱敏」需求一致）
