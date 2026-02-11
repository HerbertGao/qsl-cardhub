## 为什么

当前系统中，普通标签与地址标签打印正常，但顺丰面单打印会在部分机型（如汉印 HPRT N31D）出现跳纸、半张和空白张问题。该问题影响面单打印可用性，且具有明确的路径差异与可复现性，应优先修复。

## 变更内容

- 统一顺丰面单打印链路与普通标签链路的关键 TSPL 头参数策略，消除同一打印机下不同链路行为不一致问题。
- 在打印机配置界面与 `printer.toml` 中新增全局 TSPL 参数配置项：`GAP` 与 `DIRECTION`。
- 所有打印链路（QSL 标签、地址标签、顺丰面单）统一接入上述全局参数，避免链路间行为漂移。
- 默认参数调整为兼容基线：`GAP 2mm,0mm`、`DIRECTION 1,0`，并允许用户在配置界面修改。
- 增加顺丰打印链路的关键调试日志，便于定位机型兼容性问题。
- 完善顺丰面单打印回归验证，确保“普通标签/地址正常”的既有行为不被回归影响。

## 功能 (Capabilities)

### 新增功能
- `global-tspl-print-config`: 提供全局 TSPL 参数（GAP、DIRECTION）配置能力，并应用到所有标签打印链路。

### 修改功能
- `sf-express-integration`: 调整顺丰面单 PDF 转 TSPL 的规范要求，使其与全局 TSPL 配置一致。
- `printer-backend`: 调整图像打印与 TSPL 生成调用方式，支持从统一配置注入 GAP 与 DIRECTION。
- `configuration-management`: 扩展单配置模式的数据结构与持久化内容，支持保存全局 TSPL 参数。

## 影响

- 前端打印机配置界面：`web/src/views/ConfigView.vue`
- 前端类型定义：`web/src/types/models.ts`
- 配置模型与持久化：`src/config/models.rs`、`src/config/profile_manager.rs`
- 后端命令：`src/commands/sf_express.rs`
- 后端打印命令：`src/commands/printer.rs`
- 打印与 TSPL：`src/printer/tspl.rs`、`src/printer/backend/*`
- 顺丰 PDF 渲染：`src/sf_express/pdf_renderer.rs`
- 相关测试与日志输出
