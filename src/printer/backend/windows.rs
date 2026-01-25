// Windows 打印后端
//
// 使用 Windows Win32 API 实现 RAW 打印

#[cfg(target_os = "windows")]
use super::{ImagePrintConfig, PrinterBackend, PrintResult};

#[cfg(target_os = "windows")]
use super::pdf::PDF_TEST_PRINTER_NAME;

#[cfg(target_os = "windows")]
use crate::printer::tspl::TSPLGenerator;

#[cfg(target_os = "windows")]
use anyhow::{Context, Result};

#[cfg(target_os = "windows")]
use image::GrayImage;

#[cfg(target_os = "windows")]
use windows::core::PWSTR;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{GetLastError, WIN32_ERROR};

#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Printing::{
    ClosePrinter, DOC_INFO_1W, EndDocPrinter, EndPagePrinter, EnumPrintersW, OpenPrinterW,
    PRINTER_ENUM_LOCAL, PRINTER_HANDLE, PRINTER_INFO_2W, StartDocPrinterW, StartPagePrinter,
    WritePrinter,
};

#[cfg(target_os = "windows")]
use windows::Win32::System::Diagnostics::Debug::{FormatMessageW, FORMAT_MESSAGE_FROM_SYSTEM};

#[cfg(target_os = "windows")]
/// Windows 打印后端
pub struct WindowsBackend;

#[cfg(target_os = "windows")]
impl WindowsBackend {
    pub fn new() -> Self {
        Self
    }

    /// 获取 Win32 错误码的可读描述
    fn format_win32_error(error_code: WIN32_ERROR) -> String {
        unsafe {
            let mut buffer: [u16; 512] = [0; 512];
            let len = FormatMessageW(
                FORMAT_MESSAGE_FROM_SYSTEM,
                None,
                error_code.0,
                0,
                PWSTR(buffer.as_mut_ptr()),
                buffer.len() as u32,
                None,
            );
            if len > 0 {
                String::from_utf16_lossy(&buffer[..len as usize])
                    .trim()
                    .to_string()
            } else {
                format!("未知错误 (错误码: {})", error_code.0)
            }
        }
    }

    /// 获取并记录当前错误
    fn log_last_error(context: &str) -> String {
        unsafe {
            let error = GetLastError();
            let error_msg = Self::format_win32_error(error);
            log::error!("❌ {}: {} (错误码: {})", context, error_msg, error.0);
            error_msg
        }
    }
}

#[cfg(target_os = "windows")]
impl PrinterBackend for WindowsBackend {
    fn name(&self) -> &str {
        "Windows"
    }

    fn list_printers(&self) -> Result<Vec<String>> {
        unsafe {
            let mut needed: u32 = 0;
            let mut returned: u32 = 0;

            // 第一次调用获取所需的缓冲区大小
            let _ = EnumPrintersW(
                PRINTER_ENUM_LOCAL,
                None,
                2, // PRINTER_INFO_2W
                None,
                &mut needed,
                &mut returned,
            );

            if needed == 0 {
                return Ok(Vec::new());
            }

            // 分配缓冲区
            let mut buffer = vec![0u8; needed as usize];

            // 第二次调用获取打印机信息
            EnumPrintersW(
                PRINTER_ENUM_LOCAL,
                None,
                2,
                Some(buffer.as_mut_slice()),
                &mut needed,
                &mut returned,
            )
            .context("无法枚举打印机")?;

            // 解析打印机信息
            let mut printers = Vec::new();
            let printer_info_array = buffer.as_ptr() as *const PRINTER_INFO_2W;

            for i in 0..returned as usize {
                let printer_info = &*printer_info_array.add(i);
                if !printer_info.pPrinterName.is_null() {
                    let name = printer_info
                        .pPrinterName
                        .to_string()
                        .context("无法解析打印机名称")?;
                    printers.push(name);
                }
            }

            Ok(printers)
        }
    }

    fn owns_printer(&self, printer_name: &str) -> bool {
        // Windows 后端拥有所有非 PDF 测试打印机
        printer_name != PDF_TEST_PRINTER_NAME
    }

    fn send_raw(&self, printer_name: &str, data: &[u8]) -> Result<PrintResult> {
        log::info!(
            "🖨️ 开始打印: 打印机={}, 数据大小={}字节",
            printer_name,
            data.len()
        );

        unsafe {
            // 打开打印机
            log::debug!("📤 打开打印机...");
            let mut printer_name_wide: Vec<u16> =
                printer_name.encode_utf16().chain(Some(0)).collect();
            let mut printer_handle: PRINTER_HANDLE = PRINTER_HANDLE::default();

            if let Err(e) = OpenPrinterW(
                PWSTR(printer_name_wide.as_mut_ptr()),
                &mut printer_handle as *mut PRINTER_HANDLE,
                None,
            ) {
                let error_msg = Self::log_last_error("无法打开打印机");
                anyhow::bail!("无法打开打印机: {} ({})", error_msg, e);
            }
            log::debug!("✓ 打印机已打开");

            // 开始文档
            log::debug!("📤 开始打印文档...");
            let doc_name = "QSL Card\0".encode_utf16().collect::<Vec<u16>>();
            // 指定 RAW 数据类型，让打印后台处理程序直接传递数据给打印机
            let datatype = "RAW\0".encode_utf16().collect::<Vec<u16>>();
            let doc_info = DOC_INFO_1W {
                pDocName: PWSTR(doc_name.as_ptr() as *mut u16),
                pOutputFile: PWSTR::null(),
                pDatatype: PWSTR(datatype.as_ptr() as *mut u16),
            };

            let job_id = StartDocPrinterW(printer_handle, 1, &doc_info);
            if job_id == 0 {
                let error_msg = Self::log_last_error("无法开始打印文档");
                let _ = ClosePrinter(printer_handle);
                anyhow::bail!("无法开始打印文档: {}", error_msg);
            }
            log::info!("📋 文档 ID: {}", job_id);

            // 开始页面
            log::debug!("📤 开始打印页面...");
            if !StartPagePrinter(printer_handle).as_bool() {
                let error_msg = Self::log_last_error("无法开始打印页面");
                let _ = EndDocPrinter(printer_handle);
                let _ = ClosePrinter(printer_handle);
                anyhow::bail!("无法开始打印页面: {}", error_msg);
            }
            log::debug!("✓ 页面已开始");

            // 写入数据
            log::info!("📤 写入打印数据...");
            let mut written: u32 = 0;
            if !WritePrinter(
                printer_handle,
                data.as_ptr() as *const _,
                data.len() as u32,
                &mut written,
            )
            .as_bool()
            {
                let error_msg = Self::log_last_error("无法写入打印数据");
                let _ = EndPagePrinter(printer_handle);
                let _ = EndDocPrinter(printer_handle);
                let _ = ClosePrinter(printer_handle);
                anyhow::bail!("无法写入打印数据: {}", error_msg);
            }
            log::info!("✓ 已写入 {} 字节", written);

            // 结束页面和文档
            log::debug!("📤 结束打印...");
            let _ = EndPagePrinter(printer_handle);
            let _ = EndDocPrinter(printer_handle);
            let _ = ClosePrinter(printer_handle);

            log::info!("✅ 打印成功: 作业ID={}", job_id);

            // 构建结果
            let result = PrintResult::success_with_job_id(
                format!("打印成功: {} ({} 字节)", printer_name, written),
                job_id.to_string(),
            )
            .with_details(format!(
                "文档ID: {}, 写入字节: {}/{}",
                job_id,
                written,
                data.len()
            ));

            Ok(result)
        }
    }

    fn print_image(
        &self,
        printer_name: &str,
        image: &GrayImage,
        config: &ImagePrintConfig,
    ) -> Result<PrintResult> {
        if !self.owns_printer(printer_name) {
            anyhow::bail!("Windows 后端不支持打印机: {}", printer_name);
        }

        log::info!("Windows 后端：将图像转换为 TSPL 并打印");

        // 使用 TSPL 生成器将图像转换为 TSPL 指令
        let tspl_generator = TSPLGenerator::new();
        let tspl = tspl_generator
            .generate_from_image(image, config.width_mm, config.height_mm)
            .context("生成 TSPL 指令失败")?;

        log::info!("TSPL 指令生成成功，长度: {} 字节", tspl.len());

        // 使用 send_raw 发送到打印机
        self.send_raw(printer_name, &tspl)
    }
}

#[cfg(target_os = "windows")]
impl Default for WindowsBackend {
    fn default() -> Self {
        Self::new()
    }
}
