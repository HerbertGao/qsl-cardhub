// TSPL 生成器
//
// 接收 RenderResult 并生成 TSPL 指令

use crate::printer::render_pipeline::RenderResult;
use anyhow::Result;
use image::GrayImage;

/// TSPL 生成器
///
/// 功能：
/// - 接收 RenderResult 并生成 TSPL 打印指令
/// - 支持两种渲染模式（混合/全位图）
pub struct TSPLGenerator {
    /// 打印机 DPI（默认 203）
    dpi: u32,
}

impl TSPLGenerator {
    /// 创建新的 TSPL 生成器
    pub fn new() -> Self {
        Self { dpi: 203 }
    }

    /// 创建指定 DPI 的 TSPL 生成器
    pub fn with_dpi(dpi: u32) -> Self {
        Self { dpi }
    }

    /// 从 RenderResult 生成 TSPL 指令
    ///
    /// # 参数
    /// - `result`: 渲染结果
    /// - `paper_width_mm`: 纸张宽度(mm)
    /// - `paper_height_mm`: 纸张高度(mm)
    ///
    /// # 返回
    /// TSPL 指令字符串
    pub fn generate(
        &self,
        result: RenderResult,
        paper_width_mm: f32,
        paper_height_mm: f32,
    ) -> Result<String> {
        let mut tspl = String::new();

        // 纸张配置
        tspl.push_str(&format!("SIZE {} mm, {} mm\n", paper_width_mm, paper_height_mm));
        tspl.push_str("GAP 2 mm, 0 mm\n");
        tspl.push_str("DIRECTION 0\n");
        tspl.push_str("CLS\n");

        // 根据渲染模式生成内容
        match result {
            RenderResult::MixedMode {
                bitmaps,
                native_barcodes,
                canvas_size,
                border,
            } => {
                log::info!("生成混合模式TSPL指令");

                // 生成文本位图指令
                for (i, (x, y, bitmap)) in bitmaps.iter().enumerate() {
                    log::debug!("生成位图[{}]指令: {}x{} at ({}, {})", i, bitmap.width(), bitmap.height(), x, y);
                    tspl.push_str(&self.generate_bitmap_command(*x, *y, bitmap)?);
                }

                // 生成条形码指令
                for (i, barcode) in native_barcodes.iter().enumerate() {
                    log::debug!(
                        "生成条码[{}]指令: \"{}\" ({}) at ({}, {})",
                        i,
                        barcode.content,
                        barcode.barcode_type,
                        barcode.x,
                        barcode.y
                    );
                    tspl.push_str(&self.generate_barcode_command(barcode)?);
                }

                // 绘制边框
                if let Some(border_config) = border {
                    log::debug!("生成边框指令");
                    tspl.push_str(&self.generate_border_command(&border_config));
                }

                log::info!(
                    "混合模式: {} 个位图, {} 个条码, 画布 {}x{}",
                    bitmaps.len(),
                    native_barcodes.len(),
                    canvas_size.0,
                    canvas_size.1
                );
            }
            RenderResult::FullBitmap { canvas, canvas_size } => {
                log::info!("生成全位图模式TSPL指令");

                // 生成完整画布位图指令
                tspl.push_str(&self.generate_bitmap_command(0, 0, &canvas)?);

                log::info!("全位图模式: 画布 {}x{}", canvas_size.0, canvas_size.1);
            }
        }

        // 打印命令
        tspl.push_str("PRINT 1\n");

        Ok(tspl)
    }

    /// 生成 BITMAP 指令
    ///
    /// TSPL BITMAP 格式: BITMAP x,y,width,height,mode,data
    /// - mode: 1 = 1bpp (每像素1位)
    /// - data: 十六进制字符串，每行从左到右，按字节对齐
    fn generate_bitmap_command(&self, x: u32, y: u32, bitmap: &GrayImage) -> Result<String> {
        let width = bitmap.width();
        let height = bitmap.height();

        // 计算每行字节数（向上取整到8的倍数）
        let bytes_per_row = ((width + 7) / 8) as usize;

        // 转换为1bpp数据
        let mut data = Vec::with_capacity(bytes_per_row * height as usize);

        for row_y in 0..height {
            let mut row_bytes = vec![0u8; bytes_per_row];

            for col_x in 0..width {
                let pixel = bitmap.get_pixel(col_x, row_y);

                // 0=黑色应该设置为1，255=白色应该设置为0（TSPL位图格式）
                if pixel.0[0] == 0 {
                    let byte_idx = (col_x / 8) as usize;
                    let bit_idx = 7 - (col_x % 8); // MSB first
                    row_bytes[byte_idx] |= 1 << bit_idx;
                }
            }

            data.extend_from_slice(&row_bytes);
        }

        // 转换为十六进制字符串
        let hex_data: String = data.iter().map(|b| format!("{:02X}", b)).collect();

        // 生成 BITMAP 指令
        // BITMAP x, y, width_bytes, height, mode, data
        Ok(format!(
            "BITMAP {},{},{},{},1,{}\n",
            x, y, bytes_per_row, height, hex_data
        ))
    }

    /// 生成 BARCODE 指令
    ///
    /// TSPL BARCODE 格式: BARCODE x,y,"type",height,readable,rotation,narrow,wide,"data"
    fn generate_barcode_command(
        &self,
        barcode: &crate::printer::render_pipeline::BarcodeElement,
    ) -> Result<String> {
        // 将条码类型转换为TSPL格式
        let tspl_type = match barcode.barcode_type.to_lowercase().as_str() {
            "code128" | "128" => "128",
            _ => &barcode.barcode_type,
        };

        // 生成 BARCODE 指令
        // BARCODE x,y,"type",height,readable,rotation,narrow,wide,"data"
        // 参数说明:
        // - readable: 1 = 显示人类可读文本, 0 = 不显示
        // - rotation: 0 = 不旋转
        // - narrow/wide: 窄条和宽条的宽度（默认 2,2）
        Ok(format!(
            "BARCODE {},{},\"{}\",{},1,0,2,2,\"{}\"\n",
            barcode.x, barcode.y, tspl_type, barcode.height, barcode.content
        ))
    }

    /// 生成 BOX 边框指令
    fn generate_border_command(&self, border: &crate::printer::layout_engine::BorderConfig) -> String {
        // BOX x_start, y_start, x_end, y_end, line_thickness
        format!(
            "BOX {},{},{},{},{}\n",
            border.x,
            border.y,
            border.x + border.width,
            border.y + border.height,
            border.thickness
        )
    }
}

impl Default for TSPLGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::template::{TemplateConfig, OutputConfig};
    use crate::printer::layout_engine::LayoutEngine;
    use crate::printer::render_pipeline::RenderPipeline;
    use crate::printer::template_engine::TemplateEngine;
    use std::collections::HashMap;

    #[test]
    fn test_generate_mixed_mode() {
        // 准备测试数据
        let config = TemplateConfig::default_qsl_card();
        let mut data = HashMap::new();
        data.insert("project_name".to_string(), "CQWW DX".to_string());
        data.insert("callsign".to_string(), "BG7XXX".to_string());
        data.insert("sn".to_string(), "001".to_string());
        data.insert("qty".to_string(), "100".to_string());

        // 完整流程
        let resolved = TemplateEngine::resolve(&config, &data).unwrap();
        let mut layout_engine = LayoutEngine::new().unwrap();
        let layout_result = layout_engine.layout(&config, resolved).unwrap();

        let mut pipeline = RenderPipeline::new().unwrap();
        let output_config = OutputConfig {
            mode: "text_bitmap_plus_native_barcode".to_string(),
            threshold: 160,
        };
        let render_result = pipeline.render(layout_result, &output_config).unwrap();

        // 生成TSPL
        let generator = TSPLGenerator::new();
        let tspl = generator.generate(render_result, 76.0, 130.0).unwrap();

        println!("生成的TSPL指令:\n{}", tspl);

        // 验证指令内容
        assert!(tspl.contains("SIZE 76 mm, 130 mm"));
        assert!(tspl.contains("CLS"));
        assert!(tspl.contains("BITMAP")); // 应该有位图指令
        assert!(tspl.contains("BARCODE")); // 应该有条码指令
        assert!(tspl.contains("BOX")); // 应该有边框指令
        assert!(tspl.contains("PRINT 1"));

        // 验证条码内容
        assert!(tspl.contains("\"BG7XXX\""));
    }

    #[test]
    fn test_generate_full_bitmap() {
        let config = TemplateConfig::default_qsl_card();
        let mut data = HashMap::new();
        data.insert("project_name".to_string(), "测试".to_string());
        data.insert("callsign".to_string(), "BD7AA".to_string());
        data.insert("sn".to_string(), "999".to_string());
        data.insert("qty".to_string(), "50".to_string());

        let resolved = TemplateEngine::resolve(&config, &data).unwrap();
        let mut layout_engine = LayoutEngine::new().unwrap();
        let layout_result = layout_engine.layout(&config, resolved).unwrap();

        let mut pipeline = RenderPipeline::new().unwrap();
        let output_config = OutputConfig {
            mode: "full_bitmap".to_string(),
            threshold: 160,
        };
        let render_result = pipeline.render(layout_result, &output_config).unwrap();

        // 生成TSPL
        let generator = TSPLGenerator::new();
        let tspl = generator.generate(render_result, 76.0, 130.0).unwrap();

        println!("生成的TSPL指令 (全位图):\n{}", tspl);

        // 验证指令内容
        assert!(tspl.contains("SIZE 76 mm, 130 mm"));
        assert!(tspl.contains("CLS"));
        assert!(tspl.contains("BITMAP")); // 应该有位图指令
        assert!(!tspl.contains("BARCODE")); // 不应该有独立的条码指令
        assert!(tspl.contains("PRINT 1"));
    }

    #[test]
    fn test_bitmap_command_generation() {
        let generator = TSPLGenerator::new();

        // 创建一个简单的8x8位图用于测试
        let mut bitmap = GrayImage::new(8, 8);

        // 填充一些黑色像素形成一个十字
        for i in 0..8 {
            bitmap.put_pixel(i, 4, image::Luma([0u8])); // 水平线
            bitmap.put_pixel(4, i, image::Luma([0u8])); // 垂直线
        }

        let cmd = generator.generate_bitmap_command(10, 20, &bitmap).unwrap();

        println!("位图指令:\n{}", cmd);

        // 验证指令格式
        assert!(cmd.starts_with("BITMAP 10,20,1,8,1,")); // x=10, y=20, width_bytes=1, height=8, mode=1
        assert!(cmd.contains('\n'));

        // 十六进制数据应该包含在指令中
        // 每行1字节，8行，每行应该有交叉图案
        assert!(cmd.len() > 30); // 至少有指令头部+16个十六进制字符
    }
}
