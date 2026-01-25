// CUPS 打印后端
//
// 用于 macOS 和 Linux 平台，通过 lp 命令行工具与 CUPS 交互

#[cfg(target_family = "unix")]
use super::{ImagePrintConfig, PrinterBackend, PrintResult};

#[cfg(target_family = "unix")]
use super::pdf::PDF_TEST_PRINTER_NAME;

#[cfg(target_family = "unix")]
use crate::printer::tspl::TSPLGenerator;

#[cfg(target_family = "unix")]
use anyhow::{Context, Result};

#[cfg(target_family = "unix")]
use image::GrayImage;

#[cfg(target_family = "unix")]
use std::io::Write;

#[cfg(target_family = "unix")]
use std::process::{Command, Stdio};

#[cfg(target_family = "unix")]
/// CUPS 打印后端
pub struct CupsBackend;

#[cfg(target_family = "unix")]
impl CupsBackend {
    pub fn new() -> Self {
        Self
    }

    /// 解析 lp 命令输出获取作业 ID
    ///
    /// lp 成功时输出格式：`request id is PRINTER-123 (1 file(s))`
    fn parse_job_id(output: &str) -> Option<String> {
        if let Some(start) = output.find("request id is ") {
            let rest = &output[start + 14..];
            if let Some(end) = rest.find(' ') {
                return Some(rest[..end].to_string());
            }
            // 如果没有空格，尝试找括号
            if let Some(end) = rest.find('(') {
                return Some(rest[..end].trim().to_string());
            }
        }
        None
    }
}

#[cfg(target_family = "unix")]
impl PrinterBackend for CupsBackend {
    fn name(&self) -> &str {
        "CUPS"
    }

    fn list_printers(&self) -> Result<Vec<String>> {
        // 执行 lpstat -p 列出所有打印机
        let output = Command::new("lpstat")
            .arg("-p")
            .output()
            .context("无法执行 lpstat 命令")?;

        if !output.status.success() {
            anyhow::bail!("lpstat 命令执行失败");
        }

        // 解析输出
        // 英文格式：printer PrinterName is idle...
        // 中文格式：打印机PrinterName闲置...
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut printers = Vec::new();

        for line in stdout.lines() {
            if line.starts_with("printer ") {
                // 英文格式
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    printers.push(parts[1].to_string());
                }
            } else if line.starts_with("打印机") {
                // 中文格式：打印机PrinterName闲置... 或 打印机PrinterName已禁用...
                let rest = line.trim_start_matches("打印机");
                // 找到状态关键词的位置
                let printer_name = if let Some(pos) = rest.find("闲置") {
                    &rest[..pos]
                } else if let Some(pos) = rest.find("已禁用") {
                    &rest[..pos]
                } else if let Some(pos) = rest.find("正在打印") {
                    &rest[..pos]
                } else {
                    // 如果找不到已知状态，尝试找逗号或空格
                    rest.split(|c| c == ',' || c == '，' || c == ' ')
                        .next()
                        .unwrap_or("")
                };
                if !printer_name.is_empty() {
                    printers.push(printer_name.to_string());
                }
            }
        }

        Ok(printers)
    }

    fn owns_printer(&self, printer_name: &str) -> bool {
        // CUPS 后端拥有所有非 PDF 测试打印机
        printer_name != PDF_TEST_PRINTER_NAME
    }

    fn send_raw(&self, printer_name: &str, data: &[u8]) -> Result<PrintResult> {
        log::info!(
            "🖨️ 开始打印: 打印机={}, 数据大小={}字节",
            printer_name,
            data.len()
        );

        // 使用 lp 命令发送原始数据
        // lp -d <printer> -o raw -
        log::info!("📤 发送打印数据...");

        let mut child = Command::new("lp")
            .arg("-d")
            .arg(printer_name)
            .arg("-o")
            .arg("raw")
            .arg("-") // 从 stdin 读取
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("无法启动 lp 命令")?;

        // 写入数据到 stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(data)
                .context("无法写入打印数据到 lp 命令")?;
        }

        // 等待命令完成
        let output = child.wait_with_output().context("lp 命令执行失败")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // 记录系统响应
        if !stdout.is_empty() {
            log::info!("📋 系统响应 (stdout): {}", stdout.trim());
        }
        if !stderr.is_empty() {
            log::debug!("📋 系统响应 (stderr): {}", stderr.trim());
        }

        if !output.status.success() {
            log::error!("❌ 打印失败: {}", stderr.trim());
            log::error!("详细信息: stdout={}, stderr={}", stdout, stderr);
            anyhow::bail!("打印失败: {}", stderr.trim());
        }

        // 解析作业 ID
        let job_id = Self::parse_job_id(&stdout);
        if let Some(ref id) = job_id {
            log::debug!("📋 解析到作业 ID: {}", id);
        }

        // 构建详细信息
        let details = if !stdout.is_empty() || !stderr.is_empty() {
            Some(format!(
                "stdout: {}, stderr: {}",
                stdout.trim(),
                stderr.trim()
            ))
        } else {
            None
        };

        let result = if let Some(id) = job_id {
            log::info!("✅ 打印成功: 作业ID={}", id);
            PrintResult::success_with_job_id(
                format!("打印成功: {} ({} 字节)", printer_name, data.len()),
                id,
            )
        } else {
            log::info!("✅ 打印成功: {} ({} 字节)", printer_name, data.len());
            PrintResult::success(format!("打印成功: {} ({} 字节)", printer_name, data.len()))
        };

        Ok(if let Some(d) = details {
            result.with_details(d)
        } else {
            result
        })
    }

    fn print_image(
        &self,
        printer_name: &str,
        image: &GrayImage,
        config: &ImagePrintConfig,
    ) -> Result<PrintResult> {
        if !self.owns_printer(printer_name) {
            anyhow::bail!("CUPS 后端不支持打印机: {}", printer_name);
        }

        log::info!("CUPS 后端：将图像转换为 TSPL 并打印");

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

#[cfg(target_family = "unix")]
impl Default for CupsBackend {
    fn default() -> Self {
        Self::new()
    }

}

#[cfg(test)]
#[cfg(target_family = "unix")]
mod tests {
    use super::*;

    #[test]
    fn test_parse_job_id() {
        // 标准格式
        assert_eq!(
            CupsBackend::parse_job_id("request id is TSC_TDP-225-456 (1 file(s))"),
            Some("TSC_TDP-225-456".to_string())
        );

        // 无括号格式
        assert_eq!(
            CupsBackend::parse_job_id("request id is PRINTER-123"),
            Some("PRINTER-123".to_string())
        );

        // 无效格式
        assert_eq!(CupsBackend::parse_job_id("some random output"), None);
    }
}
