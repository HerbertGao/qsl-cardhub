## 1. Markdown 渲染工具函数

- [x] 1.1 创建 `web/src/utils/markdown.ts` 工具模块，封装 markdown-it 初始化（禁用 html 选项）和 DOMPurify 清洗逻辑，导出 `renderMarkdown(source: string): string` 函数
- [x] 1.2 在 `renderMarkdown` 中配置 DOMPurify 白名单，仅允许标题、列表、段落、链接、代码等安全标签和属性

## 2. AboutView 更新说明渲染改造

- [x] 2.1 在 `AboutView.vue` 中引入 `renderMarkdown`，将更新说明区域（第 73-78 行）从 `{{ notes }}` 纯文本插值改为 `v-html="renderedNotes"` 渲染
- [x] 2.2 添加兜底逻辑：当 notes 为"无更新说明"时保持纯文本展示，仅对有实际内容的 notes 执行 Markdown 渲染
- [x] 2.3 为渲染容器添加 `.release-notes` scoped 样式，覆盖 h1-h4、ul/ol、p、a、code、pre > code 的基础排版

## 3. 链接安全处理

- [x] 3.1 确保渲染结果中的 `<a>` 标签点击时通过 Tauri shell `open` 在外部浏览器中打开，而非在 WebView 内导航（可在容器上添加 click 事件委托拦截 `<a>` 点击）

## 4. 验证

- [x] 4.1 使用包含标题、列表、代码块、链接的典型 Release Notes Markdown 样例验证渲染效果
- [x] 4.2 验证包含 `<script>`、`<iframe>`、`onerror` 的恶意输入被正确清洗
- [x] 4.3 验证"无更新说明"兜底场景正常显示纯文本
