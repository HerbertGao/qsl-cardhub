## 上下文

web_query_service 是一个基于 Vue 3 + Vite 的移动端优先 Web 应用，部署在 Cloudflare Workers 上。当前页面已有一定的移动端适配（如 16px 基础字体、touch-action: manipulation、-webkit-appearance: none 等），但在部分手机浏览器（特别是 iOS Safari）上，点击输入框时页面仍会自动放大，且输入完成后不会恢复原始缩放。

当前 viewport meta 标签配置为：
```html
<meta name="viewport" content="width=device-width, initial-scale=1.0" />
```

现有输入框已设置 `font-size: 1rem`（基于 body 的 16px），但 MathCaptcha 组件的输入框可能缺少明确的字体大小控制。

## 目标 / 非目标

**目标：**
- 防止手机浏览器在聚焦输入框时自动缩放页面
- 确保输入完成后页面保持正确的缩放比例
- 支持 iOS Safari、Android Chrome 等主流移动浏览器
- 保持良好的无障碍访问性

**非目标：**
- 完全禁止用户手动缩放（保留双指缩放能力以满足无障碍需求）
- 重新设计页面布局或 UI 组件
- 添加桌面端适配或响应式断点

## 决策

### 1. viewport meta 标签策略

**选择：** 添加 `maximum-scale=1.0` 和 `viewport-fit=cover`

```html
<meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, viewport-fit=cover" />
```

**替代方案考虑：**
- `user-scalable=no`：会完全禁止用户缩放，影响无障碍访问性，iOS Safari 10.3+ 已忽略此属性。不采用。
- 仅靠 CSS `font-size: 16px`：理论上可以防止 iOS 自动缩放，但不够可靠，某些浏览器仍会缩放。不够全面。
- `maximum-scale=1.0`：在大多数浏览器上能有效防止自动缩放，同时 iOS Safari 14+ 仍允许用户手动双指缩放。兼顾了功能性和无障碍性。

**理由：** `maximum-scale=1.0` 是目前最广泛被推荐的解决方案，能在防止自动缩放的同时不完全阻断用户的手动缩放能力。`viewport-fit=cover` 确保页面在有刘海/圆角的设备上正确填充。

### 2. 输入框字体大小保障

**选择：** 确保所有 input/textarea/select 元素的 `font-size` 不小于 `16px`

**理由：** iOS Safari 会在输入框字体小于 16px 时触发自动缩放。这是导致问题的根本原因之一。当前全局样式中 body 已设为 16px，但需要确保没有组件级样式覆盖导致输入框字体变小。

### 3. 触摸交互优化

**选择：** 为所有表单元素统一添加 `touch-action: manipulation`

**理由：** 此属性可以禁止浏览器的双击缩放行为，减少输入框聚焦时的意外缩放。当前已在 `.btn` 上应用，需要扩展到所有表单元素。

## 风险 / 权衡

- **[无障碍影响]** `maximum-scale=1.0` 在某些旧浏览器上可能限制用户缩放 → iOS Safari 14+ 和现代 Android Chrome 已不再严格限制用户手动缩放，仅阻止自动缩放行为。对于目标用户群（业余无线电爱好者查询呼号）影响可控。
- **[浏览器兼容性]** 不同浏览器对 viewport meta 的解读有差异 → 采用多层防御策略（viewport meta + CSS font-size + touch-action），确保在各浏览器上都有效果。