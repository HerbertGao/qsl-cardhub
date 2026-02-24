## 为什么

当前应用的用户偏好设置分散存储：卡片张数显示模式（精确/大致）存在前端 `localStorage`，标签标题固定文字硬编码在 `callsign.toml` 模板文件中。这些设置不会随数据导出/导入同步，导致用户换设备或重装后需要重新配置。将它们统一存储到数据库全局配置表中，可以自动随导出/导入流程同步，同时为后续新增配置项提供统一的存储基础设施。

## 变更内容

1. **新建全局配置表 `app_settings`**：在数据库中新增一张键值对配置表，用于存储应用级别的用户偏好设置。
2. **迁移 `qty_display_mode` 到数据库**：将卡片张数精确/大致显示模式从前端 `localStorage` 迁移到 `app_settings` 表，前端通过 Tauri Command 读写。
3. **迁移标签标题固定文字到数据库**：将 `callsign.toml` 模板中 `title` 元素的固定文字值（当前为 "中国无线电协会业余分会-2区卡片局"）存入 `app_settings` 表，打印时从数据库读取替代模板硬编码值。
4. **导出/导入自动包含配置**：`app_settings` 表作为新的数据表纳入导出文件，导入时自动恢复。
5. **云端同步包含配置**：`app_settings` 表数据纳入云端同步的 `SyncData`，确保多设备间偏好设置一致。

## 功能 (Capabilities)

### 新增功能

- `app-settings`: 全局配置表的 CRUD 操作，包括数据库表定义、Tauri Command 接口、前端读写逻辑。

### 修改功能

- `cloud-database-support`: 导出/导入流程新增 `app_settings` 表的数据；云端同步 `SyncData` 新增 `app_settings` 字段。
- `cloud-backend-api`: 云端 `/sync` 端点新增 `app_settings` 表的 D1 建表、DELETE 和 INSERT 处理。
- `template-configuration`: 打印标签时，`title` 元素的固定文字从 `app_settings` 表读取，不再硬编码于模板。

## 影响

- **数据库**：新增 `app_settings` 表，需要新增迁移脚本
- **后端**：新增 `src/db/app_settings.rs` CRUD 模块；新增 Tauri Command；修改 `export.rs` / `import.rs` 包含新表；修改 `sync/client.rs` 的 `SyncData` 包含新表；修改打印流程从数据库读取标题
- **前端**：修改 `useQtyDisplayMode.ts` 从 Tauri Command 读写替代 `localStorage`；新增配置管理界面入口
- **数据格式**：导出文件 `tables` 新增 `app_settings` 字段（可选，向后兼容）
- **云端**：`SyncData` 新增 `app_settings` 字段；`web_query_service` 的 `/sync` 端点需处理新表的存储和清除