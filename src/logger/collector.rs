// 日志收集器
//
// 使用环形缓冲区收集日志，支持文件持久化

use super::models::{LogEntry, LogLevel};
use anyhow::{Context, Result};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// 日志收集器
pub struct LogCollector {
    /// 内存日志缓冲区（环形缓冲区，最多 1000 条）
    buffer: VecDeque<LogEntry>,
    /// 最大缓冲区大小
    max_buffer_size: usize,
    /// 日志文件路径
    log_file_path: Option<PathBuf>,
    /// 日志文件句柄
    log_file: Option<File>,
}

impl LogCollector {
    /// 创建新的日志收集器
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::with_capacity(1000),
            max_buffer_size: 1000,
            log_file_path: None,
            log_file: None,
        }
    }

    /// 初始化文件日志
    ///
    /// # 参数
    /// - `log_dir`: 日志文件目录
    pub fn init_file_logging(&mut self, log_dir: PathBuf) -> Result<()> {
        // 确保日志目录存在
        if !log_dir.exists() {
            std::fs::create_dir_all(&log_dir).context("创建日志目录失败")?;
        }

        // 生成日志文件名（按日期）
        let date_str = chrono::Local::now().format("%Y-%m-%d");
        let log_file_path = log_dir.join(format!("qsl-cardhub-{}.log", date_str));

        // 打开日志文件（追加模式）
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)
            .context("打开日志文件失败")?;

        self.log_file_path = Some(log_file_path.clone());
        self.log_file = Some(log_file);

        println!("📝 日志文件: {}", log_file_path.display());

        Ok(())
    }

    /// 添加日志条目
    ///
    /// 将日志添加到内存缓冲区，并写入文件（如果已配置）
    pub fn add_log(&mut self, entry: LogEntry) {
        // 写入文件
        if let Some(ref mut file) = self.log_file {
            let log_line = format!("{}\n", entry.format());
            let _ = file.write_all(log_line.as_bytes());
            let _ = file.flush();
        }

        // 添加到缓冲区
        if self.buffer.len() >= self.max_buffer_size {
            self.buffer.pop_front(); // 移除最旧的日志
        }
        self.buffer.push_back(entry);
    }

    /// 添加日志（便捷方法）
    #[allow(dead_code)]
    pub fn log(&mut self, level: LogLevel, source: &str, message: &str) {
        let entry = LogEntry::new(level, source.to_string(), message.to_string());
        self.add_log(entry);
    }

    /// 获取所有日志
    ///
    /// # 参数
    /// - `level_filter`: 日志级别过滤（None 表示不过滤）
    /// - `limit`: 最多返回的日志条数（None 表示不限制）
    pub fn get_logs(&self, level_filter: Option<LogLevel>, limit: Option<usize>) -> Vec<LogEntry> {
        let mut logs: Vec<LogEntry> = self
            .buffer
            .iter()
            .filter(|entry| {
                if let Some(filter_level) = level_filter {
                    entry.level.priority() >= filter_level.priority()
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        // 逆序（最新的在前）
        logs.reverse();

        // 限制数量
        if let Some(limit_count) = limit {
            logs.truncate(limit_count);
        }

        logs
    }

    /// 清空内存日志
    pub fn clear_logs(&mut self) {
        self.buffer.clear();
    }

    /// 导出日志到文件
    ///
    /// # 参数
    /// - `export_path`: 导出文件路径
    pub fn export_logs(&self, export_path: PathBuf) -> Result<()> {
        let mut file = File::create(&export_path).context("创建导出文件失败")?;

        for entry in self.buffer.iter() {
            writeln!(file, "{}", entry.format()).context("写入导出文件失败")?;
        }

        file.flush().context("刷新导出文件失败")?;

        Ok(())
    }

    /// 获取日志文件路径
    pub fn log_file_path(&self) -> Option<&PathBuf> {
        self.log_file_path.as_ref()
    }
}

impl Default for LogCollector {
    fn default() -> Self {
        Self::new()
    }
}
