## 1. 基础设施

- [x] 1.1 创建 Loading Store (`web/src/stores/loadingStore.ts`)
  - 定义 `LoadingState` 接口
  - 实现 `showLoading(text?)` 函数
  - 实现 `hideLoading()` 函数
  - 实现引用计数机制（支持嵌套调用）
  - 导出响应式状态 `loadingState`

- [x] 1.2 创建全局 Loading 组件 (`web/src/components/common/GlobalLoading.vue`)
  - 全屏遮罩层（z-index: 9999）
  - Element Plus 风格的加载动画
  - 可配置的文本提示
  - 淡入淡出过渡效果
  - 阻止点击穿透

- [x] 1.3 创建 useLoading composable (`web/src/composables/useLoading.ts`)
  - 实现 `withLoading(fn, text?)` 包装函数
  - 导出 `showLoading`、`hideLoading` 快捷方法
  - 导出 `isLoading` 计算属性

## 2. 集成到应用

- [x] 2.1 在 App.vue 中集成 GlobalLoading 组件
  - 导入 GlobalLoading 组件
  - 添加到模板根节点
  - 确保在所有内容之上显示

## 3. 应用到关键场景

- [x] 3.1 顺丰速运模块
  - `SFExpressConfigView.vue` - 保存配置时
  - `CreateOrderDialog.vue` - 创建订单、确认订单时
  - `SFOrderListView.vue` - 确认订单、取消订单、查询订单时
  - `SenderDialog.vue` - 保存寄件人时

- [x] 3.2 QRZ 查询模块
  - `QRZConfigView.vue` - 登录测试、保存凭据时
  - `QRZComConfigView.vue` - 登录测试、保存凭据时
  - `DistributeDialog.vue` - 查询地址时

- [x] 3.3 打印模块
  - `WaybillPrintDialog.vue` - 获取面单、打印面单时
  - `PrintView.vue` - 执行打印时

## 4. 测试验证

- [x] 4.1 功能测试
  - 验证 loading 正确显示和隐藏
  - 验证嵌套调用场景（多个请求同时进行）
  - 验证错误情况下 loading 正确关闭
  - 验证遮罩层阻止用户点击

- [x] 4.2 视觉测试
  - 验证 loading 样式与 Element Plus 风格一致
  - 验证过渡动画流畅
  - 验证在不同页面都能正确显示

## 5. 文档更新（可选）

- [ ] 5.1 更新开发文档
  - 添加 useLoading 使用说明
  - 添加最佳实践指南
  - 更新组件目录结构说明
