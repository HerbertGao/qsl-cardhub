// 打印机后端抽象层
//
// 提供跨平台打印机接口的统一抽象

use anyhow::Result;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_family = "unix")]
pub mod cups;

pub mod pdf;

#[cfg(target_os = "windows")]
pub use windows::WindowsBackend;

#[cfg(target_family = "unix")]
pub use cups::CupsBackend;

pub use pdf::PdfBackend;

// 重新导出 trait（用于测试）
pub use self::PrinterBackend as PrinterBackendTrait;

/// 打印机后端 trait
///
/// 所有平台的打印机后端必须实现此 trait
pub trait PrinterBackend: Send + Sync {
    /// 获取后端名称
    fn name(&self) -> &str;

    /// 列出所有可用的打印机
    ///
    /// # 返回
    /// 打印机名称列表
    fn list_printers(&self) -> Result<Vec<String>>;

    /// 发送原始数据到打印机
    ///
    /// # 参数
    /// - `printer_name`: 打印机名称
    /// - `data`: 原始打印数据（通常是 TSPL 指令）
    fn send_raw(&self, printer_name: &str, data: &[u8]) -> Result<()>;
}
