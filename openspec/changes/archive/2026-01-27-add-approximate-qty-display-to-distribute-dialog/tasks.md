# 任务列表

## 1. 公共工具函数

- [x] 1.1 创建数量格式化函数 `formatQty(qty: number): string`，根据当前模式返回精确值或大致值（`≤10`、`≤50`、`>50`）
- [x] 1.2 创建 localStorage 读写工具（key: `qty_display_mode`，值: `exact` | `approximate`），提供响应式 composable 供各组件共享

## 2. 各页面数量显示适配

- [x] 2.1 `CardList.vue` — 列表表格"数量"列，使用格式化函数替换直接显示 `qty`
- [x] 2.2 `CardDetailDialog.vue` — 详情弹窗的数量字段
- [x] 2.3 `DistributeDialog.vue` — 分发对话框基本信息区的数量字段
- [x] 2.4 `ReturnDialog.vue` — 退回对话框基本信息区的数量字段
- [x] 2.5 `SFOrderListView.vue` — 顺丰订单详情的数量字段
- [x] 2.6 `CardManagementView.vue` — 录入成功提示文案中的数量

## 3. 打印标签内容同步

- [x] 3.1 `CardManagementView.vue` — 录入后打印时，根据模式传入精确或大致的 qty 字符串
- [x] 3.2 `CardList.vue` — 列表补打印时，同上

## 4. 模式切换入口

- [x] 4.1 `CardInputDialog.vue` — 在数量输入下方增加模式切换开关（精确/大致），带问号说明提示
- [x] 4.2 大致模式下将 `el-input-number` 替换为按钮组（≤10、≤50、>50），存储值为 10/50/100
- [x] 4.3 切换模式时自动调整数量值，连续录入重置时也根据模式设置默认值

## 5. 验证

- [x] 5.1 验证所有页面精确模式显示原始数量
- [x] 5.2 验证所有页面大致模式：qty=10 显示 ≤10、qty=50 显示 ≤50、qty=100 显示 >50
- [x] 5.3 验证打印标签在大致模式下显示 `QTY: ≤50` 等格式
- [x] 5.4 验证切换模式后全局生效，重新打开页面后选择被保留
