# 云端同步 API 规范（自托管实现指引）

> **本文档为非规范性（non-normative）的自托管实现指引。**
> 同步面契约的**唯一真源**是 OpenSpec 主规范 [`openspec/specs/cloud-backend-api/spec.md`](../openspec/specs/cloud-backend-api/spec.md)（及其引用的 `tenant-isolation`、`cloud-sync-versioning` 规范）。本文档**跟踪**该主规范，便于自托管者实现一个能与桌面端对接的同步后端；**任何冲突一律以 `cloud-backend-api` 主规范为准**。
> 因此，下文所有端点字段以**示例 / 样例**呈现（标注「规范以 `cloud-backend-api` 为准」），而**不**写成「系统必须……」这类与主规范平行的规范性断言——避免归档后形成两份会相互漂移的契约真源。

## 概述

QSL-CardHub 支持把本地数据全量同步到云端 API（官方实现见本仓库 `web_query_service`，Cloudflare Workers + D1）。同步采用推送模式，客户端主动把数据推送到服务端，并可经 `GET /pull` 拉回云端快照用于换机 / 恢复 / 被 409 挡住后的续传。

- **同步方向**：双向（本地 → 云端推送 `POST /sync`；云端 → 本地拉取 `GET /pull`）
- **同步模式**：全量同步（每次推送 / 拉取该租户完整数据）
- **认证方式**：`Authorization: Bearer <key>`，由服务端**从 Key 解析出租户**（表驱动），可选 `X-Tenant-Id` 头交叉校验
- **并发控制**：`POST /sync` 走乐观并发版本护栏（OCC，`base_version` / `server_version`）

## 认证与租户模型

> 规范以 `cloud-backend-api` 为准（见「写入/探活端点声明租户交叉校验」「云端接收同步数据接口」需求）。下为实现指引示例。

所有同步端点（`/ping`·`/sync`·`/pull`）在请求头携带 Bearer Key：

```
Authorization: Bearer <your_key>
```

服务端的鉴权与租户解析示例约定：

- **从 Key 解析租户**：服务端对 `sha256(trim(key))` 做**表驱动**查找（在凭据表里按 `key_hash` 命中得到 `tenant_id`），鉴权只看 `key_hash` 与 `status='active'`，**不读 `scope`**。同一租户可签发多把 Key（多 Key → 同一租户）。
- **`X-Tenant-Id`（可选）交叉校验**：客户端可额外携带 `X-Tenant-Id: <声明的租户 slug>`。服务端把它与「Key 解析出的 `tenant_id`」比对：
  - **一致** → 正常处理（与不带该头时行为等价）。
  - **不一致** → **HTTP 403**，响应 `code` 为 `tenant_mismatch`。
  - **缺头 / 空串 / 纯空白**（值经 `trim` 后为空）→ 跳过交叉校验、向后兼容放行（归属仍由 Key 决定）。
  - 注意：`X-Tenant-Id` **仅**用于校验 + 回显，**绝不**当写入 / 读取目标——入库 / 读取的 `tenant_id` 恒取 Key 解析值。
- **Key 无效**（解析不命中）→ **HTTP 401**，响应 `code` 为 `auth_failed`。
- **结构化错误码是契约、`message` 不是契约**：客户端**必须**按 HTTP 状态码 + `code` 字段区分（`auth_failed` / `tenant_mismatch`），**禁止**依赖 `message` 文案（人类可读、可变）。`code` 是向后兼容的新增字段，既有客户端按状态码短路、不读 `code` 仍正常。

错误码示例：

```jsonc
// 401 Key 无效
{ "success": false, "code": "auth_failed", "message": "API Key 无效" }
// 403 声明租户与凭据不一致
{ "success": false, "code": "tenant_mismatch", "message": "申报租户与凭据不一致" }
```

**默认单租户**：自托管最简部署可只用一个租户，slug 取 `default`（须与 `env.DEFAULT_TENANT` 一致，详见部署文档「新增租户与自托管」节）。

## 端点契约（示例）

> 以下请求 / 响应样例标注了字段类型与 null 情形，**规范以 `cloud-backend-api` 为准**。

### 1. 连接测试 GET /ping

回显 Key 解析出的租户身份，供桌面端「测试连接」确认归属与模式。

**请求**

```http
GET /ping
Authorization: Bearer <key>
X-Tenant-Id: default        # 可选，交叉校验
```

**响应（成功，200）**

```jsonc
{
  "success": true,
  "message": "pong",
  "server_time": "2026-06-18T14:30:00+08:00",
  "tenant": "default",   // string，Key 解析出的 tenant_id
  "fallback": false      // boolean，true 表示本次经 env.API_KEY 兜底命中（过渡机制）
}
```

`tenant` 与 `fallback` 两字段均会返回。失败时按上节返回 401 `auth_failed` 或 403 `tenant_mismatch`。

### 2. 数据同步 POST /sync

接收客户端推送的全量数据，按 Key 解析出的租户全量替换，受 OCC 版本护栏约束。

**请求**

```http
POST /sync
Authorization: Bearer <key>
X-Tenant-Id: default        # 可选
Content-Type: application/json
```

**请求体**

```jsonc
{
  // client_id：必填请求字段（≤128，超长截断）。仅写入服务端 last_client_id 作设备溯源，
  // 永不决定数据归属（归属仅由 Key 解析）。缺失 → 400。它不是隔离键。
  "client_id": "550e8400-e29b-41d4-a716-446655440000",
  "sync_time": "2026-06-18T14:30:00+08:00",
  // base_version：可选。OCC 基线，须为「严格整数」（非整数 / 字符串如 "5" 一律按未携带处理，降级为无条件覆盖）。
  "base_version": 7,
  // force：可选。仅当 === true（布尔，非字符串 "true"）时绕过 OCC 走无条件覆盖。
  "force": false,
  "data": {
    "projects": [ /* … */ ],
    "cards": [ /* … */ ],
    "sf_senders": [ /* … */ ],
    "sf_orders": [ /* … */ ],
    "app_settings": [ { "key": "theme", "value": "dark" } ]   // 键值对数组，会被同步
  }
}
```

并发护栏行为：

- 携带**严格整数** `base_version` 且 `force` 非 `true` → 走 compare-and-swap：仅当云端 `server_version === base_version` 才全量替换并 `+1`，否则 **409 且零数据改动**。
- `force === true` 或**未携带**（含非整数） `base_version` → 无条件覆盖 + 版本单调 `+1`（兼容旧桌面端）。

**响应（成功，200）**

```jsonc
{
  "success": true,
  "message": "同步成功",
  "received_at": "2026-06-18T14:30:01+08:00",
  "server_version": 8,        // number，本次写入后的新版本，客户端据此刷新基线
  "stats": { "projects": 10, "cards": 500, "sf_senders": 5, "sf_orders": 100 }
}
```

**响应（基线陈旧，409）**

```jsonc
{
  "success": false,
  "message": "云端数据已更新，本地基线已陈旧",
  // 云端当前版本号；当 sync_meta 行不存在时为 null（禁 undefined/NaN）
  "server_version": 12
}
```

**响应（缺 client_id 或 data，400）**

```jsonc
{ "success": false, "message": "缺少 client_id 或 data" }
```

### 3. 拉取快照 GET /pull

供持写入凭据的客户端拉回本租户全量快照与当前版本（换机初始化、被 409 挡后先下载再续、主动恢复）。鉴权与租户解析复用 `/sync` 的写入 Key 路径。

**请求**

```http
GET /pull
Authorization: Bearer <key>
X-Tenant-Id: default        # 可选
```

**响应（成功，200）**

```jsonc
{
  "success": true,
  "server_version": 8,            // number；该租户尚无同步记录时为 null
  "data": {
    "projects": [ { "id": "…", "name": "…", "created_at": "…", "updated_at": "…" } ],
    "cards": [
      {
        "id": "…", "project_id": "…", "creator_id": null,
        "callsign": "BV2AAA", "qty": 1, "serial": null, "status": "pending",
        // metadata 在 D1 存为 JSON 字符串；/pull 还原为对象。NULL/空串 → null。
        "metadata": { "distribution": { "method": "邮寄" }, "return": null },
        "created_at": "…", "updated_at": "…"
      }
    ],
    "sf_senders": [
      {
        "id": "…", "name": "…", "phone": "…", "mobile": null,
        "province": "…", "city": "…", "district": "…", "address": "…",
        // is_default 在 D1 存为整数 0/1；/pull 还原为布尔。
        "is_default": true,
        "created_at": "…", "updated_at": "…"
      }
    ],
    "sf_orders": [
      {
        "id": "…", "order_id": "…", "waybill_no": null, "card_id": null,
        "status": "confirmed", "pay_method": 1, "cargo_name": "QSL卡片",
        // sender_info / recipient_info 在 D1 存为 JSON 字符串；/pull 还原为对象（空 → {}）。
        "sender_info": { /* … */ },
        "recipient_info": { /* … */ },
        "created_at": "…", "updated_at": "…"
      }
    ],
    "app_settings": [ { "key": "theme", "value": "dark" } ]
  },
  "last_client_id": "550e8400-…",   // string；尚无记录时为 null
  "sync_time": "2026-06-18T14:30:00+08:00"   // string；尚无记录时为 null
}
```

形态还原要点（实现指引）：D1 把 `cards.metadata`、`sf_orders.sender_info`、`sf_orders.recipient_info` 存为 **JSON 字符串**、把 `sf_senders.is_default` 存为**整数**；`/pull` 须把它们分别 `JSON.parse` 还原为**对象**、`!!` 还原为**布尔**，以匹配桌面端 `export_database()` 的形态，否则桌面端反序列化失败、恢复不可用。`last_client_id` / `sync_time` 在该租户尚无同步记录时为 `null`。

## 数据结构定义

> 字段形态以桌面端 `export_database()` 与 `cloud-backend-api` 为准；下为概览示例。

### Project（项目）

```jsonc
{ "id": "…", "name": "项目名称", "created_at": "…", "updated_at": "…" }
```

### Card（卡片）

```jsonc
{
  "id": "…", "project_id": "…", "creator_id": null,
  "callsign": "BV2AAA", "qty": 1, "serial": null, "status": "pending",
  "metadata": { "distribution": { "method": "邮寄", "address": "…", "remarks": "…" }, "return": null },
  "created_at": "…", "updated_at": "…"
}
```

**status 可选值**：`pending`（已录入待分发）、`distributed`（已分发）、`returned`（已退卡）。

### SFSender（顺丰寄件人）

```jsonc
{
  "id": "…", "name": "…", "phone": "…", "mobile": "13800138000",
  "province": "…", "city": "…", "district": "…", "address": "…",
  "is_default": true, "created_at": "…", "updated_at": "…"
}
```

### SFOrder（顺丰订单）

```jsonc
{
  "id": "…", "order_id": "…", "waybill_no": "…", "card_id": "…",
  "status": "confirmed", "pay_method": 1, "cargo_name": "QSL卡片",
  "sender_info": { /* 对象 */ }, "recipient_info": { /* 对象 */ },
  "created_at": "…", "updated_at": "…"
}
```

**status 可选值**：`pending`、`confirmed`、`cancelled`、`printed`。
**pay_method 可选值**：`1`（寄方付）、`2`（收方付）、`3`（第三方付）。

## 错误码（示例）

| HTTP 状态码 | `code` | 说明 |
|------------|--------|------|
| 200 | — | 请求成功 |
| 400 | — | 请求参数错误（如缺 `client_id` 或 `data`） |
| 401 | `auth_failed` | 认证失败（Key 无效 / 缺失） |
| 403 | `tenant_mismatch` | `X-Tenant-Id` 声明租户与 Key 解析租户不一致 |
| 409 | — | `POST /sync` 基线陈旧（OCC 守卫拒绝，零改动） |
| 500 | — | 服务器内部错误（响应脱敏，不回显内部结构） |

## 端点路径与租户身份

> 规范以 `cloud-backend-api`（及 `tenant-path-routing`）为准。

- **同步端点为裸路径**：`/ping`·`/sync`·`/pull` 均为裸路径，租户身份经 `Authorization: Bearer` + 可选 `X-Tenant-Id` 头解析，**不**经 URL 路径。
- **`/t/<slug>/` 前缀在这三个端点上返回 404**：`/t/<slug>/` 路径前缀仅作用于**公共查询面**（按呼号查询）；官方 worker 对 `/t/<slug>/ping`·`/t/<slug>/sync`·`/t/<slug>/pull` 一律返回 **404**。自托管者**禁止**把同步 `api_url` 配成 `/t/<slug>` 形式——应直接指向裸域名根（如 `https://<你的域名>`）。

## 实现建议

1. **租户隔离按 Key 解析的 `tenant_id`**：所有读写注入 `WHERE tenant_id = ?`，归属恒由 Key 解析决定。`client_id` 仅作设备溯源（写 `last_client_id`），**绝不**当隔离键。
2. **全量替换 + OCC**：`/sync` 在单个事务内删除该租户全量再写入；护栏路径下版本守卫覆盖全部 DELETE/INSERT，陈旧基线 409 且零改动。
3. **HTTPS**：强制使用 HTTPS 保护传输。
4. **错误脱敏**：错误响应禁止回显内部异常、SQL/约束、堆栈等细节。
5. **请求频率限制**：建议对端点做基础限流（公共查询面另有 PoW + 会话防爬，见部署文档）。

## 自托管

完整的「新增租户签发凭据」与「两类自托管」（仅实现同步 API / 自部署本仓库 worker）指引见
[web-query-service-deploy.md「新增租户与自托管」节](web-query-service-deploy.md#新增租户与自托管)。

## 更新历史

- 2026-06-18（阶段 4-C4）：重写到当前多租户契约——声明本文档非规范性、契约真源为 `cloud-backend-api`；以示例呈现 Bearer 表驱动租户解析 + 可选 `X-Tenant-Id` 交叉校验（401 `auth_failed` / 403 `tenant_mismatch`）、`POST /sync` 必填 `client_id`（仅溯源非归属）+ OCC `base_version`/`force`/409/`server_version`、新增 `GET /pull`（JSON/布尔列还原）、同步端点裸路径（`/t/<slug>/` 返 404）；删除过时单一 `API_KEY` / `client_id` 当隔离键 / Express 示例。
- 2026-01-23：初始版本（多租户改造前，已废弃）。
