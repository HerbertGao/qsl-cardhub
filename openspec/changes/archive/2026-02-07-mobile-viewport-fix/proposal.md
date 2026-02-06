## 为什么

web_query_service 页面在部分手机浏览器（特别是 iOS Safari）上，点击输入框时页面会自动放大（zoom in），输入完成后页面不会自动恢复原始缩放比例，导致用户体验不佳。这是移动端 Web 开发中常见的 viewport 缩放问题，需要通过 viewport meta 标签和 CSS 优化来解决。

## 变更内容

- 修改 `index.html` 的 viewport meta 标签，添加 `maximum-scale=1.0` 和 `viewport-fit=cover` 以防止浏览器在聚焦输入框时自动缩放
- 确保所有输入框的 `font-size` 不小于 `16px`（iOS Safari 在输入框字体小于 16px 时会触发自动缩放）
- 为 MathCaptcha 组件的数字输入框补充移动端优化样式

## 功能 (Capabilities)

### 新增功能

- `mobile-viewport-optimization`: 移动端 viewport 缩放控制与输入框适配优化

### 修改功能

（无规范级行为变更）

## 影响

- `web_query_service/index.html` — viewport meta 标签修改
- `web_query_service/src/client/style.css` — 全局输入框样式补充
- `web_query_service/src/client/components/SearchBox.vue` — 搜索输入框样式确认
- `web_query_service/src/client/components/MathCaptcha.vue` — 验证码输入框样式优化