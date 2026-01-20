// 打印机管理器
//
// 聚合所有打印后端，提供统一的打印机管理接口

use super::backend::{MockBackend, PdfBackend, PrinterBackend};
use super::tspl::TSPLGenerator;
use anyhow::{Context, Result};
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use super::backend::WindowsBackend;

#[cfg(target_family = "unix")]
use super::backend::CupsBackend;

/// 打印机管理器
pub struct PrinterManager {
    /// 所有可用的打印后端
    backends: Vec<Box<dyn PrinterBackend>>,
    /// TSPL 指令生成器
    tspl_generator: TSPLGenerator,
}

impl PrinterManager {
    /// 创建新的打印机管理器
    pub fn new(output_dir: PathBuf) -> Result<Self> {
        let mut backends: Vec<Box<dyn PrinterBackend>> = Vec::new();

        // 添加平台特定的后端
        #[cfg(target_os = "windows")]
        {
            backends.push(Box::new(WindowsBackend::new()));
        }

        #[cfg(target_family = "unix")]
        {
            backends.push(Box::new(CupsBackend::new()));
        }

        // 添加 PDF 后端（总是可用，输出到 Downloads 目录）
        match PdfBackend::with_downloads_dir() {
            Ok(pdf_backend) => backends.push(Box::new(pdf_backend)),
            Err(e) => eprintln!("⚠️  PDF 后端初始化失败: {}", e),
        }

        // 添加 Mock 后端（总是可用）
        backends.push(Box::new(MockBackend::new(output_dir)?));

        Ok(Self {
            backends,
            tspl_generator: TSPLGenerator::new(),
        })
    }

    /// 列出所有可用的打印机
    ///
    /// 聚合所有后端的打印机列表
    pub fn list_printers(&self) -> Result<Vec<String>> {
        let mut all_printers = Vec::new();

        for backend in &self.backends {
            match backend.list_printers() {
                Ok(printers) => {
                    all_printers.extend(printers);
                }
                Err(e) => {
                    eprintln!("⚠️  后端 {} 枚举打印机失败: {}", backend.name(), e);
                }
            }
        }

        // 去重
        all_printers.sort();
        all_printers.dedup();

        Ok(all_printers)
    }

    /// 打印 QSL 卡片
    ///
    /// # 参数
    /// - `printer_name`: 打印机名称
    /// - `callsign`: 呼号
    /// - `serial`: 序列号
    /// - `qty`: 打印数量
    /// - `task_name`: 任务名称（副标题，可选）
    pub fn print_qsl(
        &self,
        printer_name: &str,
        callsign: &str,
        serial: u32,
        qty: u32,
        task_name: Option<&str>,
    ) -> Result<()> {
        // 生成 TSPL 指令
        let tspl = self.tspl_generator.generate_qsl_card(callsign, serial, qty, task_name);

        // 发送到打印机
        self.send_raw(printer_name, tspl.as_bytes())
    }

    /// 打印校准页
    ///
    /// # 参数
    /// - `printer_name`: 打印机名称
    pub fn print_calibration(&self, printer_name: &str) -> Result<()> {
        // 生成校准页 TSPL 指令
        let tspl = self.tspl_generator.generate_calibration_page();

        // 发送到打印机
        self.send_raw(printer_name, tspl.as_bytes())
    }

    /// 发送原始数据到打印机
    ///
    /// 自动选择合适的后端
    fn send_raw(&self, printer_name: &str, data: &[u8]) -> Result<()> {
        // 遍历所有后端，找到支持该打印机的后端
        for backend in &self.backends {
            if let Ok(printers) = backend.list_printers() {
                if printers.contains(&printer_name.to_string()) {
                    return backend
                        .send_raw(printer_name, data)
                        .context(format!("后端 {} 打印失败", backend.name()));
                }
            }
        }

        anyhow::bail!("打印机未找到: {}", printer_name)
    }

    /// 获取 TSPL 生成器（用于测试）
    #[allow(dead_code)]
    pub fn tspl_generator(&self) -> &TSPLGenerator {
        &self.tspl_generator
    }
}
