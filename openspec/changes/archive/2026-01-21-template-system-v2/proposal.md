# 打印模板系统v2全面改造提案

## 背景

当前打印模板系统(v1)存在以下问题:

1. **死板的坐标配置**: 所有元素使用硬编码的绝对坐标(dots),缺乏自适应能力
2. **文本渲染不灵活**:
   - 使用TSPL固定字体,无法根据内容自动调整字号
   - 中文和英文混排时字号不一致,视觉效果差
   - 长文本无法自动缩放适配宽度约束
3. **PDF和打印机模式不兼容**:
   - PDF后端通过解析TSPL指令渲染,实际上是"模拟"TSPL
   - 中文渲染在PDF模式下需要加载字体文件,但TSPL打印机不支持
   - 条形码在两种模式下渲染逻辑不一致
4. **模板配置不够语义化**:
   - 元素来源(fixed/input/computed)没有明确表达
   - 缺少高度预算、间距等布局约束概念
   - 无法表达"每行独立求最大字号"等自适应布局需求

## 目标

参考 `docs/template.v2.md` 规范,进行打印模板系统的全面改造,实现:

### 核心目标

1. **模板化配置系统**: 支持 fixed/input/computed 三种元素来源
2. **自适应文本布局**: 每行独立求最大字号,自动适配宽度和高度约束
3. **统一的渲染管线**: PDF和打印机后端共享同一套布局引擎,只在最后一步分叉
4. **中文粗体支持**: 内嵌中英文粗体字体,保证视觉一致性
5. **灵活的版式控制**: 通过高度预算、间距、对齐方式等参数精确控制布局

### 技术目标

1. **分离布局与渲染**: 布局引擎计算坐标和字号,渲染后端负责输出
2. **位图优先策略**: 文本渲染为1bpp位图,保证跨打印机一致性
3. **条形码混合模式**: 支持原生BARCODE(方案A)和全位图(方案B)两种模式
4. **配置驱动**: 通过TOML配置文件完全控制布局,无需修改代码

## 方案概述

### 架构变更

```
旧架构:
Template TOML -> TSPLGenerator -> TSPL指令 -> {PDF解析渲染 | 打印机}
                                          ↑
                                     坐标写死,不灵活

新架构:
Template TOML -> TemplateEngine -> LayoutEngine -> RenderPipeline
                   (解析配置)      (计算布局)        ↓
                                                    ├─> BitmapRenderer (文本->1bpp)
                                                    ├─> BarcodeRenderer (条码)
                                                    └─> Backend
                                                        ├─> PDF (位图+条码->PNG/PDF)
                                                        └─> Printer (位图+条码->TSPL)
```

### 关键组件

1. **TemplateEngine**: 模板配置解析器
   - 读取TOML配置
   - 解析元素来源(fixed/input/computed)
   - 填充运行时数据

2. **LayoutEngine**: 布局引擎
   - 计算可用区域(扣除margin)
   - 为每个元素分配高度预算
   - 为每行文本求最大字号(二分搜索)
   - 计算整体内容块垂直居中位置
   - 输出布局结果(坐标、字号、内容)

3. **TextRenderer**: 文本渲染器
   - 使用rusttype/ab_glyph渲染文本为1bpp位图
   - 支持中英文粗体字体
   - 字体度量和宽度计算

4. **RenderPipeline**: 渲染管线
   - 协调文本渲染、条形码渲染、后端输出
   - 支持两种输出模式:
     - 方案A: 文本位图 + TSPL原生BARCODE
     - 方案B: 全部渲染为位图

5. **Backend抽象**: 后端接口
   - PDF后端: 将位图和条码组合为PNG/PDF
   - Printer后端: 将位图和条码转换为TSPL指令(BITMAP + BARCODE/PUTBMP)

### 配置文件变更

新的TOML配置格式(参考 docs/template.v2.md 第7节):

```toml
[page]
dpi = 203
width_mm = 76
height_mm = 130
margin_left_mm = 2
margin_right_mm = 2
margin_top_mm = 3
margin_bottom_mm = 3
border = true
border_thickness_mm = 0.3

[layout]
align_h = "center"
align_v = "center"
gap_mm = 2
line_gap_mm = 2.0

[fonts]
cn_bold = "SourceHanSansSC-Bold.otf"
en_bold = "Arial-Bold.ttf"
fallback_bold = "SourceHanSansSC-Bold.otf"

[[elements]]
id = "title"
type = "text"
source = "fixed"
value = "中国无线电协会业余分会-2区卡片局"
max_height_mm = 10

[[elements]]
id = "callsign"
type = "text"
source = "input"
key = "callsign"
max_height_mm = 28

[[elements]]
id = "barcode"
type = "barcode"
barcode_type = "code128"
source = "computed"
format = "{callsign}"
height_mm = 18
quiet_zone_mm = 2
human_readable = false

[[elements]]
id = "sn"
type = "text"
source = "computed"
format = "SN: {sn}"
max_height_mm = 22

[output]
mode = "text_bitmap_plus_native_barcode"  # or "full_bitmap"
threshold = 160
```

## 实施范围

### 新增模块

- `src/printer/template_engine.rs`: 模板解析引擎
- `src/printer/layout_engine.rs`: 布局计算引擎
- `src/printer/bitmap_renderer.rs`: 位图渲染器
- `src/config/template_v2.rs`: v2版本配置结构

### 改造模块

- `src/printer/text_renderer.rs`: 增强文本度量和字号计算
- `src/printer/barcode_renderer.rs`: 支持quiet zone等新特性
- `src/printer/backend/pdf.rs`: 改为接收布局结果而非TSPL指令
- `src/printer/backend/mod.rs`: 新增渲染管线抽象
- `src/printer/tspl.rs`: 改为从布局结果生成TSPL(而非模板)

## 非目标

本次改造**不包括**:

- 可视化拖拽编辑器(继续使用TOML配置)
- 多页打印/批量打印(仍然是单张)
- 二维码支持(QR code,留待后续)
- 元素条件显示(如字段缺失时隐藏元素)

## 风险与缓解

### 风险1: 字体文件体积
- **风险**: 内嵌中英文粗体字体可能增加可执行文件体积(~10-20MB)
- **缓解**: 使用字体子集化工具,只保留常用汉字和ASCII字符

### 风险2: 布局计算性能
- **风险**: 二分搜索字号可能影响渲染速度
- **缓解**:
  - 使用缓存避免重复计算
  - 设置合理的搜索范围(如8-72pt)

### 风险3: TSPL位图指令兼容性
- **风险**: 不同打印机对BITMAP/PUTBMP指令支持程度不一
- **缓解**:
  - 支持多种位图编码格式(原始/RLE压缩)
  - 提供配置选项切换编码方式

## 验收标准

改造完成后,系统应满足:

1. ✅ 使用v2配置格式,所有元素来源可配置(fixed/input/computed)
2. ✅ 每行文本字号独立计算,自动适配宽度和高度约束
3. ✅ 所有元素水平居中,整体内容块垂直居中
4. ✅ 中文和英文文本使用内嵌粗体字体渲染,视觉一致
5. ✅ PDF和打印机两种模式输出结果一致(像素级)
6. ✅ 支持两种渲染模式切换(文本位图+原生条码 / 全位图)
7. ✅ 条形码可扫描,内容可从配置派生

## 实施计划

详见 `tasks.md` 和各规范增量文档.

## 参考文档

- `docs/template.v2.md`: 完整的需求规格
- `docs/example.png`: 预期输出效果示例
- `config/templates/qsl-card-v1.toml`: 现有v1配置
