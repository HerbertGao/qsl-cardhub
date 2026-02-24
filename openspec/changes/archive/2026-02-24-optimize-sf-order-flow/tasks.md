## 1. 地址智能识别

- [x] 1.1 创建 `web/src/utils/addressParser.ts`，实现 `parseAddress(text: string)` 函数，返回 `{ name, phone, province, city, district, address }` 结构，利用 `china-regions.ts` 的省市区数据进行匹配
- [x] 1.2 在 `CreateOrderDialog.vue` 详细地址输入框旁新增「智能识别」按钮，点击后调用 `parseAddress` 并将结果填充到 `recipientForm` 各字段

## 2. 默认付款方式

- [x] 2.1 修改 `CreateOrderDialog.vue` 中 `payMethod` 默认值从 `ref(1)` 改为 `ref(2)`，同时修改 `handleClose` 中的重置值为 `2`

## 3. 合并确认流程

- [x] 3.1 修改 `CreateOrderDialog.vue` 的 `handleSubmit`，在 `sf_create_order` 成功后根据 `filter_result` 自动调用 `sf_confirm_order`，filter_result=1 时弹出 ElMessageBox 让用户选择，filter_result=3 时显示错误并中止
- [x] 3.2 确认成功后在 `handleSubmit` 中串联调用 `sf_fetch_waybill` + `sf_print_waybill` 实现自动打印，打印机未配置时跳过并提示
- [x] 3.3 修改 `CreateOrderDialog` 的按钮文案从「提交订单」改为「提交并确认订单」，移除 `order-created` emit，改为确认+打印完成后 emit `success` 传回 `SFOrder`
- [x] 3.4 修改 `DistributeDialog.vue`，移除 `ConfirmOrderDialog` 的引用和相关状态（`confirmOrderDialogVisible`、`pendingOrderData`、`handleOrderCreated`、`handleConfirmCancel`），改为监听 `CreateOrderDialog` 的 `success` 事件直接处理运单号回填
