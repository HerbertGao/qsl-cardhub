## 1. CardManagementView 加载同步配置与处理同步事件

- [x] 1.1 在 `web/src/views/CardManagementView.vue` 中新增响应式变量 `syncConfigured` (boolean, 默认 false) 和 `syncing` (boolean, 默认 false)。在 `onMounted` 中调用 `invoke('load_sync_config_cmd')` 加载同步配置，如果返回结果有 `api_url` 且 `has_api_key` 为 true，则设置 `syncConfigured = true`。
- [x] 1.2 在 `web/src/views/CardManagementView.vue` 中新增 `handleSync()` 函数：设置 `syncing = true`，调用 `invoke('execute_sync_cmd')`，成功后用 ElMessage.success 显示同步统计（项目数、卡片数），失败时用 ElMessage.error 显示错误信息，最后设置 `syncing = false`。
- [x] 1.3 将 `syncConfigured` 和 `syncing` 作为 props 传递给 `<CardList>` 组件，并监听 `@sync` 事件调用 `handleSync()`。

## 2. CardList 组件新增同步按钮

- [x] 2.1 在 `web/src/components/cards/CardList.vue` 的 props 中新增 `syncConfigured` (boolean) 和 `syncing` (boolean)，在 emits 中新增 `sync` 事件。
- [x] 2.2 在 `CardList.vue` 的 toolbar-left 区域，"导出"按钮之后，新增"同步到云端"按钮：使用 `v-if="syncConfigured"` 控制可见性，使用 `Upload` 图标，`:loading="syncing"` 控制加载状态，`@click="$emit('sync')"` 触发同步事件。
