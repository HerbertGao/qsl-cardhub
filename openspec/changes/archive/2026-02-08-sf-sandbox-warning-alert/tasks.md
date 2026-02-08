## 1. 前端状态扩展

- [x] 1.1 在 `CreateOrderDialog.vue` 的 `checkApiConfig()` 方法中，将返回的 `environment` 字段存储到新增的 `currentEnvironment` ref 变量
- [x] 1.2 新增计算属性 `isSandbox`，当 `apiConfigured` 为 true 且 `currentEnvironment` 为 `"sandbox"` 时返回 true

## 2. 沙箱环境 Warning Alert

- [x] 2.1 在模板中「API 未配置提示」alert 之后，新增一个 `v-if="isSandbox"` 的 `el-alert` 组件，type 为 `warning`，title 为「当前为沙箱环境」，show-icon，closable 为 false
- [x] 2.2 在 alert 的默认插槽中添加说明文字「订单将不会被真实派发」和可点击链接「请点击此处切换至生产环境」，点击链接调用已有的 `goToConfig()` 方法

## 3. 导航参数支持

- [x] 3.1 在 `navigationStore.ts` 中新增 `navigationParams` ref，扩展 `navigateTo` 支持可选 `params` 参数，新增 `consumeNavigationParams` 函数（读后即清）
- [x] 3.2 修改 `CreateOrderDialog.vue` 的 `go-config` 事件和 `goToConfig` 方法支持可选 tab 参数，沙箱警告传 `'api'`，寄件人未配置传 `'sender'`
- [x] 3.3 修改 `DistributeDialog.vue` 的 `handleGoConfig` 转发 tab 参数到 `navigateTo`
- [x] 3.4 修改 `SFExpressConfigView.vue` 在 `onMounted` 中通过 `consumeNavigationParams` 读取 tab 参数，有参数时直接设置 `activeTab`，无参数时走默认逻辑

## 4. 验证

- [ ] 4.1 手动验证：沙箱环境下打开下单对话框，确认 warning alert 正确显示且跳转链接正常工作
- [ ] 4.2 手动验证：生产环境下打开下单对话框，确认不显示沙箱环境警告
- [ ] 4.3 手动验证：API 未配置时打开下单对话框，确认仅显示 API 未配置提示，不显示沙箱环境警告
- [ ] 4.4 手动验证：沙箱警告跳转后配置页面展示 API 凭据配置 Tab
- [ ] 4.5 手动验证：寄件人未配置跳转后配置页面展示寄件人信息 Tab
- [ ] 4.6 手动验证：从菜单直接进入配置页面，按默认逻辑展示 Tab
