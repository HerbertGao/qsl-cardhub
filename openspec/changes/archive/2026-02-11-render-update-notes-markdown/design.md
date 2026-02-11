## 上下文

当前 `AboutView.vue` 第 73-78 行使用 `{{ updateState.updateInfo.notes }}` 配合 `white-space: pre-wrap` 以纯文本方式渲染 GitHub Release body。该 body 通常为 Markdown 格式，用户会看到 `#`、`-`、`` ``` `` 等源码标记，阅读体验差。

项目已安装 `markdown-it`（v14.1.1）和 `dompurify`（v3.3.1）两个依赖，可直接复用。

## 目标 / 非目标

**目标：**

- 将更新说明从纯文本渲染改为安全的 Markdown 富文本渲染
- 使用 DOMPurify 清洗渲染结果，防止 XSS 注入
- 提供简洁的排版样式（标题、列表、段落、链接、代码块）
- 保持现有兜底逻辑（"无更新说明"仍以纯文本展示）

**非目标：**

- 不引入复杂主题系统或可配置样式
- 不修改 `updateCheck.ts` 中的数据获取逻辑
- 不支持图片、视频等富媒体内容渲染
- 不影响 Tauri Updater 的更新检查/下载/安装流程

## 决策

### 1. Markdown 渲染库：使用已有的 markdown-it

**选择**：`markdown-it`（已安装）

**替代方案**：
- `marked`：同样流行，但项目已安装 markdown-it，无需额外引入
- `v-md-editor`：过于重量级，本场景只需只读渲染
- 手写简单正则替换：不可靠，难以覆盖完整 Markdown 语法

**理由**：零额外依赖成本，markdown-it 功能完备、可扩展。

### 2. XSS 防护：DOMPurify 清洗

**选择**：先用 markdown-it 渲染为 HTML，再用 DOMPurify 清洗后通过 `v-html` 输出。

**替代方案**：
- markdown-it 禁用 `html` 选项即可避免原始 HTML 注入：虽可阻止 `<script>`，但无法防御 markdown-it 插件或未来配置变更带来的风险，DOMPurify 是更可靠的纵深防御
- 使用 CSP（Content-Security-Policy）：Tauri WebView 已有一定保护，但 CSP 无法替代输出清洗

**理由**：DOMPurify 已安装，额外调用成本极低，且是业界标准的 XSS 防护方案。

### 3. 渲染位置：仅 AboutView.vue 更新说明区域

将 markdown-it + DOMPurify 的渲染逻辑封装为一个 composable（`useMarkdown`）或工具函数，在 `AboutView.vue` 中调用。更新说明区域将 `{{ notes }}` 替换为 `v-html="renderedNotes"`。

**理由**：集中复用渲染逻辑，若未来其他位置需要渲染 Markdown 可直接调用。

### 4. 样式方案：scoped 局部样式

在 `AboutView.vue` 的 `<style scoped>` 中为 Markdown 渲染容器添加 `.release-notes` 作用域样式，覆盖标题（h1-h4）、列表（ul/ol）、段落（p）、链接（a）、行内代码（code）和代码块（pre > code）的基础排版。

**替代方案**：
- 全局 Markdown 样式文件：过早抽象，目前仅一处使用
- 第三方 Markdown 主题包（如 github-markdown-css）：引入额外依赖，样式过多

**理由**：最小侵入，样式可控且不影响全局。

## 风险 / 权衡

- **风险**：GitHub Release body 可能包含非标准 Markdown 或 HTML 片段 → DOMPurify 会清洗掉危险标签，最坏情况下部分格式丢失但不会引发安全问题
- **风险**：markdown-it 默认配置可能渲染出意外元素（如 `<img>`） → DOMPurify 默认白名单会过滤，可进一步收紧 `ALLOWED_TAGS` 配置
- **权衡**：使用 `v-html` 代替模板插值引入了理论上的 XSS 攻击面 → 通过 DOMPurify 清洗将风险降至可接受水平
