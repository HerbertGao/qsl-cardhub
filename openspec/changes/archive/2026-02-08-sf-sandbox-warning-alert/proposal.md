## 为什么

当前顺丰速运下单界面不会提示用户当前使用的是沙箱环境还是生产环境。如果用户配置为沙箱环境并下单，订单不会被真实派发，但用户可能并未意识到这一点，导致误操作和困惑。需要在下单界面增加明显的沙箱环境提醒，帮助用户识别当前环境状态。

## 变更内容

- 在顺丰速运下单对话框（CreateOrderDialog）中，当检测到当前 API 配置为沙箱环境时，在页面顶部显示一个 `warning` 类型的 `el-alert` 提示
- 提示内容说明当前为沙箱环境，订单将不会被真实派发
- 提示中包含一个跳转链接，点击后可导航到顺丰速运配置界面切换环境
- 增强导航系统（navigationStore）支持携带参数，使下单界面跳转到配置页面时能精确指定目标 Tab（如沙箱警告跳转到 API 配置 Tab，寄件人未配置跳转到寄件人 Tab），而非依赖配置页面的自动判断逻辑

## 功能 (Capabilities)

### 新增功能

（无新增功能）

### 修改功能

- `sf-express-integration`: 在下单前配置检查中新增沙箱环境提示规则——当 API 配置的 environment 为 sandbox 时，在下单界面显示 warning alert 提醒用户；增强下单配置跳转，支持通过导航参数精确指定配置页面的目标 Tab

## 影响

- **前端组件**: `web/src/components/sf-express/CreateOrderDialog.vue` — 增加沙箱环境判断和 warning alert 展示，`go-config` 事件增加 tab 参数
- **前端组件**: `web/src/components/cards/DistributeDialog.vue` — 转发 tab 参数到导航
- **导航系统**: `web/src/stores/navigationStore.ts` — `navigateTo` 增加 params 支持，新增 `consumeNavigationParams`
- **前端视图**: `web/src/views/SFExpressConfigView.vue` — 支持通过导航参数指定初始 Tab
- **后端 API**: 无变更，`sf_load_config` 命令已返回 `environment` 字段，可直接复用