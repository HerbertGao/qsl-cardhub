// 打印机管理 Commands

use crate::printer::PrinterManager;
use std::sync::{Arc, Mutex};
use tauri::State;

/// 应用状态（打印机管理器部分）
pub struct PrinterState {
    pub manager: Arc<Mutex<PrinterManager>>,
}

/// 获取所有可用的打印机列表
#[tauri::command]
pub async fn get_printers(state: State<'_, PrinterState>) -> Result<Vec<String>, String> {
    log::info!("正在获取打印机列表");
    let manager = state
        .manager
        .lock()
        .map_err(|e| format!("锁定失败: {}", e))?;
    let printers = manager
        .list_printers()
        .map_err(|e| format!("获取打印机列表失败: {}", e))?;
    log::info!("找到 {} 台打印机", printers.len());
    Ok(printers)
}

/// 打印 QSL 卡片
///
/// # 参数
/// - `printer_name`: 打印机名称
/// - `callsign`: 呼号
/// - `serial`: 序列号
/// - `qty`: 打印数量
/// - `task_name`: 任务名称（副标题）
#[tauri::command]
pub async fn print_qsl(
    printer_name: String,
    callsign: String,
    serial: u32,
    qty: u32,
    task_name: Option<String>,
    state: State<'_, PrinterState>,
) -> Result<(), String> {
    log::info!(
        "开始打印 QSL 卡片: 呼号={}, 序列号={}, 数量={}, 任务={}, 打印机={}",
        callsign,
        serial,
        qty,
        task_name.as_deref().unwrap_or("无"),
        printer_name
    );
    let manager = state
        .manager
        .lock()
        .map_err(|e| format!("锁定失败: {}", e))?;
    let result = manager.print_qsl(&printer_name, &callsign, serial, qty, task_name.as_deref());

    match &result {
        Ok(_) => log::info!("QSL 卡片打印成功"),
        Err(e) => log::error!("QSL 卡片打印失败: {}", e),
    }

    result.map_err(|e| format!("打印失败: {}", e))
}

/// 打印校准页
///
/// # 参数
/// - `printer_name`: 打印机名称
#[tauri::command]
pub async fn print_calibration(
    printer_name: String,
    state: State<'_, PrinterState>,
) -> Result<(), String> {
    log::info!("开始打印校准页: 打印机={}", printer_name);
    let manager = state
        .manager
        .lock()
        .map_err(|e| format!("锁定失败: {}", e))?;
    let result = manager.print_calibration(&printer_name);

    match &result {
        Ok(_) => log::info!("校准页打印成功"),
        Err(e) => log::error!("校准页打印失败: {}", e),
    }

    result.map_err(|e| format!("打印校准页失败: {}", e))
}
