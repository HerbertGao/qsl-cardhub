## 上下文

`app_settings` 后端（数据库表、Tauri Commands、导出/导入/同步）已完整实现。前端目前在不同页面分散操作配置项：`qty_display_mode` 在卡片录入对话框的 Switch 控制，`label_title` 在模板设置页的输入框编辑。缺少集中入口。

现有菜单结构：「数据配置」子菜单下有 QRZ.cn、QRZ.com、顺丰速运、数据管理四个菜单项，通过 `activeMenu` 字符串匹配渲染对应 View 组件。导航跳转通过 `navigationStore.navigateTo(target)` 实现。

## 目标 / 非目标

**目标：**
- 新增 `GlobalSettingsView.vue` 页面，集中展示和编辑 `app_settings` 配置项
- 在「数据配置」子菜单第一位添加「全局配置」菜单项
- 标题文本：文本输入框 + 跳转到标签模板配置页预览的文字链接
- 数量显示模式：复用 CardInputDialog 同款 el-switch 样式（active-text="大致"，inactive-text="精确"）

**非目标：**
- 不修改后端（复用已有 `get_app_setting_cmd` / `set_app_setting_cmd`）
- 不修改 `useQtyDisplayMode` composable 逻辑
- 不新增配置项

## 决策

### 决策 1：页面结构使用 el-card 分组

每个配置项用独立的 el-form-item 呈现，整体包在一个 el-card 内。与 DataTransferView 等现有页面风格一致。

替代方案：使用 el-collapse 折叠面板 → 配置项仅 2 个，折叠反而增加操作步骤，放弃。

### 决策 2：标题文本变更使用防抖自动保存

输入框通过 `@input` 事件触发 500ms 防抖写入数据库，与 TemplateView 的自动保存策略一致。

替代方案：`@change`（失焦或回车）触发保存 → 用户可能以为没保存就跳走导致丢失，放弃。

### 决策 3：跳转预览使用 navigateTo

点击"跳转到模板配置预览"链接调用 `navigateTo('print-config-template')`，复用已有导航机制切换到标签模板配置页。

### 决策 4：数量模式开关直接操作 useQtyDisplayMode 共享状态

全局配置页和卡片录入对话框共享同一个 `qtyDisplayMode` ref，修改即时生效。无需额外通知机制。

## 风险 / 权衡

- [风险] 标题文本输入框为空时保存空值 → 允许空值（此时打印将使用模板默认标题），不做非空校验。
