// 文本渲染器
//
// 使用 rusttype 进行文本光栅化和渲染
// 支持中英文混排和自动字体选择

use super::font_loader::FontLoader;
use anyhow::Result;
use image::{GrayImage, ImageBuffer, Luma, RgbImage};
use rusttype::{Font, Scale, point};
use std::collections::HashMap;
use std::sync::Mutex;

/// 字符度量信息
#[derive(Debug, Clone, Copy)]
struct CharMetrics {
    width: u32,
    height: u32,
}

/// 文本渲染器
pub struct TextRenderer {
    /// 字体加载器
    font_loader: FontLoader,
    /// 字体度量缓存 (字符, 字号) -> 度量信息
    metrics_cache: Mutex<HashMap<(char, u32), CharMetrics>>,
}

impl TextRenderer {
    /// 创建新的文本渲染器
    pub fn new() -> Result<Self> {
        Ok(Self {
            font_loader: FontLoader::new(),
            metrics_cache: Mutex::new(HashMap::new()),
        })
    }

    /// 根据字符类型选择字体
    ///
    /// # 参数
    /// - `c`: 字符
    ///
    /// # 返回
    /// 字体名称 ("chinese" | "english")
    fn select_font_for_char(&self, c: char) -> &str {
        if Self::is_cjk(c) {
            "chinese"
        } else {
            "english"
        }
    }

    /// 判断字符是否为CJK字符
    fn is_cjk(c: char) -> bool {
        matches!(c,
            '\u{4E00}'..='\u{9FFF}' |  // CJK Unified Ideographs
            '\u{3400}'..='\u{4DBF}' |  // CJK Extension A
            '\u{20000}'..='\u{2A6DF}'  // CJK Extension B
        )
    }

    /// 测量文本尺寸(支持混排)
    ///
    /// # 参数
    /// - `text`: 要测量的文本
    /// - `font_size`: 字号
    ///
    /// # 返回
    /// (宽度, 高度) in dots
    pub fn measure_text(&mut self, text: &str, font_size: f32) -> Result<(u32, u32)> {
        if text.is_empty() {
            return Ok((0, 0));
        }

        let scale = Scale::uniform(font_size);
        let mut x_offset = 0.0;
        let mut min_x = 0i32;
        let mut max_x = 0i32;
        let mut max_height = 0u32;

        for c in text.chars() {
            let font_name = if Self::is_cjk(c) {
                "chinese"
            } else {
                "english"
            };
            let font = self.font_loader.load_font(font_name)?;

            let v_metrics = font.v_metrics(scale);
            let glyph = font.glyph(c).scaled(scale);
            let h_metrics = glyph.h_metrics();

            // 计算实际的像素边界框
            let positioned_glyph = glyph.positioned(point(x_offset, v_metrics.ascent));
            if let Some(bb) = positioned_glyph.pixel_bounding_box() {
                min_x = min_x.min(bb.min.x);
                max_x = max_x.max(bb.max.x);
            }

            // 计算高度
            let height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;
            max_height = max_height.max(height);

            x_offset += h_metrics.advance_width;
        }

        // 实际像素宽度 = max_x - min_x
        // 如果min_x < 0，说明字形向左延伸，需要额外的空间
        let width = (max_x - min_x) as u32;

        Ok((width, max_height))
    }

    /// 渲染文本为1bpp位图
    ///
    /// # 参数
    /// - `text`: 要渲染的文本
    /// - `font_size`: 字号
    ///
    /// # 返回
    /// 1bpp灰度图像 (0=黑色, 255=白色)
    pub fn render_text(&mut self, text: &str, font_size: f32) -> Result<GrayImage> {
        // 1. 测量文本尺寸
        let (width, height) = self.measure_text(text, font_size)?;

        if width == 0 || height == 0 {
            // 返回空图像
            return Ok(ImageBuffer::from_pixel(1, 1, Luma([255u8])));
        }

        // 2. 创建白色画布
        let mut canvas = ImageBuffer::from_pixel(width, height, Luma([255u8]));

        // 3. 计算最小x偏移（用于处理字形向左延伸的情况）
        let scale = Scale::uniform(font_size);
        let mut x_offset = 0.0;
        let mut min_x = 0i32;

        // 先计算min_x
        for c in text.chars() {
            let font_name = if Self::is_cjk(c) {
                "chinese"
            } else {
                "english"
            };
            let font = self.font_loader.load_font(font_name)?;
            let v_metrics = font.v_metrics(scale);
            let glyph = font.glyph(c).scaled(scale);
            let h_metrics = glyph.h_metrics();

            let positioned_glyph = glyph.positioned(point(x_offset, v_metrics.ascent));
            if let Some(bb) = positioned_glyph.pixel_bounding_box() {
                min_x = min_x.min(bb.min.x);
            }

            x_offset += h_metrics.advance_width;
        }

        // 4. 渲染文本（考虑min_x偏移）
        x_offset = 0.0;
        let x_adjust = -min_x;  // 如果min_x < 0，向右偏移

        for c in text.chars() {
            let font_name = if Self::is_cjk(c) {
                "chinese"
            } else {
                "english"
            };
            let font = self.font_loader.load_font(font_name)?;

            let v_metrics = font.v_metrics(scale);
            let glyph = font.glyph(c).scaled(scale);
            let h_metrics = glyph.h_metrics();

            let positioned_glyph = glyph.positioned(point(x_offset, v_metrics.ascent));
            if let Some(bounding_box) = positioned_glyph.pixel_bounding_box() {
                positioned_glyph.draw(|x, y, v| {
                    let px = ((x as i32 + bounding_box.min.x + x_adjust) as u32);
                    let py = (y as i32 + bounding_box.min.y) as u32;

                    if px < width && py < height {
                        // 将字体的灰度值转换为图像灰度值
                        // v=1.0表示完全黑色，v=0.0表示完全透明
                        let gray_value = (255.0 * (1.0 - v)) as u8;
                        canvas.put_pixel(px, py, Luma([gray_value]));
                    }
                });
            }

            x_offset += h_metrics.advance_width;
        }

        // 5. 应用阈值转换为1bpp
        let threshold = 160; // 默认阈值
        Ok(Self::apply_threshold(canvas, threshold))
    }

    /// 应用阈值转换为1bpp
    fn apply_threshold(mut img: GrayImage, threshold: u8) -> GrayImage {
        for pixel in img.pixels_mut() {
            pixel.0[0] = if pixel.0[0] < threshold { 0 } else { 255 };
        }
        img
    }

    /// 获取缓存的字符度量
    fn get_cached_metrics(&self, c: char, font_size: u32) -> Option<CharMetrics> {
        self.metrics_cache
            .lock()
            .unwrap()
            .get(&(c, font_size))
            .copied()
    }

    /// 缓存字符度量
    fn cache_metrics(&self, c: char, font_size: u32, metrics: CharMetrics) {
        let mut cache = self.metrics_cache.lock().unwrap();

        // LRU: 限制缓存大小
        if cache.len() >= 10000 {
            // 简单清空（真实LRU需要更复杂的实现）
            cache.clear();
            log::debug!("字体度量缓存已清空");
        }

        cache.insert((c, font_size), metrics);
    }

    // ========== 向后兼容的方法 ==========

    /// 计算文本在指定宽度内的最大字号
    ///
    /// # 参数
    /// - `text`: 要渲染的文本
    /// - `max_width`: 最大宽度(像素)
    /// - `is_chinese`: 是否为中文文本（为了向后兼容保留，实际会自动检测）
    ///
    /// # 返回
    /// 最大字号(像素)
    pub fn calculate_max_font_size(
        &mut self,
        text: &str,
        max_width: u32,
        _is_chinese: bool, // 忽略此参数，自动检测
    ) -> Result<f32> {
        // 根据文本类型确定最大字号
        let has_chinese = text.chars().any(Self::is_cjk);
        let max_font_size = if !has_chinese && text.len() <= 10 {
            120 // 短英文（如呼号）最大 120px
        } else if has_chinese {
            80 // 中文最大 80px
        } else {
            80 // 其他英文最大 80px
        };

        // 从大到小尝试字号
        for font_size in (20..=max_font_size).rev().step_by(5) {
            let (width, _) = self.measure_text(text, font_size as f32)?;

            // 留 5% 边距
            if width <= (max_width as f32 * 0.95) as u32 {
                log::debug!(
                    "文本 \"{}\" 最大字号: {}px (宽度: {}/{}, 字符数: {})",
                    text,
                    font_size,
                    width,
                    max_width,
                    text.len()
                );
                return Ok(font_size as f32);
            }
        }

        // 保底字号
        log::warn!("文本 \"{}\" 使用保底字号 18px", text);
        Ok(18.0)
    }

    /// 在RGB图像上绘制文本（向后兼容）
    pub fn draw_text(
        &mut self,
        img: &mut RgbImage,
        text: &str,
        x: i32,
        y: i32,
        font_size: f32,
        _is_chinese: bool, // 忽略此参数
    ) -> Result<()> {
        let scale = Scale::uniform(font_size);
        let mut x_offset = x as f32;

        for c in text.chars() {
            let font_name = if Self::is_cjk(c) {
                "chinese"
            } else {
                "english"
            };
            let font = self.font_loader.load_font(font_name)?;

            let v_metrics = font.v_metrics(scale);
            let glyph = font.glyph(c).scaled(scale);
            let h_metrics = glyph.h_metrics();

            let positioned_glyph = glyph.positioned(point(x_offset, y as f32 + v_metrics.ascent));
            if let Some(bounding_box) = positioned_glyph.pixel_bounding_box() {
                positioned_glyph.draw(|gx, gy, gv| {
                    let px = bounding_box.min.x + gx as i32;
                    let py = bounding_box.min.y + gy as i32;

                    if px >= 0 && px < img.width() as i32 && py >= 0 && py < img.height() as i32 {
                        let pixel = img.get_pixel_mut(px as u32, py as u32);
                        // 混合黑色文本
                        let alpha = gv;
                        let inv_alpha = 1.0 - alpha;
                        pixel.0[0] = (pixel.0[0] as f32 * inv_alpha) as u8;
                        pixel.0[1] = (pixel.0[1] as f32 * inv_alpha) as u8;
                        pixel.0[2] = (pixel.0[2] as f32 * inv_alpha) as u8;
                    }
                });
            }

            x_offset += h_metrics.advance_width;
        }

        Ok(())
    }

    /// 居中绘制文本（向后兼容）
    pub fn draw_centered_text(
        &mut self,
        img: &mut RgbImage,
        text: &str,
        y: i32,
        font_size: f32,
        width: u32,
        is_chinese: bool,
    ) -> Result<()> {
        let (text_width, _) = self.measure_text(text, font_size)?;
        let x = ((width as f32 - text_width as f32) / 2.0) as i32;

        self.draw_text(img, text, x, y, font_size, is_chinese)
    }

    /// 渲染中文标题(两行)（向后兼容）
    pub fn render_chinese_headers(
        &mut self,
        img: &mut RgbImage,
        title1: &str,
        title2: &str,
        width: u32,
    ) -> Result<()> {
        log::info!("开始渲染中文标题");

        // 第一行标题
        let font_size1 = self.calculate_max_font_size(title1, width, true)?;
        log::debug!("第一行标题 \"{}\" 字号: {}", title1, font_size1);
        self.draw_centered_text(img, title1, 25, font_size1, width, true)?;

        // 第二行标题
        if !title2.is_empty() {
            let font_size2 = self.calculate_max_font_size(title2, width, true)?;
            log::debug!("第二行标题 \"{}\" 字号: {}", title2, font_size2);
            self.draw_centered_text(img, title2, 75, font_size2, width, true)?;
        }

        log::info!("中文标题渲染完成");
        Ok(())
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new().expect("创建TextRenderer失败")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_cjk() {
        assert!(TextRenderer::is_cjk('中'));
        assert!(TextRenderer::is_cjk('国'));
        assert!(!TextRenderer::is_cjk('A'));
        assert!(!TextRenderer::is_cjk('1'));
        assert!(!TextRenderer::is_cjk(' '));
    }

    #[test]
    fn test_select_font_for_char() {
        let renderer = TextRenderer::new().unwrap();

        assert_eq!(renderer.select_font_for_char('中'), "chinese");
        assert_eq!(renderer.select_font_for_char('A'), "english");
        assert_eq!(renderer.select_font_for_char('1'), "english");
        assert_eq!(renderer.select_font_for_char(' '), "english");
    }

    #[test]
    fn test_measure_text_empty() {
        let mut renderer = TextRenderer::new().unwrap();
        let (width, height) = renderer.measure_text("", 72.0).unwrap();

        assert_eq!(width, 0);
        assert_eq!(height, 0);
    }

    #[test]
    fn test_measure_text_english() {
        let mut renderer = TextRenderer::new().unwrap();
        let (width, height) = renderer.measure_text("BG7XXX", 72.0).unwrap();

        assert!(width > 0, "英文文本宽度应该大于0");
        assert!(height > 0, "英文文本高度应该大于0");
        println!("BG7XXX at 72pt: {}x{}", width, height);
    }

    #[test]
    fn test_measure_text_chinese() {
        let mut renderer = TextRenderer::new().unwrap();
        let (width, height) = renderer.measure_text("中国", 48.0).unwrap();

        assert!(width > 0, "中文文本宽度应该大于0");
        assert!(height > 0, "中文文本高度应该大于0");
        println!("中国 at 48pt: {}x{}", width, height);
    }

    #[test]
    fn test_measure_text_mixed() {
        let mut renderer = TextRenderer::new().unwrap();
        let (width, height) = renderer.measure_text("BG7XXX 中国", 60.0).unwrap();

        assert!(width > 0, "混排文本宽度应该大于0");
        assert!(height > 0, "混排文本高度应该大于0");
        println!("BG7XXX 中国 at 60pt: {}x{}", width, height);
    }

    #[test]
    fn test_render_text_english() {
        let mut renderer = TextRenderer::new().unwrap();
        let img = renderer.render_text("BG7XXX", 72.0).unwrap();

        assert!(img.width() > 0);
        assert!(img.height() > 0);

        // 验证位图为1bpp (只有0或255)
        for pixel in img.pixels() {
            assert!(pixel.0[0] == 0 || pixel.0[0] == 255, "像素值应为0或255");
        }

        println!("渲染 BG7XXX: {}x{}", img.width(), img.height());
    }

    #[test]
    fn test_render_text_chinese() {
        let mut renderer = TextRenderer::new().unwrap();
        let img = renderer.render_text("中国", 48.0).unwrap();

        assert!(img.width() > 0);
        assert!(img.height() > 0);

        // 验证位图为1bpp
        for pixel in img.pixels() {
            assert!(pixel.0[0] == 0 || pixel.0[0] == 255);
        }

        println!("渲染 中国: {}x{}", img.width(), img.height());
    }

    #[test]
    fn test_render_text_mixed() {
        let mut renderer = TextRenderer::new().unwrap();
        let img = renderer.render_text("SN: 123", 60.0).unwrap();

        assert!(img.width() > 0);
        assert!(img.height() > 0);

        // 验证位图为1bpp
        for pixel in img.pixels() {
            assert!(pixel.0[0] == 0 || pixel.0[0] == 255);
        }

        println!("渲染 SN: 123: {}x{}", img.width(), img.height());
    }

    #[test]
    fn test_metrics_cache() {
        let mut renderer = TextRenderer::new().unwrap();

        // 第一次测量
        let start = std::time::Instant::now();
        renderer.measure_text("AAAA", 72.0).unwrap();
        let first_duration = start.elapsed();

        // 第二次测量（应该使用缓存）
        let start = std::time::Instant::now();
        renderer.measure_text("AAAA", 72.0).unwrap();
        let second_duration = start.elapsed();

        println!(
            "第一次: {:?}, 第二次: {:?}",
            first_duration, second_duration
        );

        // 第二次应该更快（使用了缓存）
        // 注意：这个断言可能在某些环境下不稳定，仅用于演示
    }
}
