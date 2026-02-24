## 为什么

当前顺丰下单流程需要 3 个弹窗（创建→确认→打印）和多次手动操作，效率低下。收件人地址需要手动逐级选择省市区，且默认付款方式为寄方付，与实际使用中到付为主的场景不符。这些摩擦点在高频分卡场景下严重影响操作效率。

## 变更内容

1. **默认付款方式改为到付**：硬编码将 `payMethod` 默认值从 `1`（寄方付）改为 `2`（收方付）
2. **智能地址识别**：在收件人区域新增文本输入框和「智能识别」按钮，用户粘贴一段包含姓名、手机号、省市区、详细地址的文本后，一键自动拆分并填充到对应表单字段
3. **合并确认流程**：去掉独立的 ConfirmOrderDialog，将「提交订单」按钮改为「提交并确认订单」，后台自动串联 create + confirm 两步 API 调用，仅在 filter_result 异常时才弹出提示
4. **确认后自动打印**：订单确认成功后自动调用 fetch_waybill + print_waybill，无需手动打开打印弹窗；打印机未配置时跳过并提示

## 功能 (Capabilities)

### 新增功能
- `address-smart-parse`: 纯前端地址智能解析，从一段自由文本中提取姓名、手机号、省市区、详细地址并填充到表单

### 修改功能
- `sf-express-integration`: 下单默认付款方式改为到付；合并创建+确认为一步操作；确认成功后自动打印面单；去掉独立的确认弹窗

## 影响

- **前端组件**：
  - `CreateOrderDialog.vue` — 主要修改：默认付款方式、新增智能识别区域、提交按钮逻辑改为 create+confirm+print 串联
  - `ConfirmOrderDialog.vue` — 不再从 CreateOrderDialog 独立弹出（订单列表页的确认功能保留）
  - `AddressSelector.vue` — 无需修改，智能识别填充后仍通过现有组件展示和微调
- **后端**：无需修改，现有 `sf_create_order`、`sf_confirm_order`、`sf_fetch_waybill`、`sf_print_waybill` 命令已足够支撑
- **数据库**：无变更
- **依赖**：无新增依赖，地址解析使用现有 `china-regions.ts` 数据
