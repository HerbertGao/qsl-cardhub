## 新增需求

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

## 修改需求

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
