// PDF 后端 v2
//
// 接收 RenderResult 并生成 PNG/PDF 文件

use crate::printer::barcode_renderer::BarcodeRenderer;
use crate::printer::render_pipeline::{BarcodeElement, RenderResult};
use anyhow::{Context, Result};
use image::{GrayImage, ImageBuffer, Luma, Rgb, RgbImage};
use std::fs;
use std::path::PathBuf;

/// PDF 后端 v2
///
/// 功能：
/// - 接收 RenderResult 并渲染为图像
/// - 支持两种渲染模式（混合/全位图）
/// - 保存为 PNG 格式
pub struct PdfBackendV2 {
    /// 输出目录
    output_dir: PathBuf,
    /// 条形码渲染器
    barcode_renderer: BarcodeRenderer,
}

impl PdfBackendV2 {
    /// 创建新的 PDF 后端
    pub fn new(output_dir: PathBuf) -> Result<Self> {
        // 确保输出目录存在
        if !output_dir.exists() {
            fs::create_dir_all(&output_dir).context("创建输出目录失败")?;
        }

        Ok(Self {
            output_dir,
            barcode_renderer: BarcodeRenderer::new(),
        })
    }

    /// 使用 Downloads 目录创建后端
    pub fn with_downloads_dir() -> Result<Self> {
        let downloads_dir = dirs::download_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));

        Self::new(downloads_dir)
    }

    /// 渲染 RenderResult 并保存为文件
    ///
    /// # 参数
    /// - `result`: 渲染结果
    ///
    /// # 返回
    /// PNG 文件路径
    pub fn render(&mut self, result: RenderResult) -> Result<PathBuf> {
        log::info!("PDF后端开始渲染");

        // 根据渲染模式生成画布
        let canvas = match result {
            RenderResult::MixedMode {
                bitmaps,
                native_barcodes,
                canvas_size,
                border,
            } => {
                log::info!("处理混合模式结果");
                self.render_mixed_mode(bitmaps, native_barcodes, canvas_size, border)?
            }
            RenderResult::FullBitmap { canvas, .. } => {
                log::info!("处理全位图模式结果");
                canvas
            }
        };

        // 转换为RGB图像
        let rgb_canvas = self.gray_to_rgb(&canvas);

        // 生成文件名
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("qsl_{}.png", timestamp);
        let png_path = self.output_dir.join(&filename);

        // 保存PNG
        rgb_canvas
            .save(&png_path)
            .with_context(|| format!("保存PNG到 {} 失败", png_path.display()))?;

        log::info!("✅ 保存PNG: {}", png_path.display());

        Ok(png_path)
    }

    /// 渲染混合模式结果
    fn render_mixed_mode(
        &mut self,
        bitmaps: Vec<(u32, u32, GrayImage)>,
        native_barcodes: Vec<BarcodeElement>,
        canvas_size: (u32, u32),
        border: Option<crate::printer::layout_engine::BorderConfig>,
    ) -> Result<GrayImage> {
        log::debug!(
            "混合模式: {} 个位图, {} 个条码",
            bitmaps.len(),
            native_barcodes.len()
        );

        // 创建白色背景画布
        let mut canvas = ImageBuffer::from_pixel(canvas_size.0, canvas_size.1, Luma([255u8]));

        // 叠加文本位图
        for (i, (x, y, bitmap)) in bitmaps.iter().enumerate() {
            log::debug!(
                "叠加位图[{}]: {}x{} at ({}, {})",
                i,
                bitmap.width(),
                bitmap.height(),
                x,
                y
            );
            self.overlay(&mut canvas, bitmap, *x, *y);
        }

        // 渲染条形码（PDF中也渲染为位图）
        for (i, barcode) in native_barcodes.iter().enumerate() {
            log::debug!(
                "渲染条码[{}]: \"{}\" ({}) at ({}, {})",
                i,
                barcode.content,
                barcode.barcode_type,
                barcode.x,
                barcode.y
            );

            let barcode_bitmap = self
                .barcode_renderer
                .render_barcode(&barcode.content, &barcode.barcode_type, barcode.height)
                .with_context(|| format!("渲染条码 {} 失败", barcode.content))?;

            self.overlay(&mut canvas, &barcode_bitmap, barcode.x, barcode.y);
        }

        // 绘制边框
        if let Some(border_config) = border {
            self.draw_border(&mut canvas, &border_config);
        }

        Ok(canvas)
    }

    /// 叠加位图到画布
    fn overlay(&self, canvas: &mut GrayImage, bitmap: &GrayImage, x: u32, y: u32) {
        for (bx, by, pixel) in bitmap.enumerate_pixels() {
            let cx = x + bx;
            let cy = y + by;

            if cx < canvas.width() && cy < canvas.height() {
                // 只叠加黑色像素
                if pixel.0[0] == 0 {
                    canvas.put_pixel(cx, cy, *pixel);
                }
            }
        }
    }

    /// 绘制边框
    fn draw_border(
        &self,
        canvas: &mut GrayImage,
        border: &crate::printer::layout_engine::BorderConfig,
    ) {
        let x = border.x;
        let y = border.y;
        let width = border.width;
        let height = border.height;
        let thickness = border.thickness;

        // 绘制四条边
        for ty in 0..thickness {
            // 上边
            for tx in 0..width {
                let px = x + tx;
                let py = y + ty;
                if px < canvas.width() && py < canvas.height() {
                    canvas.put_pixel(px, py, Luma([0u8]));
                }
            }

            // 下边
            for tx in 0..width {
                let px = x + tx;
                let py = y + height - thickness + ty;
                if px < canvas.width() && py < canvas.height() {
                    canvas.put_pixel(px, py, Luma([0u8]));
                }
            }
        }

        for ty in 0..height {
            // 左边
            for tx in 0..thickness {
                let px = x + tx;
                let py = y + ty;
                if px < canvas.width() && py < canvas.height() {
                    canvas.put_pixel(px, py, Luma([0u8]));
                }
            }

            // 右边
            for tx in 0..thickness {
                let px = x + width - thickness + tx;
                let py = y + ty;
                if px < canvas.width() && py < canvas.height() {
                    canvas.put_pixel(px, py, Luma([0u8]));
                }
            }
        }
    }

    /// 灰度图转RGB图
    fn gray_to_rgb(&self, gray: &GrayImage) -> RgbImage {
        ImageBuffer::from_fn(gray.width(), gray.height(), |x, y| {
            let gray_value = gray.get_pixel(x, y).0[0];
            Rgb([gray_value, gray_value, gray_value])
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::template_v2::TemplateV2Config;
    use crate::printer::layout_engine::LayoutEngine;
    use crate::printer::render_pipeline::RenderPipeline;
    use crate::printer::template_engine::TemplateEngine;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn test_render_mixed_mode() {
        let temp_dir = TempDir::new().unwrap();
        let mut backend = PdfBackendV2::new(temp_dir.path().to_path_buf()).unwrap();

        // 准备测试数据
        let config = TemplateV2Config::default_qsl_card_v2();
        let mut data = HashMap::new();
        data.insert("task_name".to_string(), "测试".to_string());
        data.insert("callsign".to_string(), "BG7XXX".to_string());
        data.insert("sn".to_string(), "001".to_string());
        data.insert("qty".to_string(), "100".to_string());

        let resolved = TemplateEngine::resolve(&config, &data).unwrap();
        let mut layout_engine = LayoutEngine::new().unwrap();
        let layout_result = layout_engine.layout(&config, resolved).unwrap();

        let mut pipeline = RenderPipeline::new().unwrap();
        let output_config = crate::config::template_v2::OutputConfig {
            mode: "text_bitmap_plus_native_barcode".to_string(),
            threshold: 160,
        };
        let render_result = pipeline.render(layout_result, &output_config).unwrap();

        // 渲染并保存
        let png_path = backend.render(render_result).unwrap();

        assert!(png_path.exists());
        println!("保存到: {}", png_path.display());
    }

    #[test]
    fn test_render_full_bitmap() {
        let temp_dir = TempDir::new().unwrap();
        let mut backend = PdfBackendV2::new(temp_dir.path().to_path_buf()).unwrap();

        let config = TemplateV2Config::default_qsl_card_v2();
        let mut data = HashMap::new();
        data.insert("task_name".to_string(), "测试".to_string());
        data.insert("callsign".to_string(), "BG7XXX".to_string());
        data.insert("sn".to_string(), "001".to_string());
        data.insert("qty".to_string(), "100".to_string());

        let resolved = TemplateEngine::resolve(&config, &data).unwrap();
        let mut layout_engine = LayoutEngine::new().unwrap();
        let layout_result = layout_engine.layout(&config, resolved).unwrap();

        let mut pipeline = RenderPipeline::new().unwrap();
        let output_config = crate::config::template_v2::OutputConfig {
            mode: "full_bitmap".to_string(),
            threshold: 160,
        };
        let render_result = pipeline.render(layout_result, &output_config).unwrap();

        let png_path = backend.render(render_result).unwrap();

        assert!(png_path.exists());
        println!("保存到: {}", png_path.display());
    }
}
