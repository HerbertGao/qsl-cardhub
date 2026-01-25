# 提案：顺丰速运前端Bug修复及优化

## 概述

本提案旨在修复顺丰速运模块的四个前端问题，并实现官方API参数预配置功能。

## 问题分析

### Bug 1：去配置按钮无法跳转

**现状：**
- `CreateOrderDialog.vue` 中的 `goToConfig()` 函数只发出了 `go-config` 事件
- 但 `DistributeDialog.vue` 中引用 `CreateOrderDialog` 时没有监听 `@go-config` 事件
- 因此点击"去配置"按钮后无法跳转到顺丰速运配置界面

**位置：** `web/src/components/sf-express/CreateOrderDialog.vue:309-312`

### Bug 2：收件人数据未清空及验证提示问题

**现状：**
- `CreateOrderDialog.vue` 的 `handleClose()` 函数会重置 `recipientForm`
- 但 watch 中打开对话框时，如果没有 `defaultRecipient`，不会主动清空表单
- 同时，省市县验证只检查了 `province` 字段（规则中），但 `city` 和 `district` 的验证是在 `handleSubmit` 中单独进行的
- 当重新打开时，如果上次填写了地区但数据还在，会出现"已有数据却提示未填写"的情况

**位置：** `web/src/components/sf-express/CreateOrderDialog.vue:283-296`

### Bug 3：缺少API配置状态预检查

**现状：**
- 下单界面打开时只加载寄件人信息
- 没有检查顺丰API凭据是否已配置
- 用户可能在未配置API的情况下尝试下单，直到提交时才发现错误

**位置：** `web/src/components/sf-express/CreateOrderDialog.vue:283-296`

### Bug 5：生产环境凭据保存失败

**现状：**
- 生产环境下，顺丰速运 API 配置保存后显示"已保存"
- 但页面刷新或重新打开后，状态显示"未配置"
- 怀疑是系统钥匙串在某些环境下存在兼容性问题

**根因分析：**
- 当前实现在生产模式优先使用系统钥匙串（`credentials.rs:53-60`）
- `is_available()` 测试可能通过，但实际保存/读取时失败
- macOS 上未签名应用可能无法正确使用钥匙串
- Windows/Linux 上钥匙串实现（secret-service、credential-manager）也可能有兼容性问题

**解决方案：**
- 移除对系统钥匙串的支持
- 统一使用本地加密文件存储凭据
- 简化代码，提高跨平台稳定性

**位置：**
- `src/security/credentials.rs`
- `src/security/keyring_storage.rs`（可删除）
- `src/commands/security.rs`

---

### Bug 4：配置界面改为左侧 Tab 标签页

**现状：**
- `SFExpressConfigView.vue` 中使用折叠面板（el-collapse）展示配置区域
- 折叠面板在展开/收起时会改变页面高度，用户体验不够流畅

**改进：**
- 将折叠面板改为左侧 Tab 标签页（el-tabs with tab-position="left"）
- 提供更直观的导航体验，保持页面布局稳定
- 默认选中的 Tab：
  - 如果未配置API，默认选中"API 凭据配置"
  - 如果已配置API，默认选中"寄件人信息"

**位置：** `web/src/views/SFExpressConfigView.vue`

### 需求 5：日志界面优化

**需求：**
1. 增加"开启DEBUG日志"开关，默认关闭
   - 关闭时：隐藏DEBUG级别日志，日志级别过滤器中也不显示DEBUG选项
   - 开启时：显示DEBUG级别日志，日志级别过滤器中显示DEBUG选项
2. 日志列表高度与窗口大小关联
   - 避免页面出现两个滚动条（页面滚动条 + 表格滚动条）
   - 表格高度应自适应填满剩余空间
3. TSPL指令DEBUG日志输出
   - 在 `tspl.rs` 生成 TSPL 指令后，通过 DEBUG 级别日志打印完整指令内容
   - 方便开发者调试打印问题

**现状分析：**
- 当前日志级别过滤器包含DEBUG选项（第48-49行）
- 表格高度固定为600px（第125行 `max-height="600"`）
- 页面有 `page-content` 样式，导致可能出现双滚动条
- `tspl.rs` 已有部分日志输出（info/debug），但未输出完整TSPL指令

**位置：**
- `web/src/views/LogView.vue`
- `src/printer/tspl.rs`

---

### Bug 6：二次展示分发/退回界面时没有带出之前的信息

**现状：**
- 当卡片已被分发（状态为 distributed）时，再次打开分发对话框
- 对话框总是重置表单为默认值（快递、空备注、空代收呼号）
- 没有加载 `card.metadata.distribution` 中保存的历史分发信息
- 退回对话框同样存在此问题，没有加载 `card.metadata.return` 中的历史退回信息

**根因分析：**
- `DistributeDialog.vue` 的 watch 监听 `props.visible`
- 打开对话框时（第 839-864 行）总是将表单重置为默认值
- 没有检查 `props.card.metadata.distribution` 是否存在
- `ReturnDialog.vue` 的 watch 监听 `props.visible`（第 201-214 行）同样有此问题
- 没有检查 `props.card.metadata.return` 是否存在

**解决方案：**
- 在 watch 中打开对话框时，检查对应的 metadata 字段
- 如果存在历史信息，用历史数据填充表单
- 否则使用默认值

**位置：**
- `web/src/components/cards/DistributeDialog.vue:839-864`
- `web/src/components/cards/ReturnDialog.vue:201-214`

---

### 需求 7：优化顺丰速运下单界面布局

**需求：**
- 优化下单界面的展示，使布局更紧凑
- 尽量在不滚动的情况下展示全部内容
- 保持信息完整性的同时减少垂直空间占用

**现状分析：**
- 当前对话框宽度为 700px，内容区域 `max-height: 60vh`
- 各 section 有 24px 的 margin-bottom 和 16px 的 padding
- 寄件人信息、托寄物信息、收件人信息分三个区块展示
- 表单 label-width 为 80px，有一定的空间浪费

**优化方案：**
1. 减少区块之间的间距（24px → 16px）
2. 减少区块内部的 padding（16px → 12px）
3. 寄件人信息改为单行展示（姓名、电话、地址三列）
4. 托寄物信息与付款方式可以放在同一行（使用 el-row）
5. 收件人表单保持现有布局，但减少行间距
6. 调整对话框高度限制，尽量避免滚动

**位置：** `web/src/components/sf-express/CreateOrderDialog.vue`

---

### 需求 8：二次确认独立对话框

**需求：**
- 下单成功后弹出独立的确认对话框，展示所有发送给顺丰的字段
- 让用户在确认前能够核对完整的订单信息
- 包括但不限于：寄件人、收件人、托寄物、付款方式、产品类型等

**现状分析：**
- 当前下单成功后只在同一对话框中显示"下单成功"和运单号
- 没有展示发送给顺丰的完整请求内容
- 用户无法在确认前核对订单详情
- 确认和下单混在同一个对话框中，界面切换不够清晰

**设计方案：**
- 新建 `ConfirmOrderDialog.vue` 组件作为独立的二次确认对话框
- 下单成功后，关闭下单对话框，打开确认对话框
- 确认对话框使用 `el-descriptions` 组件以表格形式展示完整订单信息

**需要展示的字段：**
1. **寄件人信息**：姓名、电话、省市区、详细地址
2. **收件人信息**：姓名、电话、省市区、详细地址
3. **托寄物信息**：物品名称
4. **订单信息**：
   - 客户订单号（order_id）
   - 快件产品类别（express_type_id，如"顺丰标快"）
   - 付款方式（pay_method，如"寄方付"/"收方付"/"第三方付"）
5. **顺丰返回信息**：
   - 运单号（waybill_no）
   - 原寄地区域代码（origin_code）
   - 目的地区域代码（dest_code）
   - 筛单结果（filter_result，转为可读文本）

**后端改造：**
- `sf_create_order` 返回值需要包含构建请求时使用的完整数据
- 新增 `CreateOrderResponse` 中的字段：
  - `sender_info`: 寄件人完整信息
  - `recipient_info`: 收件人完整信息
  - `cargo_name`: 托寄物名称
  - `pay_method`: 付款方式
  - `express_type_id`: 快件产品类别

**位置：**
- 新增 `web/src/components/sf-express/ConfirmOrderDialog.vue`
- 修改 `web/src/components/sf-express/CreateOrderDialog.vue`
- 修改 `web/src/components/cards/DistributeDialog.vue`（调用方）
- 修改 `src/commands/sf_express.rs`（后端返回值扩展）

---

### 需求 6：默认API参数预配置方案

**需求：**
- 预定义一组默认API参数（partnerID、沙箱/生产校验码）
- 允许用户切换到自定义参数模式
- 选择默认参数时显示风险提示
- 自定义模式下显示顺丰开放平台链接提示

**设计方案：**
- 使用后端配置文件（TOML）存储预定义参数，而非硬编码在前端
- 前端提供"使用默认参数"/"使用自定义参数"切换
- 选择默认参数时，显示警告提示：「该参数不可滥用，有随时停用或更换的风险，请尽量使用自定义参数」
- 选择自定义参数时，显示顺丰开放平台申请提示和链接

## 影响范围

### 修改文件

1. `web/src/components/sf-express/CreateOrderDialog.vue` - Bug 1, 2, 3, 需求 7, 需求 8
2. `web/src/components/sf-express/ConfirmOrderDialog.vue` - 需求 8（新增文件）
3. `web/src/components/cards/DistributeDialog.vue` - Bug 1, Bug 6, 需求 8
4. `web/src/components/cards/ReturnDialog.vue` - Bug 6
5. `web/src/views/SFExpressConfigView.vue` - Bug 4, 需求 6
6. `web/src/views/LogView.vue` - 需求 5（DEBUG开关、高度自适应）
7. `web/src/App.vue` - Bug 1（需要暴露导航方法或使用事件总线）
8. `src/security/credentials.rs` - Bug 5（移除钥匙串支持，统一使用加密文件）
9. `src/commands/security.rs` - Bug 5（移除 `check_keyring_available` 命令或始终返回 false）
10. `src/commands/sf_express.rs` - 需求 8（扩展返回值）
11. `src-tauri/src/` 相关后端文件 - 需求 6（读取预定义参数）

### 可删除文件

1. `src/security/keyring_storage.rs` - Bug 5（移除钥匙串支持后不再需要）

### 新增文件

1. `config/sf_express_default.toml.example` - 默认参数模板（提交到Git）
2. `config/sf_express_default.toml` - 实际默认参数（被gitignore，不提交）

### Git忽略配置

在 `.gitignore` 中添加：
```
config/sf_express_default.toml
```

### 新增类型定义

```typescript
// web/src/types/models.ts
interface SFOfficialConfig {
  partner_id: string
  checkword_sandbox: string | null
  checkword_prod: string | null
}
```

## 兼容性

- 向后兼容：现有配置文件结构不变
- 用户已保存的自定义配置继续有效
- 预定义参数作为可选默认值

## 验收标准

1. Bug 1：点击"去配置"按钮能正确跳转到顺丰速运配置页面
2. Bug 2：重复打开下单对话框时，收件人表单被正确重置
3. Bug 3：打开下单界面时检查API配置，未配置时显示提示
4. Bug 4：配置界面改为左侧 Tab 标签页，根据API配置状态智能选中对应 Tab
5. Bug 5：生产环境凭据保存正常工作，重启后配置仍然存在
6. Bug 6：二次打开已分发卡片的分发对话框时，能正确显示之前的分发信息
5. 需求 5：日志界面优化
   - 默认不显示DEBUG级别日志
   - 开启"显示DEBUG日志"后才显示DEBUG级别
   - 日志表格高度自适应窗口，页面只有一个滚动条
   - 打印时可在DEBUG日志中看到完整TSPL指令
6. 需求 6：
   - 配置界面显示"默认参数"/"自定义参数"切换
   - 默认参数从配置文件读取，不硬编码
   - 选择默认参数时显示风险警告提示
   - 自定义参数模式下显示申请提示和链接
7. 需求 7：下单界面布局紧凑
   - 无需滚动即可看到完整的下单表单
   - 寄件人信息单行展示
   - 托寄物和付款方式同行展示
8. 需求 8：二次确认页面完整展示
   - 展示寄件人完整信息（姓名、电话、地址）
   - 展示收件人完整信息（姓名、电话、地址）
   - 展示托寄物、付款方式、产品类型
   - 展示顺丰返回的运单号、区域代码等信息
