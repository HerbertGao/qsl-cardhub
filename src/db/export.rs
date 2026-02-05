// 数据导出模块
//
// 将本地数据库导出为 JSON 格式文件

use crate::db::models::{Card, Project};
use crate::db::sqlite::{get_connection, get_db_version, format_version};
use crate::error::AppError;
use crate::sf_express::{RecipientInfo, SFOrder, SenderInfo};
use serde::{Deserialize, Serialize};

/// 导出格式版本
pub const EXPORT_FORMAT_VERSION: &str = "1.0";

/// 导出数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    /// 导出格式版本
    pub version: String,
    /// 数据库版本号（整数）
    pub db_version: i32,
    /// 可读版本号（如 "2026.1.23.003"）
    pub db_version_display: String,
    /// 应用版本号
    pub app_version: String,
    /// 导出时间戳（ISO 8601 格式）
    pub exported_at: String,
    /// 表数据
    pub tables: ExportTables,
}

/// 导出的表数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportTables {
    /// 项目列表
    pub projects: Vec<Project>,
    /// 卡片列表
    pub cards: Vec<Card>,
    /// 顺丰寄件人列表
    pub sf_senders: Vec<SenderInfo>,
    /// 顺丰订单列表
    pub sf_orders: Vec<SFOrder>,
}

/// 导出统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStats {
    /// 项目数
    pub projects: u32,
    /// 卡片数
    pub cards: u32,
    /// 寄件人数
    pub sf_senders: u32,
    /// 订单数
    pub sf_orders: u32,
}

/// 导出所有数据
pub fn export_database() -> Result<ExportData, AppError> {
    let conn = get_connection()?;

    // 获取数据库版本
    let db_version = get_db_version(&conn)?;
    let db_version_display = format_version(db_version);

    // 获取应用版本
    let app_version = env!("CARGO_PKG_VERSION").to_string();

    // 导出时间
    let exported_at = crate::db::models::format_datetime(&crate::db::models::now_china());

    // 导出所有项目
    let projects = export_projects(&conn)?;

    // 导出所有卡片
    let cards = export_cards(&conn)?;

    // 导出所有寄件人
    let sf_senders = export_senders(&conn)?;

    // 导出所有订单
    let sf_orders = export_orders(&conn)?;

    log::info!(
        "📦 导出数据完成: {} 个项目, {} 张卡片, {} 个寄件人, {} 个订单",
        projects.len(),
        cards.len(),
        sf_senders.len(),
        sf_orders.len()
    );

    Ok(ExportData {
        version: EXPORT_FORMAT_VERSION.to_string(),
        db_version,
        db_version_display,
        app_version,
        exported_at,
        tables: ExportTables {
            projects,
            cards,
            sf_senders,
            sf_orders,
        },
    })
}

/// 导出项目列表
fn export_projects(conn: &rusqlite::Connection) -> Result<Vec<Project>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, name, created_at, updated_at FROM projects ORDER BY created_at")
        .map_err(|e| AppError::Other(format!("准备项目查询失败: {}", e)))?;

    let projects = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(|e| AppError::Other(format!("查询项目失败: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Other(format!("读取项目数据失败: {}", e)))?;

    Ok(projects)
}

/// 导出卡片列表
fn export_cards(conn: &rusqlite::Connection) -> Result<Vec<Card>, AppError> {
    use crate::db::models::CardStatus;

    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, creator_id, callsign, qty, serial, status, metadata, created_at, updated_at
             FROM cards ORDER BY created_at"
        )
        .map_err(|e| AppError::Other(format!("准备卡片查询失败: {}", e)))?;

    let cards = stmt
        .query_map([], |row| {
            let status_str: String = row.get(6)?;
            let metadata_str: Option<String> = row.get(7)?;

            Ok(Card {
                id: row.get(0)?,
                project_id: row.get(1)?,
                creator_id: row.get(2)?,
                callsign: row.get(3)?,
                qty: row.get(4)?,
                serial: row.get(5)?,
                status: CardStatus::from_str(&status_str).unwrap_or(CardStatus::Pending),
                metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| AppError::Other(format!("查询卡片失败: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Other(format!("读取卡片数据失败: {}", e)))?;

    Ok(cards)
}

/// 导出寄件人列表
fn export_senders(conn: &rusqlite::Connection) -> Result<Vec<SenderInfo>, AppError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, phone, mobile, province, city, district, address, is_default, created_at, updated_at
             FROM sf_senders ORDER BY created_at"
        )
        .map_err(|e| AppError::Other(format!("准备寄件人查询失败: {}", e)))?;

    let senders = stmt
        .query_map([], |row| {
            Ok(SenderInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                phone: row.get(2)?,
                mobile: row.get(3)?,
                province: row.get(4)?,
                city: row.get(5)?,
                district: row.get(6)?,
                address: row.get(7)?,
                is_default: row.get::<_, i32>(8)? != 0,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| AppError::Other(format!("查询寄件人失败: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Other(format!("读取寄件人数据失败: {}", e)))?;

    Ok(senders)
}

/// 导出订单列表
fn export_orders(conn: &rusqlite::Connection) -> Result<Vec<SFOrder>, AppError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, order_id, waybill_no, card_id, status, pay_method, cargo_name, sender_info, recipient_info, created_at, updated_at
             FROM sf_orders ORDER BY created_at"
        )
        .map_err(|e| AppError::Other(format!("准备订单查询失败: {}", e)))?;

    let orders = stmt
        .query_map([], |row| {
            let sender_info_json: String = row.get(7)?;
            let recipient_info_json: String = row.get(8)?;

            let sender_info: SenderInfo = serde_json::from_str(&sender_info_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    7,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))?;
            let recipient_info: RecipientInfo = serde_json::from_str(&recipient_info_json)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    8,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))?;

            Ok(SFOrder {
                id: row.get(0)?,
                order_id: row.get(1)?,
                waybill_no: row.get(2)?,
                card_id: row.get(3)?,
                status: row.get(4)?,
                pay_method: row.get(5)?,
                cargo_name: row.get(6)?,
                sender_info,
                recipient_info,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| AppError::Other(format!("查询订单失败: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Other(format!("读取订单数据失败: {}", e)))?;

    Ok(orders)
}

/// 获取导出统计信息
pub fn get_export_stats(data: &ExportData) -> ExportStats {
    ExportStats {
        projects: data.tables.projects.len() as u32,
        cards: data.tables.cards.len() as u32,
        sf_senders: data.tables.sf_senders.len() as u32,
        sf_orders: data.tables.sf_orders.len() as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format_version() {
        assert_eq!(EXPORT_FORMAT_VERSION, "1.0");
    }
}
