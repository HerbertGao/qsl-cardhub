# cloud-sync-versioning 规范

## 目的
待定 - 由归档变更 add-sync-robustness 创建。归档后请更新目的。
## 需求
### 需求：上传乐观并发版本护栏

云端**必须**以每租户单调递增的 `sync_meta.server_version` 作为乐观并发（OCC）基准。当 `/sync` 请求携带 `base_version` 且未带 `force=true` 时，写入**必须**走 compare-and-swap：**仅当**云端当前 `server_version` 等于请求的 `base_version` 才执行全量替换并把版本 +1；否则**必须**返回 **409** 且**保证零数据改动**（不删除、不写入任何业务行、不推进版本）。

由于 D1 的 `DB.batch()` 原子但**不能在某条语句 `changes==0` 时中途中止**，实现**必须**满足以下原子性约束（见 design 决策 1），**禁止**用「CAS 放首句、0 行即 409」或「batch 外预读比对」这类存在 TOCTOU/假 409 的写法：

- 5 张业务表的 `DELETE` 与全部 `INSERT` **必须**各自带版本守卫（`… AND (SELECT server_version FROM sync_meta WHERE tenant_id=?)=?`，`INSERT` 用 `INSERT…SELECT…WHERE 守卫`），使陈旧时这些写语句净影响 0 行；
- 递增版本的 CAS（`UPDATE sync_meta SET server_version=server_version+1,… WHERE tenant_id=? AND server_version=?`）**必须**置于 batch **最后一条**，使前面所有守卫都看到原始 `base_version`；
- 409 判定（**仅守卫路径**——无条件/`force` 路径不做版本比较、永不 409）**必须**读守卫路径 batch 中 **CAS 那条结果**的 `result.meta.changes`：`1`→成功、`0`→409（守卫路径 CAS 即该 batch 末条；**禁** `SELECT changes()`）；
- 全过程**必须**在单个 `DB.batch()` 内（D1 不支持用户侧 `BEGIN/COMMIT`，原子性只能靠单 batch）。

#### 场景：基线匹配则替换并推进版本

- **当** `/sync` 携带 `base_version = N`、未带 `force`，且云端 `server_version = N`
- **那么** 服务端**必须**全量替换该租户数据，并把 `server_version` 置为 `N+1`
- **并且** 响应**必须**返回 200 且回传新的 `server_version = N+1`

#### 场景：基线陈旧或被并发抢先则 409 且零改动

- **当** `/sync` 携带 `base_version = N`、未带 `force`，但云端 `server_version ≠ N`（已被其他设备推进到 `N+k`）
- **那么** 服务端**必须**返回 409，**禁止**删除或写入任何业务行，**禁止**改变 `server_version`
- **并且** 经原子性约束保证：该次请求对数据库的净影响为 0 行（不存在「已删未写」或「部分覆盖」中间态）

#### 场景：两设备持同一基线并发上传仅一端成功

- **当** 两台设备各持 `base_version = N` 几乎同时上传（均未 `force`）
- **那么** 依 SQLite 写事务库级串行，先提交者把版本推进到 `N+1`、返回 200；后提交者的守卫与 CAS 均落空、返回 409 且零改动
- **并且** 后者**必须**通过下载最新（`/pull`）或强制覆盖（`force`）才能再次写入

#### 场景：sync_meta 行缺失时守卫路径返回 409 且版本字段不为 undefined

- **当** 携带 `base_version` 的守卫路径 `/sync` 执行时该租户 `sync_meta` 行不存在（守卫子查询 `SELECT server_version` 得 NULL）
- **那么** 所有守卫的 DELETE/INSERT **必须**净 0 行、末条 CAS `changes==0` → 返回 409（行为安全、零改动）
- **并且** 该 409 响应体的 `server_version` 字段**必须**为 `null`（或省略该字段），**禁止**回 `undefined`/`NaN`（否则客户端据此把本地 `base_version` 写坏）
- **并且** 该场景属边界：现役活跃租户行恒存在；新租户开通**必须**先经无条件/`force` 路径或开通流程建 `sync_meta` 行（阶段 4 租户开通 seed，本期记录、不实现）

### 需求：强制覆盖与缺省基线的兼容降级

`force=true` 的 `/sync`，或**未携带** `base_version` 的 `/sync`（含未升级的旧桌面端），**必须**走「无条件覆盖」路径：不做版本比较、直接全量替换，并以 upsert 把 `server_version` 单调 +1（`server_version` 行不存在时建行并置为 1）。此路径使**未升级桌面端零改动继续工作**，护栏仅在客户端开始回传 `base_version` 后对该客户端生效。

#### 场景：force 跳过比较强制覆盖

- **当** `/sync` 携带 `force=true`（无论是否带 `base_version`、无论云端当前版本）
- **那么** 服务端**必须**无条件全量替换该租户数据，并把 `server_version` 推进为「当前 +1」
- **并且** 响应**必须**返回 200 且回传新的 `server_version`

#### 场景：旧桌面端缺 base_version 按无条件覆盖（兼容）

- **当** `/sync` 未携带 `base_version`（旧客户端，未参与并发协议）
- **那么** 服务端**必须**按无条件覆盖处理（不返回 409）、全量替换并把 `server_version` 单调 +1
- **并且** 该行为与阶段 1 的现状写入语义一致，保证旧桌面端零改动继续工作

### 需求：上传响应回传版本号与客户端基线义务

`/sync` 的 200 响应**必须**回传写入后的新 `server_version`；409 响应**必须**回传云端**当前** `server_version`。客户端**必须**持久化 `base_version`、在每次上传时回传所持基线，并在收到 200 后**立即把本地 `base_version` 刷新为响应回传的 `server_version` 并落盘**——否则下次上传仍带旧基线会自发 409。

#### 场景：成功响应携带新版本且客户端刷新基线

- **当** 客户端上传成功（200）
- **那么** 响应体**必须**含新的 `server_version`
- **并且** 客户端**必须**把本地持久化的 `base_version` 更新为该值并落盘
- **并且** 在两端之间无其他设备写入时，客户端**连续两次**上传的第二次**必须**仍为 200（验证基线刷新生效，而非每次 409）

#### 场景：冲突响应携带云端当前版本

- **当** 上传因基线陈旧返回 409
- **那么** 响应体**必须**含云端**当前** `server_version`，供客户端提示「云端已到版本 N」并据此引导下载或强制覆盖

### 需求：按写入 Key 下载租户全量快照

云端**必须**提供 `GET /pull`：以**写入凭据**（`Authorization: Bearer {写入 Key}`，复用 `tenant-isolation` 的 `resolveTenant` 解析租户）鉴权，返回该租户的全量业务数据快照与当前 `server_version`。`/pull` 返回含 PII 的整租户数据，**禁止**用面向公众的查询签名 key 鉴权——只能给持写凭据的属主。

#### 场景：凭写入 Key 拉回全量快照与版本

- **当** 客户端以有效写入 Key 发起 `GET /pull`
- **那么** 服务端**必须**按 Key 解析出的 `tenant_id` 返回该租户全量数据（projects、cards、sf_senders、sf_orders、app_settings）与当前 `server_version`
- **并且** `data` 各表字段形态**必须**逐字段等于桌面端导出（`export_database()`）的**对象/布尔**形态、而非 D1 原始存储形态：worker 入库时把 `cards.metadata`、`sf_orders.sender_info`、`sf_orders.recipient_info` 存为 **JSON 字符串**、`sf_senders.is_default` 存为**整数**；`/pull` 读出后**必须**将这三个字符串字段 `JSON.parse` 还原为对象（NULL/空串容错）、`is_default` 还原为布尔，否则客户端按对象/布尔类型反序列化**直接失败**、恢复不可用
- **并且** 若某行存的非合法 JSON（数据损坏），`/pull` **必须** fail-closed 返回 500（脱敏），**禁止**静默跳过该行或返回半快照
- **并且** 客户端据此恢复后**必须**把本地 `base_version` 设为返回的 `server_version`，使后续上传基线对齐、直接 200

#### 场景：无效或缺失写入 Key 拒绝

- **当** `/pull` 的 Bearer Key 既未命中 `tenant_credentials` 也不等于兜底 `env.API_KEY`（或缺失）
- **那么** 服务端**必须**返回 401、**禁止**返回任何租户数据，且错误响应**禁止**回显内部结构（与「错误响应脱敏」一致）

