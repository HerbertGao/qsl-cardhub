// 卡片导出 Tauri 命令
//
// 提供将卡片列表导出为 Excel 文件的功能

use crate::db::{self, CardStatus, CardWithProject, Project};
use chrono::Local;
use rust_xlsxwriter::{Workbook, Format};
use std::io::Cursor;

/// 数量显示模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QtyDisplayMode {
    /// 精确显示
    Exact,
    /// 大致显示
    Approximate,
}

impl QtyDisplayMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "approximate" => Self::Approximate,
            _ => Self::Exact,
        }
    }
}

/// 格式化数量（根据显示模式）
fn format_qty(qty: i32, mode: QtyDisplayMode) -> String {
    match mode {
        QtyDisplayMode::Exact => qty.to_string(),
        QtyDisplayMode::Approximate => {
            if qty <= 10 {
                "≤10".to_string()
            } else if qty <= 50 {
                "≤50".to_string()
            } else {
                ">50".to_string()
            }
        }
    }
}

/// 格式化状态为中文
fn format_status(status: &CardStatus) -> &'static str {
    match status {
        CardStatus::Pending => "待分发",
        CardStatus::Distributed => "已分发",
        CardStatus::Returned => "已退卡",
    }
}

/// 格式化序号为三位数
fn format_serial(serial: Option<i32>) -> String {
    match serial {
        Some(s) => format!("{:03}", s),
        None => String::new(),
    }
}

/// 清理文件名中的非法字符
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if "/\\:*?\"<>|".contains(c) {
                '_'
            } else {
                c
            }
        })
        .collect()
}

/// 生成默认文件名
fn generate_filename(project_name: &str) -> String {
    let sanitized_name = sanitize_filename(project_name);
    let timestamp = Local::now().format("%Y%m%d%H%M%S");
    format!("{}_{}.xlsx", sanitized_name, timestamp)
}

/// 生成 Excel 文件内容
fn generate_excel(cards: &[CardWithProject], qty_mode: QtyDisplayMode) -> Result<Vec<u8>, String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // 设置表头格式
    let header_format = Format::new().set_bold();

    // 写入表头
    worksheet
        .write_string_with_format(0, 0, "序号", &header_format)
        .map_err(|e| format!("写入表头失败: {}", e))?;
    worksheet
        .write_string_with_format(0, 1, "呼号", &header_format)
        .map_err(|e| format!("写入表头失败: {}", e))?;
    worksheet
        .write_string_with_format(0, 2, "数量", &header_format)
        .map_err(|e| format!("写入表头失败: {}", e))?;
    worksheet
        .write_string_with_format(0, 3, "状态", &header_format)
        .map_err(|e| format!("写入表头失败: {}", e))?;

    // 写入数据行
    for (i, card) in cards.iter().enumerate() {
        let row = (i + 1) as u32;

        // 序号
        let serial_str = format_serial(card.serial);
        worksheet
            .write_string(row, 0, &serial_str)
            .map_err(|e| format!("写入数据失败: {}", e))?;

        // 呼号
        worksheet
            .write_string(row, 1, &card.callsign)
            .map_err(|e| format!("写入数据失败: {}", e))?;

        // 数量
        let qty_str = format_qty(card.qty, qty_mode);
        worksheet
            .write_string(row, 2, &qty_str)
            .map_err(|e| format!("写入数据失败: {}", e))?;

        // 状态
        let status_str = format_status(&card.status);
        worksheet
            .write_string(row, 3, status_str)
            .map_err(|e| format!("写入数据失败: {}", e))?;
    }

    // 设置列宽
    worksheet.set_column_width(0, 10).ok(); // 序号
    worksheet.set_column_width(1, 15).ok(); // 呼号
    worksheet.set_column_width(2, 10).ok(); // 数量
    worksheet.set_column_width(3, 10).ok(); // 状态

    // 保存到内存
    let mut buffer = Cursor::new(Vec::new());
    workbook
        .save_to_writer(&mut buffer)
        .map_err(|e| format!("保存 Excel 失败: {}", e))?;

    Ok(buffer.into_inner())
}

/// 导出结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExportResult {
    /// 是否成功
    pub success: bool,
    /// 导出的文件路径（成功时）
    pub file_path: Option<String>,
    /// 错误消息（失败时）
    pub error: Option<String>,
    /// 是否被用户取消
    pub cancelled: bool,
}

/// 导出卡片到 Excel
///
/// 返回导出结果，包含是否成功、文件路径或错误信息
#[tauri::command]
pub async fn export_cards_to_excel(
    app: tauri::AppHandle,
    project_id: String,
    qty_display_mode: String,
) -> Result<ExportResult, String> {
    use tauri_plugin_dialog::DialogExt;

    // 在后台线程中查询数据
    let project_id_clone = project_id.clone();
    let (project, cards) = tokio::task::spawn_blocking(move || {
        // 获取项目信息
        let project = db::get_project(&project_id_clone)
            .map_err(|e| format!("获取项目失败: {}", e))?
            .ok_or_else(|| format!("项目不存在: {}", project_id_clone))?;

        // 获取所有卡片（不分页）
        let filter = db::CardFilter {
            project_id: Some(project_id_clone),
            callsign: None,
            status: None,
        };
        let pagination = db::Pagination {
            page: 1,
            page_size: 100000, // 足够大以获取所有卡片
        };
        let paged = db::list_cards(filter, pagination)
            .map_err(|e| format!("获取卡片列表失败: {}", e))?;

        Ok::<(Project, Vec<CardWithProject>), String>((project, paged.items))
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))??;

    // 检查是否有卡片
    if cards.is_empty() {
        return Ok(ExportResult {
            success: false,
            file_path: None,
            error: Some("当前项目没有卡片可导出".to_string()),
            cancelled: false,
        });
    }

    // 生成 Excel 内容
    let qty_mode = QtyDisplayMode::from_str(&qty_display_mode);
    let excel_data = generate_excel(&cards, qty_mode)?;

    // 生成默认文件名
    let default_filename = generate_filename(&project.name);

    // 使用 channel 等待对话框结果
    let (tx, rx) = tokio::sync::oneshot::channel();

    // 弹出保存对话框
    app.dialog()
        .file()
        .set_file_name(&default_filename)
        .add_filter("Excel 文件", &["xlsx"])
        .save_file(move |file_path| {
            let _ = tx.send(file_path);
        });

    // 等待用户选择
    let file_path = rx.await.map_err(|_| "对话框已关闭".to_string())?;

    let file_path = match file_path {
        Some(path) => path,
        None => {
            // 用户取消
            return Ok(ExportResult {
                success: false,
                file_path: None,
                error: None,
                cancelled: true,
            });
        }
    };

    // 写入文件
    let path_str = file_path.to_string();
    std::fs::write(&path_str, &excel_data)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    log::info!("✅ 导出卡片到 Excel 成功: {}", path_str);

    Ok(ExportResult {
        success: true,
        file_path: Some(path_str),
        error: None,
        cancelled: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_serial() {
        assert_eq!(format_serial(Some(1)), "001");
        assert_eq!(format_serial(Some(12)), "012");
        assert_eq!(format_serial(Some(123)), "123");
        assert_eq!(format_serial(None), "");
    }

    #[test]
    fn test_format_qty_exact() {
        assert_eq!(format_qty(1, QtyDisplayMode::Exact), "1");
        assert_eq!(format_qty(15, QtyDisplayMode::Exact), "15");
        assert_eq!(format_qty(100, QtyDisplayMode::Exact), "100");
    }

    #[test]
    fn test_format_qty_approximate() {
        // ≤10
        assert_eq!(format_qty(8, QtyDisplayMode::Approximate), "≤10");
        assert_eq!(format_qty(10, QtyDisplayMode::Approximate), "≤10");
        // ≤50
        assert_eq!(format_qty(11, QtyDisplayMode::Approximate), "≤50");
        assert_eq!(format_qty(35, QtyDisplayMode::Approximate), "≤50");
        assert_eq!(format_qty(50, QtyDisplayMode::Approximate), "≤50");
        // >50
        assert_eq!(format_qty(51, QtyDisplayMode::Approximate), ">50");
        assert_eq!(format_qty(100, QtyDisplayMode::Approximate), ">50");
    }

    #[test]
    fn test_format_status() {
        assert_eq!(format_status(&CardStatus::Pending), "待分发");
        assert_eq!(format_status(&CardStatus::Distributed), "已分发");
        assert_eq!(format_status(&CardStatus::Returned), "已退卡");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Test/Project:2026"), "Test_Project_2026");
        assert_eq!(sanitize_filename("Normal Name"), "Normal Name");
        assert_eq!(sanitize_filename("a<b>c*d?e"), "a_b_c_d_e");
    }

    #[test]
    fn test_generate_filename() {
        let filename = generate_filename("Test Project");
        assert!(filename.starts_with("Test Project_"));
        assert!(filename.ends_with(".xlsx"));
        // 文件名应该包含时间戳（14位数字）
        let parts: Vec<&str> = filename.split('_').collect();
        assert!(parts.len() >= 2);
    }
}
