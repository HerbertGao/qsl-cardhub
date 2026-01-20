// 日志模块
//
// 提供日志收集、过滤和导出功能

mod collector;
mod models;

pub use collector::LogCollector;
pub use models::{LogEntry, LogLevel};

use std::sync::{Arc, Mutex};

/// 全局日志收集器实例
static LOG_COLLECTOR: once_cell::sync::Lazy<Arc<Mutex<LogCollector>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(LogCollector::new())));

/// 获取全局日志收集器
pub fn get_collector() -> Arc<Mutex<LogCollector>> {
    Arc::clone(&LOG_COLLECTOR)
}

/// 初始化日志系统
///
/// 集成 log crate 和自定义日志收集器
pub fn init_logger(log_dir: std::path::PathBuf) -> anyhow::Result<()> {
    // 初始化日志收集器
    let mut collector = LOG_COLLECTOR.lock().unwrap();
    collector.init_file_logging(log_dir)?;
    drop(collector);

    // 配置 env_logger，同时将日志写入收集器
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format(|buf, record| {
            use std::io::Write;

            // 记录到我们的收集器
            let level = match record.level() {
                log::Level::Error => LogLevel::Error,
                log::Level::Warn => LogLevel::Warning,
                log::Level::Info => LogLevel::Info,
                log::Level::Debug => LogLevel::Debug,
                log::Level::Trace => LogLevel::Debug,
            };

            let source = record
                .module_path()
                .unwrap_or("unknown")
                .to_string();
            let message = format!("{}", record.args());

            let entry = LogEntry::new(level, source, message);

            // 添加到收集器
            if let Ok(mut collector) = LOG_COLLECTOR.lock() {
                collector.add_log(entry);
            }

            // 格式化输出到控制台
            writeln!(
                buf,
                "[{}] [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();

    log::info!("日志系统初始化完成");

    Ok(())
}
