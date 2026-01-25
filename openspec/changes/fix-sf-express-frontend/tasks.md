# 任务列表：顺丰速运前端Bug修复及优化

## 阶段一：Bug 修复

### 任务 1.1：修复去配置按钮跳转问题
**优先级：** 高
**依赖：** 无

**步骤：**
1. 在 `DistributeDialog.vue` 中监听 `CreateOrderDialog` 的 `@go-config` 事件
2. 实现事件处理函数，关闭分发对话框
3. 使用 `emit` 向上层传递导航请求，或使用事件总线/状态管理
4. 在 `App.vue` 中处理导航到 `data-config-sf-express`

**验证：**
- 在分发对话框中打开顺丰下单
- 无寄件人时点击"去配置"
- 确认跳转到顺丰速运配置页面

---

### 任务 1.2：修复收件人数据清空问题
**优先级：** 高
**依赖：** 无

**步骤：**
1. 在 `CreateOrderDialog.vue` 的 watch 中，打开对话框时先重置 `recipientForm`
2. 然后再根据 `defaultRecipient` 填充数据
3. 调用 `formRef.value?.clearValidate()` 清除验证状态

**验证：**
- 打开下单对话框，填写收件人信息
- 关闭对话框
- 重新打开，确认收件人信息已清空

---

### 任务 1.3：添加API配置状态预检查
**优先级：** 中
**依赖：** 无

**步骤：**
1. 在 `CreateOrderDialog.vue` 中添加 `apiConfigured` 状态
2. 在 watch 中打开对话框时，调用 `sf_load_config` 检查配置状态
3. 如果未配置（partnerId 为空或校验码缺失），显示提示信息
4. 提示信息中包含"去配置"按钮

**验证：**
- 清除顺丰API配置
- 打开下单对话框
- 确认显示"请先配置顺丰速运API"提示

---

### 任务 1.4：配置界面改为左侧 Tab 标签页
**优先级：** 中
**依赖：** 无

**步骤：**
1. 将 `SFExpressConfigView.vue` 中的 `el-collapse` 替换为 `el-tabs`
2. 设置 `tab-position="left"` 使用左侧 Tab 布局
3. 将 `el-collapse-item` 替换为 `el-tab-pane`
4. 修改 `activePanel` 变量名为 `activeTab`，类型保持为字符串
5. 在 `loadConfig` 函数中检查API配置状态：
   - 如果未配置API（partnerId 为空），设置 `activeTab.value = 'api'`
   - 如果已配置API，设置 `activeTab.value = 'sender'`
6. 调整样式，确保左侧 Tab 和右侧内容区域布局合理

**UI 设计：**
```
┌────────────────┬─────────────────────────────────────┐
│                │                                     │
│  API 凭据配置  │   [当前选中 Tab 的内容区域]          │
│                │                                     │
│  寄件人信息    │                                     │
│                │                                     │
└────────────────┴─────────────────────────────────────┘
```

**验证：**
- 打开配置页面，确认显示为左侧 Tab 标签页布局
- 清除顺丰API配置，打开配置页面，确认"API 凭据配置" Tab 默认选中
- 配置顺丰API后，重新打开配置页面，确认"寄件人信息" Tab 默认选中
- 点击不同 Tab 切换内容，确认切换流畅

---

### 任务 1.5：修复生产环境凭据保存失败
**优先级：** 高
**依赖：** 无

**问题描述：**
生产环境下保存顺丰API配置显示成功，但实际未保存，怀疑系统钥匙串兼容性问题。

**解决方案：移除钥匙串支持，统一使用加密文件**

**步骤：**
1. 修改 `src/security/credentials.rs`：
   - 移除 `StorageStrategy::Keyring` 分支
   - `get_credential_storage()` 始终返回 `EncryptedFileStorage`
   - 移除 `#[cfg(debug_assertions)]` 条件编译
   - `is_keyring_available()` 始终返回 `false`

2. 修改 `src/commands/security.rs`：
   - `check_keyring_available` 命令始终返回 `false`
   - 或完全移除此命令（需同步修改前端）

3. 修改前端 `SFExpressConfigView.vue`：
   - 移除钥匙串相关的提示文字
   - 存储方式提示改为"本地加密文件"

4. 可选：删除 `src/security/keyring_storage.rs`
5. 可选：移除 `Cargo.toml` 中的 `keyring` 依赖

**验证：**
- 生产环境打包后，保存顺丰API配置
- 重启应用，确认配置仍然存在
- 在 Windows、macOS、Linux 上测试

---

### 任务 1.6：修复二次展示分发/退回界面时没有带出之前的信息
**优先级：** 中
**依赖：** 无

**问题描述：**
当卡片已被分发（状态为 distributed）或退回（状态为 returned）后，再次打开对应对话框时，表单总是重置为默认值，没有显示之前的信息。

**步骤：**

**1. 修复 DistributeDialog.vue：**
1. 在 watch 中（打开对话框时）
2. 检查 `props.card.metadata.distribution` 是否存在
3. 如果存在且卡片状态为 distributed：
   - 用 `distribution.method` 填充分发方式
   - 用 `distribution.remarks` 填充备注
   - 用 `distribution.proxy_callsign`（如有）填充代收呼号
4. 否则使用默认值

**代码示例（DistributeDialog）：**
```typescript
watch(() => props.visible, (newVal: boolean): void => {
  if (newVal) {
    // 检查是否有历史分发信息
    const distribution = props.card.metadata?.distribution
    if (distribution && props.card.status === 'distributed') {
      form.value = {
        method: distribution.method || '快递',
        remarks: distribution.remarks || '',
        proxyCallsign: distribution.proxy_callsign || ''
      }
    } else {
      // 重置表单为默认值
      form.value = {
        method: '快递',
        remarks: '',
        proxyCallsign: ''
      }
    }
    // ... 其余代码
  }
})
```

**2. 修复 ReturnDialog.vue：**
1. 在 watch 中（打开对话框时）
2. 检查 `props.card.metadata.return` 是否存在
3. 如果存在且卡片状态为 returned：
   - 用 `return.method` 填充处理方式
   - 用 `return.remarks` 填充备注
4. 否则使用默认值

**代码示例（ReturnDialog）：**
```typescript
watch(() => props.visible, (newVal: boolean): void => {
  if (newVal) {
    // 检查是否有历史退回信息
    const returnInfo = props.card?.metadata?.return
    if (returnInfo && props.card?.status === 'returned') {
      form.value = {
        method: returnInfo.method || 'NOT FOUND',
        remarks: returnInfo.remarks || ''
      }
    } else {
      // 重置表单为默认值
      form.value = {
        method: 'NOT FOUND',
        remarks: ''
      }
    }
    // ... 其余代码
  }
})
```

**验证：**
1. **分发对话框：**
   - 创建或选择一张卡片，打开分发对话框
   - 选择"面交"方式，填写备注"测试备注"，点击分发
   - 再次点击该卡片打开分发对话框
   - 确认分发方式显示"面交"，备注显示"测试备注"
   - 对于代收方式，确认代收呼号也能正确回显

2. **退回对话框：**
   - 选择一张卡片，打开退回对话框
   - 选择"REFUSED"方式，填写备注"拒收测试"，点击退回
   - 再次点击该卡片打开退回对话框
   - 确认处理方式显示"REFUSED"，备注显示"拒收测试"

---

### 任务 1.7：优化下单界面布局
**优先级：** 中
**依赖：** 无

**问题描述：**
当前下单界面垂直空间占用较多，需要滚动才能看到完整内容。

**步骤：**
1. 减少区块间距：`.section` 的 `margin-bottom` 从 24px 改为 16px
2. 减少区块内部填充：`.section` 的 `padding` 从 16px 改为 12px
3. 寄件人信息改为单行紧凑展示：
   ```html
   <div class="sender-info-compact">
     <span><strong>{{ sender.name }}</strong> {{ sender.phone }}</span>
     <span class="address">{{ sender.province }}{{ sender.city }}{{ sender.district }}{{ sender.address }}</span>
   </div>
   ```
4. 托寄物信息和付款方式放在同一行：
   ```html
   <el-row :gutter="16">
     <el-col :span="12">
       <el-form-item label="物品名称">...</el-form-item>
     </el-col>
     <el-col :span="12">
       <el-form-item label="付款方式">...</el-form-item>
     </el-col>
   </el-row>
   ```
5. 减小表单行间距
6. 移除 `max-height: 60vh` 限制，让对话框自适应内容高度

**验证：**
- 在 1080p 分辨率下，下单表单无需滚动即可完整显示
- 所有信息仍然清晰可读

---

### 任务 1.8：创建二次确认独立对话框
**优先级：** 中
**依赖：** 无

**问题描述：**
下单成功后需要在独立对话框中展示完整订单信息供用户确认。

**步骤：**

**1. 后端扩展返回值（sf_express.rs）：**
修改 `CreateOrderResponse` 结构，新增字段：
```rust
pub struct CreateOrderResponse {
    // 现有字段
    pub order_id: String,
    pub waybill_no_list: Vec<String>,
    pub local_order: SFOrder,
    // 新增字段
    pub sender_info: SenderDisplayInfo,
    pub recipient_info: RecipientDisplayInfo,
    pub cargo_name: String,
    pub pay_method: i32,
    pub express_type_id: i32,
    pub origin_code: Option<String>,
    pub dest_code: Option<String>,
    pub filter_result: Option<i32>,
}

pub struct SenderDisplayInfo {
    pub name: String,
    pub phone: String,
    pub address: String,  // 省市区+详细地址
}

pub struct RecipientDisplayInfo {
    pub name: String,
    pub phone: String,
    pub address: String,  // 省市区+详细地址
}
```

**2. 创建 ConfirmOrderDialog.vue：**
```vue
<template>
  <el-dialog v-model="dialogVisible" title="确认订单" width="600px">
    <!-- 订单基本信息 -->
    <el-descriptions title="订单信息" :column="2" border>
      <el-descriptions-item label="客户订单号">{{ orderData.order_id }}</el-descriptions-item>
      <el-descriptions-item label="运单号">{{ orderData.waybill_no_list[0] }}</el-descriptions-item>
      <el-descriptions-item label="产品类型">{{ expressTypeName }}</el-descriptions-item>
      <el-descriptions-item label="付款方式">{{ payMethodName }}</el-descriptions-item>
    </el-descriptions>

    <!-- 寄件人信息 -->
    <el-descriptions title="寄件人" :column="1" border style="margin-top: 16px">
      <el-descriptions-item label="姓名/电话">
        {{ orderData.sender_info.name }} {{ orderData.sender_info.phone }}
      </el-descriptions-item>
      <el-descriptions-item label="地址">{{ orderData.sender_info.address }}</el-descriptions-item>
    </el-descriptions>

    <!-- 收件人信息 -->
    <el-descriptions title="收件人" :column="1" border style="margin-top: 16px">
      <el-descriptions-item label="姓名/电话">
        {{ orderData.recipient_info.name }} {{ orderData.recipient_info.phone }}
      </el-descriptions-item>
      <el-descriptions-item label="地址">{{ orderData.recipient_info.address }}</el-descriptions-item>
    </el-descriptions>

    <!-- 托寄物信息 -->
    <el-descriptions title="托寄物" :column="1" border style="margin-top: 16px">
      <el-descriptions-item label="物品名称">{{ orderData.cargo_name }}</el-descriptions-item>
    </el-descriptions>

    <!-- 顺丰返回信息 -->
    <el-descriptions title="顺丰返回" :column="2" border style="margin-top: 16px">
      <el-descriptions-item label="原寄地代码">{{ orderData.origin_code || '-' }}</el-descriptions-item>
      <el-descriptions-item label="目的地代码">{{ orderData.dest_code || '-' }}</el-descriptions-item>
      <el-descriptions-item label="筛单结果">{{ filterResultText }}</el-descriptions-item>
    </el-descriptions>

    <template #footer>
      <el-button @click="handleCancel">稍后确认</el-button>
      <el-button type="primary" :loading="confirming" @click="handleConfirm">
        立即确认订单
      </el-button>
    </template>
  </el-dialog>
</template>
```

**3. 修改 CreateOrderDialog.vue：**
- 下单成功后，发出 `order-created` 事件，传递完整的 `CreateOrderResponse`
- 关闭下单对话框
- 移除对话框内的"下单结果"区块

**4. 修改 DistributeDialog.vue：**
- 添加 `ConfirmOrderDialog` 组件
- 监听 `CreateOrderDialog` 的 `order-created` 事件
- 打开 `ConfirmOrderDialog` 进行二次确认

**验证：**
1. 下单成功后，下单对话框关闭
2. 确认对话框打开，显示完整订单信息
3. 信息包括：订单号、运单号、寄件人、收件人、托寄物、付款方式等
4. 点击"立即确认"后订单确认成功
5. 点击"稍后确认"关闭对话框，订单保持待确认状态

---

## 阶段二：日志界面优化

### 任务 2.1：添加DEBUG日志开关
**优先级：** 中
**依赖：** 无

**步骤：**
1. 在 `LogView.vue` 中添加 `showDebug` 响应式变量，默认为 `false`
2. 在过滤选项区域添加开关：「显示DEBUG日志」
3. 修改日志级别过滤器：
   - `showDebug=false` 时，隐藏 DEBUG 选项
   - `showDebug=true` 时，显示 DEBUG 选项
4. 修改日志获取逻辑：
   - `showDebug=false` 且 `selectedLevel=''`（全部）时，过滤掉 DEBUG 级别
5. 将开关状态保存到 `localStorage`，下次打开时恢复

**验证：**
- 默认不显示DEBUG日志
- 开启开关后，日志级别过滤器出现DEBUG选项
- 选择"全部"时能看到DEBUG级别日志

---

### 任务 2.2：日志表格高度自适应
**优先级：** 中
**依赖：** 无

**步骤：**
1. 移除表格固定的 `max-height="600"`
2. 使用 CSS Flexbox 布局使表格填满剩余空间
3. 修改 `.page-content` 样式：
   - 设置 `height: 100%` 和 `display: flex; flex-direction: column`
   - 设置 `overflow: hidden` 避免页面滚动条
4. 设置日志表格容器 `flex: 1` 和 `overflow: hidden`
5. 表格使用 `height="100%"` 自适应容器

**CSS 方案示意：**
```css
.page-content {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.log-table-card {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
```

**验证：**
- 窗口大小变化时，日志表格高度自动调整
- 页面只有一个滚动条（表格内部滚动）
- 不同窗口尺寸下布局正常

---

### 任务 2.3：TSPL指令DEBUG日志输出
**优先级：** 中
**依赖：** 无

**步骤：**
1. 在 `src/printer/tspl.rs` 的 `generate` 函数末尾添加 DEBUG 日志
2. 将生成的 TSPL 指令（文本部分）通过 `log::debug!` 输出
3. 由于 TSPL 指令可能包含二进制数据（BITMAP），需要特殊处理：
   - 分离文本指令和二进制数据
   - 或者只输出非 BITMAP 的指令部分
   - 或者将二进制数据以十六进制摘要形式显示

**示例输出：**
```
[DEBUG] TSPL指令生成完成，总长度: 12345 字节
[DEBUG] TSPL指令内容:
SIZE 76 mm, 130 mm
GAP 2 mm, 0 mm
DIRECTION 1
CLS
BITMAP 0,0,76,1040,0,<binary: 79040 bytes>
BARCODE 100,200,"128",60,1,0,2,2,"BG7XXX"
BOX 0,0,608,1040,2
PRINT 1
```

**验证：**
- 打印标签时，开启DEBUG日志可看到TSPL指令内容
- 日志内容可用于排查打印问题

---

## 阶段三：默认参数预配置

### 任务 3.1：设计配置文件结构
**优先级：** 中
**依赖：** 无

**步骤：**
1. 创建模板文件 `config/sf_express_default.toml.example`（提交到Git）
2. 在 `.gitignore` 中添加 `config/sf_express_default.toml`
3. 定义字段：`enabled`, `partner_id`, `checkword_sandbox`, `checkword_prod`
4. 添加注释说明这些是默认预配置参数

**模板文件示例（sf_express_default.toml.example）：**
```toml
# 顺丰速运默认预配置参数模板
# 复制此文件为 sf_express_default.toml 并填入实际参数
# 注意：sf_express_default.toml 已被 .gitignore 忽略

enabled = true
partner_id = ""
checkword_sandbox = ""
checkword_prod = ""
```

**安全说明：**
- 实际配置文件不提交到Git，避免敏感信息泄露
- 打包发布时通过CI/CD注入实际参数

---

### 任务 3.2：后端支持读取默认参数
**优先级：** 中
**依赖：** 任务 3.1

**步骤：**
1. 创建新的 Tauri 命令 `sf_get_default_api_config`
2. 从配置文件读取默认参数
3. 返回给前端（敏感信息需要脱敏处理标记）

---

### 任务 3.3：前端配置界面改造
**优先级：** 中
**依赖：** 任务 3.2

**步骤：**
1. 在 `SFExpressConfigView.vue` 中添加配置模式选择
2. 新增单选按钮组："使用默认参数" / "使用自定义参数"
3. 默认参数模式：
   - 调用 `sf_get_default_api_config` 获取参数
   - 显示为只读状态
   - 显示风险警告提示
   - 保存时直接使用默认参数
4. 自定义参数模式：
   - 显示当前输入框
   - 添加提示文字和链接到顺丰开放平台

**UI 设计：**
```
[API 凭据配置]

参数来源：
  ○ 使用默认参数
  ○ 使用自定义参数（推荐）

  [默认参数模式下的提示]
  ⚠️ 该参数不可滥用，有随时停用或更换的风险，请尽量使用自定义参数。

  [自定义参数模式下的提示]
  ⓘ 前往 顺丰开放平台 申请您自己的 API 凭据
     https://open.sf-express.com/

环境选择：...
顾客编码：...（默认模式下只读）
校验码：...（默认模式下隐藏输入）
```

---

### 任务 3.4：保存逻辑适配
**优先级：** 中
**依赖：** 任务 3.3

**步骤：**
1. 修改保存逻辑，支持两种模式
2. 默认模式：保存时标记为使用默认参数
3. 自定义模式：保存用户输入的参数
4. 后端需要区分两种模式，使用默认参数时从配置文件读取实际值

---

## 阶段四：测试验证

### 任务 4.1：功能测试
**优先级：** 低
**依赖：** 所有修改任务

**测试用例：**
1. 去配置跳转功能
2. 收件人数据重置功能
3. API配置预检查功能
4. 配置界面智能展开面板
5. 分发/退回历史信息回显
6. 下单界面无需滚动即可展示完整表单
7. 二次确认对话框展示完整订单信息
8. DEBUG日志开关功能
9. 日志表格高度自适应（不同窗口尺寸）
10. 默认/自定义参数切换功能
11. 两种模式下的保存和读取

---

## 任务依赖关系

```
任务 1.1 ─┐
任务 1.2 ─┤
任务 1.3 ─┤
任务 1.4 ─┤
任务 1.5 ─┼─> 任务 4.1
任务 1.6 ─┤
任务 1.7 ─┤
任务 1.8 ─┤
          │
任务 2.1 ─┤
任务 2.2 ─┤
任务 2.3 ─┤
          │
任务 3.1 ─┼─> 任务 3.2 ──> 任务 3.3 ──> 任务 3.4 ──> 任务 4.1
```

## 预计工作量

| 任务 | 估计 | 说明 |
|------|------|------|
| 1.1 | 小 | 事件传递和导航处理 |
| 1.2 | 小 | 表单重置逻辑调整 |
| 1.3 | 小 | 添加配置检查和提示 |
| 1.4 | 小 | 配置界面面板展开逻辑 |
| 1.5 | 中 | 移除钥匙串支持，统一加密文件 |
| 1.6 | 小 | 分发/退回历史信息回显 |
| 1.7 | 小 | 下单界面布局优化 |
| 1.8 | 中 | 二次确认独立对话框（含后端改造） |
| 2.1 | 小 | DEBUG日志开关 |
| 2.2 | 中 | 日志表格高度自适应 |
| 2.3 | 小 | TSPL指令DEBUG日志输出 |
| 3.1 | 小 | 配置文件结构设计 |
| 3.2 | 中 | 后端命令实现 |
| 3.3 | 中 | 前端界面改造 |
| 3.4 | 中 | 保存逻辑适配 |
| 4.1 | 小 | 测试验证 |
