# 规范：自动打开新建配置弹框

**功能**: 自动打开新建配置弹框
**模块**: web/ConfigView + web/App
**状态**: 草稿

---

## 新增需求

### 需求：ConfigView 必须支持自动打开新建配置弹框

`ConfigView` 组件必须接收一个可选的 prop `autoOpenNewDialog`（布尔类型）。当该 prop 为 `true` 时，组件在挂载完成后必须自动打开新建配置对话框。自动打开逻辑必须在 `onMounted` 钩子中执行，并使用 `nextTick` 确保 DOM 已完全渲染。自动打开行为必须只触发一次，避免重复打开。prop 默认值必须为 `false`，确保向后兼容。

---

#### 场景：首次启动应用且无配置时自动打开弹框

**前置条件**:
- 应用首次启动
- `config/profiles/` 目录为空或不存在任何配置文件
- App.vue 检测到无配置，跳转到 ConfigView

**操作**:
1. App.vue 传递 `autoOpenNewDialog="true"` 给 ConfigView
2. ConfigView 组件挂载

**预期结果**:
- ConfigView 完全渲染后，新建配置对话框自动打开
- `newConfigDialogVisible` 被设置为 `true`
- 用户直接看到新建配置表单

---

#### 场景：正常访问配置管理页面时不自动打开弹框

**前置条件**:
- 应用已有至少一个配置文件
- 用户从导航菜单点击"配置管理"

**操作**:
1. App.vue 不传递 `autoOpenNewDialog` prop（或传递 `false`）
2. ConfigView 组件挂载

**预期结果**:
- ConfigView 正常渲染，显示配置列表
- 新建配置对话框**不会**自动打开
- `newConfigDialogVisible` 保持为 `false`

---

#### 场景：用户手动点击新建按钮

**前置条件**:
- ConfigView 已渲染
- `autoOpenNewDialog` 为 `false` 或 `undefined`

**操作**:
1. 用户点击左上角的"新建"按钮
2. 触发 `handleNewConfig` 方法

**预期结果**:
- 新建配置对话框打开
- `newConfigDialogVisible` 被设置为 `true`
- 功能与自动打开时完全一致

---

### 需求：App.vue 必须在无配置时触发自动打开逻辑

`App.vue` 在 `onMounted` 钩子中检测配置状态时，如果发现配置列表为空（`profiles.length === 0`）或没有默认配置（`!defaultId`），必须设置一个响应式变量 `shouldAutoOpenNewConfig` 为 `true`，并将该变量作为 prop 传递给 `ConfigView` 组件。传递的 prop 名称必须为 `autoOpenNewDialog`。检测逻辑必须在跳转到配置管理页面（`activeMenu.value = 'config'`）之前或同时执行。

---

#### 场景：App.vue 检测到无配置时设置自动打开参数

**前置条件**:
- 应用启动
- `invoke('get_profiles')` 返回空数组或 `null`

**操作**:
1. App.vue 的 `onMounted` 钩子执行
2. 调用 `invoke('get_profiles')` 和 `invoke('get_default_profile_id')`

**预期结果**:
- `activeMenu.value` 被设置为 `'config'`
- `shouldAutoOpenNewConfig` 被设置为 `true`
- `<ConfigView :autoOpenNewDialog="shouldAutoOpenNewConfig" />` 接收到 `true`

---

#### 场景：App.vue 检测到有配置时不设置自动打开参数

**前置条件**:
- 应用启动
- `invoke('get_profiles')` 返回至少一个配置

**操作**:
1. App.vue 的 `onMounted` 钩子执行
2. 调用 `invoke('get_profiles')` 和 `invoke('get_default_profile_id')`

**预期结果**:
- `activeMenu.value` 保持为 `'print'`（默认页面）
- `shouldAutoOpenNewConfig` 保持为 `false`
- ConfigView 不会自动打开新建弹框（即使稍后用户手动导航到配置页面）

---

## 修改需求

无

---

## 移除需求

无

---

## 重命名需求

无

---

## 参考

- [Vue 3 Props 文档](https://vuejs.org/guide/components/props.html)
- [Vue 3 nextTick 文档](https://vuejs.org/api/general.html#nexttick)
- ConfigView 源码：`web/src/views/ConfigView.vue`
- App 源码：`web/src/App.vue`
