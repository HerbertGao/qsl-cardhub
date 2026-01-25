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
    /// TSPL 指令字节数组（包含二进制位图数据）
    pub fn generate(
        &self,
        result: RenderResult,
        paper_width_mm: f32,
        paper_height_mm: f32,
    ) -> Result<Vec<u8>> {
        let mut tspl: Vec<u8> = Vec::new();

        // 纸张配置（使用 \r\n 作为行尾符，TSPL 标准要求）
        tspl.extend_from_slice(format!("SIZE {} mm, {} mm\r\n", paper_width_mm, paper_height_mm).as_bytes());
        tspl.extend_from_slice(b"GAP 2 mm, 0 mm\r\n");
        // DIRECTION 1: 打印方向旋转 180 度
        // 这是针对实际打印机硬件测试后确定的正确方向，确保标签正向出纸时内容朝向正确
        // DIRECTION 0 会导致标签内容上下颠倒
        tspl.extend_from_slice(b"DIRECTION 1\r\n");
        tspl.extend_from_slice(b"CLS\r\n");

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
                    let width = bitmap.width();
                    let height = bitmap.height();
                    let bytes_per_row = ((width + 7) / 8) as usize;
                    log::info!(
                        "生成位图[{}]: {}x{} 像素, {}字节/行, 总{}字节 at ({}, {})",
                        i, width, height, bytes_per_row, bytes_per_row * height as usize, x, y
                    );
                    tspl.extend_from_slice(&self.generate_bitmap_command(*x, *y, bitmap)?);
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
                    tspl.extend_from_slice(&self.generate_barcode_command(barcode)?);
                }

                // 绘制边框
                if let Some(border_config) = border {
                    log::debug!("生成边框指令");
                    tspl.extend_from_slice(&self.generate_border_command(&border_config));
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
                tspl.extend_from_slice(&self.generate_bitmap_command(0, 0, &canvas)?);

                log::info!("全位图模式: 画布 {}x{}", canvas_size.0, canvas_size.1);
            }
        }

        // 打印命令
        tspl.extend_from_slice(b"PRINT 1\r\n");

        // 输出 DEBUG 日志，显示生成的 TSPL 指令内容
        log::debug!("TSPL指令生成完成，总长度: {} 字节", tspl.len());
        self.log_tspl_content(&tspl);

        Ok(tspl)
    }

    /// 从灰度图像直接生成 TSPL 指令
    ///
    /// 用于打印机后端的 print_image 方法，直接从图像生成完整的 TSPL 打印指令
    ///
    /// # 参数
    /// - `image`: 灰度图像
    /// - `paper_width_mm`: 纸张宽度(mm)
    /// - `paper_height_mm`: 纸张高度(mm)
    ///
    /// # 返回
    /// TSPL 指令字节数组（包含二进制位图数据）
    pub fn generate_from_image(
        &self,
        image: &GrayImage,
        paper_width_mm: f32,
        paper_height_mm: f32,
    ) -> Result<Vec<u8>> {
        log::info!(
            "从图像生成 TSPL 指令: 图像 {}x{}, 纸张 {}x{} mm",
            image.width(),
            image.height(),
            paper_width_mm,
            paper_height_mm
        );

        let mut tspl: Vec<u8> = Vec::new();

        // 纸张配置
        tspl.extend_from_slice(
            format!("SIZE {} mm, {} mm\r\n", paper_width_mm, paper_height_mm).as_bytes(),
        );
        tspl.extend_from_slice(b"GAP 0 mm, 0 mm\r\n");
        // DIRECTION 1,0: 打印方向旋转 180 度，镜像关闭
        tspl.extend_from_slice(b"DIRECTION 1,0\r\n");
        tspl.extend_from_slice(b"CLS\r\n");

        // 生成位图指令
        tspl.extend_from_slice(&self.generate_bitmap_command(0, 0, image)?);

        // 打印命令
        tspl.extend_from_slice(b"PRINT 1\r\n");

        log::debug!("TSPL 指令生成完成，总长度: {} 字节", tspl.len());
        self.log_tspl_content(&tspl);

        Ok(tspl)
    }

    /// 将 TSPL 指令内容输出到 DEBUG 日志
    /// 由于 TSPL 指令包含二进制数据（BITMAP），需要特殊处理
    fn log_tspl_content(&self, tspl: &[u8]) {
        let content = String::from_utf8_lossy(tspl);
        let mut readable_content = String::new();

        // 按行处理，对 BITMAP 指令中的二进制数据进行摘要显示
        for line in content.split("\r\n") {
            if line.starts_with("BITMAP ") {
                // 解析 BITMAP 指令: BITMAP x,y,width_bytes,height,mode,<binary_data>
                // 找到第5个逗号的位置（mode参数之后）
                let mut comma_count = 0;
                let mut comma_pos = None;
                for (i, c) in line.char_indices() {
                    if c == ',' {
                        comma_count += 1;
                        if comma_count == 5 {
                            comma_pos = Some(i + 1);
                            break;
                        }
                    }
                }
                if let Some(pos) = comma_pos {
                    // 提取 BITMAP 头部（到 mode 参数之后的逗号为止）
                    let header = &line[..pos];
                    // 计算二进制数据长度
                    let binary_len = line.len() - header.len();
                    readable_content.push_str(&format!("{}<binary: {} bytes>\n", header, binary_len));
                } else {
                    readable_content.push_str(line);
                    readable_content.push('\n');
                }
            } else if !line.is_empty() {
                readable_content.push_str(line);
                readable_content.push('\n');
            }
        }

        log::debug!("TSPL指令内容:\n{}", readable_content.trim_end());
    }

    /// 生成 BITMAP 指令
    ///
    /// TSPL BITMAP 格式: BITMAP x,y,width,height,mode,data
    /// - mode: 0 = OVERWRITE
    /// - data: 二进制数据，每行从左到右，按字节对齐
    ///
    /// 在当前实现中（基于实测打印结果，实际打印正确）：
    /// - 0 = 打印黑点
    /// - 1 = 不打印（白色）
    fn generate_bitmap_command(&self, x: u32, y: u32, bitmap: &GrayImage) -> Result<Vec<u8>> {
        let width = bitmap.width();
        let height = bitmap.height();

        // 计算每行字节数（向上取整到8的倍数）
        let bytes_per_row = ((width + 7) / 8) as usize;

        // 转换为1bpp数据
        let mut bitmap_data = Vec::with_capacity(bytes_per_row * height as usize);

        for row_y in 0..height {
            // 初始化为 0xFF（全部为1=白色/不打印）
            // 这样 padding 位也是白色，不会打印出黑线
            let mut row_bytes = vec![0xFFu8; bytes_per_row];

            for col_x in 0..width {
                let pixel = bitmap.get_pixel(col_x, row_y);

                // 渲染后的位图: 文字=黑色(0), 背景=白色(255)
                // TSPL 位图中: 0=打印(黑), 1=不打印(白)
                // 像素值 < 128 (偏黑/文字) -> 清除位为0 (打印黑点)
                // 像素值 >= 128 (偏白/背景) -> 保持位为1 (不打印)
                if pixel.0[0] < 128 {
                    let byte_idx = (col_x / 8) as usize;
                    let bit_idx = 7 - (col_x % 8); // MSB first
                    row_bytes[byte_idx] &= !(1 << bit_idx); // 清除该位
                }
            }

            bitmap_data.extend_from_slice(&row_bytes);
        }

        // 生成 BITMAP 指令（使用二进制数据）
        // BITMAP x, y, width_bytes, height, mode, data
        // mode 0 = OVERWRITE
        let mut result = Vec::new();
        result.extend_from_slice(
            format!("BITMAP {},{},{},{},0,", x, y, bytes_per_row, height).as_bytes()
        );
        result.extend_from_slice(&bitmap_data);
        // BITMAP 数据后需要换行以分隔下一条指令
        result.extend_from_slice(b"\r\n");

        Ok(result)
    }

    /// 生成 BARCODE 指令
    ///
    /// TSPL BARCODE 格式: BARCODE x,y,"type",height,readable,rotation,narrow,wide,"data"
    fn generate_barcode_command(
        &self,
        barcode: &crate::printer::render_pipeline::BarcodeElement,
    ) -> Result<Vec<u8>> {
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
        let readable = if barcode.human_readable { 1 } else { 0 };
        Ok(format!(
            "BARCODE {},{},\"{}\",{},{},0,2,2,\"{}\"\r\n",
            barcode.x, barcode.y, tspl_type, barcode.height, readable, barcode.content
        ).into_bytes())
    }

    /// 生成 BOX 边框指令
    fn generate_border_command(&self, border: &crate::printer::layout_engine::BorderConfig) -> Vec<u8> {
        // BOX x_start, y_start, x_end, y_end, line_thickness
        format!(
            "BOX {},{},{},{},{}\r\n",
            border.x,
            border.y,
            border.x + border.width,
            border.y + border.height,
            border.thickness
        ).into_bytes()
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
        let tspl_bytes = generator.generate(render_result, 76.0, 130.0).unwrap();
        let tspl = String::from_utf8_lossy(&tspl_bytes);

        println!("生成的TSPL指令长度: {} 字节", tspl_bytes.len());

        // 验证指令内容（只检查文本部分）
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
        let tspl_bytes = generator.generate(render_result, 76.0, 130.0).unwrap();
        let tspl = String::from_utf8_lossy(&tspl_bytes);

        println!("生成的TSPL指令长度 (全位图): {} 字节", tspl_bytes.len());

        // 验证指令内容（只检查文本部分）
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

        let cmd_bytes = generator.generate_bitmap_command(10, 20, &bitmap).unwrap();
        let cmd = String::from_utf8_lossy(&cmd_bytes);

        println!("位图指令长度: {} 字节", cmd_bytes.len());

        // 验证指令格式（mode=0 OVERWRITE）
        assert!(cmd.starts_with("BITMAP 10,20,1,8,0,")); // x=10, y=20, width_bytes=1, height=8, mode=0

        // 8x8 位图，每行1字节，共8字节数据 + 指令头部 + \r\n
        // 指令头部 "BITMAP 10,20,1,8,0," = 19 字节 + 8 字节数据 + 2 字节换行 = 29 字节
        assert_eq!(cmd_bytes.len(), 19 + 8 + 2);
    }
}
