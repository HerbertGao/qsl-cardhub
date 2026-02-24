## 1. 新建全局配置页面

- [x] 1.1 创建 `web/src/views/GlobalSettingsView.vue`，包含页面标题和 el-card 容器
- [x] 1.2 在 el-card 内添加标题文本配置项：el-form-item + el-input 输入框，onMounted 时调用 `get_app_setting_cmd` 加载 `label_title` 值，输入时 500ms 防抖调用 `set_app_setting_cmd` 保存
- [x] 1.3 在标题文本输入框旁添加跳转链接，点击调用 `navigateTo('print-config-template')` 导航到标签模板配置页
- [x] 1.4 在 el-card 内添加数量显示模式配置项：使用 `useQtyDisplayMode` 获取 `qtyDisplayMode`，通过 computed 转换为 boolean 绑定 el-switch（active-text="大致"，inactive-text="精确"），复用 CardInputDialog 同款样式

## 2. 注册菜单与路由

- [x] 2.1 在 `App.vue` 的「数据配置」子菜单中，在 QRZ.cn 菜单项之前添加「全局配置」菜单项（index="data-config-global-settings"）
- [x] 2.2 在 `App.vue` 中 import `GlobalSettingsView` 并添加对应的 `v-if="activeMenu === 'data-config-global-settings'"` 渲染条件
