## 上下文

卡片管理页面由 `CardManagementView.vue` 和子组件 `CardList.vue` 组成。`CardList.vue` 的工具栏左侧有"录入卡片"和"导出"按钮，右侧有搜索和筛选控件。

云端同步功能已在 `DataTransferView.vue` 中完整实现，核心是调用 `execute_sync_cmd` Tauri 命令。同步配置通过 `load_sync_config_cmd` 加载，返回 `{ api_url, client_id, has_api_key, last_sync_at }` 结构。

## 目标 / 非目标

**目标：**
- 在 CardList 工具栏左侧新增"同步到云端"按钮，紧跟在"导出"按钮之后
- 完全复用已有的 `execute_sync_cmd` 后端命令，不引入新的同步逻辑
- 仅在同步 API 已配置（api_url 非空且 has_api_key 为 true）时显示按钮

**非目标：**
- 不在卡片管理页面实现同步配置 UI（配置仍在数据管理页面）
- 不显示同步详细进度或统计信息面板（仅用 ElMessage 提示）
- 不添加同步状态全局 store（本地 ref 状态即可）

## 决策

### 决策 1：按钮放置在 CardList 组件内部

**选择**：将同步按钮直接放在 `CardList.vue` 的 toolbar-left 区域，而非在 `CardManagementView.vue` 中添加。

**理由**：工具栏的其他操作按钮（录入、导出）都在 `CardList.vue` 内，保持一致性。通过 props 传递同步配置状态，通过 emit 事件通知父组件触发同步。

### 决策 2：同步配置状态在 CardManagementView 中加载

**选择**：在 `CardManagementView.vue` 的 `onMounted` 中调用 `load_sync_config_cmd` 获取同步配置状态，通过 prop `syncConfigured` (boolean) 传递给 `CardList.vue`。

**理由**：
- CardList 是纯展示组件，不应直接调用 Tauri 命令
- 同步配置在应用运行期间不会频繁变化，onMounted 时加载一次即可
- 使用单一 boolean prop 比传递完整配置对象更简洁

### 决策 3：同步逻辑在 CardManagementView 中处理

**选择**：CardList 通过 `emit('sync')` 事件通知父组件，由 `CardManagementView.vue` 调用 `execute_sync_cmd` 并处理结果/错误。同步 loading 状态通过 prop `syncing` (boolean) 回传给 CardList 控制按钮状态。

**理由**：与现有的事件模式一致（CardList 已使用 emit 模式处理 `add`、`search` 等事件）。

## 风险 / 权衡

**[权衡] 同步配置变更不实时反映**
→ 如果用户在数据管理页面修改同步配置后回到卡片管理页面，按钮可见性不会立即更新（需刷新页面）。这是可接受的，因为配置修改是低频操作。
