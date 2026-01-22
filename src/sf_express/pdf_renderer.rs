// PDF 渲染器
//
// 负责：
// - 将 PDF 渲染为位图
// - 转换为 1bpp 点阵
// - 生成 TSPL BITMAP 指令

use anyhow::{Context, Result, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use image::{GrayImage, Luma, DynamicImage};
use pdfium_render::prelude::*;
use std::io::Cursor;

/// 获取当前平台的 pdfium 库目录名
fn get_platform_dir() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "macos-arm64"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "macos-x64"
    }
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    {
        "windows-arm64"
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "windows-x64"
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "linux-x64"
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        "linux-arm64"
    }
    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
    )))]
    {
        "unknown"
    }
}

/// 获取可执行文件所在目录
fn get_executable_dir() -> Option<std::path::PathBuf> {
    std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()))
}

/// 创建 Pdfium 实例
fn create_pdfium() -> Result<Pdfium> {
    let platform_dir = get_platform_dir();
    log::info!("当前平台: {}", platform_dir);

    // 构建搜索路径列表
    let mut search_paths: Vec<std::path::PathBuf> = Vec::new();

    // 1. 相对于可执行文件的 resources 目录（打包后的应用）
    if let Some(exe_dir) = get_executable_dir() {
        // macOS: App.app/Contents/MacOS/exe -> App.app/Contents/Resources
        #[cfg(target_os = "macos")]
        {
            let resources_dir = exe_dir.join("../Resources/resources/pdfium").join(platform_dir);
            search_paths.push(resources_dir);
        }
        // Windows: 安装目录/exe -> 安装目录/resources
        #[cfg(target_os = "windows")]
        {
            let resources_dir = exe_dir.join("resources/pdfium").join(platform_dir);
            search_paths.push(resources_dir);
        }
        // 通用：相对于 exe 的 resources 目录
        let resources_dir = exe_dir.join("resources/pdfium").join(platform_dir);
        search_paths.push(resources_dir);
    }

    // 2. 开发环境：项目根目录的 resources
    search_paths.push(std::path::PathBuf::from(format!("./resources/pdfium/{}", platform_dir)));
    search_paths.push(std::path::PathBuf::from(format!("resources/pdfium/{}", platform_dir)));

    // 3. 系统路径
    #[cfg(target_os = "macos")]
    {
        search_paths.push(std::path::PathBuf::from("/usr/local/lib"));
        search_paths.push(std::path::PathBuf::from("/usr/lib"));
    }

    // 尝试从各个路径加载
    for path in &search_paths {
        let lib_path = Pdfium::pdfium_platform_library_name_at_path(path);
        log::info!("尝试加载 pdfium: {}", lib_path.display());

        if let Ok(bindings) = Pdfium::bind_to_library(&lib_path) {
            log::info!("成功加载 pdfium: {}", lib_path.display());
            return Ok(Pdfium::new(bindings));
        }
    }

    // 最后尝试系统默认路径
    log::info!("尝试从系统默认路径加载 pdfium");
    Pdfium::bind_to_system_library()
        .map(Pdfium::new)
        .map_err(|e| {
            let paths_tried = search_paths.iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            anyhow::anyhow!(
                "无法加载 pdfium 库: {}。已尝试路径: {}。请确保 pdfium 库已正确打包。",
                e, paths_tried
            )
        })
}

/// 面单尺寸配置
pub struct WaybillSize {
    /// 宽度（毫米）
    pub width_mm: f32,
    /// 高度（毫米）
    pub height_mm: f32,
    /// 打印机 DPI
    pub dpi: u32,
}

impl Default for WaybillSize {
    fn default() -> Self {
        Self {
            width_mm: 76.0,
            height_mm: 130.0,
            dpi: 203,
        }
    }
}

impl WaybillSize {
    /// 获取目标宽度（像素）
    pub fn width_pixels(&self) -> u32 {
        ((self.width_mm / 25.4) * self.dpi as f32) as u32
    }

    /// 获取目标高度（像素）
    pub fn height_pixels(&self) -> u32 {
        ((self.height_mm / 25.4) * self.dpi as f32) as u32
    }

    /// 获取每行字节数
    pub fn bytes_per_row(&self) -> usize {
        ((self.width_pixels() + 7) / 8) as usize
    }
}

/// PDF 渲染器
pub struct PdfRenderer {
    size: WaybillSize,
}

impl PdfRenderer {
    /// 创建新的渲染器
    pub fn new() -> Self {
        Self {
            size: WaybillSize::default(),
        }
    }

    /// 使用自定义尺寸创建渲染器
    pub fn with_size(size: WaybillSize) -> Self {
        Self { size }
    }

    /// 将 PDF 数据渲染为灰度位图
    pub fn render_pdf_to_grayscale(&self, pdf_data: &[u8]) -> Result<GrayImage> {
        log::info!("开始渲染 PDF，目标尺寸: {}x{} 像素",
            self.size.width_pixels(), self.size.height_pixels());

        // 创建 Pdfium 实例
        let pdfium = create_pdfium()?;

        // 加载 PDF 文档
        let document = pdfium
            .load_pdf_from_byte_slice(pdf_data, None)
            .context("加载 PDF 文档失败")?;

        // 获取第一页
        let pages = document.pages();
        if pages.len() == 0 {
            bail!("PDF 文档没有页面");
        }

        let page = pages.get(0).context("获取 PDF 页面失败")?;

        // 计算渲染配置
        let target_width = self.size.width_pixels() as i32;
        let target_height = self.size.height_pixels() as i32;

        log::debug!("渲染 PDF 页面到 {}x{}", target_width, target_height);

        // 渲染页面为位图
        let render_config = PdfRenderConfig::new()
            .set_target_width(target_width)
            .set_target_height(target_height)
            .render_form_data(true)
            .render_annotations(true);

        let bitmap = page
            .render_with_config(&render_config)
            .context("渲染 PDF 页面失败")?;

        // 转换为 image crate 的格式
        let image = bitmap.as_image();

        // 转换为灰度图像
        let gray_image = image.to_luma8();

        log::info!("PDF 渲染完成，灰度图像尺寸: {}x{}",
            gray_image.width(), gray_image.height());

        Ok(gray_image)
    }

    /// 将 PDF 数据渲染为 PNG 预览图像（Base64 编码）
    ///
    /// # 参数
    /// - `pdf_data`: PDF 文件数据
    ///
    /// # 返回
    /// Base64 编码的 PNG 图像数据
    pub fn render_pdf_to_preview(&self, pdf_data: &[u8]) -> Result<String> {
        log::info!("渲染 PDF 预览图像");

        // 创建 Pdfium 实例
        let pdfium = create_pdfium()?;

        // 加载 PDF 文档
        let document = pdfium
            .load_pdf_from_byte_slice(pdf_data, None)
            .context("加载 PDF 文档失败")?;

        // 获取第一页
        let pages = document.pages();
        if pages.len() == 0 {
            bail!("PDF 文档没有页面");
        }

        let page = pages.get(0).context("获取 PDF 页面失败")?;

        // 计算渲染配置（预览用更高分辨率以便查看）
        let target_width = self.size.width_pixels() as i32;
        let target_height = self.size.height_pixels() as i32;

        log::debug!("渲染 PDF 预览到 {}x{}", target_width, target_height);

        // 渲染页面为位图
        let render_config = PdfRenderConfig::new()
            .set_target_width(target_width)
            .set_target_height(target_height)
            .render_form_data(true)
            .render_annotations(true);

        let bitmap = page
            .render_with_config(&render_config)
            .context("渲染 PDF 页面失败")?;

        // 转换为 image crate 的格式
        let image = bitmap.as_image();
        let dynamic_image = DynamicImage::ImageRgba8(image.to_rgba8());

        // 编码为 PNG
        let mut png_data = Vec::new();
        let mut cursor = Cursor::new(&mut png_data);
        dynamic_image
            .write_to(&mut cursor, image::ImageFormat::Png)
            .context("编码 PNG 失败")?;

        // Base64 编码
        let base64_data = STANDARD.encode(&png_data);

        log::info!("PDF 预览渲染完成，PNG 大小: {} 字节，Base64 大小: {} 字节",
            png_data.len(), base64_data.len());

        Ok(base64_data)
    }

    /// 将灰度图像二值化为 1bpp 点阵
    ///
    /// # 参数
    /// - `image`: 灰度图像
    /// - `threshold`: 二值化阈值（0-255，默认 128）
    ///
    /// # 返回
    /// 1bpp 点阵数据（MSB first）
    pub fn binarize(&self, image: &GrayImage, threshold: u8) -> Vec<u8> {
        let width = image.width();
        let height = image.height();
        let bytes_per_row = self.size.bytes_per_row();

        log::info!("二值化图像: {}x{}, 阈值: {}, 每行字节数: {}",
            width, height, threshold, bytes_per_row);

        let mut data = Vec::with_capacity(bytes_per_row * height as usize);

        for y in 0..height {
            let mut row_bytes = vec![0u8; bytes_per_row];

            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                let value = pixel.0[0];

                // 灰度值小于阈值为黑色（打印点），设置位为 1
                // TSPL BITMAP 格式：1 = 打印点（黑色），0 = 空白（白色）
                if value < threshold {
                    let byte_idx = (x / 8) as usize;
                    let bit_idx = 7 - (x % 8); // MSB first
                    if byte_idx < bytes_per_row {
                        row_bytes[byte_idx] |= 1 << bit_idx;
                    }
                }
            }

            data.extend_from_slice(&row_bytes);
        }

        log::info!("二值化完成，数据大小: {} 字节", data.len());

        data
    }

    /// 生成 TSPL BITMAP 指令
    ///
    /// # 参数
    /// - `bitmap_data`: 1bpp 点阵数据
    ///
    /// # 返回
    /// TSPL 指令字符串
    pub fn generate_tspl(&self, bitmap_data: &[u8]) -> String {
        let width_bytes = self.size.bytes_per_row();
        let height = self.size.height_pixels();

        log::info!("生成 TSPL 指令: SIZE {} mm x {} mm, BITMAP {}x{}",
            self.size.width_mm, self.size.height_mm, width_bytes, height);

        let mut tspl = String::new();

        // 纸张配置
        tspl.push_str(&format!("SIZE {} mm, {} mm\n", self.size.width_mm, self.size.height_mm));
        tspl.push_str("GAP 0 mm, 0 mm\n");
        tspl.push_str("DIRECTION 1,0\n");
        tspl.push_str("CLS\n");

        // BITMAP 指令
        // 格式: BITMAP x,y,width_bytes,height,mode,data
        // mode 0 = 覆盖模式（白色背景上打印黑点）
        tspl.push_str(&format!("BITMAP 0,0,{},{},0,", width_bytes, height));

        // 添加二进制数据（十六进制格式）
        for byte in bitmap_data {
            tspl.push_str(&format!("{:02X}", byte));
        }

        tspl.push('\n');

        // 打印指令
        tspl.push_str("PRINT 1,1\n");

        tspl
    }

    /// 完整流程：PDF -> TSPL
    ///
    /// # 参数
    /// - `pdf_data`: PDF 文件数据
    /// - `threshold`: 二值化阈值（默认 128）
    ///
    /// # 返回
    /// TSPL 指令字符串
    pub fn pdf_to_tspl(&self, pdf_data: &[u8], threshold: Option<u8>) -> Result<String> {
        let threshold = threshold.unwrap_or(128);

        // 1. 渲染 PDF 为灰度图像
        let gray_image = self.render_pdf_to_grayscale(pdf_data)?;

        // 2. 二值化为 1bpp 点阵
        let bitmap_data = self.binarize(&gray_image, threshold);

        // 3. 生成 TSPL 指令
        let tspl = self.generate_tspl(&bitmap_data);

        Ok(tspl)
    }

    /// 获取 TSPL 原始二进制数据（用于直接发送到打印机）
    pub fn pdf_to_tspl_bytes(&self, pdf_data: &[u8], threshold: Option<u8>) -> Result<Vec<u8>> {
        let tspl = self.pdf_to_tspl(pdf_data, threshold)?;
        Ok(tspl.into_bytes())
    }
}

impl Default for PdfRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waybill_size_default() {
        let size = WaybillSize::default();
        assert_eq!(size.width_mm, 76.0);
        assert_eq!(size.height_mm, 130.0);
        assert_eq!(size.dpi, 203);

        // 验证像素计算
        // 76mm / 25.4 * 203 ≈ 608
        // 130mm / 25.4 * 203 ≈ 1039
        assert_eq!(size.width_pixels(), 607);
        assert_eq!(size.height_pixels(), 1039);
    }

    #[test]
    fn test_bytes_per_row() {
        let size = WaybillSize::default();
        // 607 像素 / 8 = 75.875，向上取整 = 76 字节
        assert_eq!(size.bytes_per_row(), 76);
    }

    #[test]
    fn test_binarize_simple() {
        let renderer = PdfRenderer::new();

        // 创建一个简单的 8x8 灰度图像
        let mut image = GrayImage::new(8, 8);

        // 填充一些测试数据
        // 第一行：全黑（灰度值 0）
        for x in 0..8 {
            image.put_pixel(x, 0, Luma([0u8]));
        }
        // 第二行：全白（灰度值 255）
        for x in 0..8 {
            image.put_pixel(x, 1, Luma([255u8]));
        }

        let size = WaybillSize {
            width_mm: 2.0,
            height_mm: 2.0,
            dpi: 100, // 使得 8 像素 = 1 字节
        };
        let renderer = PdfRenderer::with_size(size);

        let data = renderer.binarize(&image, 128);

        // 第一行应该是 0xFF（8 个黑点）
        assert_eq!(data[0], 0xFF);
        // 第二行应该是 0x00（8 个白点）
        assert_eq!(data[1], 0x00);
    }
}
