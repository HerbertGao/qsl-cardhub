// 条形码渲染模块
//
// 使用 barcoders 库生成 Code128 条形码并渲染到图像

use anyhow::{Context, Result};
use barcoders::sym::code128::Code128;
use image::{GrayImage, ImageBuffer, Luma, Rgb, RgbImage};

/// 条形码渲染器
pub struct BarcodeRenderer;

impl BarcodeRenderer {
    /// 创建新的条形码渲染器
    pub fn new() -> Self {
        Self
    }

    /// 渲染条形码为1bpp灰度图像
    ///
    /// # 参数
    /// - `data`: 条形码数据
    /// - `barcode_type`: 条形码类型（如"code128"）
    /// - `height`: 条形码高度(dots)
    ///
    /// # 返回
    /// 1bpp灰度图像 (0=黑色, 255=白色)
    pub fn render_barcode(&self, data: &str, barcode_type: &str, height: u32) -> Result<GrayImage> {
        match barcode_type.to_lowercase().as_str() {
            "code128" | "128" => self.render_code128_bitmap(data, height),
            _ => anyhow::bail!("不支持的条形码类型: {}", barcode_type),
        }
    }

    /// 渲染Code128条形码为位图
    fn render_code128_bitmap(&self, data: &str, height: u32) -> Result<GrayImage> {
        // 添加Code128前缀
        let prefixed_data = format!("\u{0181}{}", data);

        // 创建条形码编码器
        let barcode = Code128::new(&prefixed_data).context("创建 Code128 条形码失败")?;

        // 获取编码
        let encoded = barcode.encode();

        if encoded.is_empty() {
            anyhow::bail!("条形码编码为空");
        }

        // 每个模块(module)宽度为2像素
        let module_width = 2u32;
        let width = encoded.len() as u32 * module_width;

        // 创建白色背景画布
        let mut bitmap = ImageBuffer::from_pixel(width, height, Luma([255u8]));

        // 渲染条形码
        for (i, &bit) in encoded.iter().enumerate() {
            if bit == 1 {
                // 黑色条
                let x_start = i as u32 * module_width;
                for dy in 0..height {
                    for dx in 0..module_width {
                        let x = x_start + dx;
                        if x < width {
                            bitmap.put_pixel(x, dy, Luma([0u8]));
                        }
                    }
                }
            }
        }

        log::debug!("渲染条形码位图: \"{}\" -> {}x{} dots", data, width, height);

        Ok(bitmap)
    }

    /// 渲染 Code128 条形码到图像
    ///
    /// # 参数
    /// - `img`: 目标图像
    /// - `data`: 条形码数据（如呼号）
    /// - `x`: 左上角 X 坐标
    /// - `y`: 左上角 Y 坐标
    /// - `width`: 条形码宽度
    /// - `height`: 条形码高度
    pub fn render_code128(
        &self,
        img: &mut RgbImage,
        data: &str,
        x: i32,
        y: i32,
        target_width: u32,
        height: u32,
    ) -> Result<()> {
        // Code128 需要字符集前缀
        // Set B (Ɓ = \u{0181}): 适用于字母和数字混合
        // Set C (Ć = \u{0106}): 适用于纯数字（双密度编码）
        let prefixed_data = format!("\u{0181}{}", data); // 使用 Set B

        log::debug!("原始数据: '{}', 添加前缀后: '{}'", data, prefixed_data);

        // 创建 Code128 条形码编码器
        let barcode = Code128::new(&prefixed_data).context("创建 Code128 条形码失败")?;

        // 获取条形码的二进制表示（1=黑条，0=白条）
        let encoded = barcode.encode();

        log::debug!(
            "条形码编码: 数据='{}', 编码长度={} bits",
            data,
            encoded.len()
        );

        // 计算每个条（bar）的宽度（像素）
        let bar_width = if encoded.len() > 0 {
            (target_width as f32 / encoded.len() as f32).max(1.0) as u32
        } else {
            1
        };

        // 计算实际渲染宽度（由于取整，可能与目标宽度不同）
        let actual_width = bar_width * encoded.len() as u32;

        log::debug!(
            "条形码渲染参数: x={}, y={}, target_width={}, actual_width={}, height={}, bar_width={}",
            x,
            y,
            target_width,
            actual_width,
            height,
            bar_width
        );

        // 渲染每个条
        for (i, &bit) in encoded.iter().enumerate() {
            let bar_x = x + (i as u32 * bar_width) as i32;

            // 确定颜色：1=黑色，0=白色
            let color = if bit == 1 {
                Rgb([0u8, 0u8, 0u8]) // 黑色
            } else {
                Rgb([255u8, 255u8, 255u8]) // 白色
            };

            // 绘制竖条
            for dy in 0..height {
                for dx in 0..bar_width {
                    let px = bar_x + dx as i32;
                    let py = y + dy as i32;

                    // 边界检查
                    if px >= 0 && px < img.width() as i32 && py >= 0 && py < img.height() as i32 {
                        img.put_pixel(px as u32, py as u32, color);
                    }
                }
            }
        }

        log::info!("条形码渲染成功: '{}'", data);
        Ok(())
    }

    /// 渲染 Code128 条形码（使用 TSPL 参数）
    ///
    /// 对应 TSPL 命令: BARCODE x,y,"128",height,readable,rotation,narrow,wide,code
    ///
    /// # 参数
    /// - `img`: 目标图像
    /// - `x`: X 坐标（dots）
    /// - `y`: Y 坐标（dots，将自动调整以适应标题）
    /// - `height`: 条形码高度（dots）
    /// - `code`: 条形码数据
    pub fn render_tspl_barcode(
        &self,
        img: &mut RgbImage,
        x: i32,
        mut y: i32,
        height: u32,
        code: &str,
    ) -> Result<()> {
        // Y 坐标下移 60 dots 为标题腾出空间（与文本渲染一致）
        y += 60;

        // Code128 条形码通常宽度是动态的，基于内容长度
        // 估算宽度：每个字符约 11 modules，每个 module 2-3 像素
        let estimated_width = (code.len() * 11 * 2) as u32;

        log::debug!(
            "渲染 TSPL 条形码: data='{}', x={}, y={}, height={}, estimated_width={}",
            code,
            x,
            y,
            height,
            estimated_width
        );

        self.render_code128(img, code, x, y, estimated_width, height)
    }

    /// 渲染居中的 Code128 条形码
    ///
    /// # 参数
    /// - `img`: 目标图像
    /// - `code`: 条形码数据
    /// - `y`: Y 坐标（dots）
    /// - `width`: 条形码宽度（dots，不包括静止区）
    /// - `height`: 条形码高度（dots）
    /// - `canvas_width`: 画布总宽度（用于计算居中位置）
    /// - `quiet_zone`: 静止区宽度（dots）
    pub fn render_centered_barcode(
        &self,
        img: &mut RgbImage,
        code: &str,
        y: i32,
        width: u32,
        height: u32,
        canvas_width: u32,
        _quiet_zone: u32,
    ) -> Result<()> {
        // 先编码以获取实际的条数
        let prefixed_data = format!("\u{0181}{}", code);
        let barcode = Code128::new(&prefixed_data).context("创建 Code128 条形码失败")?;
        let encoded = barcode.encode();

        // 计算每个条的宽度
        let bar_width = if encoded.len() > 0 {
            (width as f32 / encoded.len() as f32).max(1.0) as u32
        } else {
            1
        };

        // 计算实际渲染宽度（考虑取整）
        let actual_width = bar_width * encoded.len() as u32;

        // 按实际渲染宽度居中
        let x = ((canvas_width - actual_width) / 2) as i32;

        log::debug!(
            "居中条形码: code='{}', x={}, y={}, target_width={}, actual_width={}, canvas_width={}, bar_width={}",
            code,
            x,
            y,
            width,
            actual_width,
            canvas_width,
            bar_width
        );

        self.render_code128(img, code, x, y, width, height)
    }
}

impl Default for BarcodeRenderer {
    fn default() -> Self {
        Self::new()
    }
}
