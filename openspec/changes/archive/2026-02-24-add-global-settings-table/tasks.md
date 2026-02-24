## 1. 数据库层

- [x] 1.1 创建迁移脚本 `migrations/2026.1.24.004_add_app_settings.sql`：建表 `app_settings(key TEXT PRIMARY KEY, value TEXT NOT NULL)`，并插入默认值 `label_title` = `中国无线电协会业余分会-2区卡片局`
- [x] 1.2 新建 `src/db/app_settings.rs`：实现 `get_setting(key) -> Option<String>`、`set_setting(key, value)`、`get_all_settings() -> Vec<AppSetting>` 函数，其中 `AppSetting` 结构体包含 `key: String` 和 `value: String`
- [x] 1.3 在 `src/db/models.rs` 中添加 `AppSetting` 结构体（derive Serialize, Deserialize, Clone, Debug）
- [x] 1.4 在 `src/db/mod.rs` 中导出 `app_settings` 模块

## 2. Tauri Command 接口

- [x] 2.1 新建 `src/commands/app_settings.rs`：实现 `get_app_setting_cmd(key)`, `set_app_setting_cmd(key, value)`, `get_all_app_settings_cmd()` 三个 Tauri Command
- [x] 2.2 在 `src/main.rs` 中注册新的 Tauri Command

## 3. 导出/导入集成

- [x] 3.1 在 `src/db/export.rs` 的 `ExportTables` 结构体中新增 `app_settings: Option<Vec<AppSetting>>`（带 `#[serde(default, skip_serializing_if = "Option::is_none")]`）
- [x] 3.2 在 `export_database()` 函数中查询 `app_settings` 表并填入导出数据
- [x] 3.3 在 `src/db/import.rs` 中处理导入文件的 `app_settings` 字段：如果存在则清空并恢复该表数据，不存在则跳过
- [x] 3.4 确保 v1.0 兼容类型 `ExportTablesV1_0` 不受影响（旧版文件无 `app_settings` 字段）

## 4. 云端同步集成

- [x] 4.1 在 `src/sync/client.rs` 的 `SyncData` 结构体中新增 `app_settings: Vec<AppSetting>` 字段
- [x] 4.2 在 `execute_sync()` 函数中，从 `export_data.tables` 获取 `app_settings` 填入 `SyncData`
- [x] 4.3 在 `web_query_service/src/worker/index.js` 中新建 D1 `app_settings` 表（`client_id TEXT, key TEXT, value TEXT, PRIMARY KEY (client_id, key)`）
- [x] 4.4 在 `/sync` 端点 DELETE 阶段新增清除 `app_settings` 表中该 `client_id` 的记录
- [x] 4.5 在 `/sync` 端点 INSERT 阶段新增写入 `data.app_settings` 中的键值对（如果字段存在）

## 5. 打印命令集成

- [x] 5.1 修改 `src/commands/printer.rs` 中的打印流程：加载模板后，从 `app_settings` 读取 `label_title`，如果存在则将其注入到运行时数据 HashMap 中，并将模板 `title` 元素的 `source` 从 `fixed` 改为 `input`（`key = "label_title"`）

## 6. 前端改造

- [x] 6.1 修改 `web/src/composables/useQtyDisplayMode.ts`：将持久化层从 `localStorage` 改为调用 `get_app_setting` / `set_app_setting` Tauri Command，应用启动时从后端加载初始值
- [x] 6.2 在 `useQtyDisplayMode.ts` 中添加 localStorage 迁移逻辑：首次加载时如果 localStorage 有旧值且数据库无记录，写入数据库后清除 localStorage
- [x] 6.3 在 `web/src/views/TemplateView.vue` 的模板配置界面中，为 `title` 元素的 `value` 字段增加可编辑功能，修改后调用 `set_app_setting` 保存到数据库（键 `label_title`）
