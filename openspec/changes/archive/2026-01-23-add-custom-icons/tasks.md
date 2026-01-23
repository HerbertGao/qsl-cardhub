## 1. 基础配置

- [x] 1.1 安装依赖
  - 安装 `unplugin-icons`
  - 可选：安装常用图标集 `@iconify-json/mdi`

- [x] 1.2 配置 Vite 插件 (`web/vite.config.ts`)
  - 导入 `unplugin-icons/vite`
  - 配置 `Icons` 插件
  - 设置 `compiler: 'vue3'`
  - 配置自定义图标集（从 `src/assets/icons/` 加载）

- [x] 1.3 创建自定义图标目录
  - 创建 `web/src/assets/icons/` 目录
  - 添加示例图标文件

## 2. 类型支持

- [x] 2.1 添加类型声明
  - 在 `web/src/types/icons.d.ts` 中添加 `~icons/*` 模块声明
  - 确保 TypeScript 能正确识别图标组件

## 3. 使用验证

- [x] 3.1 测试 Iconify 图标
  - 在测试组件中使用 MDI 图标
  - 验证图标正常显示
  - 验证图标大小和颜色可配置

- [x] 3.2 测试自定义图标
  - 添加顺丰图标 SVG 到 `assets/icons/sf-express.svg`
  - 在组件中使用自定义图标
  - 验证图标正常显示

- [x] 3.3 测试与 el-icon 配合
  - 将 unplugin-icons 图标放入 `<el-icon>` 中
  - 验证 size 和 color 属性生效

## 4. 应用到实际场景

- [x] 4.1 替换需要的图标
  - 顺丰订单模块：使用自定义 `sf-express` 图标
  - App.vue 菜单中的顺丰订单图标
  - DistributeDialog.vue 中的顺丰速运下单按钮图标

## 5. 文档更新（可选）

- [ ] 5.1 添加图标使用说明
  - 如何使用 Iconify 图标
  - 如何添加自定义图标
  - 命名规则说明
