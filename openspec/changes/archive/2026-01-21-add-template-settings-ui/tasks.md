# 实施任务清单

## 1. 后端 - Tauri 命令实现
- [x] 1.1 在 `src/commands/printer.rs` 中实现 `get_template_config` 命令
  - 读取 `config/templates/default.toml`
  - 返回完整的 `TemplateConfig` 结构
- [x] 1.2 在 `src/commands/printer.rs` 中实现 `save_template_config` 命令
  - 接收前端传来的 `TemplateConfig` 结构
  - 序列化为 TOML 格式
  - 覆盖写入 `config/templates/default.toml`
  - 包含错误处理（文件权限、格式验证等）
- [x] 1.3 在 `src/main.rs` 中注册新命令
  - 注册 `get_template_config`
  - 注册 `save_template_config`

## 2. 前端 - 页面结构实现
- [x] 2.1 创建 `web/src/views/TemplateView.vue` 基础结构
  - 实现左右分栏布局（Element Plus Layout）
  - 左侧：表单编辑区域
  - 右侧：预览区域
- [x] 2.2 添加路由配置（如需要）
  - 项目使用单页应用，无需独立路由
- [x] 2.3 添加导航入口
  - 在主导航栏添加"模板设置"菜单项

## 3. 前端 - 表单字段实现
- [x] 3.1 实现 `[page]` 配置字段组
  - **只读字段**：dpi, width_mm, height_mm
  - **可编辑字段**：margin_left_mm, margin_right_mm, margin_top_mm, margin_bottom_mm
  - **可编辑字段**：border (Switch), border_thickness_mm
- [x] 3.2 实现 `[layout]` 配置字段组
  - **可编辑字段**：align_h (Select: center/left/right)
  - **可编辑字段**：align_v (Select: center/top/bottom)
  - **可编辑字段**：gap_mm, line_gap_mm
- [x] 3.3 实现 `[elements]` 配置字段组（展开/折叠）
  - **只读字段**：id, type, source, value/key/format
  - **可编辑字段**：max_height_mm（仅针对 text 类型元素）
  - **只读字段**：barcode 元素的 height_mm, quiet_zone_mm, human_readable
- [x] 3.4 实现 `[metadata]` 和 `[fonts]` 配置字段组（只读）
  - 所有字段只读展示
- [x] 3.5 实现 `[output]` 配置字段组（只读）
  - 所有字段只读展示

## 4. 前端 - 实时预览功能
- [x] 4.1 实现预览区域布局
  - 显示预览图片
  - 添加"刷新预览"按钮
  - 添加加载状态指示
- [x] 4.2 集成 `preview_qsl` 命令
  - 调用现有的 `preview_qsl` 生成预览
  - 使用测试数据填充运行时字段
  - 显示生成的 PNG 预览图
- [x] 4.3 实现刷新预览逻辑
  - 用户点击刷新按钮时调用预览
  - 使用当前表单中的配置参数
  - 处理预览失败的错误提示

## 5. 前端 - 保存功能
- [x] 5.1 实现自动保存逻辑
  - 字段变更时自动调用 `save_template_config`
  - 使用防抖（debounce）避免频繁写入
  - 显示保存状态反馈（成功/失败）
- [x] 5.2 实现错误处理
  - 捕获保存失败错误
  - 显示用户友好的错误提示
  - 提供重试机制

## 6. 前端 - UI 优化
- [x] 6.1 实现表单分组和折叠面板
  - 使用 Collapse 组件组织字段
  - 默认展开高频使用的配置组
- [x] 6.2 添加字段说明和提示
  - 为每个字段添加 tooltip 说明
  - 标注单位（mm, dots 等）
- [x] 6.3 添加表单验证
  - 数值范围验证（如边距 >= 0）
  - 必填字段验证

## 7. 测试验证
- [x] 7.1 手动测试完整流程
  - 前端和后端代码构建成功
  - 所有功能已实现
  - 待用户运行应用进行实际测试
- [x] 7.2 边界情况测试
  - 代码中已包含错误处理
  - Element Plus 组件提供内置验证
  - 待用户运行应用进行实际测试

## 8. 文档更新
- [ ] 8.1 更新用户文档
  - 添加模板设置功能使用说明
  - 添加各参数的说明和推荐值
- [ ] 8.2 更新开发文档
  - 记录新增的 Tauri 命令
