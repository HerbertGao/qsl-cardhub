## 上下文

当前顺丰下单流程由 3 个独立弹窗组成：`CreateOrderDialog`（填写表单）→ `ConfirmOrderDialog`（展示确认信息）→ `WaybillPrintDialog`（获取+打印面单）。每一步都需要用户手动操作。收件人地址需要通过三级联动逐级选择省市区，付款方式默认为寄方付。

调用方 `DistributeDialog` 通过事件串联三个弹窗：`order-created` → 打开确认弹窗 → `success` → 保存运单号。

## 目标 / 非目标

**目标：**
- 将下单流程从 3 个弹窗缩减为 1 个弹窗
- 支持粘贴一段文本自动拆分为姓名、手机号、省市区、详细地址
- 默认付款方式改为到付（收方付）
- 提交订单后自动完成确认和打印，无需额外操作

**非目标：**
- 不修改后端 Rust 代码或 Tauri Commands，现有 API 已足够
- 不修改数据库 schema
- 不接入外部地址解析 API
- 不改变订单列表页（`SFOrderListView`）的确认/打印流程
- 不重构 `ConfirmOrderDialog` 组件本身（订单列表页仍在使用）

## 决策

### 1. 默认付款方式：硬编码改为到付

`CreateOrderDialog.vue` 中 `payMethod` 默认值从 `ref(1)` 改为 `ref(2)`。同时 `handleClose` 中的重置值也改为 `2`。

**替代方案：** 做成全局配置可选项 → 用户明确不需要，硬编码即可。

### 2. 地址智能识别：纯前端文本解析

在 `CreateOrderDialog` 的收件人区域上方新增一个文本输入框和「智能识别」按钮。解析逻辑作为独立工具函数 `parseAddress` 放在 `web/src/utils/addressParser.ts`。

**解析策略（按优先级顺序）：**

1. **提取手机号**：正则 `1[3-9]\d{9}`，支持中间有横杠/空格的情况（先清洗再匹配）
2. **提取省份**：遍历 `getProvinces()` 列表，在剩余文本中查找匹配。支持带/不带"省"/"市"（直辖市）/"自治区"等后缀
3. **提取城市**：在匹配省份下，遍历 `getCities()` 查找匹配
4. **提取区县**：在匹配城市下，遍历 `getDistricts()` 查找匹配
5. **剩余部分**：手机号和省市区之外的文本，较短的部分（≤5字符且非数字开头）作为姓名，较长的作为详细地址

**利用现有基础设施：** `china-regions.ts` 已有 `findProvinceByName`、`findCityByName`、`findDistrictByName` 等查找函数，但这些函数基于名称精确/包含匹配，需要先从原始文本中提取候选片段。解析函数将直接使用 `getProvinces()`、`getCities()`、`getDistricts()` 遍历匹配。

**替代方案：** 接入高德/百度地址解析 API → 增加外部依赖和网络需求，QSL 场景下地址格式相对规范，纯前端够用。

### 3. 合并确认流程：提交即创建+确认

修改 `CreateOrderDialog` 的 `handleSubmit` 逻辑：

```
用户点击「提交并确认订单」
  ↓
sf_create_order（创建订单）
  ↓
检查 filter_result：
  - 2（可收派）→ 自动调用 sf_confirm_order → 继续打印流程
  - 1（人工确认）→ ElMessageBox.confirm 提示用户，可选择继续确认
  - 3（不可收派）→ ElMessage.error 提示，订单保留 pending 状态
  ↓
sf_fetch_waybill（获取面单 PDF）
  ↓
sf_print_waybill（发送到打印机）
  ↓
关闭弹窗，emit('success', order)
```

**关键变更：**
- `CreateOrderDialog` 不再 emit `order-created` 事件
- `DistributeDialog` 不再引用 `ConfirmOrderDialog`（但组件本身保留，订单列表页还在用）
- `CreateOrderDialog` 新增 emit `success` 事件直接传回最终的 `SFOrder`
- 整个流程使用 `useLoading` 的 `withLoading` 显示全局 loading 状态，分步更新文案

**替代方案：** 保留确认弹窗但自动打开 → 仍然多一次点击，不如直接合并彻底。

### 4. 自动打印：确认后串联 fetch + print

确认成功后，自动调用 `sf_fetch_waybill` 获取面单 PDF，再调用 `sf_print_waybill` 打印。

**打印机配置检查：** 调用 `get_printer_config` 获取打印机名称。如果未配置打印机（名称为空），跳过打印步骤，仅提示「订单已确认，请配置打印机后手动打印面单」。

**错误处理：** 打印失败不影响订单确认状态。如果 fetch 或 print 失败，显示警告但不回滚确认。

## 风险 / 权衡

- **[地址解析准确率]** 纯前端匹配对非标准格式（如缺省/缺市）可能失败 → 解析结果填充后用户可手动微调，不影响表单提交校验
- **[打印机未配置]** 用户可能未配置打印机就使用一键流程 → 检查打印机配置，未配置时跳过打印并提示
- **[SF API 超时]** 串联 3 个 API 调用增加总耗时 → 使用全局 loading 动态更新步骤文案，用户感知进度
- **[ConfirmOrderDialog 复用]** 订单列表页仍在使用该组件 → 仅在 DistributeDialog 中移除引用，组件本身不删除
