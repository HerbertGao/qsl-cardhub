## 为什么

`app_settings` 表已实现后端存储和导出/导入/同步支持，但用户目前只能在分散的界面中间接操作配置项（标签标题在模板设置页编辑，张数模式在卡片录入时切换）。缺少一个集中的配置入口，不利于用户发现和管理全局偏好。

## 变更内容

- 在「数据配置」子菜单中新增「全局配置」菜单项，放在第一位（排在 QRZ.cn 之前）
- 新建 `GlobalSettingsView.vue` 页面，集中展示 `app_settings` 配置项：
  - **标题文本**：文本输入框编辑 `label_title`，旁边提供跳转到标签模板配置页预览的链接/按钮
  - **数量显示模式**：精确/大致开关，复用卡片录入页同款的 Switch 组件样式，操作 `qty_display_mode`
- 配置变更实时写入数据库（通过已有的 `set_app_setting_cmd`），无需手动保存按钮

## 功能 (Capabilities)

### 新增功能
- `global-settings-ui`: 全局配置界面，在数据配置菜单提供集中管理 app_settings 的页面

### 修改功能

## 影响

- **前端**：新增 `GlobalSettingsView.vue`；修改 `App.vue` 菜单注册和路由
- **后端**：无变更，复用已有的 `get_app_setting_cmd` / `set_app_setting_cmd` 命令
- **依赖**：无新增依赖
