# 功能: 渲染管线 (rendering-pipeline)

## 概述

渲染管线负责协调文本渲染、条形码渲染和后端输出,支持两种渲染模式(文本位图+原生条码 / 全位图),并适配PDF和打印机两种后端.

---

## 新增需求

### 需求: 支持两种渲染模式

渲染管线必须根据输出配置支持两种渲染模式:文本位图+原生条码(方案A)和全位图(方案B),并返回相应的渲染结果.

#### 场景: 模式A - 文本位图+原生条码

- **给定** 输出配置为:
  ```toml
  [output]
  mode = "text_bitmap_plus_native_barcode"
  ```
- **并且** 布局结果包含文本元素和条形码元素
- **当** 调用 `RenderPipeline::render(layout_result, output_config)`
- **则** 应:
  - 文本元素渲染为1bpp位图
  - 条形码元素保留原始信息(不渲染为位图)
  - 返回 `RenderResult::MixedMode { bitmaps, native_barcodes }`

#### 场景: 模式B - 全位图

- **给定** 输出配置为:
  ```toml
  [output]
  mode = "full_bitmap"
  ```
- **并且** 布局结果包含文本元素和条形码元素
- **当** 调用 `RenderPipeline::render(layout_result, output_config)`
- **则** 应:
  - 文本元素渲染为1bpp位图
  - 条形码元素渲染为1bpp位图
  - 所有位图合成到统一画布
  - 返回 `RenderResult::FullBitmap(canvas)`

---

### 需求: 渲染文本元素为位图

渲染管线必须调用TextRenderer将所有文本元素渲染为1bpp位图,保留每个位图的坐标信息.

#### 场景: 渲染单个文本元素

- **给定** 布局结果包含一个文本元素:
  - `content = "BG7XXX"`
  - `font_size = 72pt`
  - `x = 104`, `y = 344`
- **当** 调用 `RenderPipeline::render_text_element(element)`
- **则** 应:
  - 调用 `TextRenderer::render_text("BG7XXX", 72.0)`
  - 返回位图及其坐标 `(104, 344, bitmap)`

#### 场景: 渲染多个文本元素

- **给定** 布局结果包含6个文本元素(标题、副标题、呼号、SN、QTY)
- **当** 调用 `RenderPipeline::render(...)`
- **则** 应为每个文本元素生成位图
- **并且** 位图列表应包含6个条目

---

### 需求: 处理条形码元素

渲染管线必须根据渲染模式决定条形码元素的处理方式:方案A保留原生信息,方案B渲染为位图.

#### 场景: 模式A - 保留原生条形码信息

- **给定** 渲染模式为 `text_bitmap_plus_native_barcode`
- **并且** 布局结果包含一个条形码元素:
  - `content = "BG7XXX"`
  - `barcode_type = "code128"`
  - `x = 138`, `y = 580`
  - `height = 140 dots`
- **当** 调用 `RenderPipeline::render(...)`
- **则** 应保留条形码信息(不渲染为位图)
- **并且** 返回结果应包含 `native_barcodes` 列表

#### 场景: 模式B - 渲染条形码为位图

- **给定** 渲染模式为 `full_bitmap`
- **并且** 布局结果包含一个条形码元素
- **当** 调用 `RenderPipeline::render(...)`
- **则** 应:
  - 调用 `BarcodeRenderer::render_barcode(...)`
  - 将条形码位图绘制到画布上
  - 不在 `native_barcodes` 列表中包含该元素

---

### 需求: 合成全位图画布(模式B)

渲染管线在全位图模式下必须创建白色背景画布,依次叠加所有文本和条形码位图,绘制边框,生成完整画布.

#### 场景: 创建白色背景画布

- **给定** 渲染模式为 `full_bitmap`
- **并且** 画布尺寸为 608×1024 dots
- **当** 调用 `RenderPipeline::create_canvas(width, height)`
- **则** 应创建一个 `ImageBuffer<Luma<u8>, Vec<u8>>`
- **并且** 所有像素应初始化为白色(255)

#### 场景: 叠加文本位图到画布

- **给定** 文本位图 (宽400, 高100)
- **并且** 目标坐标为 (104, 344)
- **当** 调用 `overlay(&mut canvas, &text_bitmap, 104, 344)`
- **则** 应将文本位图的黑色像素(0)叠加到画布上
- **并且** 白色像素(255)应保持画布背景不变

#### 场景: 叠加条形码位图到画布

- **给定** 条形码位图 (宽332, 高140)
- **并且** 目标坐标为 (138, 580)
- **当** 调用 `overlay(&mut canvas, &barcode_bitmap, 138, 580)`
- **则** 应将条形码位图叠加到画布上
- **并且** 不应覆盖已有的文本

#### 场景: 绘制边框(如果启用)

- **给定** 布局结果包含边框配置:
  - `x = 16`, `y = 24`
  - `width = 576`, `height = 976`
  - `thickness = 2`
- **当** 调用 `RenderPipeline::draw_border(&mut canvas, border_config)`
- **则** 应在画布上绘制矩形边框
- **并且** 边框线宽为2 dots

---

### 需求: 返回渲染结果

渲染管线必须根据渲染模式返回不同类型的RenderResult:MixedMode包含位图列表和原生条码信息,FullBitmap包含完整画布.

#### 场景: 返回混合模式结果(模式A)

- **给定** 渲染模式为 `text_bitmap_plus_native_barcode`
- **并且** 渲染完成
- **当** 返回 `RenderResult`
- **则** 应返回:
  ```rust
  RenderResult::MixedMode {
      bitmaps: Vec<(u32, u32, ImageBuffer<Luma<u8>, Vec<u8>>)>,
      native_barcodes: Vec<BarcodeElement>,
      canvas_size: (u32, u32),
      border: Option<BorderConfig>,
  }
  ```

#### 场景: 返回全位图结果(模式B)

- **给定** 渲染模式为 `full_bitmap`
- **并且** 渲染完成
- **当** 返回 `RenderResult`
- **则** 应返回:
  ```rust
  RenderResult::FullBitmap {
      canvas: ImageBuffer<Luma<u8>, Vec<u8>>,
      canvas_size: (u32, u32),
  }
  ```

---

### 需求: PDF后端适配

PDF后端必须接收RenderResult并根据类型生成PNG/PDF文件:MixedMode需叠加位图并渲染条码,FullBitmap直接使用画布.

#### 场景: PDF后端处理混合模式结果

- **给定** 渲染结果为 `RenderResult::MixedMode`
- **当** 调用 `PdfBackend::render(result)`
- **则** 应:
  1. 创建白色背景画布
  2. 叠加所有文本位图
  3. 将条形码渲染为位图(PDF中也用位图)
  4. 叠加条形码位图
  5. 绘制边框(如果有)
  6. 保存为PNG和PDF

#### 场景: PDF后端处理全位图结果

- **给定** 渲染结果为 `RenderResult::FullBitmap`
- **当** 调用 `PdfBackend::render(result)`
- **则** 应:
  1. 直接使用提供的画布
  2. 保存为PNG和PDF

#### 场景: 生成带时间戳的文件名

- **给定** 当前时间为 `2026-01-20 14:30:45`
- **当** 调用 `PdfBackend::render(...)`
- **则** 应生成文件名:
  - PNG: `qsl_20260120_143045.png`
  - PDF: `qsl_20260120_143045.pdf`

---

### 需求: 打印机后端适配

打印机后端必须接收RenderResult并生成TSPL指令:MixedMode生成BITMAP+BARCODE指令,FullBitmap生成整张BITMAP指令.

#### 场景: 打印机后端处理混合模式结果

- **给定** 渲染结果为 `RenderResult::MixedMode`
- **当** 调用 `PrinterBackend::render(result)`
- **则** 应生成TSPL指令:
  ```tspl
  SIZE 76 mm, 130 mm
  GAP 2 mm, 0 mm
  CLS

  ; 文本位图 -> BITMAP指令
  BITMAP 104,344,400,100,<1bpp_data>
  BITMAP 104,500,350,80,<1bpp_data>
  ...

  ; 条形码 -> BARCODE指令
  BARCODE 138,580,"128",140,0,0,2,2,"BG7XXX"

  PRINT 1
  ```

#### 场景: 打印机后端处理全位图结果

- **给定** 渲染结果为 `RenderResult::FullBitmap`
- **当** 调用 `PrinterBackend::render(result)`
- **则** 应生成TSPL指令:
  ```tspl
  SIZE 76 mm, 130 mm
  GAP 2 mm, 0 mm
  CLS

  ; 整张画布 -> BITMAP指令
  BITMAP 0,0,608,1024,<1bpp_data>

  PRINT 1
  ```

#### 场景: 编码1bpp位图数据为TSPL格式

- **给定** 一个1bpp位图(宽400, 高100)
- **当** 调用 `PrinterBackend::encode_bitmap_1bpp(&bitmap)`
- **则** 应:
  - 将位图转换为TSPL的二进制格式
  - 每8个像素打包为1个字节(从高位到低位)
  - 返回十六进制字符串或Base64编码

---

### 需求: 错误处理

渲染管线必须在文本或条形码渲染失败、位图叠加超出边界时返回错误或警告,确保错误信息清晰.

#### 场景: 文本渲染失败

- **给定** 文本元素包含无法渲染的内容
- **当** 调用 `RenderPipeline::render(...)`
- **则** 应返回错误: `"文本渲染失败: [详细信息]"`

#### 场景: 条形码渲染失败

- **给定** 条形码内容不符合Code128规范
- **当** 调用 `BarcodeRenderer::render_barcode(...)`
- **则** 应返回错误: `"条形码渲染失败: 内容不符合Code128规范"`

#### 场景: 位图叠加超出画布边界

- **给定** 文本位图坐标或尺寸超出画布范围
- **当** 调用 `overlay(...)`
- **则** 应:
  - 输出警告日志: `"位图部分超出画布边界,已裁剪"`
  - 只叠加画布内的部分

---

## 修改需求

### 需求: 改造PDF后端为接收布局结果

必须重构PDF后端移除TSPL解析逻辑,改为直接接收RenderResult进行渲染,保持输出格式不变.

#### 场景: 重构PDF后端的render方法

- **给定** 现有的 `PdfBackend` 通过解析TSPL指令渲染
- **当** 重构为接收 `RenderResult`
- **则** 应:
  - 移除TSPL解析逻辑
  - 新增 `render(result: RenderResult) -> Result<PathBuf>` 方法
  - 支持两种渲染结果类型
- **并且** 保持输出格式不变(PNG + PDF)

---

### 需求: 改造TSPL生成器为从布局结果生成

必须重构TSPLGenerator移除模板解析逻辑,改为从RenderResult生成TSPL指令,根据渲染结果类型生成不同指令.

#### 场景: 重构TSPLGenerator

- **给定** 现有的 `TSPLGenerator` 从模板配置生成TSPL
- **当** 重构为从 `RenderResult` 生成TSPL
- **则** 应:
  - 移除模板解析逻辑
  - 新增 `generate_from_render_result(result: RenderResult) -> String` 方法
  - 根据渲染结果类型生成不同的TSPL指令
- **并且** 保持TSPL指令格式正确

---

## 移除需求

### 需求: 移除PDF后端的TSPL解析功能

- **说明**: 新架构中,PDF后端直接接收布局结果,不再需要解析TSPL指令
- **影响**: `src/printer/backend/pdf.rs` 中的 `parse_tspl`, `parse_tspl_line`, `render_commands` 方法应移除

---

## 重命名需求

无

---

## 技术说明

### 核心数据结构

```rust
/// 渲染结果
#[derive(Debug, Clone)]
pub enum RenderResult {
    /// 混合模式: 文本位图 + 原生条码
    MixedMode {
        /// 文本位图列表: (x, y, bitmap)
        bitmaps: Vec<(u32, u32, ImageBuffer<Luma<u8>, Vec<u8>>)>,
        /// 原生条形码元素
        native_barcodes: Vec<BarcodeElement>,
        /// 画布尺寸
        canvas_size: (u32, u32),
        /// 边框配置
        border: Option<BorderConfig>,
    },
    /// 全位图模式
    FullBitmap {
        /// 完整画布
        canvas: ImageBuffer<Luma<u8>, Vec<u8>>,
        /// 画布尺寸
        canvas_size: (u32, u32),
    },
}

/// 条形码元素信息
#[derive(Debug, Clone)]
pub struct BarcodeElement {
    pub x: u32,
    pub y: u32,
    pub content: String,
    pub barcode_type: String,
    pub height: u32,
    pub quiet_zone: u32,
    pub human_readable: bool,
}
```

### RenderPipeline API

```rust
pub struct RenderPipeline {
    text_renderer: TextRenderer,
    barcode_renderer: BarcodeRenderer,
}

impl RenderPipeline {
    /// 执行渲染管线
    pub fn render(
        &self,
        layout_result: LayoutResult,
        output_config: &OutputConfig,
    ) -> Result<RenderResult>;

    /// 渲染混合模式(方案A)
    fn render_mixed_mode(&self, layout: LayoutResult) -> Result<RenderResult>;

    /// 渲染全位图模式(方案B)
    fn render_full_bitmap(&self, layout: LayoutResult) -> Result<RenderResult>;

    /// 创建空白画布
    fn create_canvas(&self, width: u32, height: u32) -> ImageBuffer<Luma<u8>, Vec<u8>>;

    /// 叠加位图到画布
    fn overlay(
        &self,
        canvas: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
        bitmap: &ImageBuffer<Luma<u8>, Vec<u8>>,
        x: u32,
        y: u32,
    );

    /// 绘制边框
    fn draw_border(
        &self,
        canvas: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
        border: &BorderConfig,
    );
}
```

### 位图叠加算法

```rust
fn overlay(
    &self,
    canvas: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
    bitmap: &ImageBuffer<Luma<u8>, Vec<u8>>,
    x: u32,
    y: u32,
) {
    let canvas_width = canvas.width();
    let canvas_height = canvas.height();

    for (bx, by, pixel) in bitmap.enumerate_pixels() {
        let cx = x + bx;
        let cy = y + by;

        // 边界检查
        if cx >= canvas_width || cy >= canvas_height {
            continue;
        }

        // 只叠加黑色像素(0),白色像素(255)保持透明
        if pixel.0[0] < 128 {
            canvas.put_pixel(cx, cy, *pixel);
        }
    }
}
```

### TSPL位图编码

```rust
impl PrinterBackend {
    fn encode_bitmap_1bpp(&self, bitmap: &ImageBuffer<Luma<u8>, Vec<u8>>) -> String {
        let width = bitmap.width();
        let height = bitmap.height();
        let row_bytes = (width + 7) / 8;  // 向上取整

        let mut data = Vec::new();

        for y in 0..height {
            let mut byte = 0u8;
            let mut bit_index = 7;

            for x in 0..width {
                let pixel = bitmap.get_pixel(x, y).0[0];
                let bit = if pixel < 128 { 1 } else { 0 };  // 黑色为1
                byte |= bit << bit_index;

                if bit_index == 0 {
                    data.push(byte);
                    byte = 0;
                    bit_index = 7;
                } else {
                    bit_index -= 1;
                }
            }

            // 处理行尾不足8位的情况
            if width % 8 != 0 {
                data.push(byte);
            }
        }

        // 转换为十六进制字符串
        data.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<String>()
    }
}
```

---

## 依赖关系

- **前置功能**:
  - `layout-system` (提供布局结果)
  - `text-rendering` (文本位图渲染)
  - 已有的 `barcode_renderer` (条形码渲染)
- **后续功能**: 无(输出到后端)

---

## 测试要求

### 单元测试

- 测试两种渲染模式的分支逻辑
- 测试位图叠加(边界情况、透明度)
- 测试边框绘制
- 测试TSPL位图编码

### 集成测试

- 测试完整渲染管线(布局 -> 渲染 -> 输出)
- 测试PDF后端输出(验证PNG文件)
- 测试打印机后端输出(验证TSPL指令)

### 对比测试

- 对比方案A和方案B的输出(PDF模式)
- 验证条形码可扫描性

### 性能测试

- 渲染管线总耗时应 < 200ms

---

**创建日期**: 2026-01-20
