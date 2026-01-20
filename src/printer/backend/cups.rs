// CUPS 打印后端
//
// 用于 macOS 和 Linux 平台，通过 lp 命令行工具与 CUPS 交互

#[cfg(target_family = "unix")]
use super::PrinterBackend;

#[cfg(target_family = "unix")]
use anyhow::{Context, Result};

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

    fn send_raw(&self, printer_name: &str, data: &[u8]) -> Result<()> {
        // 使用 lp 命令发送原始数据
        // lp -d <printer> -o raw -
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

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("打印失败: {}", stderr);
        }

        println!("✅ 打印成功: {} ({} 字节)", printer_name, data.len());

        Ok(())
    }
}

#[cfg(target_family = "unix")]
impl Default for CupsBackend {
    fn default() -> Self {
        Self::new()
    }
}
