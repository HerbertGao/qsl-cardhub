// 日志相关的 Tauri Commands

use crate::logger::{get_collector, LogEntry, LogLevel};

/// 获取日志列表
///
/// # 参数
/// - `level`: 日志级别过滤（debug/info/warning/error）
/// - `limit`: 最多返回的日志条数
///
/// # 返回
/// 日志条目列表（最新的在前）
#[tauri::command]
pub fn get_logs(level: Option<String>, limit: Option<usize>) -> Result<Vec<LogEntry>, String> {
    let collector = get_collector();
    let collector = collector.lock().map_err(|e| format!("锁定日志收集器失败: {}", e))?;

    // 解析日志级别
    let level_filter = level.map(|s| LogLevel::from_str(&s));

    // 获取日志
    let logs = collector.get_logs(level_filter, limit);

    Ok(logs)
}

/// 清空内存日志
#[tauri::command]
pub fn clear_logs() -> Result<(), String> {
    let collector = get_collector();
    let mut collector = collector.lock().map_err(|e| format!("锁定日志收集器失败: {}", e))?;

    collector.clear_logs();

    log::info!("内存日志已清空");

    Ok(())
}

/// 导出日志到文件
///
/// # 参数
/// - `export_path`: 导出文件路径
#[tauri::command]
pub fn export_logs(export_path: String) -> Result<String, String> {
    let collector = get_collector();
    let collector = collector.lock().map_err(|e| format!("锁定日志收集器失败: {}", e))?;

    let export_path_buf = std::path::PathBuf::from(&export_path);
    collector
        .export_logs(export_path_buf)
        .map_err(|e| format!("导出日志失败: {}", e))?;

    log::info!("日志已导出到: {}", export_path);

    Ok(export_path)
}

/// 获取日志文件路径
#[tauri::command]
pub fn get_log_file_path() -> Result<Option<String>, String> {
    let collector = get_collector();
    let collector = collector.lock().map_err(|e| format!("锁定日志收集器失败: {}", e))?;

    let path = collector
        .log_file_path()
        .map(|p| p.to_string_lossy().to_string());

    Ok(path)
}
