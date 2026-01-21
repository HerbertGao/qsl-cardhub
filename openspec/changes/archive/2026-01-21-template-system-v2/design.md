# 打印模板系统v2架构设计

## 1. 架构概览

### 1.1 整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        用户/API调用                               │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                      TemplateEngine                              │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ 1. 加载TOML配置 (TemplateV2Config)                         │ │
│  │ 2. 解析元素来源 (fixed/input/computed)                     │ │
│  │ 3. 填充运行时数据 (ResolvedElement)                        │ │
│  └────────────────────────────────────────────────────────────┘ │
└───────────────────────────┬─────────────────────────────────────┘
                            │ ResolvedElements
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                       LayoutEngine                               │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ 1. 计算可用区域 (扣除margin)                                │ │
│  │ 2. 分配高度预算 (基于max_height_mm)                        │ │
│  │ 3. 为每行文本求最大字号 (二分搜索)                         │ │
│  │ 4. 计算整体内容块垂直居中位置                              │ │
│  │ 5. 生成布局结果 (LayoutResult)                             │ │
│  └────────────────────────────────────────────────────────────┘ │
└───────────────────────────┬─────────────────────────────────────┘
                            │ LayoutResult
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                      RenderPipeline                              │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ 根据output.mode决定渲染策略:                                │ │
│  │                                                              │ │
│  │ 方案A: text_bitmap_plus_native_barcode                      │ │
│  │   ├─> TextRenderer: 文本->1bpp位图                          │ │
│  │   ├─> BarcodeRenderer: 保留条码信息(后端原生渲染)          │ │
│  │   └─> Backend: 位图+原生条码->输出                          │ │
│  │                                                              │ │
│  │ 方案B: full_bitmap                                           │ │
│  │   ├─> TextRenderer: 文本->1bpp位图                          │ │
│  │   ├─> BarcodeRenderer: 条码->1bpp位图                       │ │
│  │   └─> Backend: 全位图->输出                                 │ │
│  └────────────────────────────────────────────────────────────┘ │
└───────────────────────────┬─────────────────────────────────────┘
                            │ RenderResult
                            ▼
                 ┌──────────┴──────────┐
                 │                     │
                 ▼                     ▼
        ┌────────────────┐    ┌────────────────┐
        │  PDF Backend   │    │ Printer Backend│
        │                │    │                │
        │ 位图->PNG/PDF   │    │ 位图->TSPL     │
        └────────────────┘    └────────────────┘
```

### 1.2 数据流

```
TOML配置 + 运行时数据
    ↓
ResolvedElements (已填充内容)
    ↓
LayoutResult (坐标、字号、位图)
    ↓
RenderResult (最终输出格式)
    ↓
{PNG/PDF | TSPL指令}
```

## 2. 核心组件设计

### 2.1 TemplateEngine (模板引擎)

**职责**: 解析配置文件,填充运行时数据

**关键数据结构**:

```rust
/// 元素来源类型
pub enum ElementSource {
    /// 固定值
    Fixed { value: String },
    /// 运行时输入
    Input { key: String },
    /// 从其他字段计算
    Computed { format: String },
}

/// 已解析的元素(内容已填充)
pub struct ResolvedElement {
    pub id: String,
    pub element_type: ElementType,  // Text | Barcode
    pub content: String,             // 填充后的实际内容
    pub config: ElementConfig,       // 元素配置(高度、字体等)
}
```

**核心算法**:

1. 加载TOML -> `TemplateV2Config`
2. 遍历 `[[elements]]`:
   - 如果 `source = "fixed"`, 使用 `value`
   - 如果 `source = "input"`, 从运行时数据中取 `data[key]`
   - 如果 `source = "computed"`, 使用简单模板引擎替换 `{field}`
3. 输出 `Vec<ResolvedElement>`

**模板引擎实现**:

```rust
impl TemplateEngine {
    pub fn resolve_format(format: &str, data: &HashMap<String, String>) -> String {
        let mut result = format.to_string();
        for (key, value) in data {
            result = result.replace(&format!("{{{}}}", key), value);
        }
        result
    }
}
```

### 2.2 LayoutEngine (布局引擎)

**职责**: 计算每个元素的坐标、字号,生成布局结果

**关键数据结构**:

```rust
/// 布局结果
pub struct LayoutResult {
    /// 画布尺寸(dots)
    pub canvas_width: u32,
    pub canvas_height: u32,
    /// 边框配置
    pub border: Option<BorderConfig>,
    /// 已布局的元素列表
    pub elements: Vec<LayoutedElement>,
}

/// 已布局的元素
pub struct LayoutedElement {
    pub id: String,
    pub element_type: ElementType,
    pub content: String,
    /// 元素的绝对位置(dots)
    pub x: u32,
    pub y: u32,
    /// 文本字号(pt) - 仅文本元素
    pub font_size: Option<f32>,
    /// 条形码高度(dots) - 仅条形码元素
    pub barcode_height: Option<u32>,
    /// 其他配置
    pub config: ElementConfig,
}
```

**核心算法**:

1. **计算可用区域**:
   ```
   canvas_width = page.width_mm * dpi / 25.4
   canvas_height = page.height_mm * dpi / 25.4
   available_width = canvas_width - margin_left - margin_right
   available_height = canvas_height - margin_top - margin_bottom
   ```

2. **分配高度预算**:
   ```
   对于每个元素:
     if type == "text":
       height_budget = max_height_mm * dpi / 25.4
     elif type == "barcode":
       height_budget = height_mm * dpi / 25.4
   ```

3. **求每行文本的最大字号**(二分搜索):
   ```rust
   fn find_max_font_size(
       text: &str,
       available_width: u32,
       height_budget: u32,
       font: &Font,
   ) -> f32 {
       let mut left = 8.0;
       let mut right = 120.0;
       let mut result = left;

       while right - left > 0.5 {
           let mid = (left + right) / 2.0;
           let (width, height) = measure_text(text, mid, font);

           if width <= available_width && height <= height_budget {
               result = mid;
               left = mid;
           } else {
               right = mid;
           }
       }

       result
   }
   ```

4. **计算内容块垂直居中**:
   ```
   total_content_height = sum(element_heights) + sum(gaps)
   y_offset = (available_height - total_content_height) / 2

   对于每个元素:
     element.y = margin_top + y_offset + cumulative_height
     cumulative_height += element_height + line_gap
   ```

5. **水平居中**:
   ```
   对于文本元素:
     text_width = measure_text(content, font_size, font)
     element.x = margin_left + (available_width - text_width) / 2

   对于条形码元素:
     barcode_width = estimate_barcode_width(content) + 2 * quiet_zone
     element.x = margin_left + (available_width - barcode_width) / 2
   ```

6. **全局防溢出校验**:
   ```
   if total_content_height > available_height:
       scale_factor = available_height / total_content_height
       for element in elements:
           element.font_size *= scale_factor
           element.y *= scale_factor
   ```

### 2.3 TextRenderer (文本渲染器)

**职责**: 将文本渲染为1bpp位图

**技术选型**: 使用 `rusttype` 或 `ab_glyph` + `imageproc`

**关键接口**:

```rust
pub struct TextRenderer {
    cn_font: Font,
    en_font: Font,
    fallback_font: Font,
}

impl TextRenderer {
    /// 测量文本尺寸(像素)
    pub fn measure_text(&self, text: &str, font_size: f32) -> (u32, u32);

    /// 渲染文本为1bpp位图
    pub fn render_text(
        &self,
        text: &str,
        font_size: f32,
    ) -> ImageBuffer<Luma<u8>, Vec<u8>>;
}
```

**字体选择策略**:

```rust
fn select_font_for_char(c: char) -> &Font {
    if c.is_ascii() {
        &en_font
    } else if is_cjk(c) {
        &cn_font
    } else {
        &fallback_font
    }
}
```

### 2.4 RenderPipeline (渲染管线)

**职责**: 协调文本渲染、条形码渲染、后端输出

**核心流程**:

```rust
impl RenderPipeline {
    pub fn render(
        &self,
        layout: LayoutResult,
        output_config: &OutputConfig,
    ) -> Result<RenderResult> {
        match output_config.mode.as_str() {
            "text_bitmap_plus_native_barcode" => self.render_mode_a(layout),
            "full_bitmap" => self.render_mode_b(layout),
            _ => Err(anyhow!("未知渲染模式")),
        }
    }

    fn render_mode_a(&self, layout: LayoutResult) -> Result<RenderResult> {
        let mut bitmaps = Vec::new();
        let mut native_barcodes = Vec::new();

        for elem in layout.elements {
            match elem.element_type {
                ElementType::Text => {
                    let bitmap = self.text_renderer.render_text(
                        &elem.content,
                        elem.font_size.unwrap(),
                    );
                    bitmaps.push((elem.x, elem.y, bitmap));
                }
                ElementType::Barcode => {
                    native_barcodes.push(elem);
                }
            }
        }

        Ok(RenderResult::MixedMode { bitmaps, native_barcodes })
    }

    fn render_mode_b(&self, layout: LayoutResult) -> Result<RenderResult> {
        let mut canvas = ImageBuffer::from_pixel(
            layout.canvas_width,
            layout.canvas_height,
            Luma([255u8]),
        );

        for elem in layout.elements {
            match elem.element_type {
                ElementType::Text => {
                    let bitmap = self.text_renderer.render_text(...);
                    overlay(&mut canvas, &bitmap, elem.x, elem.y);
                }
                ElementType::Barcode => {
                    let bitmap = self.barcode_renderer.render_barcode(...);
                    overlay(&mut canvas, &bitmap, elem.x, elem.y);
                }
            }
        }

        Ok(RenderResult::FullBitmap(canvas))
    }
}
```

### 2.5 Backend抽象

**PDF Backend**:

```rust
impl PdfBackend {
    pub fn render(&self, result: RenderResult) -> Result<PathBuf> {
        match result {
            RenderResult::MixedMode { bitmaps, native_barcodes } => {
                let mut canvas = create_white_canvas();

                // 贴文本位图
                for (x, y, bitmap) in bitmaps {
                    overlay(&mut canvas, &bitmap, x, y);
                }

                // 渲染条形码(PDF中也用位图)
                for barcode_elem in native_barcodes {
                    let bitmap = self.barcode_renderer.render(...);
                    overlay(&mut canvas, &bitmap, ...);
                }

                self.save_as_png_and_pdf(canvas)
            }
            RenderResult::FullBitmap(canvas) => {
                self.save_as_png_and_pdf(canvas)
            }
        }
    }
}
```

**Printer Backend**:

```rust
impl PrinterBackend {
    pub fn render(&self, result: RenderResult) -> Result<String> {
        let mut tspl = String::new();

        // 纸张配置
        tspl.push_str("SIZE 76 mm, 130 mm\n");
        tspl.push_str("GAP 2 mm, 0 mm\n");
        tspl.push_str("CLS\n");

        match result {
            RenderResult::MixedMode { bitmaps, native_barcodes } => {
                // 文本位图 -> BITMAP/PUTBMP指令
                for (x, y, bitmap) in bitmaps {
                    let bitmap_data = self.encode_bitmap_1bpp(&bitmap);
                    tspl.push_str(&format!(
                        "BITMAP {},{},{},{},{}\n",
                        x, y, bitmap.width(), bitmap.height(), bitmap_data
                    ));
                }

                // 条形码 -> BARCODE指令
                for barcode_elem in native_barcodes {
                    tspl.push_str(&format!(
                        "BARCODE {},{},\"{}\",{},...,\"{}\"\n",
                        barcode_elem.x,
                        barcode_elem.y,
                        barcode_elem.config.barcode_type,
                        barcode_elem.barcode_height.unwrap(),
                        barcode_elem.content,
                    ));
                }
            }
            RenderResult::FullBitmap(canvas) => {
                // 整张位图 -> BITMAP指令
                let bitmap_data = self.encode_bitmap_1bpp(&canvas);
                tspl.push_str(&format!(
                    "BITMAP 0,0,{},{},{}\n",
                    canvas.width(), canvas.height(), bitmap_data
                ));
            }
        }

        tspl.push_str("PRINT 1\n");
        Ok(tspl)
    }
}
```

## 3. 关键技术决策

### 3.1 为什么选择位图渲染文本?

**问题**: TSPL的TEXT指令不支持TrueType字体,中文渲染效果差

**方案对比**:

| 方案 | 优点 | 缺点 |
|------|------|------|
| TSPL TEXT | 指令简单,打印机原生支持 | 不支持TTF,中文效果差 |
| 位图渲染 | 完全控制字体和渲染,跨机型一致 | 指令体积大,传输慢 |

**决策**: 使用**位图渲染文本**,理由:
- 中文粗体是核心需求,TSPL TEXT无法满足
- 现代打印机(USB连接)传输速度足够快
- 可使用RLE压缩减少TSPL指令体积

### 3.2 为什么保留两种渲染模式?

**方案A**(文本位图 + 原生条码):
- **优点**: 条形码由打印机原生渲染,更锐利,扫码成功率高
- **缺点**: PDF预览和实际打印输出可能略有差异

**方案B**(全位图):
- **优点**: PDF和打印输出完全一致(像素级)
- **缺点**: 条形码作为位图传输,可能略微失真

**决策**: 默认使用**方案A**,允许配置切换到方案B

### 3.3 为什么使用二分搜索求最大字号?

**问题**: 需要在宽度和高度约束下找到最大字号

**方案对比**:

| 方案 | 优点 | 缺点 |
|------|------|------|
| 从大到小尝试 | 实现简单 | 效率低(最坏O(n)) |
| 二分搜索 | 效率高(O(log n)) | 需要字体度量单调性保证 |

**决策**: 使用**二分搜索**,搜索范围8-120pt,精度0.5pt

### 3.4 字体加载策略

**问题**: 中英文粗体字体文件大小

**方案**:

1. **内嵌字体**: 使用 `include_bytes!` 编译时嵌入
2. **字体子集化**: 使用 `fonttools` 提取常用汉字(3500字) + ASCII
3. **字体选择**:
   - 英文: Arial Bold / Liberation Sans Bold (~200KB)
   - 中文: Source Han Sans Bold 子集 (~2-3MB)

**决策**: 优先使用**子集化字体**,提供完整字体作为fallback

## 4. 兼容性设计

### 4.1 配置版本检测

```toml
# v1配置(没有version字段)
[metadata]
name = "QSL Card v1"
version = "1.0"

[paper]
width_mm = 76
# ...

# v2配置(有明确的template_version字段)
[metadata]
template_version = "2.0"
name = "QSL Card v2"

[page]
# ...
```

检测逻辑:

```rust
impl TemplateConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;

        // 尝试解析为v2
        if let Ok(v2) = toml::from_str::<TemplateV2Config>(&content) {
            return Ok(TemplateConfig::V2(v2));
        }

        // 尝试解析为v1
        if let Ok(v1) = toml::from_str::<TemplateV1Config>(&content) {
            return Ok(TemplateConfig::V1(v1));
        }

        Err(anyhow!("无法解析模板配置"))
    }
}
```

### 4.2 v1到v2迁移工具

```bash
cargo run --bin migrate-template -- \
    --input config/templates/qsl-card-v1.toml \
    --output config/templates/default.toml
```

迁移逻辑:

1. 读取v1配置
2. 转换为v2格式:
   - `[paper]` -> `[page]`
   - 固定坐标 -> 高度预算(通过逆向计算)
   - 添加 `[layout]`, `[fonts]`, `[output]` 默认配置
3. 保存v2配置

## 5. 测试策略

### 5.1 单元测试

- `TemplateEngine`: 测试fixed/input/computed三种来源
- `LayoutEngine`: 测试字号计算、垂直居中、防溢出
- `TextRenderer`: 测试中英文混排、字体选择、位图生成
- `RenderPipeline`: 测试两种渲染模式

### 5.2 集成测试

- 端到端测试: TOML -> PDF输出
- 对比测试: 方案A和方案B输出对比
- 回归测试: 与v1输出效果对比

### 5.3 性能测试

- 布局计算耗时(目标: <50ms)
- 位图渲染耗时(目标: <200ms)
- TSPL指令生成耗时(目标: <100ms)

## 6. 实施顺序

建议按以下顺序实施,每个阶段可独立验证:

1. **阶段1: 配置解析** (`TemplateEngine`)
   - 定义v2配置结构
   - 实现配置加载和数据填充
   - 单元测试

2. **阶段2: 布局引擎** (`LayoutEngine`)
   - 实现字号计算算法
   - 实现垂直居中和高度分配
   - 单元测试

3. **阶段3: 文本渲染** (`TextRenderer`)
   - 集成rusttype/ab_glyph
   - 实现1bpp位图渲染
   - 单元测试

4. **阶段4: 渲染管线** (`RenderPipeline`)
   - 实现方案A(混合模式)
   - 实现方案B(全位图模式)
   - 单元测试

5. **阶段5: 后端适配**
   - 改造PDF后端
   - 改造Printer后端
   - 集成测试

6. **阶段6: 兼容性和工具**
   - 实现配置版本检测
   - 实现v1到v2迁移工具
   - 端到端测试

## 7. 性能优化

### 7.1 字体加载优化

- 使用 `lazy_static` 全局加载字体(避免重复加载)
- 字体度量结果缓存(HashMap<(char, font_size), Metrics>)

### 7.2 布局计算优化

- 二分搜索范围收窄(8-72pt已覆盖绝大多数场景)
- 对于相同内容缓存字号计算结果

### 7.3 位图编码优化

- TSPL位图使用RLE压缩(对于大面积白色背景效果显著)
- 提供原始编码作为fallback(兼容性更好)

## 8. 未来扩展

### 8.1 元素条件显示

```toml
[[elements]]
id = "optional_field"
type = "text"
source = "input"
key = "optional"
visible_when = "optional != ''"  # 简单条件表达式
```

### 8.2 多模板管理

```
config/templates/
  ├── qsl-standard.toml
  ├── qsl-compact.toml
  └── qsl-bilingual.toml
```

前端提供模板选择下拉菜单.

### 8.3 二维码支持

```toml
[[elements]]
type = "qrcode"
source = "computed"
format = "https://qrz.com/{callsign}"
size_mm = 20
```

使用 `qrcode` crate 渲染为位图.

---

**设计完成日期**: 2026-01-20
