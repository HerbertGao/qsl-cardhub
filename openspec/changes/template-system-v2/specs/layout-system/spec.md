# 功能: 布局系统 (layout-system)

## 概述

布局系统负责计算每个元素在画布上的精确位置、文本的最大字号,以及整体内容块的垂直居中.实现"每行独立求最大字号"和"整体垂直居中"的核心布局算法.

---

## 新增需求

### 需求: 计算画布和可用区域

布局引擎必须能够根据页面配置(DPI、尺寸、边距)计算画布总尺寸和扣除边距后的可用区域,为后续布局提供基准.

#### 场景: 从mm单位转换为dots单位

- **给定** 页面配置为:
  ```toml
  [page]
  dpi = 203
  width_mm = 76
  height_mm = 130
  ```
- **当** 调用 `LayoutEngine::calculate_canvas_size(page_config)`
- **则** 应返回:
  - `canvas_width = 608 dots` (76mm * 203dpi / 25.4)
  - `canvas_height = 1024 dots` (130mm * 203dpi / 25.4)

#### 场景: 计算可用区域(扣除边距)

- **给定** 页面配置为:
  ```toml
  [page]
  dpi = 203
  width_mm = 76
  height_mm = 130
  margin_left_mm = 2
  margin_right_mm = 2
  margin_top_mm = 3
  margin_bottom_mm = 3
  ```
- **当** 调用 `LayoutEngine::calculate_available_area(page_config)`
- **则** 应返回:
  - `left = 16 dots` (2mm * 203/25.4)
  - `right = 592 dots` (608 - 16)
  - `top = 24 dots` (3mm * 203/25.4)
  - `bottom = 1000 dots` (1024 - 24)
  - `available_width = 576 dots`
  - `available_height = 976 dots`

---

### 需求: 为每行文本求最大字号

布局引擎必须为每个文本元素独立计算最大字号,使用二分搜索算法在宽度和高度约束下找到尽可能大的字号.

#### 场景: 简单文本的最大字号计算

- **给定** 一个文本元素:
  - `content = "BG7XXX"`
  - `max_height_mm = 28`
  - `available_width = 576 dots`
- **并且** 字体为Arial Bold
- **当** 调用 `LayoutEngine::calculate_max_font_size(element, available_width, font)`
- **则** 应通过二分搜索找到最大字号
- **并且** 满足以下约束:
  - `text_width("BG7XXX", font_size) <= available_width`
  - `text_height(font_size) <= max_height_mm * dpi / 25.4`
- **并且** 字号应尽可能大(在0.5pt精度内)

#### 场景: 中文长文本的字号自动缩小

- **给定** 一个文本元素:
  - `content = "中国无线电协会业余分会-2区卡片局"`
  - `max_height_mm = 10`
  - `available_width = 576 dots`
- **当** 调用 `LayoutEngine::calculate_max_font_size(element, available_width, font)`
- **则** 应返回一个较小的字号(如14pt)
- **并且** 该字号应使文本宽度不超过 `available_width`
- **并且** 文本高度不超过高度预算

#### 场景: 不同行独立计算字号

- **给定** 两个文本元素:
  1. `content = "中国无线电协会业余分会-2区卡片局"`, `max_height_mm = 10`
  2. `content = "BG7XXX"`, `max_height_mm = 28`
- **当** 分别调用 `calculate_max_font_size` 计算字号
- **则** 应返回不同的字号:
  - 标题: ~14pt (中文较长)
  - 呼号: ~72pt (英文较短)
- **说明**: 每行独立求最大字号,不要求统一

#### 场景: 二分搜索的性能要求

- **给定** 字号搜索范围为 8pt ~ 120pt
- **并且** 精度为 0.5pt
- **当** 执行二分搜索
- **则** 迭代次数应不超过 `log2((120-8)/0.5) ≈ 8` 次
- **并且** 单次字号计算耗时应小于 10ms

---

### 需求: 计算整体内容块垂直居中

布局引擎必须计算所有元素的总高度(含行间距),并计算垂直偏移量使整体内容块在可用区域内垂直居中.

#### 场景: 计算内容块总高度

- **给定** 已解析的元素列表包含:
  1. 标题: 高度 80 dots
  2. 副标题: 高度 120 dots
  3. 呼号: 高度 220 dots
  4. 条形码: 高度 140 dots
  5. SN: 高度 180 dots
  6. QTY: 高度 180 dots
- **并且** 行间距 `line_gap_mm = 2` (约16 dots)
- **当** 调用 `LayoutEngine::calculate_total_content_height(elements, line_gap)`
- **则** 应返回:
  - `total_content_height = 80 + 120 + 220 + 140 + 180 + 180 + 5*16 = 1000 dots`

#### 场景: 计算垂直居中偏移量

- **给定** 可用高度为 976 dots
- **并且** 内容块总高度为 800 dots
- **当** 调用 `LayoutEngine::calculate_vertical_offset(available_height, total_content_height)`
- **则** 应返回:
  - `y_offset = (976 - 800) / 2 = 88 dots`

#### 场景: 为每个元素分配y坐标

- **给定** 垂直偏移量为 88 dots
- **并且** 元素高度列表为 `[80, 120, 220, 140, 180, 180]`
- **并且** 行间距为 16 dots
- **当** 调用 `LayoutEngine::assign_y_positions(elements, y_offset, line_gap)`
- **则** 应返回以下y坐标:
  1. 标题: y = 24 + 88 = 112 dots (margin_top + offset)
  2. 副标题: y = 112 + 80 + 16 = 208 dots
  3. 呼号: y = 208 + 120 + 16 = 344 dots
  4. 条形码: y = 344 + 220 + 16 = 580 dots
  5. SN: y = 580 + 140 + 16 = 736 dots
  6. QTY: y = 736 + 180 + 16 = 932 dots

---

### 需求: 计算水平居中

布局引擎必须为每个元素计算水平居中的x坐标,支持文本元素和条形码元素的不同宽度计算方式.

#### 场景: 文本元素水平居中

- **给定** 一个文本元素:
  - `content = "BG7XXX"`
  - `font_size = 72pt`
  - `text_width = 400 dots` (测量结果)
- **并且** 可用宽度为 576 dots
- **并且** 左边距为 16 dots
- **当** 调用 `LayoutEngine::calculate_horizontal_center(text_width, available_width, left_margin)`
- **则** 应返回:
  - `x = 16 + (576 - 400) / 2 = 104 dots`

#### 场景: 条形码元素水平居中

- **给定** 一个条形码元素:
  - `content = "BG7XXX"`
  - `barcode_type = "code128"`
  - `quiet_zone_mm = 2` (约16 dots)
- **并且** 条形码估算宽度为 300 dots
- **并且** 总宽度为 `300 + 2*16 = 332 dots`
- **当** 调用 `LayoutEngine::calculate_horizontal_center(...)`
- **则** 应返回:
  - `x = 16 + (576 - 332) / 2 = 138 dots`

---

### 需求: 全局防溢出校验

布局引擎必须在所有元素布局完成后检测是否溢出可用高度,如果溢出则全局缩放所有元素并重新计算坐标.

#### 场景: 内容高度未溢出时不缩放

- **给定** 内容块总高度为 800 dots
- **并且** 可用高度为 976 dots
- **当** 调用 `LayoutEngine::apply_overflow_protection(elements, available_height)`
- **则** 不应进行任何缩放
- **并且** 所有元素的字号和y坐标保持不变

#### 场景: 内容高度溢出时全局缩放

- **给定** 内容块总高度为 1100 dots
- **并且** 可用高度为 976 dots
- **当** 调用 `LayoutEngine::apply_overflow_protection(elements, available_height)`
- **则** 应计算缩放比例:
  - `scale_factor = 976 / 1100 ≈ 0.887`
- **并且** 所有元素的字号应乘以 `scale_factor`:
  - 原字号72pt → 缩放后63.9pt
- **并且** 所有元素的高度应乘以 `scale_factor`
- **并且** 重新计算y坐标

#### 场景: 缩放后重新计算垂直居中

- **给定** 内容溢出,已进行缩放
- **当** 缩放完成后
- **则** 应重新计算内容块总高度
- **并且** 重新计算垂直偏移量
- **并且** 内容块应在可用区域内垂直居中

---

### 需求: 生成布局结果

布局引擎必须整合所有布局计算结果,生成包含每个元素精确坐标、字号和配置信息的LayoutResult结构.

#### 场景: 输出完整的布局结果

- **给定** 已完成所有布局计算
- **当** 调用 `LayoutEngine::layout(config, resolved_elements)`
- **则** 应返回 `LayoutResult`:
  - `canvas_width`: 画布宽度(dots)
  - `canvas_height`: 画布高度(dots)
  - `border`: 边框配置(如果启用)
  - `elements`: 已布局的元素列表

#### 场景: 已布局元素包含完整信息

- **给定** 布局完成
- **当** 检查 `LayoutResult.elements`
- **则** 每个 `LayoutedElement` 应包含:
  - `id`: 元素ID
  - `element_type`: 元素类型(Text/Barcode)
  - `content`: 填充后的内容
  - `x`: 绝对x坐标(dots)
  - `y`: 绝对y坐标(dots)
  - `font_size`: 文本字号(pt,仅文本元素)
  - `barcode_height`: 条形码高度(dots,仅条形码元素)
  - `config`: 原始配置信息

---

### 需求: 处理边框配置

布局引擎必须根据页面配置中的border参数,计算边框的位置、尺寸和线宽,包含在布局结果中.

#### 场景: 启用边框时计算边框区域

- **给定** 页面配置为:
  ```toml
  [page]
  border = true
  border_thickness_mm = 0.3
  ```
- **当** 调用 `LayoutEngine::layout(...)`
- **则** 布局结果应包含边框配置:
  - `border.enabled = true`
  - `border.thickness = 2 dots` (0.3mm * 203/25.4)
  - `border.x = 16 dots` (left_margin)
  - `border.y = 24 dots` (top_margin)
  - `border.width = 576 dots` (available_width)
  - `border.height = 976 dots` (available_height)

#### 场景: 未启用边框时跳过边框计算

- **给定** 页面配置为:
  ```toml
  [page]
  border = false
  ```
- **当** 调用 `LayoutEngine::layout(...)`
- **则** 布局结果的 `border` 字段应为 `None`

---

## 修改需求

无

---

## 移除需求

无

---

## 重命名需求

无

---

## 技术说明

### 核心数据结构

```rust
/// 布局结果
#[derive(Debug, Clone)]
pub struct LayoutResult {
    /// 画布尺寸(dots)
    pub canvas_width: u32,
    pub canvas_height: u32,
    /// 边框配置(可选)
    pub border: Option<BorderConfig>,
    /// 已布局的元素列表
    pub elements: Vec<LayoutedElement>,
}

/// 已布局的元素
#[derive(Debug, Clone)]
pub struct LayoutedElement {
    pub id: String,
    pub element_type: ElementType,
    pub content: String,
    /// 绝对坐标(dots)
    pub x: u32,
    pub y: u32,
    /// 文本字号(pt) - 仅文本元素
    pub font_size: Option<f32>,
    /// 元素高度(dots)
    pub height: u32,
    /// 条形码配置 - 仅条形码元素
    pub barcode_config: Option<BarcodeConfig>,
}

/// 边框配置
#[derive(Debug, Clone)]
pub struct BorderConfig {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub thickness: u32,
}
```

### LayoutEngine API

```rust
pub struct LayoutEngine {
    text_renderer: TextRenderer,
}

impl LayoutEngine {
    /// 执行完整布局计算
    pub fn layout(
        &self,
        config: &TemplateV2Config,
        resolved_elements: Vec<ResolvedElement>,
    ) -> Result<LayoutResult>;

    /// 计算画布尺寸
    fn calculate_canvas_size(&self, page_config: &PageConfig) -> (u32, u32);

    /// 计算可用区域
    fn calculate_available_area(&self, page_config: &PageConfig) -> (u32, u32, u32, u32);

    /// 为单个文本元素求最大字号(二分搜索)
    fn calculate_max_font_size(
        &self,
        content: &str,
        max_height_dots: u32,
        available_width_dots: u32,
        font: &Font,
    ) -> f32;

    /// 计算内容块总高度
    fn calculate_total_content_height(
        &self,
        elements: &[LayoutedElement],
        line_gap_dots: u32,
    ) -> u32;

    /// 计算水平居中x坐标
    fn calculate_horizontal_center(
        &self,
        element_width: u32,
        available_width: u32,
        left_margin: u32,
    ) -> u32;

    /// 全局防溢出校验和缩放
    fn apply_overflow_protection(
        &self,
        elements: &mut Vec<LayoutedElement>,
        available_height: u32,
        line_gap_dots: u32,
    ) -> Result<()>;
}
```

### 二分搜索算法实现

```rust
fn calculate_max_font_size(
    &self,
    content: &str,
    max_height_dots: u32,
    available_width_dots: u32,
    font: &Font,
) -> f32 {
    let mut left = 8.0;   // 最小字号
    let mut right = 120.0; // 最大字号
    let mut result = left;

    while right - left > 0.5 {
        let mid = (left + right) / 2.0;

        // 测量文本尺寸
        let (width, height) = self.text_renderer.measure_text(content, mid, font);

        if width <= available_width_dots && height <= max_height_dots {
            result = mid;
            left = mid;
        } else {
            right = mid;
        }
    }

    result
}
```

### 坐标系说明

- **原点**: 画布左上角为(0, 0)
- **x轴**: 向右为正
- **y轴**: 向下为正
- **单位**: dots(像素)
- **DPI换算**: `dots = mm * dpi / 25.4`

---

## 依赖关系

- **前置功能**:
  - `template-engine` (使用已解析的元素列表)
  - `text-rendering` (依赖文本度量功能)
- **后续功能**:
  - `rendering-pipeline` (使用布局结果进行渲染)

---

## 测试要求

### 单元测试

- 测试画布尺寸和可用区域计算
- 测试二分搜索字号算法(不同文本长度)
- 测试垂直居中和y坐标分配
- 测试水平居中和x坐标计算
- 测试防溢出缩放逻辑

### 集成测试

- 测试完整布局流程(从配置到布局结果)
- 测试极端情况(超长文本、超高内容块)

### 性能测试

- 布局计算总耗时应 < 50ms
- 单次字号计算应 < 10ms

### 测试数据

- 提供不同文本长度的测试用例
- 提供溢出和不溢出的布局场景

---

**创建日期**: 2026-01-20
