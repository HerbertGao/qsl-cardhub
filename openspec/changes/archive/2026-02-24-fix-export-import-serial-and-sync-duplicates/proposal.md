## 为什么

数据导出/导入流程存在两个 Bug：（1）导入时 INSERT 语句遗漏了 `serial` 列，导致序号丢失；（2）导出文件不包含云端同步的 `client_id`，换设备或重装后导入再同步会产生新的 `client_id`，而云端查询接口不按 `client_id` 去重，导致同一呼号返回重复记录。这两个问题直接影响数据完整性和用户体验，需要立即修复。

## 变更内容

1. **修复导入丢失序号**：在 `import.rs` 的卡片 INSERT 语句中补上 `serial` 列及对应绑定参数。
2. **导出/导入包含 `client_id`**：将 `client_id` 纳入导出文件，导入时自动恢复到 `sync.toml`，确保备份恢复后同步身份一致。
3. **云端同步清除全部 client 数据**：修改 `web_query_service` 的 `/sync` 端点，在 DELETE 阶段不仅清除当前 `client_id` 的数据，同时清除所有 client 中与本次同步卡片 UUID 重叠的记录，解决已存在的重复数据问题。

## 功能 (Capabilities)

### 新增功能

（无）

### 修改功能

- `cloud-database-support`: 导出文件新增 `client_id` 字段；导入时恢复 `client_id` 到同步配置；导入 INSERT 补全 `serial` 列。
- `cloud-backend-api`: `/sync` 端点在写入前额外清除其他 client 中 ID 重复的卡片/项目/订单记录，防止跨 client 重复。

## 影响

- **后端**：`src/db/import.rs`（补 serial 列 + 恢复 client_id）、`src/db/export.rs`（导出 client_id）、`src/sync/config.rs`（新增写入接口）
- **云端**：`web_query_service/src/worker/index.js`（`/sync` 端点 DELETE 逻辑调整）
- **数据格式**：导出格式版本保持 1.1（`client_id` 为可选字段，向后兼容旧版导出文件）
- **无破坏性变更**：旧版导出文件不含 `client_id`，导入时跳过恢复即可