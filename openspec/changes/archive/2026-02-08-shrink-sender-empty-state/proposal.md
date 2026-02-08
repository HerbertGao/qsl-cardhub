## 为什么

在顺丰速运下单对话框中，当寄件人未配置时，`el-empty` 组件的默认尺寸远大于寄件人已配置时的信息展示区域，导致视觉上不协调。需要缩小未配置状态的占位大小，使其与已配置状态保持视觉平衡。

## 变更内容

- 缩小 `CreateOrderDialog.vue` 中寄件人未配置时 `el-empty` 组件的尺寸，使其与已配置时的紧凑展示保持一致

## 功能 (Capabilities)

### 新增功能

（无）

### 修改功能

- `sf-express-integration`: 调整下单界面寄件人未配置状态的 UI 尺寸

## 影响

- **前端组件**: `web/src/components/sf-express/CreateOrderDialog.vue` — 调整 `el-empty` 组件样式
