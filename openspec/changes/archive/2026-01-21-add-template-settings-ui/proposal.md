# 变更：增加模板设置功能

## 为什么
当前系统使用硬编码的模板配置（`config/templates/default.toml`），用户无法通过界面调整模板参数（如边距、对齐方式、间距等）。每次需要修改这些参数时，用户必须手动编辑 TOML 文件，对普通用户不友好，也容易出错。

需要提供一个可视化的模板设置界面，让用户能够：
1. 查看当前模板的所有配置参数
2. 调整高频使用的参数（边距、对齐、间距等）
3. 实时预览修改效果
4. 保存修改到模板文件

## 变更内容
- 新增 `TemplateView.vue` 前端页面，提供模板参数编辑界面
- 新增 Tauri 命令 `get_template_config` 读取当前模板配置
- 新增 Tauri 命令 `save_template_config` 保存模板配置到文件
- 实现左右分栏布局：左侧表单编辑，右侧实时预览
- 实现表单字段的可编辑/只读控制
- 集成现有的 `preview_qsl` 命令进行实时预览

## 影响
- 受影响规范：`template-configuration`（模板配置系统）
- 受影响代码：
  - 新增：`web/src/views/TemplateView.vue` - 模板设置页面
  - 新增：`web/src/router/index.js` - 路由配置（如需要）
  - 修改：`src/commands/printer.rs` - 新增模板配置读写命令
  - 修改：`src/main.rs` - 注册新的 Tauri 命令
  - 修改：`web/src/App.vue` - 添加导航入口（如需要）
- 向后兼容性：完全向后兼容，不影响现有打印功能
- 用户体验：显著提升，用户无需手动编辑 TOML 文件
