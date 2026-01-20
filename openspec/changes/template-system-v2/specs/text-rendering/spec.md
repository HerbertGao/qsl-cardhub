# 功能: 文本渲染 (text-rendering)

## 概述

文本渲染模块负责使用内嵌的中英文粗体字体将文本渲染为1bpp黑白位图,并提供文本度量功能供布局引擎使用.

---

## 新增需求

### 需求: 加载内嵌的中英文粗体字体

文本渲染器必须在编译时嵌入中英文粗体字体文件,并在初始化时加载为Font对象,无需运行时文件系统访问.

#### 场景: 在编译时嵌入字体文件

- **给定** 项目包含以下字体文件:
  - `assets/fonts/LiberationSans-Bold.ttf` (英文粗体)
  - `assets/fonts/SourceHanSansSC-Bold.otf` (中文粗体)
- **当** 编译应用程序
- **则** 字体文件应通过 `include_bytes!` 宏嵌入到二进制文件中
- **并且** 不需要在运行时读取文件系统

#### 场景: 初始化TextRenderer时加载字体

- **给定** 嵌入的字体数据
- **当** 调用 `TextRenderer::new()`
- **则** 应成功加载三个字体:
  - `cn_font`: 中文粗体(SourceHanSansSC-Bold)
  - `en_font`: 英文粗体(LiberationSans-Bold)
  - `fallback_font`: fallback字体(使用中文字体)
- **并且** 如果字体加载失败,应返回明确的错误信息

---

### 需求: 根据字符类型自动选择字体

文本渲染器必须根据字符类型(ASCII/CJK/其他)自动选择合适的字体,支持中英文混排文本的正确渲染.

#### 场景: 纯英文文本使用英文字体

- **给定** 文本内容为 `"BG7XXX"`
- **当** 调用 `TextRenderer::render_text("BG7XXX", 72.0)`
- **则** 应使用 `en_font` (Arial Bold) 渲染所有字符

#### 场景: 纯中文文本使用中文字体

- **给定** 文本内容为 `"中国无线电协会"`
- **当** 调用 `TextRenderer::render_text(...)`
- **则** 应使用 `cn_font` (SourceHanSansSC-Bold) 渲染所有字符

#### 场景: 中英文混排自动切换字体

- **给定** 文本内容为 `"BG7XXX 中国"`
- **当** 调用 `TextRenderer::render_text(...)`
- **则** 应:
  - 使用 `en_font` 渲染 `"BG7XXX "`
  - 使用 `cn_font` 渲染 `"中国"`
- **并且** 渲染结果应正确拼接

#### 场景: 数字和标点符号使用英文字体

- **给定** 文本内容为 `"SN: 123"`
- **当** 调用 `TextRenderer::render_text(...)`
- **则** 所有字符应使用 `en_font`

#### 场景: 未知字符使用fallback字体

- **给定** 文本包含emoji或特殊字符 `"QSL 📻"`
- **当** 调用 `TextRenderer::render_text(...)`
- **则** 应:
  - `"QSL "` 使用 `en_font`
  - emoji使用 `fallback_font` (如果支持) 或跳过

---

### 需求: 测量文本尺寸

文本渲染器必须提供文本尺寸测量功能,准确计算指定字号下文本的宽度和高度(dots),支持混排文本.

#### 场景: 测量纯英文文本尺寸

- **给定** 文本为 `"BG7XXX"`, 字号为 72pt
- **当** 调用 `TextRenderer::measure_text("BG7XXX", 72.0)`
- **则** 应返回:
  - `width`: 文本宽度(dots)
  - `height`: 文本高度(dots)
- **并且** 尺寸应与实际渲染结果一致

#### 场景: 测量中文文本尺寸

- **给定** 文本为 `"中国无线电"`, 字号为 48pt
- **当** 调用 `TextRenderer::measure_text(...)`
- **则** 应返回准确的宽度和高度(dots)

#### 场景: 测量中英文混排文本尺寸

- **给定** 文本为 `"BG7XXX 中国"`, 字号为 60pt
- **当** 调用 `TextRenderer::measure_text(...)`
- **则** 应:
  - 分别测量英文部分和中文部分
  - 累加宽度
  - 高度取最大值

#### 场景: 空字符串的尺寸

- **给定** 文本为 `""`
- **当** 调用 `TextRenderer::measure_text("", 72.0)`
- **则** 应返回:
  - `width = 0`
  - `height = 0`

---

### 需求: 渲染文本为1bpp黑白位图

文本渲染器必须将文本渲染为1bpp(纯黑白)位图,使用指定字号和自动选择的字体,支持混排文本的字形拼接.

#### 场景: 渲染纯英文文本

- **给定** 文本为 `"BG7XXX"`, 字号为 72pt
- **当** 调用 `TextRenderer::render_text("BG7XXX", 72.0)`
- **则** 应返回一个 `ImageBuffer<Luma<u8>, Vec<u8>>`:
  - 位图宽度为测量宽度
  - 位图高度为测量高度
  - 背景为白色(255)
  - 文字为黑色(0)

#### 场景: 渲染中文文本

- **给定** 文本为 `"中国"`, 字号为 48pt
- **当** 调用 `TextRenderer::render_text(...)`
- **则** 应返回1bpp位图
- **并且** 中文笔画应清晰,粗体效果明显

#### 场景: 渲染中英文混排文本

- **给定** 文本为 `"SN: 123"`, 字号为 60pt
- **当** 调用 `TextRenderer::render_text(...)`
- **则** 应:
  - 正确拼接不同字体渲染的字符
  - 保持基线对齐
  - 返回统一的位图

#### 场景: 渲染带特殊字符的文本

- **给定** 文本为 `"SN: 123 (测试)"`
- **当** 调用 `TextRenderer::render_text(...)`
- **则** 应正确渲染所有字符,包括括号和空格

---

### 需求: 1bpp位图格式

文本渲染器生成的位图必须为1bpp格式,所有像素值为0(黑色)或255(白色),通过阈值转换消除灰度.

#### 场景: 位图像素值为纯黑(0)或纯白(255)

- **给定** 渲染完成的位图
- **当** 遍历所有像素
- **则** 每个像素的值应为 `0` (黑色) 或 `255` (白色)
- **并且** 不应有灰度值(如128)

#### 场景: 通过阈值将灰度图转换为1bpp

- **给定** 字体渲染产生的灰度图(抗锯齿)
- **并且** 阈值为 `160`
- **当** 应用阈值转换
- **则** 像素值应转换为:
  - `value < 160` → `0` (黑色)
  - `value >= 160` → `255` (白色)

---

### 需求: 字体度量缓存

文本渲染器必须缓存字符度量结果以提升性能,避免重复测量相同字符和字号的宽度.

#### 场景: 缓存字符宽度以提升性能

- **给定** 已经测量过 `'A'` 在72pt下的宽度
- **当** 再次测量相同字符和字号
- **则** 应从缓存中返回结果,而非重新测量
- **并且** 缓存查询应 < 1ms

#### 场景: 缓存大小限制

- **给定** 字体度量缓存
- **当** 缓存条目数超过 10000
- **则** 应使用LRU策略淘汰旧条目

---

### 需求: 错误处理

文本渲染器必须在字体加载失败或渲染无法显示的字符时返回明确错误或跳过无法渲染的字符.

#### 场景: 字体加载失败

- **给定** 嵌入的字体数据损坏
- **当** 调用 `TextRenderer::new()`
- **则** 应返回错误: `"加载字体失败: [字体名称]"`

#### 场景: 渲染包含无法显示的字符

- **给定** 文本包含字体不支持的字符
- **当** 调用 `TextRenderer::render_text(...)`
- **则** 应:
  - 跳过无法显示的字符
  - 输出警告日志: `"字符 [char] 无法渲染,已跳过"`
  - 继续渲染其他字符

---

## 修改需求

### 需求: 增强现有TextRenderer的字体选择逻辑

必须重构现有TextRenderer支持多字体自动切换,添加字体选择方法,保持向后兼容.

#### 场景: 重构现有的字体选择方法

- **给定** 现有的 `src/printer/text_renderer.rs` 只支持单一字体
- **当** 重构为支持中英文字体自动切换
- **则** 应:
  - 添加 `select_font_for_char(c: char) -> &Font` 方法
  - 更新 `render_text` 方法,支持混排
- **并且** 保持向后兼容(现有测试不受影响)

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
pub struct TextRenderer {
    /// 中文粗体字体
    cn_font: Font,
    /// 英文粗体字体
    en_font: Font,
    /// fallback字体
    fallback_font: Font,
    /// 字体度量缓存
    metrics_cache: Mutex<HashMap<(char, u32), CharMetrics>>,
}

#[derive(Clone, Copy)]
struct CharMetrics {
    width: u32,
    height: u32,
}
```

### TextRenderer API

```rust
impl TextRenderer {
    /// 创建新的文本渲染器
    pub fn new() -> Result<Self>;

    /// 测量文本尺寸(dots)
    pub fn measure_text(&self, text: &str, font_size: f32) -> (u32, u32);

    /// 渲染文本为1bpp位图
    pub fn render_text(
        &self,
        text: &str,
        font_size: f32,
    ) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>>;

    /// 根据字符选择字体
    fn select_font_for_char(&self, c: char) -> &Font;

    /// 应用阈值转换为1bpp
    fn apply_threshold(
        &self,
        grayscale_image: ImageBuffer<Luma<u8>, Vec<u8>>,
        threshold: u8,
    ) -> ImageBuffer<Luma<u8>, Vec<u8>>;
}
```

### 字体选择算法

```rust
fn select_font_for_char(&self, c: char) -> &Font {
    if c.is_ascii() {
        &self.en_font
    } else if is_cjk(c) {
        &self.cn_font
    } else {
        &self.fallback_font
    }
}

fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}' |  // CJK Unified Ideographs
        '\u{3400}'..='\u{4DBF}' |  // CJK Extension A
        '\u{20000}'..='\u{2A6DF}'  // CJK Extension B
    )
}
```

### 字体嵌入

```rust
// src/printer/font_loader.rs 或 text_renderer.rs
const CN_FONT_DATA: &[u8] = include_bytes!("../../assets/fonts/SourceHanSansSC-Bold.otf");
const EN_FONT_DATA: &[u8] = include_bytes!("../../assets/fonts/LiberationSans-Bold.ttf");

impl TextRenderer {
    pub fn new() -> Result<Self> {
        let cn_font = Font::try_from_bytes(CN_FONT_DATA)
            .context("加载中文字体失败")?;
        let en_font = Font::try_from_bytes(EN_FONT_DATA)
            .context("加载英文字体失败")?;
        let fallback_font = cn_font.clone();

        Ok(Self {
            cn_font,
            en_font,
            fallback_font,
            metrics_cache: Mutex::new(HashMap::new()),
        })
    }
}
```

### 位图渲染流程

```rust
pub fn render_text(
    &self,
    text: &str,
    font_size: f32,
) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>> {
    // 1. 测量总尺寸
    let (width, height) = self.measure_text(text, font_size);

    // 2. 创建空白画布(白色背景)
    let mut canvas = ImageBuffer::from_pixel(width, height, Luma([255u8]));

    // 3. 逐字符渲染
    let mut x_offset = 0;
    let scale = Scale::uniform(font_size);

    for c in text.chars() {
        let font = self.select_font_for_char(c);
        let glyph = font.glyph(c).scaled(scale);

        // 渲染字形到画布
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                let px = x_offset + x + bounding_box.min.x as u32;
                let py = y + bounding_box.min.y as u32;
                if px < width && py < height {
                    let gray_value = (255.0 * (1.0 - v)) as u8;
                    canvas.put_pixel(px, py, Luma([gray_value]));
                }
            });

            x_offset += glyph.h_metrics().advance_width as u32;
        }
    }

    // 4. 应用阈值转换为1bpp
    Ok(self.apply_threshold(canvas, 160))
}
```

---

## 依赖关系

- **前置功能**: `template-config` (字体配置)
- **后续功能**:
  - `layout-system` (依赖文本度量)
  - `rendering-pipeline` (依赖位图渲染)

---

## 测试要求

### 单元测试

- 测试字体加载(中文、英文、fallback)
- 测试字符类型判断(ASCII、CJK、其他)
- 测试字体选择逻辑
- 测试文本尺寸测量(纯英文、纯中文、混排)
- 测试位图渲染(不同字号)
- 测试阈值转换(灰度->1bpp)

### 组件测试

- 使用 `tests/components/text_rendering.rs` 测试文本渲染
- 测试渲染结果的像素值(0或255)
- 测试混排文本的基线对齐

### 性能测试

- 字体加载应 < 100ms
- 单次文本测量应 < 5ms
- 单次文本渲染应 < 50ms

### 测试数据

- 提供不同字号的测试用例(8pt ~ 120pt)
- 提供不同语言的测试文本(英文、中文、混排)

---

**创建日期**: 2026-01-20
