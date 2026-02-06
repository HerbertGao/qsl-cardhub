## 为什么

用户在卡片管理页面录入或分发卡片后，经常需要立即同步数据到云端。目前云端同步功能仅在"数据管理 → 云端同步"页面提供，用户需要切换页面才能触发同步，操作路径较长。在卡片管理界面直接提供同步入口可以显著提升工作效率。

## 变更内容

1. **在卡片管理界面的 CardList 工具栏中新增"同步到云端"按钮**：仅在云端同步 API 已配置时显示，点击后调用与"数据管理 → 云端同步 → 立即同步"完全相同的 `execute_sync_cmd` Tauri 命令。
2. **同步状态反馈**：按钮需显示同步中的 loading 状态，并在成功/失败时给出 ElMessage 提示。

## 功能 (Capabilities)

### 新增功能

- `card-list-sync-button`: 卡片列表工具栏中的云端同步快捷按钮，在已配置同步 API 的前提下，提供一键同步入口。

### 修改功能

（无规范层面的行为变更，仅为 UI 入口扩展）

## 影响

- **前端组件**：`web/src/components/cards/CardList.vue` — 工具栏新增同步按钮
- **前端页面**：`web/src/views/CardManagementView.vue` — 需传递同步配置状态和处理同步事件
- **无后端变更**：复用已有的 `execute_sync_cmd` 和 `load_sync_config_cmd` Tauri 命令
- **无 Breaking Change**
