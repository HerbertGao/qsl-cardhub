## 为什么

当前云端是**伪多租户**：业务表虽有 `client_id` 列，但 `/sync` 用无 `WHERE` 的 `DELETE FROM` 全表清空后重写，多 `client_id` 物理上无法共存（实质单租户）；`/api/query` 零隔离（仅 `WHERE callsign`）。要把云端演进为真多租户，必须先铺「行级 `tenant_id` 隔离 + 按写入 Key 解析租户」的地基。

本期是多租户演进的**阶段 1（地基与写入隔离）**：只铺地基，**对外可见行为不变**——`tenant_id` 全程恒为 `default`（host/path→tenant 路由是阶段 4），现有桌面端、现有移动端零改动继续工作。同时顺手修一个既有缺陷：`/sync` 的 `DELETE` 与 `INSERT` 当前不在同一事务，中途失败即把云端清空。

## 变更内容

- **新增租户模型三表**：`tenants` / `tenant_credentials` / `tenant_routes`；现有全局 API Key 的 `sha256` 登记为 `default` 的写凭据。
- **业务表主键迁移** **BREAKING**（schema 层，非 API 层）：`projects` / `cards` / `sf_senders` / `sf_orders` 主键 `(client_id, id)` → `(tenant_id, id)`，`app_settings` 为 `(client_id, key)` → `(tenant_id, key)`，**删除 `client_id` 列**；`sync_meta` 主键 `client_id` → `tenant_id` 并一次性建成终态（加 `last_client_id`、`server_version`）。生产数据经迁移回填到 `default`。
- **`/sync` 改造**：按写入 Key 查 `tenant_credentials` 解析 `tenant_id`（表驱动为主 + `env.API_KEY` 直比兜底一期）；`DELETE … WHERE tenant_id=?` 与全部 `INSERT` 合入**单个 `DB.batch` 事务**按租户全量替换（修非原子缺陷）；删除 `/sync` 内联的旧 `client_id` 建表语句。
- **`/api/query` 与顺丰 route-push join 改造**：注入/改用 `tenant_id`（本期恒为 `default`），不信任请求体自报。
- **迁移交付**：单一 SQL 文件，由用户在自己终端 `wrangler d1 execute --file --remote`，迁前 `wrangler d1 export` 备份 + 单一所有者校验。

## 功能 (Capabilities)

### 新增功能
- `tenant-isolation`: 租户隔离模型——`tenants`/`tenant_credentials`/`tenant_routes` 表；按写入 Key 的 `sha256` 命中解析 `tenant_id`（支持「多 Key → 同一租户」）；业务表行级 `tenant_id` 隔离与主键演进 `(tenant_id, id)`；生产数据迁移（建新表 + `INSERT…SELECT` 回填 `default` + 单一所有者前置校验 + `d1 export` 回滚点）；服务端强制注入 `tenant_id`、绝不信任请求体自报。

### 修改功能
- `cloud-backend-api`: `/sync` 从「删除所有 `client_id` 全表后重写」改为「按 Key 解析 `tenant_id` + 单事务 `DELETE WHERE tenant_id=?` + 全量 `INSERT`」；`/api/query`、`/api/callsigns/:callsign` 注入 `tenant_id` 过滤；`app_settings` 及其余业务表结构 `client_id` → `tenant_id`；`/ping` 正常路径（有效 secret）行为不变，但**边角收紧**：对 `env.API_KEY` 补 `trim`、且 `env.API_KEY` 为空时改为 401（与 `/sync` 一致、消除 fail-open，详见 spec）。

## 影响

- **代码**：`web_query_service/src/worker/index.js`（`/sync` 鉴权与全量替换、`/api/query`、顺丰 route-push join、删除内联建表）；`web_query_service/schema.sql`（新增三表、业务表结构演进）。
- **新增**：迁移 SQL 文件（如 `web_query_service/migrations/0001_tenant_foundation.sql`），含表重建 + `default` 凭据 seed。
- **生产 D1**：需用户手动执行迁移（含迁前 `wrangler d1 export` 备份与单一所有者校验）；离线算 `sha256(API_KEY)` 也由用户在自己终端完成（AI 不接触 secret 明文）。
- **零改动验收**：现有桌面端（继续用全局 API_KEY、落 `default`）、现有移动端（裸域查询落 `default`）行为不变。
- **不影响（后续阶段）**：OCC 版本护栏 + `/pull`（阶段 2）；PoW 会话 + CDN 真实 IP 来源修正（阶段 3）；host/path 租户路由 + 桌面端租户身份 + 第二个真实租户（阶段 4）。
- **本期不加 `tenant_id` 的全局表（已知二次迁表点）**：`callsign_openid_bindings`、`sf_route_log`（callsign 为全局键）。本期「一次到位钉死」仅覆盖业务 5 表 + `sync_meta`；这两张订阅/日志表的租户化**显式延后到阶段 4，且阶段 4 上线第二真实租户前必须先迁**（否则两租户同呼号订阅会跨租户推送）——见 `tenant-isolation` spec 与 design 风险段。
