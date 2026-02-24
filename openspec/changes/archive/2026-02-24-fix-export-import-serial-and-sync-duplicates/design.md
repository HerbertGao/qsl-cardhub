## 上下文

数据导出/导入流程 (`src/db/export.rs` / `src/db/import.rs`) 存在两个 Bug：

1. **序号丢失**：`import.rs` 中卡片 INSERT 语句遗漏了 `serial` 列，导致导入后所有卡片的序号变为 NULL。
2. **云端同步重复**：`client_id` 存储在独立文件 `sync.toml` 中，不包含在导出文件中。用户换设备或重装后导入备份会生成新的 `client_id`，同步到云端后同一数据存在于两个 `client_id` 下。云端查询接口 (`/api/callsigns/:callsign`) 不按 `client_id` 过滤，导致返回重复记录。

当前相关代码位置：
- 导出：`src/db/export.rs` — `ExportData` 结构体和 `export_database()` 函数
- 导入：`src/db/import.rs` — `execute_import()` 函数
- 同步配置：`src/sync/config.rs` — `SyncConfig` 结构体（存储在 `sync.toml`）
- 云端 `/sync`：`web_query_service/src/worker/index.js` 第 297-376 行

## 目标 / 非目标

**目标：**
- 修复导入时序号丢失的 Bug
- 导出文件包含 `client_id`，导入时恢复，确保备份恢复后同步身份一致
- 云端 `/sync` 端点在清除数据时，额外清除其他 client 中 ID 重复的记录，解决已存在的重复问题
- 保持导出格式向后兼容（旧版文件不含 `client_id`，导入时跳过恢复）

**非目标：**
- 不改变导出格式版本号（`client_id` 作为可选字段加入现有结构）
- 不改变同步协议（仍为全量推送）
- 不实现增量同步或 change tracking

## 决策

### 决策 1：`client_id` 放在导出文件的顶层字段

**选择**：在 `ExportData` 结构体中新增 `client_id: Option<String>` 顶层字段。

**替代方案**：
- 方案 B：在 `ExportTables` 中新增 `sync_config` 表 — 过重，sync 配置不属于业务数据表
- 方案 C：单独导出 `sync.toml` 文件 — 增加用户操作复杂度，需管理两个文件

**理由**：`client_id` 是导出文件的元信息，与 `version`、`app_version` 同级更合理。使用 `Option<String>` 保证旧文件（无此字段）反序列化不会失败。

### 决策 2：导入时恢复 `client_id` 到 `sync.toml`

**选择**：在 `execute_import()` 中，如果导出文件包含 `client_id`，则更新本地 `sync.toml` 的 `client_id` 字段（保留 `api_url` 和 `last_sync_at` 不变）；如果本地不存在 `sync.toml`，则创建一个仅含 `client_id` 的默认配置。

**理由**：只覆盖 `client_id`，避免丢失用户已配置的 API 地址和密钥。

### 决策 3：云端 `/sync` 清除重复记录的策略

**选择**：在现有 DELETE 批处理中，除了按 `client_id` 清除当前客户端数据，额外对每条记录的 UUID `id` 执行跨 client 清除。具体做法：在插入数据前，对本次同步的所有记录 ID 执行 `DELETE FROM <table> WHERE id IN (...)` （不限 `client_id`），清除其他 client 中 ID 相同的残留数据。

**替代方案**：
- 方案 B：查询端去重（`SELECT DISTINCT ON callsign`）— 只是掩盖问题，云端仍存冗余数据
- 方案 C：不做处理，依赖 `client_id` 恢复解决未来问题 — 不能修复已存在的重复数据

**理由**：从源头清除重复数据，无需改变查询逻辑，一次性解决已有和未来的重复问题。

## 风险 / 权衡

- **[风险] 跨 client 删除可能误删其他真实客户端的数据** → 缓解：仅按记录 UUID 匹配删除。同一 UUID 不可能属于两个不同的「真实」客户端（UUID 由客户端本地生成，全局唯一）。出现 ID 重叠仅因为是同一份数据在不同 client_id 下被同步了两次。
- **[风险] 大量数据时 `DELETE WHERE id IN (...)` 性能** → 缓解：D1 支持批处理，可将 ID 列表分批执行。实际场景中单次同步的记录数通常在数千级别，性能风险极低。
- **[权衡] 保持导出格式版本 1.1 不变** → `client_id` 字段为 `Option`，旧文件解析不受影响。但旧版客户端（<此修复版本）导出的文件不含 `client_id`，导入到新版后不会恢复同步身份。这是可接受的降级。