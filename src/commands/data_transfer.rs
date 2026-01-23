// 数据传输命令
//
// 提供数据导出和导入的 Tauri 命令

use crate::db::export::{export_database, get_export_stats, ExportStats};
use crate::db::import::{execute_import, preview_import, ImportPreview};
use std::fs;
use tauri::command;

/// 导出数据到文件
#[command]
pub async fn export_data(file_path: String) -> Result<ExportStats, String> {
    log::info!("📤 导出数据到: {}", file_path);

    // 导出数据
    let data = export_database().map_err(|e| e.to_string())?;

    // 获取统计信息
    let stats = get_export_stats(&data);

    // 序列化为 JSON
    let json = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("序列化数据失败: {}", e))?;

    // 写入文件
    fs::write(&file_path, json)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    log::info!(
        "✅ 数据导出成功: {} 个项目, {} 张卡片, {} 个寄件人, {} 个订单",
        stats.projects,
        stats.cards,
        stats.sf_senders,
        stats.sf_orders
    );

    Ok(stats)
}

/// 预览导入文件
#[command]
pub async fn preview_import_data(file_path: String) -> Result<ImportPreview, String> {
    log::info!("📂 预览导入文件: {}", file_path);
    preview_import(&file_path).map_err(|e| e.to_string())
}

/// 执行数据导入
#[command]
pub async fn import_data(file_path: String) -> Result<ExportStats, String> {
    log::info!("📥 导入数据从: {}", file_path);

    let stats = execute_import(&file_path).map_err(|e| e.to_string())?;

    log::info!(
        "✅ 数据导入成功: {} 个项目, {} 张卡片, {} 个寄件人, {} 个订单",
        stats.projects,
        stats.cards,
        stats.sf_senders,
        stats.sf_orders
    );

    Ok(stats)
}
