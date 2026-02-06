## 1. Viewport Meta 标签修改

- [x] 1.1 修改 `web_query_service/index.html` 的 viewport meta 标签，添加 `maximum-scale=1.0` 和 `viewport-fit=cover`

## 2. 全局 CSS 表单元素样式优化

- [x] 2.1 在 `web_query_service/src/client/style.css` 中为所有 input、textarea、select 元素添加 `font-size: 16px` 保底样式
- [x] 2.2 在 `web_query_service/src/client/style.css` 中为所有表单交互元素添加 `touch-action: manipulation`

## 3. MathCaptcha 组件输入框优化

- [x] 3.1 检查并确保 `web_query_service/src/client/components/MathCaptcha.vue` 中验证码输入框的 `font-size` 不小于 16px
