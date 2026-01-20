// 渲染管道
//
// 协调文本渲染、条形码渲染和后端输出
// 支持两种渲染模式: 文本位图+原生条码 / 全位图

use crate::config::template_v2::OutputConfig;
use crate::printer::barcode_renderer::BarcodeRenderer;
use crate::printer::layout_engine::{BorderConfig, ElementType, LayoutResult, LayoutedElement};
use crate::printer::text_renderer::TextRenderer;
use anyhow::{Context, Result};
use image::{GrayImage, ImageBuffer, Luma};

/// 条形码元素信息（用于原生条码模式）
#[derive(Debug, Clone)]
pub struct BarcodeElement {
    /// 条形码内容
    pub content: String,
    /// 条形码类型
    pub barcode_type: String,
    /// x坐标(dots)
    pub x: u32,
    /// y坐标(dots)
    pub y: u32,
    /// 高度(dots)
    pub height: u32,
    /// 左右留白(dots)
    pub quiet_zone: u32,
    /// 是否显示人类可读文本
    pub human_readable: bool,
}

/// 渲染结果
#[derive(Debug, Clone)]
pub enum RenderResult {
    /// 混合模式: 文本位图 + 原生条码
    MixedMode {
        /// 文本位图列表: (x, y, bitmap)
        bitmaps: Vec<(u32, u32, GrayImage)>,
        /// 原生条形码元素
        native_barcodes: Vec<BarcodeElement>,
        /// 画布尺寸
        canvas_size: (u32, u32),
        /// 边框配置
        border: Option<BorderConfig>,
    },
    /// 全位图模式: 所有元素合成到一个画布
    FullBitmap {
        /// 完整画布
        canvas: GrayImage,
        /// 画布尺寸
        canvas_size: (u32, u32),
    },
}

/// 渲染管道
pub struct RenderPipeline {
    text_renderer: TextRenderer,
    barcode_renderer: BarcodeRenderer,
}

impl RenderPipeline {
    /// 创建新的渲染管道
    pub fn new() -> Result<Self> {
        Ok(Self {
            text_renderer: TextRenderer::new()?,
            barcode_renderer: BarcodeRenderer::new(),
        })
    }

    /// 执行渲染
    ///
    /// # 参数
    /// - `layout_result`: 布局结果
    /// - `output_config`: 输出配置
    ///
    /// # 返回
    /// 渲染结果（根据模式返回不同类型）
    pub fn render(
        &mut self,
        layout_result: LayoutResult,
        output_config: &OutputConfig,
    ) -> Result<RenderResult> {
        log::info!("开始渲染，模式: {}", output_config.mode);

        match output_config.mode.as_str() {
            "text_bitmap_plus_native_barcode" => {
                self.render_mixed_mode(layout_result, output_config)
            }
            "full_bitmap" => self.render_full_bitmap_mode(layout_result, output_config),
            _ => anyhow::bail!("不支持的渲染模式: {}", output_config.mode),
        }
    }

    /// 渲染混合模式: 文本位图 + 原生条码
    fn render_mixed_mode(
        &mut self,
        layout_result: LayoutResult,
        _output_config: &OutputConfig,
    ) -> Result<RenderResult> {
        log::info!("渲染混合模式");

        let mut bitmaps = Vec::new();
        let mut native_barcodes = Vec::new();

        for element in &layout_result.elements {
            match element.element_type {
                ElementType::Text => {
                    // 渲染文本为位图
                    let (x, y, bitmap) = self.render_text_element(element)?;
                    bitmaps.push((x, y, bitmap));
                }
                ElementType::Barcode => {
                    // 保留原生条形码信息
                    let barcode_elem = self.extract_barcode_element(element)?;
                    native_barcodes.push(barcode_elem);
                }
            }
        }

        log::info!(
            "✅ 混合模式渲染完成: {} 个位图, {} 个原生条码",
            bitmaps.len(),
            native_barcodes.len()
        );

        Ok(RenderResult::MixedMode {
            bitmaps,
            native_barcodes,
            canvas_size: (layout_result.canvas_width, layout_result.canvas_height),
            border: layout_result.border,
        })
    }

    /// 渲染全位图模式: 所有元素合成到一个画布
    fn render_full_bitmap_mode(
        &mut self,
        layout_result: LayoutResult,
        _output_config: &OutputConfig,
    ) -> Result<RenderResult> {
        log::info!("渲染全位图模式");

        // 1. 创建白色背景画布
        let mut canvas = self.create_canvas(layout_result.canvas_width, layout_result.canvas_height);

        // 2. 渲染所有元素到画布
        for element in &layout_result.elements {
            match element.element_type {
                ElementType::Text => {
                    // 渲染文本位图并叠加
                    let (x, y, bitmap) = self.render_text_element(element)?;
                    self.overlay(&mut canvas, &bitmap, x, y);
                }
                ElementType::Barcode => {
                    // 渲染条形码位图并叠加
                    let (x, y, bitmap) = self.render_barcode_element(element)?;
                    self.overlay(&mut canvas, &bitmap, x, y);
                }
            }
        }

        // 3. 绘制边框（如果启用）
        if let Some(border) = &layout_result.border {
            self.draw_border(&mut canvas, border);
        }

        log::info!("✅ 全位图模式渲染完成");

        Ok(RenderResult::FullBitmap {
            canvas,
            canvas_size: (layout_result.canvas_width, layout_result.canvas_height),
        })
    }

    /// 渲染文本元素为位图
    fn render_text_element(&mut self, element: &LayoutedElement) -> Result<(u32, u32, GrayImage)> {
        let font_size = element
            .font_size
            .ok_or_else(|| anyhow::anyhow!("文本元素缺少font_size字段"))?;

        log::debug!(
            "渲染文本: \"{}\" ({}pt) at ({}, {})",
            element.content,
            font_size,
            element.x,
            element.y
        );

        let bitmap = self
            .text_renderer
            .render_text(&element.content, font_size)
            .with_context(|| format!("渲染文本 \"{}\" 失败", element.content))?;

        Ok((element.x, element.y, bitmap))
    }

    /// 渲染条形码元素为位图
    fn render_barcode_element(&mut self, element: &LayoutedElement) -> Result<(u32, u32, GrayImage)> {
        let barcode_config = element
            .barcode_config
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("条形码元素缺少barcode_config字段"))?;

        log::debug!(
            "渲染条形码: \"{}\" ({}) at ({}, {})",
            element.content,
            barcode_config.barcode_type,
            element.x,
            element.y
        );

        let bitmap = self
            .barcode_renderer
            .render_barcode(
                &element.content,
                &barcode_config.barcode_type,
                barcode_config.height_dots,
            )
            .with_context(|| format!("渲染条形码 \"{}\" 失败", element.content))?;

        Ok((element.x, element.y, bitmap))
    }

    /// 提取条形码元素信息（用于原生条码模式）
    fn extract_barcode_element(&self, element: &LayoutedElement) -> Result<BarcodeElement> {
        let barcode_config = element
            .barcode_config
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("条形码元素缺少barcode_config字段"))?;

        log::debug!(
            "提取原生条码: \"{}\" ({}) at ({}, {})",
            element.content,
            barcode_config.barcode_type,
            element.x,
            element.y
        );

        Ok(BarcodeElement {
            content: element.content.clone(),
            barcode_type: barcode_config.barcode_type.clone(),
            x: element.x,
            y: element.y,
            height: barcode_config.height_dots,
            quiet_zone: barcode_config.quiet_zone_dots,
            human_readable: barcode_config.human_readable,
        })
    }

    /// 创建白色背景画布
    fn create_canvas(&self, width: u32, height: u32) -> GrayImage {
        ImageBuffer::from_pixel(width, height, Luma([255u8]))
    }

    /// 叠加位图到画布
    ///
    /// # 参数
    /// - `canvas`: 目标画布
    /// - `bitmap`: 要叠加的位图
    /// - `x`: 目标x坐标
    /// - `y`: 目标y坐标
    fn overlay(&self, canvas: &mut GrayImage, bitmap: &GrayImage, x: u32, y: u32) {
        let canvas_width = canvas.width();
        let canvas_height = canvas.height();

        for (bx, by, pixel) in bitmap.enumerate_pixels() {
            let cx = x + bx;
            let cy = y + by;

            // 边界检查
            if cx < canvas_width && cy < canvas_height {
                // 只叠加黑色像素(0)，保持白色背景(255)
                if pixel.0[0] == 0 {
                    canvas.put_pixel(cx, cy, *pixel);
                }
            } else {
                // 第一次超出边界时警告
                if bx == 0 && by == 0 {
                    log::warn!(
                        "位图部分超出画布边界: 位图尺寸{}x{} at ({}, {}), 画布尺寸{}x{}",
                        bitmap.width(),
                        bitmap.height(),
                        x,
                        y,
                        canvas_width,
                        canvas_height
                    );
                }
                break;
            }
        }
    }

    /// 绘制边框
    fn draw_border(&self, canvas: &mut GrayImage, border: &BorderConfig) {
        log::debug!(
            "绘制边框: {}x{} at ({}, {}), 线宽 {}",
            border.width,
            border.height,
            border.x,
            border.y,
            border.thickness
        );

        let x = border.x;
        let y = border.y;
        let width = border.width;
        let height = border.height;
        let thickness = border.thickness;

        // 绘制四条边
        // 上边
        for ty in 0..thickness {
            for tx in 0..width {
                let px = x + tx;
                let py = y + ty;
                if px < canvas.width() && py < canvas.height() {
                    canvas.put_pixel(px, py, Luma([0u8]));
                }
            }
        }

        // 下边
        for ty in 0..thickness {
            for tx in 0..width {
                let px = x + tx;
                let py = y + height - thickness + ty;
                if px < canvas.width() && py < canvas.height() {
                    canvas.put_pixel(px, py, Luma([0u8]));
                }
            }
        }

        // 左边
        for ty in 0..height {
            for tx in 0..thickness {
                let px = x + tx;
                let py = y + ty;
                if px < canvas.width() && py < canvas.height() {
                    canvas.put_pixel(px, py, Luma([0u8]));
                }
            }
        }

        // 右边
        for ty in 0..height {
            for tx in 0..thickness {
                let px = x + width - thickness + tx;
                let py = y + ty;
                if px < canvas.width() && py < canvas.height() {
                    canvas.put_pixel(px, py, Luma([0u8]));
                }
            }
        }
    }
}

impl Default for RenderPipeline {
    fn default() -> Self {
        Self::new().expect("创建RenderPipeline失败")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::template_v2::{TemplateV2Config, OutputConfig};
    use crate::printer::layout_engine::LayoutEngine;
    use crate::printer::template_engine::TemplateEngine;
    use std::collections::HashMap;

    #[test]
    fn test_create_canvas() {
        let pipeline = RenderPipeline::new().unwrap();
        let canvas = pipeline.create_canvas(608, 1039);

        assert_eq!(canvas.width(), 608);
        assert_eq!(canvas.height(), 1039);

        // 验证所有像素都是白色
        for pixel in canvas.pixels() {
            assert_eq!(pixel.0[0], 255);
        }
    }

    #[test]
    fn test_overlay() {
        let pipeline = RenderPipeline::new().unwrap();
        let mut canvas = pipeline.create_canvas(100, 100);

        // 创建一个小的黑色方块
        let mut bitmap = ImageBuffer::from_pixel(10, 10, Luma([255u8]));
        for x in 0..10 {
            for y in 0..10 {
                bitmap.put_pixel(x, y, Luma([0u8]));
            }
        }

        // 叠加到画布
        pipeline.overlay(&mut canvas, &bitmap, 20, 30);

        // 验证叠加区域是黑色
        for x in 20..30 {
            for y in 30..40 {
                assert_eq!(canvas.get_pixel(x, y).0[0], 0);
            }
        }

        // 验证其他区域是白色
        assert_eq!(canvas.get_pixel(0, 0).0[0], 255);
        assert_eq!(canvas.get_pixel(50, 50).0[0], 255);
    }

    #[test]
    fn test_draw_border() {
        let pipeline = RenderPipeline::new().unwrap();
        let mut canvas = pipeline.create_canvas(100, 100);

        let border = BorderConfig {
            x: 10,
            y: 10,
            width: 80,
            height: 80,
            thickness: 2,
        };

        pipeline.draw_border(&mut canvas, &border);

        // 验证边框像素是黑色
        assert_eq!(canvas.get_pixel(10, 10).0[0], 0); // 左上角
        assert_eq!(canvas.get_pixel(89, 10).0[0], 0); // 右上角
        assert_eq!(canvas.get_pixel(10, 89).0[0], 0); // 左下角
        assert_eq!(canvas.get_pixel(89, 89).0[0], 0); // 右下角

        // 验证中心区域是白色
        assert_eq!(canvas.get_pixel(50, 50).0[0], 255);
    }

    #[test]
    fn test_render_mixed_mode() {
        let config = TemplateV2Config::default_qsl_card_v2();
        let mut data = HashMap::new();
        data.insert("task_name".to_string(), "测试".to_string());
        data.insert("callsign".to_string(), "BG7XXX".to_string());
        data.insert("sn".to_string(), "001".to_string());
        data.insert("qty".to_string(), "100".to_string());

        let resolved_elements = TemplateEngine::resolve(&config, &data).unwrap();
        let mut layout_engine = LayoutEngine::new().unwrap();
        let layout_result = layout_engine.layout(&config, resolved_elements).unwrap();

        let mut pipeline = RenderPipeline::new().unwrap();
        let output_config = OutputConfig {
            mode: "text_bitmap_plus_native_barcode".to_string(),
            threshold: 160,
        };

        let result = pipeline.render(layout_result, &output_config).unwrap();

        match result {
            RenderResult::MixedMode {
                bitmaps,
                native_barcodes,
                canvas_size,
                border,
            } => {
                assert_eq!(bitmaps.len(), 5, "应该有5个文本位图");
                assert_eq!(native_barcodes.len(), 1, "应该有1个原生条码");
                assert_eq!(canvas_size, (608, 1039));
                assert!(border.is_some(), "应该有边框");

                println!("✓ 混合模式: {} 位图, {} 条码", bitmaps.len(), native_barcodes.len());
            }
            _ => panic!("应该返回MixedMode"),
        }
    }

    #[test]
    fn test_render_full_bitmap_mode() {
        let config = TemplateV2Config::default_qsl_card_v2();
        let mut data = HashMap::new();
        data.insert("task_name".to_string(), "测试".to_string());
        data.insert("callsign".to_string(), "BG7XXX".to_string());
        data.insert("sn".to_string(), "001".to_string());
        data.insert("qty".to_string(), "100".to_string());

        let resolved_elements = TemplateEngine::resolve(&config, &data).unwrap();
        let mut layout_engine = LayoutEngine::new().unwrap();
        let layout_result = layout_engine.layout(&config, resolved_elements).unwrap();

        let mut pipeline = RenderPipeline::new().unwrap();
        let output_config = OutputConfig {
            mode: "full_bitmap".to_string(),
            threshold: 160,
        };

        let result = pipeline.render(layout_result, &output_config).unwrap();

        match result {
            RenderResult::FullBitmap { canvas, canvas_size } => {
                assert_eq!(canvas.width(), 608);
                assert_eq!(canvas.height(), 1039);
                assert_eq!(canvas_size, (608, 1039));

                // 验证画布包含黑色像素（有内容）
                let has_black = canvas.pixels().any(|p| p.0[0] == 0);
                assert!(has_black, "画布应该包含黑色像素");

                println!("✓ 全位图模式: {}x{}", canvas.width(), canvas.height());
            }
            _ => panic!("应该返回FullBitmap"),
        }
    }
}
