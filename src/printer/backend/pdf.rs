// PDF 打印后端
//
// 将 TSPL 指令忠实渲染为 PDF 文件，作为真实打印的预览
// 不做动态布局调整，完全按照 TSPL 坐标渲染

use crate::printer::backend::PrinterBackend;
use crate::printer::barcode_renderer::BarcodeRenderer;
use crate::printer::font_loader::FontLoader;
use crate::printer::text_renderer::TextRenderer;
use anyhow::{Context, Result};
use image::{ImageBuffer, Rgb, RgbImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_hollow_rect_mut};
use imageproc::rect::Rect;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

/// PDF 打印后端
///
/// 特点：
/// - 忠实渲染 TSPL 指令，作为真实打印的预览
/// - 保存为 PNG 和 PDF 两种格式
/// - 分辨率：203 DPI
/// - 支持真实的文本和中文渲染（通过 TSPL TEXT 命令）
/// - 支持 Code128 条形码渲染（通过 TSPL BARCODE 命令）
/// - 支持元数据注释（TITLE, SUBTITLE）
pub struct PdfBackend {
    /// PDF 输出目录（默认为 Downloads）
    output_dir: PathBuf,
    /// 字体加载器（使用 Mutex 实现线程安全的内部可变性）
    font_loader: Mutex<FontLoader>,
}

impl PdfBackend {
    /// 创建新的 PDF 后端
    ///
    /// # 参数
    /// - `output_dir`: PDF 文件输出目录
    pub fn new(output_dir: PathBuf) -> Result<Self> {
        // 确保输出目录存在
        if !output_dir.exists() {
            fs::create_dir_all(&output_dir).context("创建 PDF 输出目录失败")?;
        }

        Ok(Self {
            output_dir,
            font_loader: Mutex::new(FontLoader::new()),
        })
    }

    /// 从系统获取 Downloads 目录
    pub fn with_downloads_dir() -> Result<Self> {
        let downloads_dir = dirs::download_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));

        Self::new(downloads_dir)
    }

    /// 解析 TSPL 指令并生成 PNG（和 PDF，如果可能）
    fn render_tspl(&self, tspl: &str) -> Result<(PathBuf, Option<PathBuf>)> {
        // 解析 TSPL 指令
        let commands = self.parse_tspl(tspl);

        // 提取纸张尺寸
        let (width, height) = self.extract_size(&commands);

        // 创建画布（白色背景）
        let mut img: RgbImage = ImageBuffer::from_pixel(width, height, Rgb([255u8, 255u8, 255u8]));

        // 忠实渲染 TSPL 命令
        self.render_commands(&mut img, &commands)?;

        // 生成文件名（带时间戳）
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename_base = format!("qsl_{}", timestamp);
        let png_path = self.output_dir.join(format!("{}.png", filename_base));

        // 保存为 PNG
        img.save(&png_path).context("保存 PNG 失败")?;

        // 尝试转换为 PDF（如果失败也不影响PNG输出）
        let pdf_path = self.output_dir.join(format!("{}.pdf", filename_base));
        let pdf_result = self.image_to_pdf(&img, width, height, &pdf_path);

        match pdf_result {
            Ok(_) => Ok((png_path, Some(pdf_path))),
            Err(e) => {
                println!("⚠️  PDF 生成失败: {}，已保存 PNG", e);
                Ok((png_path, None))
            }
        }
    }

    /// 将图像转换为 PDF
    ///
    /// 注意：由于依赖版本冲突，PDF 生成功能暂时禁用
    /// 目前只生成 PNG 文件
    fn image_to_pdf(
        &self,
        _img: &RgbImage,
        _width: u32,
        _height: u32,
        _pdf_path: &PathBuf,
    ) -> Result<()> {
        // TODO: 修复 image/imageproc/printpdf 版本冲突后重新启用
        // 目前暂时返回错误，不影响 PNG 生成
        Err(anyhow::anyhow!("PDF 生成功能暂时禁用（依赖版本冲突）"))
    }

    /// 解析 TSPL 指令
    fn parse_tspl(&self, tspl: &str) -> Vec<(String, Vec<String>)> {
        let mut commands = Vec::new();

        for line in tspl.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // 处理注释行（元数据）
            if line.starts_with(';') {
                // 解析元数据注释，格式：; KEY: VALUE
                if let Some(colon_pos) = line.find(':') {
                    let key = line[1..colon_pos].trim().to_uppercase();
                    let value = line[colon_pos + 1..].trim().to_string();
                    // 将注释作为特殊命令存储
                    commands.push((format!("_META_{}", key), vec![value]));
                }
                continue;
            }

            let (cmd, params) = self.parse_tspl_line(line);
            commands.push((cmd, params));
        }

        commands
    }

    /// 解析单行 TSPL 命令
    fn parse_tspl_line(&self, line: &str) -> (String, Vec<String>) {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.is_empty() {
            return (String::new(), Vec::new());
        }

        let cmd = parts[0].to_uppercase();
        if parts.len() < 2 {
            return (cmd, Vec::new());
        }

        // 解析参数（处理逗号和引号）
        let params_str = parts[1];
        let mut params = Vec::new();
        let mut current = String::new();
        let mut in_quote = false;

        for ch in params_str.chars() {
            if ch == '"' {
                in_quote = !in_quote;
            } else if ch == ',' && !in_quote {
                if !current.is_empty() {
                    params.push(current.trim().to_string());
                    current.clear();
                }
            } else {
                current.push(ch);
            }
        }

        if !current.is_empty() {
            params.push(current.trim().to_string());
        }

        (cmd, params)
    }

    /// 从命令列表中提取纸张尺寸
    fn extract_size(&self, commands: &[(String, Vec<String>)]) -> (u32, u32) {
        for (cmd, params) in commands {
            if cmd == "SIZE" && params.len() >= 2 {
                // SIZE 76 mm, 130 mm
                let width_mm: f32 = params[0].replace("mm", "").trim().parse().unwrap_or(76.0);
                let height_mm: f32 = params[1].replace("mm", "").trim().parse().unwrap_or(130.0);
                // 1mm ≈ 8 dots @ 203 DPI
                return ((width_mm * 8.0) as u32, (height_mm * 8.0) as u32);
            }
        }
        // 默认尺寸 76mm x 130mm
        (608, 1040)
    }

    /// 渲染所有 TSPL 命令
    fn render_commands(
        &self,
        img: &mut RgbImage,
        commands: &[(String, Vec<String>)],
    ) -> Result<()> {
        log::info!("开始渲染 TSPL 命令，共 {} 条", commands.len());

        for (cmd, params) in commands {
            // 跳过元数据命令（已经被解析，不需要渲染）
            if cmd.starts_with("_META_") {
                log::debug!("跳过元数据: {} = {:?}", cmd, params);
                continue;
            }

            match cmd.as_str() {
                "TEXT" => self.render_text(img, params),
                "BARCODE" => self.render_barcode(img, params),
                "BAR" => self.render_bar(img, params),
                "BOX" => self.render_box(img, params),
                "SIZE" | "GAP" | "DIRECTION" | "CLS" | "PRINT" => {
                    // 这些命令不需要渲染
                    log::debug!("忽略控制命令: {}", cmd);
                }
                _ => {
                    log::warn!("未知的 TSPL 命令: {}", cmd);
                }
            }
        }

        log::info!("TSPL 命令渲染完成");
        Ok(())
    }

    /// 渲染 TEXT 命令（支持居中对齐）
    fn render_text(&self, img: &mut RgbImage, params: &[String]) {
        if params.len() < 7 {
            log::warn!("TEXT 命令参数不足: {:?}", params);
            return;
        }

        let x: i32 = params[0].parse().unwrap_or(0);
        let y: i32 = params[1].parse().unwrap_or(0);
        // let font_num = &params[2]; // TSPL 字体号（PDF 后端不使用）
        // let rotation = &params[3]; // 旋转（暂不支持）
        let x_scale: i32 = params[4].parse().unwrap_or(1);
        let y_scale: i32 = params[5].parse().unwrap_or(1);
        let text = params[6].trim_matches('"');

        // 计算字体大小（基于缩放系数）
        let font_size = 16.0 * x_scale.max(y_scale) as f32;

        // 使用 TextRenderer 渲染文本
        let mut text_renderer = match TextRenderer::new() {
            Ok(renderer) => renderer,
            Err(e) => {
                log::error!("创建TextRenderer失败: {}", e);
                return;
            }
        };

        // 检测是否为中文（用于选择字体）
        let is_chinese = text
            .chars()
            .any(|c| c as u32 > 0x4E00 && (c as u32) < 0x9FA5);

        // 检测是否需要居中（x 坐标接近画布中心 304，范围 280-330）
        let should_center = x >= 280 && x <= 330;

        if should_center {
            // 居中渲染
            if let Err(e) = text_renderer.draw_centered_text(
                img, text, y, font_size, 608, // 画布宽度
                is_chinese,
            ) {
                log::warn!("渲染居中文本失败: {}, 使用占位符", e);
                let text_width = text.len() as i32 * (font_size as i32 / 2);
                let text_height = font_size as i32;
                let rect =
                    Rect::at(x - text_width / 2, y).of_size(text_width as u32, text_height as u32);
                draw_hollow_rect_mut(img, rect, Rgb([0u8, 0u8, 0u8]));
            }
            log::debug!(
                "📝 TEXT (居中) at Y={}: \"{}\" (size: {})",
                y,
                text,
                font_size
            );
        } else {
            // 左对齐渲染
            if let Err(e) = text_renderer.draw_text(img, text, x, y, font_size, is_chinese) {
                log::warn!("渲染文本失败: {}, 使用占位符", e);
                let text_width = text.len() as i32 * (font_size as i32 / 2);
                let text_height = font_size as i32;
                let rect = Rect::at(x, y).of_size(text_width as u32, text_height as u32);
                draw_hollow_rect_mut(img, rect, Rgb([0u8, 0u8, 0u8]));
            }
            log::debug!(
                "📝 TEXT at ({}, {}): \"{}\" (size: {})",
                x,
                y,
                text,
                font_size
            );
        }
    }

    /// 渲染 BARCODE 命令（支持居中对齐）
    fn render_barcode(&self, img: &mut RgbImage, params: &[String]) {
        if params.len() < 9 {
            log::warn!("BARCODE 命令参数不足: {:?}", params);
            return;
        }

        let x: i32 = params[0].parse().unwrap_or(0);
        let y: i32 = params[1].parse().unwrap_or(0);
        let barcode_type = params[2].trim_matches('"');
        let height: u32 = params[3].parse().unwrap_or(80);
        let _human_readable: i32 = params[4].parse().unwrap_or(1);
        let data = params[8].trim_matches('"');

        // 只支持 Code128
        if barcode_type != "128" {
            log::warn!("不支持的条形码类型: {}", barcode_type);
            return;
        }

        let barcode_renderer = BarcodeRenderer::new();

        // 检测是否需要居中（x 坐标接近画布中心 304，范围 280-330）
        let should_center = x >= 280 && x <= 330;

        if should_center {
            // 居中渲染（使用标准宽度计算）
            let barcode_width = ((11 * (data.len() + 3) + 35) * 2) as u32;
            let quiet_zone = 20u32;

            if let Err(e) = barcode_renderer.render_centered_barcode(
                img,
                data,
                y,
                barcode_width,
                height,
                608, // 画布宽度
                quiet_zone,
            ) {
                log::warn!("条形码居中渲染失败: {}", e);
            }
            log::debug!(
                "📊 BARCODE (居中) at Y={}: \"{}\" (height={})",
                y,
                data,
                height
            );
        } else {
            // 左对齐渲染
            if let Err(e) = barcode_renderer.render_tspl_barcode(img, x, y, height, data) {
                log::warn!("条形码渲染失败，使用占位符: {}", e);

                // 失败时使用占位符（条纹矩形）
                let width = data.len() as i32 * 15;
                let rect = Rect::at(x, y).of_size(width as u32, height);
                draw_hollow_rect_mut(img, rect, Rgb([0u8, 0u8, 0u8]));

                for i in (0..width).step_by(6) {
                    let bar_rect = Rect::at(x + i, y).of_size(3, height);
                    draw_filled_rect_mut(img, bar_rect, Rgb([0u8, 0u8, 0u8]));
                }
            }
            log::debug!(
                "📊 BARCODE at ({}, {}): \"{}\" (height={})",
                x,
                y,
                data,
                height
            );
        }
    }

    /// 渲染 BAR 命令（填充矩形）
    fn render_bar(&self, img: &mut RgbImage, params: &[String]) {
        if params.len() < 4 {
            return;
        }

        let x: i32 = params[0].parse().unwrap_or(0);
        let y: i32 = params[1].parse().unwrap_or(0);
        let width: u32 = params[2].parse().unwrap_or(0);
        let height: u32 = params[3].parse().unwrap_or(0);

        let rect = Rect::at(x, y).of_size(width, height);
        draw_filled_rect_mut(img, rect, Rgb([0u8, 0u8, 0u8]));

        println!("▬ BAR at ({}, {}): {}x{}", x, y, width, height);
    }

    /// 渲染 BOX 命令（空心矩形）
    fn render_box(&self, img: &mut RgbImage, params: &[String]) {
        if params.len() < 5 {
            return;
        }

        let x1: i32 = params[0].parse().unwrap_or(0);
        let y1: i32 = params[1].parse().unwrap_or(0);
        let x2: i32 = params[2].parse().unwrap_or(0);
        let y2: i32 = params[3].parse().unwrap_or(0);
        let line_width: i32 = params[4].parse().unwrap_or(1);

        let width = (x2 - x1).abs() as u32;
        let height = (y2 - y1).abs() as u32;

        // 绘制外框
        for i in 0..line_width {
            let rect = Rect::at(x1 + i, y1 + i).of_size(
                width.saturating_sub(2 * i as u32),
                height.saturating_sub(2 * i as u32),
            );
            draw_hollow_rect_mut(img, rect, Rgb([0u8, 0u8, 0u8]));
        }

        println!(
            "▭ BOX at ({}, {}) to ({}, {}): width {}",
            x1, y1, x2, y2, line_width
        );
    }
}

impl PrinterBackend for PdfBackend {
    fn name(&self) -> &str {
        "PDF"
    }

    fn list_printers(&self) -> Result<Vec<String>> {
        // PDF 后端提供一个虚拟打印机
        Ok(vec!["PDF 测试打印机".to_string()])
    }

    fn send_raw(&self, _printer_name: &str, data: &[u8]) -> Result<()> {
        // 将 TSPL 数据解析并生成 PNG/PDF
        let tspl = String::from_utf8_lossy(data);

        let (png_path, pdf_path_opt) = self.render_tspl(&tspl).context("渲染失败")?;

        println!("\n✅ 文件已生成:");
        println!("  PNG: {}", png_path.display());
        if let Some(pdf_path) = pdf_path_opt {
            println!("  PDF: {}", pdf_path.display());
        }

        Ok(())
    }
}
