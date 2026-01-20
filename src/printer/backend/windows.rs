// Windows 打印后端
//
// 使用 Windows Win32 API 实现 RAW 打印

#[cfg(target_os = "windows")]
use super::PrinterBackend;

#[cfg(target_os = "windows")]
use anyhow::{Context, Result};

#[cfg(target_os = "windows")]
use windows::core::PWSTR;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{BOOL, HANDLE};

#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Printing::{
    ClosePrinter, EndDocPrinter, EndPagePrinter, EnumPrintersW, OpenPrinterW, StartDocPrinterW,
    StartPagePrinter, WritePrinter, DOC_INFO_1W, PRINTER_ENUM_LOCAL, PRINTER_INFO_2W,
};

#[cfg(target_os = "windows")]
/// Windows 打印后端
pub struct WindowsBackend;

#[cfg(target_os = "windows")]
impl WindowsBackend {
    pub fn new() -> Self {
        Self
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
                0,
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
                Some(buffer.as_mut_ptr() as *mut u8),
                needed,
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
                    let name = printer_info.pPrinterName.to_string()
                        .context("无法解析打印机名称")?;
                    printers.push(name);
                }
            }

            Ok(printers)
        }
    }

    fn send_raw(&self, printer_name: &str, data: &[u8]) -> Result<()> {
        unsafe {
            // 打开打印机
            let mut printer_name_wide: Vec<u16> = printer_name.encode_utf16().chain(Some(0)).collect();
            let mut printer_handle: HANDLE = HANDLE::default();

            OpenPrinterW(
                PWSTR(printer_name_wide.as_mut_ptr()),
                &mut printer_handle,
                None,
            )
            .context("无法打开打印机")?;

            // 开始文档
            let doc_name = "QSL Card\0".encode_utf16().collect::<Vec<u16>>();
            let mut doc_info = DOC_INFO_1W {
                pDocName: PWSTR(doc_name.as_ptr() as *mut u16),
                pOutputFile: PWSTR::null(),
                pDatatype: PWSTR::null(),
            };

            let job_id = StartDocPrinterW(printer_handle, 1, &mut doc_info as *mut _ as *mut u8);
            if job_id == 0 {
                ClosePrinter(printer_handle);
                anyhow::bail!("无法开始打印文档");
            }

            // 开始页面
            if !StartPagePrinter(printer_handle).as_bool() {
                EndDocPrinter(printer_handle);
                ClosePrinter(printer_handle);
                anyhow::bail!("无法开始打印页面");
            }

            // 写入数据
            let mut written: u32 = 0;
            if !WritePrinter(
                printer_handle,
                data.as_ptr() as *const _,
                data.len() as u32,
                &mut written,
            )
            .as_bool()
            {
                EndPagePrinter(printer_handle);
                EndDocPrinter(printer_handle);
                ClosePrinter(printer_handle);
                anyhow::bail!("无法写入打印数据");
            }

            // 结束页面和文档
            EndPagePrinter(printer_handle);
            EndDocPrinter(printer_handle);
            ClosePrinter(printer_handle);

            println!("✅ 打印成功: {} ({} 字节)", printer_name, written);

            Ok(())
        }
    }
}

#[cfg(target_os = "windows")]
impl Default for WindowsBackend {
    fn default() -> Self {
        Self::new()
    }
}
