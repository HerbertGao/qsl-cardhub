// 日志数据模型

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// 调试信息
    Debug,
    /// 一般信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
}

impl LogLevel {
    /// 从字符串解析日志级别
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warning" | "warn" => LogLevel::Warning,
            "error" => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
        }
    }

    /// 获取日志级别的优先级（用于过滤）
    pub fn priority(&self) -> u8 {
        match self {
            LogLevel::Debug => 0,
            LogLevel::Info => 1,
            LogLevel::Warning => 2,
            LogLevel::Error => 3,
        }
    }
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// 时间戳
    pub timestamp: DateTime<Local>,
    /// 日志级别
    pub level: LogLevel,
    /// 日志来源（模块名）
    pub source: String,
    /// 日志消息
    pub message: String,
}

impl LogEntry {
    /// 创建新的日志条目
    pub fn new(level: LogLevel, source: String, message: String) -> Self {
        Self {
            timestamp: Local::now(),
            level,
            source,
            message,
        }
    }

    /// 格式化为字符串（用于文件输出）
    pub fn format(&self) -> String {
        format!(
            "[{}] [{}] [{}] {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.level.as_str(),
            self.source,
            self.message
        )
    }
}
