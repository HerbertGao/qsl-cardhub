## 1. 修复导入丢失序号

- [x] 1.1 在 `src/db/import.rs` 的 `execute_import()` 函数中，卡片 INSERT 语句补上 `serial` 列和对应的绑定参数 `card.serial`

## 2. 导出包含 client_id

- [x] 2.1 在 `src/db/export.rs` 的 `ExportData` 结构体中新增 `client_id: Option<String>` 字段（带 `#[serde(skip_serializing_if = "Option::is_none")]`）
- [x] 2.2 在 `export_database()` 函数中，读取 `sync.toml` 获取 `client_id`，填入 `ExportData`（读取失败则设为 None）

## 3. 导入恢复 client_id

- [x] 3.1 在 `src/db/import.rs` 中的 `ExportData` 反序列化支持可选的 `client_id` 字段（需在导入使用的 `ExportData` 类型或其解析逻辑中添加 `#[serde(default)]`）
- [x] 3.2 在 `execute_import()` 函数中，当导出文件包含非空 `client_id` 时，调用 `sync::config` 模块更新本地 `sync.toml` 的 `client_id`（保留其他字段不变）

## 4. 云端同步清除重复数据

- [x] 4.1 在 `web_query_service/src/worker/index.js` 的 `/sync` 处理逻辑中，在现有 DELETE 批处理之后、INSERT 之前，对本次同步数据的所有记录 ID 执行跨 client_id 的 DELETE（`DELETE FROM <table> WHERE id IN (...)`），清除其他 client 下的重复记录
