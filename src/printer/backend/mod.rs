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

/// 打印操作结果
///
/// 包含打印操作的详细结果信息，用于日志记录和调试
#[derive(Debug, Clone)]
pub struct PrintResult {
    /// 是否成功
    pub success: bool,
    /// 打印作业 ID（如果系统提供）
    pub job_id: Option<String>,
    /// 结果消息
    pub message: String,
    /// 详细信息（stdout/stderr 等）
    pub details: Option<String>,
}

impl PrintResult {
    /// 创建成功结果
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            job_id: None,
            message: message.into(),
            details: None,
        }
    }

    /// 创建带作业 ID 的成功结果
    pub fn success_with_job_id(message: impl Into<String>, job_id: impl Into<String>) -> Self {
        Self {
            success: true,
            job_id: Some(job_id.into()),
            message: message.into(),
            details: None,
        }
    }

    /// 设置详细信息
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

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
    ///
    /// # 返回
    /// PrintResult 包含打印结果的详细信息
    fn send_raw(&self, printer_name: &str, data: &[u8]) -> Result<PrintResult>;
}
