// Mock 打印后端
//
// 用于开发和测试，将打印数据保存到文件而不是实际打印

use super::PrinterBackend;
use anyhow::{Context, Result};
use chrono::Local;
use std::fs;
use std::path::PathBuf;

/// Mock 打印后端
pub struct MockBackend {
    /// 输出目录
    output_dir: PathBuf,
}

impl MockBackend {
    /// 创建新的 Mock 后端
    ///
    /// # 参数
    /// - `output_dir`: 输出目录（默认为 "output"）
    pub fn new(output_dir: PathBuf) -> Result<Self> {
        // 确保输出目录存在
        fs::create_dir_all(&output_dir)
            .context("无法创建 Mock 打印输出目录")?;

        Ok(Self { output_dir })
    }

    /// 使用默认输出目录创建 Mock 后端
    #[allow(dead_code)]
    pub fn default() -> Result<Self> {
        Self::new(PathBuf::from("output"))
    }
}

impl PrinterBackend for MockBackend {
    fn name(&self) -> &str {
        "Mock"
    }

    fn list_printers(&self) -> Result<Vec<String>> {
        Ok(vec![
            "Mock Printer".to_string(),
            "Mock Printer 2".to_string(),
        ])
    }

    fn send_raw(&self, printer_name: &str, data: &[u8]) -> Result<()> {
        // 生成文件名：print_YYYYMMDD_HHMMSS.tspl
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("print_{}_{}.tspl", timestamp, printer_name.replace(' ', "_"));
        let filepath = self.output_dir.join(filename);

        // 写入文件
        fs::write(&filepath, data)
            .context("无法写入 Mock 打印文件")?;

        // 记录日志
        println!("📄 Mock 打印: {} -> {}", printer_name, filepath.display());

        Ok(())
    }
}
