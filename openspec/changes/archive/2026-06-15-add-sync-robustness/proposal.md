## 为什么

阶段 1 把 `/sync` 修成「按租户单事务全量替换」，但**多设备并发/陈旧覆盖**仍未闭合：`/sync` 仍是无条件覆盖，两台桌面端（同一租户、各持一把 Key）交替同步会**静默互相覆盖**——后同步者用自己的本地快照盖掉先同步者的改动，且双方都看到「同步成功」，数据无声丢失。同时**换机/重装/误删本地库**后无法从云端拉回（云端定位是「桌面端的备份镜像」，却只能写不能读）。

阶段 1 已为此预埋 `sync_meta.server_version INTEGER NOT NULL DEFAULT 0` 列（只占列、无逻辑）。本阶段把它激活成乐观并发护栏，并补齐下载侧。

## 变更内容

- **`/sync` 乐观并发版本护栏（OCC）**：桌面端上传时回传持有的 `base_version`；服务端用条件写做 compare-and-swap（CAS）。基线陈旧/被并发抢先 → 返回 **409** 且**不删除/不修改任何数据**；命中 → 全量替换并把 `server_version` 单调 +1。
- **`force=true` 覆盖逃生门**：人工确认「确实要用本机快照盖掉云端」时跳过版本比较，无条件覆盖并把版本推进到「当前+1」。
- **新增 `GET /pull`**：按写入 Key 解析租户，返回该租户全量快照 + 当前 `server_version`。用途：换机初始化、被 409 挡住后先下载再续、主动从云端恢复。
- **桌面端**：持久化 `base_version`；同步成功后用响应回传的新版本更新本地基线；遇 409 引导用户「先下载云端最新」或「强制覆盖」；新增「从云端恢复」入口（走 `/pull` 重建本地库）。
- **向后兼容（BREAKING-避免）**：旧桌面端不发 `base_version`——服务端把「缺 `base_version`」按**旧式无条件覆盖**处理并照常推进版本，使**未升级的桌面端零改动继续工作**；护栏只在桌面端升级后才对该客户端生效。

## 功能 (Capabilities)

### 新增功能
- `cloud-sync-versioning`: 同步的乐观并发版本模型——`server_version` 单调语义、CAS 护栏与 409 契约、`force` 覆盖语义、`/pull` 全量快照+版本的下载契约、缺省 `base_version` 的兼容降级。覆盖服务端强制与客户端义务两侧的规范级行为。

### 修改功能
- `cloud-backend-api`: `/sync` 请求体新增 `base_version`/`force`、新增 409 响应分支与响应体回传 `server_version`；新增 `GET /pull` 端点（按 Key 解析租户、返回全量快照+版本）。
- `cloud-database-support`: 桌面端「云端数据同步」需求新增 `base_version` 持久化与回传、200 后刷新基线、409 处理（引导下载或强制覆盖）；新增「桌面端从云端恢复同步数据」需求（走 `/pull` 重建本地库）。端点契约（`/pull`、`/sync` 的 `base_version`/`force`/409/`server_version`）以 `cloud-backend-api` 为准。

## 影响

- **Worker**：`web_query_service/src/worker/index.js`（`/sync` 改 CAS + 新增 `/pull`）、`web_query_service/schema.sql`（`server_version` 列已存在，无 DDL 变更）；`web_query_service/verify/`（新增 OCC/pull smoke 断言）。
- **桌面端（Rust/Tauri）**：`src/sync/config.rs`（`SyncConfig` 加 `base_version`）、`src/sync/client.rs`（请求加 `base_version`/`force`、解析 409 与新版本、新增 `pull_data`）、`src/commands/sync.rs`（同步命令处理 409、新增恢复命令）、`src/db/`（新增「从云端快照重建本地库」的导入路径）、前端「数据管理」页（409 引导 UI + 恢复入口）。
- **不在本期**：防爬与读取侧动态化（阶段 3）；host/path → 租户路由与多租户前端（阶段 4）。本期路由未引入，写入侧由 Key 解析，当前 `env.API_KEY` 兜底恒解析为创始租户 `bh2ro`。
- **数据无变更**：不改表结构、不改既有行；OCC 仅改变写入控制流，回滚=退 worker 版本（数据兼容）。
