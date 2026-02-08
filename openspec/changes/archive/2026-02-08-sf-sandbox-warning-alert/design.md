## 上下文

当前 `CreateOrderDialog.vue` 在打开时会调用 `checkApiConfig()` 获取配置信息（含 `environment` 字段），但仅用于检查凭据是否完整，未利用环境信息向用户提示当前环境状态。

现有代码中已有类似的 `el-alert` 模式——当 API 未配置时显示 warning alert 并提供「去配置」按钮，本次变更可复用相同的 UI 模式和跳转逻辑。

## 目标 / 非目标

**目标：**
- 在下单对话框中，当 environment 为 sandbox 时显示 warning alert 提醒用户
- 提供跳转到顺丰速运配置页面的链接，方便用户切换到生产环境

**非目标：**
- 不阻止用户在沙箱环境下提交订单（仅提醒，不禁用按钮）
- 不修改后端逻辑或 API 配置接口

## 决策

### 决策 1：环境信息获取方式

**选择**：复用现有 `checkApiConfig()` 中已获取的 `environment` 字段，新增 `currentEnvironment` ref 存储。

**替代方案**：新增独立的环境查询 API。

**理由**：`sf_load_config` 已返回 `environment` 字段，无需新增后端接口，只需在前端保存该值即可。

### 决策 2：Alert 展示位置

**选择**：放在「API 未配置提示」之后、「寄件人信息」区域之前，与现有 API 未配置 alert 平级。

**替代方案**：放在对话框标题栏或底部。

**理由**：与现有 alert 保持视觉一致性，用户打开对话框时第一眼可见。当 API 未配置时两个 alert 不会同时出现（未配置时无 environment 信息）。

### 决策 3：跳转方式

**选择**：复用现有 `goToConfig()` 方法，通过 `emit('go-config', tab?)` 事件携带目标 Tab 参数通知父组件导航到配置页面。

**理由**：该跳转逻辑已存在且经过验证，只需扩展事件签名支持可选的 tab 参数。

### 决策 4：导航参数传递机制

**选择**：扩展现有 `navigationStore` 的 `navigateTo` 函数，增加可选的 `params` 参数（`Record<string, string>`）。目标页面通过 `consumeNavigationParams()` 一次性读取并清除参数。

**替代方案 A**：通过 Props 从 App.vue 传递给 SFExpressConfigView。

**替代方案 B**：通过 localStorage 传递。

**理由**：`navigationStore` 已是项目中唯一的跨组件导航机制，在其上扩展最自然。Props 方案需要 App.vue 维护额外状态；localStorage 方案不够响应式且存在清理问题。`consumeNavigationParams` 采用读后即清的模式，避免参数残留。

**时序处理**：`clearNavigationTarget()` 只清除 target 不清除 params，确保目标页面在 `onMounted` 中仍能通过 `consumeNavigationParams()` 读取到参数。

## 风险 / 权衡

- **[风险]** 用户可能忽略 warning alert 仍然下单 → 这是预期行为，沙箱环境允许测试下单，仅做提醒不做阻断
- **[风险]** API 未配置和沙箱环境两个 alert 同时展示导致页面臃肿 → 实际不会发生：API 未配置时 `checkApiConfig` 的 catch 分支将 `apiConfigured` 设为 false，此时不应判断环境；仅当配置完整时才检查环境
