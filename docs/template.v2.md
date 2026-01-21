# 76×130 标签打印模板系统需求规格

## 1. 目标与范围

### 1.1 目标
实现一个**可模板化配置**的标签打印系统，标签尺寸固定为 **76mm × 130mm**，版式如示例图所示：  
包含以下元素（从上到下）：

1) 标题（Title）
2) 副标题（Subtitle）
3) 呼号（Callsign）
4) 条形码（Barcode）
5) SN（序列号）
6) QTY（数量）

所有元素**水平居中**；整体内容块在标签画布上**视觉居中**（上下留白一致感），外框可选。

### 1.2 非目标（暂不做）
- 不做可视化拖拽编辑器（先用 TOML 配置驱动）
- 不做多页打印/分页排版（每次打印 1 张标签）
- 不要求打印机原生支持 PDF（优先 TSPL/位图路径）

---

## 2. 核心约束

### 2.1 字体与文本规则
- 系统已内嵌**中英文各自的粗体字体**（Bold）
- 每一行文本必须使用粗体
- **每一行独立求最大字号**：  
  在不超出该行的宽度/高度预算约束下，该行字号尽可能大；不同的行允许不同字号（例如中文长则更小）
- 文本允许包含中文、英文、数字、符号（如 `SN: 123`、`QTY: 50`）

### 2.2 对齐规则
- 每个元素内容**水平居中**（x 方向）
- 所有内容（文本块 + 条码块）作为整体在画布可用区域内**垂直居中**（y 方向）

### 2.3 条形码规则
- 条形码内容可配置：固定值 / 从外部传入 / 从某字段派生（例如呼号）
- 初版默认条码类型建议 **Code128**（通用、密度高）；允许在配置中切换

### 2.4 模板化规则（关键）
每一项文字（标题、副标题、呼号、SN、QTY）以及条形码内容，都必须支持在配置文件中声明其来源：
- `fixed`：固定文字（写死在配置里）
- `input`：运行时从外部数据传入（如 API / 表单 / CSV）
- `computed`：由其他字段计算/拼接生成（例如 `SN: {sn}`）

---

## 3. 输入与输出

### 3.1 输入 = 模板配置（TOML）+ 数据载荷（JSON/Dict）
- TOML 描述：画布、边距、元素顺序、每个元素的高度预算、字体、条码参数、外框等
- 运行时数据：包含模板里引用的字段（如 `callsign`, `sn`, `qty`）

### 3.2 输出（推荐路径）
- **渲染端输出为 1bpp 黑白位图**（保证中文粗体一致性与跨机型一致）
- 下发给打印机的方式：
  - 方案 A（推荐）：文本位图 + TSPL 原生 BARCODE
  - 方案 B（最一致）：整张标签渲染为位图 + TSPL BITMAP 一把梭

> 初版建议默认走 **方案 A**：文字稳定，条码更锐利更易扫；遇到机型差异可切换到方案 B。

---

## 4. 版式模型与布局规则

### 4.1 画布与可用区域
- 画布固定：`76mm × 130mm`
- DPI 可配置（常见 203/300）
- 可用区域 = 画布扣除 `margin_*`
- 内容块 = 文本块 + （可选间距） + 条码块
- 内容块在可用区域内垂直居中

### 4.2 元素顺序（固定）
`Title → Subtitle → Callsign → Barcode → SN → QTY`

### 4.3 高度预算（可配置）
每个元素（含条码）必须有“高度预算”，用于：
- 限制该元素的最大字号（文本）
- 限制条码高度
- 允许未来做“某元素隐藏/缩放”的自适应

高度预算表达方式（TOML 可二选一）：
- `height_mm`：直观
- `height_ratio`：按比例分配剩余高度（更自适应）

---

## 5. 文本字号求解算法（每行独立最大字号）

对每一行文本 i：
- 给定：可用宽度 `Wc`、该行可用高度 `Hi`、字体（粗体）
- 求最大字号 `Si`，使得：
  - `text_width(text_i, Si) <= Wc`
  - `text_height(Si) <= Hi`

建议实现方式：
- 二分搜索或从大到小尝试
- 以像素（dots）为单位测量文字宽高（font metrics）
- 每行水平居中：`x = left_margin + (Wc - measured_width)/2`

### 5.1 全局兜底：防溢出缩放（建议）
即使每行各自不超高，仍可能出现整体高度溢出（行距、配置误差等）。  
因此必须做一次整体校验：
- 若 `sum(line_heights) + gaps + barcode_height > content_area_height`  
  则整体缩放所有文本字号：`Si = floor(Si * k)`，k 为比例系数，再复测一次。

---

## 6. 条形码布局规则

- 条码区域有固定高度预算 `Hb`
- 条码水平居中
- 条码两侧保留 quiet zone（可配置，默认 2mm）
- 条码内容来源可配置（fixed/input/computed）

---

## 7. 配置文件（TOML）规范（初版）

### 7.1 示例 TOML（可直接作为初版模板）
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
gap_mm = 2                # 文本块与条码块之间的额外间距
line_gap_mm = 2.0         # 文本行之间间距（同一套用于 title/subtitle/sn/qty 的垂直间距）

[fonts]
cn_bold = "CN_BOLD.ttf"
en_bold = "EN_BOLD.ttf"
fallback_bold = "CN_BOLD.ttf"

# 元素定义：顺序即渲染顺序
[[elements]]
id = "title"
type = "text"
source = "fixed"
value = "中国无线电协会业余分会-2区卡片局"
max_height_mm = 10

[[elements]]
id = "subtitle"
type = "text"
source = "fixed"
value = "模板测试"
max_height_mm = 16

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

[[elements]]
id = "qty"
type = "text"
source = "computed"
format = "QTY: {qty}"
max_height_mm = 22

[output]
mode = "text_bitmap_plus_native_barcode"  # or "full_bitmap"
threshold = 160                           # 灰度转黑白阈值（热敏更干净）
```

### 7.2 字段说明（简要）
- `type`：`text` | `barcode`
- `source`：
  - `fixed`：使用 `value`
  - `input`：使用 `key` 从数据载荷取值
  - `computed`：使用 `format`，支持 `{field}` 插值
- `max_height_mm`：文本元素的最大高度预算（用于求最大字号）
- `barcode`：
  - `barcode_type`：如 code128
  - `height_mm`：条码高度预算
  - `quiet_zone_mm`：左右留白
  - `human_readable`：是否显示人可读文字

---

## 8. 运行时数据载荷格式（示例）

```json
{
  "callsign": "BG7XXX",
  "sn": "123",
  "qty": "50"
}
```

---

## 9. 打印与下发（TSPL）要求

### 9.1 推荐实现（默认）
- 文本：渲染为 1bpp 位图（保证中文粗体）
- 条码：TSPL 原生 `BARCODE`（更利于扫码）

### 9.2 兜底实现（兼容/一致性最强）
- 全部内容渲染为整张 1bpp 位图
- TSPL 只发 `BITMAP 0,0,...`

### 9.3 TSPL 基本控制项（实现侧要支持）
- `SIZE 76 mm, 130 mm`
- `GAP ...`（按耗材设置，可配置）
- `DIRECTION`（默认 1）
- `CLS`
- `BITMAP/PUTBMP`（位图输出）
- `BARCODE`（原生条码输出，若启用）
- `PRINT 1,1`

---

## 10. 验收标准（能一眼判定对不对）

1) 标签尺寸固定 76×130mm，边距符合配置
2) 标题/副标题/呼号/条码/SN/QTY 的垂直顺序与示例一致
3) 所有元素水平居中；整体内容块视觉垂直居中
4) 每行文本为粗体；**每行字号尽可能大且不溢出**（呼号通常最大，标题较小但仍尽可能大）
5) 文本内容可由 fixed/input/computed 三种来源控制，修改 TOML 不改代码即可换内容来源
6) 条形码可扫；条码内容可配置为呼号或其他字段
7) 输出模式可切换：`text_bitmap_plus_native_barcode` 与 `full_bitmap`

---

## 11. 可扩展点（为后续留钩子）
- 条码类型扩展：Code39、EAN、QR（二维码）等
- 元素可选显示：当某字段缺失时自动隐藏并重新分配高度
- 每个元素独立 `line_gap_before/after`
- “呼号条码 + 二维码”组合块
- 多模板版本管理（模板 ID + 版本号）
