## 目的

QSL CardHub 的云端后端 API（Cloudflare Workers + D1）：接收桌面端全量同步数据、提供按呼号查询收卡信息接口，供「按呼号查询页」与微信订阅入口使用，并承载错误响应脱敏、订阅端点限流、密钥配置卫生等安全约束。
## 需求
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

- **当** `GET /api/query`、`GET /api/callsigns/:callsign` 的请求缺少有效会话 token、伪造会话签名，或会话配额已用尽
- **那么** 访问控制**必须**为「有效会话 token + 会话签名 + 会话配额」（语义见 `query-antibot-session`），**取代**原静态签名鉴权：无有效会话/伪造签名→401/403，配额用尽→429
- **并且** 拒绝码优先级**必须**为：Layer0 纯 IP 限流查询桶 `ratelimit:<ip>`（429）**先于**会话校验——「被限流」一律先返 429（无论有无会话），「未被限流但无有效会话/伪造签名」才返 401/403
- **并且** 不返回任何卡片数据

#### 场景：查询接口供查询页与订阅入口使用

- **当** 部署「根据呼号查询收卡信息的单独页面」时
- **那么** 该页面必须调用本按呼号查询接口展示该呼号下的收卡信息
- **并且** 在展示结果时提供「订阅收卡」按钮与提示（订阅后将收到该呼号的卡片分发/物流信息），点击后进入微信授权并完成呼号–openid 绑定（见 wechat-push 规范）

### 需求：服务端错误响应脱敏

云端服务向客户端返回错误时**必须**脱敏，**禁止**回显内部异常细节或上游服务的原始错误结构（异常消息、堆栈、SQL/数据库约束、第三方 API 的 errcode/errmsg/序列化响应体）。覆盖范围**必须**包含顶层未捕获异常处理与各业务分支显式构造的错误响应。

#### 场景：内部异常返回通用错误

- **当** 请求处理过程中发生未预期的内部异常（顶层 catch）
- **那么** 服务端必须返回通用 `{ "success": false, "message": "服务器错误" }`（HTTP 500）
- **并且** **禁止**在响应体中包含原始异常消息或内部实现细节
- **并且** 详细异常仅记录于服务端日志

#### 场景：上游服务错误不回显原始结构

- **当** 业务分支（如微信授权回调）从上游服务收到错误响应
- **那么** 服务端必须向客户端返回通用、用户可读的失败信息（如「微信授权失败」），**禁止**回显上游原始 errcode/errmsg 或序列化的响应结构
- **并且** 上游原始错误仅记录于服务端日志

### 需求：签名密钥与服务凭据的配置卫生

云端服务的各服务凭据（会话 HMAC 密钥 `SESSION_SECRET`、顺丰 checkword、微信 secret 等）**必须**遵循配置卫生：真实值**禁止**写入纳入版本控制的文件（含 `wrangler.toml.example`、README 等文档），**必须**经 Cloudflare Secret 或部署期注入提供。查询签名密钥已改为**会话专属、短时**的 `sk`（经 `POST /api/session` 响应下发，见 `query-antibot-session`）；静态 `CLIENT_SIGN_KEY` 与算术验证码密钥 `CAPTCHA_SECRET` **已退役**，**禁止**再经 `/api/config` 或任何静态途径下发可公开的查询签名密钥。

> 说明：原静态 `CLIENT_SIGN_KEY` 经 `/api/config` 明文下发、属「可公开值」、对查询防爬零收益；本阶段以会话动态签名取代之，查询访问的抬成本防护由 PoW+会话+配额承担（`query-antibot-session`）。

#### 场景：密钥不写入版本控制文件

- **当** 配置或部署云端服务
- **那么** 服务端密钥（`SESSION_SECRET` 等）的真实值**必须**经 Secret 提供
- **并且** 纳入版本控制的文件中**禁止**出现真实值，仅可出现占位符
- **并且** **禁止**经 `/api/config` 或静态途径下发任何查询签名密钥（`CLIENT_SIGN_KEY` 退役）

#### 场景：退役密钥不再下发

- **当** 客户端请求 `GET /api/config`
- **那么** 响应**禁止**包含 `sign_key`（或任何静态查询签名密钥）与算术验证码相关配置
- **并且** 查询签名密钥仅经 `POST /api/session` 响应以会话专属 `sk` 形式下发

### 需求：订阅绑定端点的基础限流

微信订阅绑定回调（`/api/wechat/auth-callback`）**必须**复用现有 IP 限流机制（`checkRateLimit`）纳入限流，以在防爬体系（后续阶段）就位前，阻止对绑定接口的自动化批量调用。限流**必须**使用**独立于查询端点的计数键**（如 `ratelimit:authcb:${ip}`），**禁止**与查询端点共用同一计数桶，避免查询流量与订阅流量互相挤占预算导致正常订阅被饿死。计数键中的 IP **必须**取自「可信真实客户端 IP 解析」（见 `trusted-client-ip` 规范）得到的真实用户 IP，**禁止**直接使用 `CF-Connecting-IP` 或任何客户端可注入头（`X-Forwarded-For` 首段等）作安全计数键——在前置阿里云 CDN 架构下，经 CDN 回源时 `CF-Connecting-IP` 是 CDN 回源节点 IP（会把大量真实用户归并到少数桶、限流粒度失真），而 `X-Forwarded-For` 在 append 语义下首段为客户端可伪造值（采信即被单请求绕过、使本防护沦为剧场）；唯有经密钥回源头（`X-Origin-Auth`）校验、采信 CDN 写入的不可伪造真实 IP 头的可信解析，能在两种入口下都按真实用户计数。

> 说明：此处「复用 `checkRateLimit`」指复用其 IP 限流**机制**，阈值当前可与查询一致；不绑定查询端点未来的防护形态（查询端点在后续阶段将升级为 PoW+会话，订阅回调作为 OAuth 被动跳转**不**随之升级）。本限流是**抬高自动化批量调用成本**（anti-abuse），**不是访问控制**；防止滥用绑定的真正闸门是微信 OAuth `code` 必须有效（攻击者拿不到任意呼号订阅者的 code）。

#### 场景：绑定回调按独立 IP 桶限流

- **当** 同一真实用户 IP 在限流窗口内对 `/api/wechat/auth-callback` 的请求超过阈值
- **那么** 服务端**必须**按独立计数键拒绝超额请求
- **并且** 该限流**禁止**消耗或被查询端点的限流预算消耗（独立计数桶）
- **并且** 正常单次订阅不受影响

#### 场景：CDN 路径下按真实用户 IP 计数

- **当** 请求经阿里云 CDN 回源到达 `/api/wechat/auth-callback`（`CF-Connecting-IP` 为 CDN 回源节点、真实用户 IP 由 CDN 写入受信真实 IP 头）
- **那么** 限流计数键中的 IP **必须**为经密钥回源头校验后采信的真实用户 IP（取自 CDN 写入的不可伪造头，非 `X-Forwarded-For` 首段）
- **并且** **禁止**将不同真实用户归并到同一 CDN 回源节点 IP 的计数桶
- **并且** 客户端伪造 `X-Forwarded-For` 经 CDN 透传后**禁止**绕过或污染该限流

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

### 需求：会话握手端点

云端服务**必须**提供查询前的会话握手端点，承载防爬 PoW + 短时会话（语义见 `query-antibot-session`）：

- `GET /api/session/challenge`：下发 PoW 题 `{seed, difficulty}`。
- `POST /api/session`：校验 PoW（`{seed, nonce}`）通过后签发会话，响应 `{token, sk, exp, quota}`（`sk`=会话专属签名密钥，经响应体下发）。

二者**必须**复用 `RATE_LIMIT` KV（seed 一次性防重放、会话状态/配额、按真实 IP 的建会话频率）；KV 未绑定时**必须** fail-closed（503），**禁止**静默放行。

#### 场景：会话握手两步

- **当** 客户端先 `GET /api/session/challenge` 取 `{seed, difficulty}`、算出满足难度的 `nonce`、再 `POST /api/session {seed, nonce}`
- **那么** PoW 通过时服务端**必须**返回 `{token, sk, exp, quota}` 并将 `seed` 标记已用
- **并且** PoW 不足或 `seed` 重放/过期时**必须**拒绝签发（4xx）

#### 场景：握手依赖 KV，未绑定 fail-closed

- **当** `RATE_LIMIT` KV 未绑定
- **那么** 会话握手端点**必须**返回 503（功能不可用），**禁止**因缺 KV 而静默放行无会话查询

