# 提案：修复条形码打印机兼容性问题

## 变更 ID
`fix-barcode-printer-compatibility`

## 动机

当前系统默认使用 `text_bitmap_plus_native_barcode`（混合模式）打印标签，其中条形码通过 TSPL 原生 `BARCODE` 指令发送给打印机。但部分热敏打印机（如汉印 HPRT N31D）不支持或无法正确解析 TSPL `BARCODE` 指令，导致条形码无法打印。

系统已有 `full_bitmap`（全位图模式）作为替代方案，将所有元素（包括条形码）渲染为位图后发送给打印机，但该模式目前需要用户手动修改模板 TOML 文件中的 `output.mode` 字段，对普通用户不友好。

## 问题分析

1. **根因**：部分打印机固件不完整支持 TSPL 协议中的 `BARCODE` 指令
2. **现状**：系统已实现全位图模式（`full_bitmap`），条形码通过 `barcode_renderer.rs` 渲染为位图后合成到画布
3. **差距**：缺少用户友好的切换方式，用户需手动编辑 TOML 文件

## 解决方案

### 方案选择

项目已有两种渲染模式：
- **混合模式** (`text_bitmap_plus_native_barcode`)：文本渲染为位图 + 条形码使用 TSPL 原生指令
- **全位图模式** (`full_bitmap`)：所有元素（文本 + 条形码）统一渲染为一张完整位图

针对不支持 TSPL `BARCODE` 指令的打印机，**将默认渲染模式从混合模式切换为全位图模式**，并在模板设置界面提供渲染模式选择功能，让用户可以根据打印机兼容性自行切换。

### 具体变更

1. **修改默认渲染模式**：将硬编码的默认模板和内置模板的 `output.mode` 从 `text_bitmap_plus_native_barcode` 改为 `full_bitmap`
2. **模板设置界面增加渲染模式选项**：在模板设置页面的"输出配置"分组中，将 `output.mode` 字段改为可编辑的下拉选择框，提供两个选项并附带说明
3. **模板文件迁移**：已有用户的模板文件保持不变（不做自动迁移），仅影响新安装或重置后的默认模板

## 影响范围

- `src/main.rs`：硬编码的默认呼号模板
- `config/templates/callsign.toml`：呼号标签模板文件
- `config/templates/address.toml`：地址标签模板文件
- `src/config/template.rs`：默认 OutputConfig 构造
- `web/src/views/TemplateView.vue`：模板设置界面
- 相关单元测试

## 不在范围内

- 不修改渲染管道（`render_pipeline.rs`）的逻辑，已有的全位图模式工作正常
- 不修改 TSPL 生成器（`tspl.rs`）的逻辑
- 不引入自动检测打印机能力的机制（复杂度过高，收益有限）
- 不做已有用户模板的自动迁移（避免覆盖用户自定义配置）
