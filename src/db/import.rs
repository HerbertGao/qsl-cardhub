// 数据导入模块
//
// 从 JSON 格式文件导入数据到本地数据库

use crate::db::export::{ExportData, ExportStats, ExportTables, EXPORT_FORMAT_VERSION};
use crate::db::models::{Card, Project};
use crate::db::sqlite::{format_version, get_connection, get_db_version};
use crate::error::AppError;
use crate::sf_express::{RecipientInfo, SFOrder, SenderInfo};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 支持的导出格式版本
const SUPPORTED_VERSIONS: &[&str] = &["1.0", "1.1"];

// ==================== v1.0 兼容类型 ====================

/// v1.0 格式的订单（sender_info/recipient_info 是 JSON 字符串）
#[derive(Debug, Clone, Deserialize)]
struct SFOrderV1_0 {
    pub id: String,
    pub order_id: String,
    pub waybill_no: Option<String>,
    pub card_id: Option<String>,
    pub status: String,
    pub pay_method: Option<i32>,
    pub cargo_name: Option<String>,
    pub sender_info: String,      // v1.0: JSON 字符串
    pub recipient_info: String,   // v1.0: JSON 字符串
    pub created_at: String,
    pub updated_at: String,
}

/// v1.0 格式的表数据
#[derive(Debug, Clone, Deserialize)]
struct ExportTablesV1_0 {
    pub projects: Vec<Project>,
    pub cards: Vec<Card>,
    pub sf_senders: Vec<SenderInfo>,
    pub sf_orders: Vec<SFOrderV1_0>,
}

/// v1.0 格式的导出数据
#[derive(Debug, Clone, Deserialize)]
struct ExportDataV1_0 {
    pub version: String,
    pub db_version: i32,
    pub db_version_display: String,
    pub app_version: String,
    pub exported_at: String,
    pub tables: ExportTablesV1_0,
}

impl ExportDataV1_0 {
    /// 转换为当前版本格式
    fn into_current(self) -> Result<ExportData, AppError> {
        let mut sf_orders = Vec::with_capacity(self.tables.sf_orders.len());

        for order in self.tables.sf_orders {
            let sender_info: SenderInfo = serde_json::from_str(&order.sender_info)
                .map_err(|e| AppError::Other(format!(
                    "解析订单 {} 的寄件人信息失败: {}", order.id, e
                )))?;
            let recipient_info: RecipientInfo = serde_json::from_str(&order.recipient_info)
                .map_err(|e| AppError::Other(format!(
                    "解析订单 {} 的收件人信息失败: {}", order.id, e
                )))?;

            sf_orders.push(SFOrder {
                id: order.id,
                order_id: order.order_id,
                waybill_no: order.waybill_no,
                card_id: order.card_id,
                status: order.status,
                pay_method: order.pay_method,
                cargo_name: order.cargo_name,
                sender_info,
                recipient_info,
                created_at: order.created_at,
                updated_at: order.updated_at,
            });
        }

        Ok(ExportData {
            version: EXPORT_FORMAT_VERSION.to_string(), // 升级到当前版本
            db_version: self.db_version,
            db_version_display: self.db_version_display,
            app_version: self.app_version,
            exported_at: self.exported_at,
            tables: ExportTables {
                projects: self.tables.projects,
                cards: self.tables.cards,
                sf_senders: self.tables.sf_senders,
                sf_orders,
            },
        })
    }
}

/// 解析导出数据，支持多版本
fn parse_export_data(content: &str) -> Result<ExportData, AppError> {
    // 先解析版本号
    let version_check: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| AppError::Other(format!("文件格式错误: {}", e)))?;

    let version = version_check.get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Other("文件缺少版本信息".to_string()))?;

    // 检查是否支持该版本
    if !SUPPORTED_VERSIONS.contains(&version) {
        return Err(AppError::Other(format!(
            "不支持的导出格式版本: {}，当前支持版本: {}",
            version,
            SUPPORTED_VERSIONS.join(", ")
        )));
    }

    // 根据版本解析
    match version {
        "1.0" => {
            log::info!("📦 检测到 v1.0 格式，将自动转换为当前格式");
            let data_v1: ExportDataV1_0 = serde_json::from_str(content)
                .map_err(|e| AppError::Other(format!("解析 v1.0 格式失败: {}", e)))?;
            data_v1.into_current()
        }
        "1.1" | _ => {
            serde_json::from_str(content)
                .map_err(|e| AppError::Other(format!("解析文件失败: {}", e)))
        }
    }
}

/// 导入预览信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    /// 文件格式版本
    pub version: String,
    /// 数据库版本号（整数）
    pub db_version: i32,
    /// 可读版本号
    pub db_version_display: String,
    /// 应用版本号
    pub app_version: String,
    /// 导出时间
    pub exported_at: String,
    /// 数据统计
    pub stats: ExportStats,
    /// 是否可以导入
    pub can_import: bool,
    /// 错误信息（如果不能导入）
    pub error_message: Option<String>,
    /// 本地数据库版本号
    pub local_db_version: i32,
    /// 本地可读版本号
    pub local_db_version_display: String,
}

/// 预览导入文件
///
/// 解析导入文件并检查版本兼容性，返回预览信息
pub fn preview_import<P: AsRef<Path>>(file_path: P) -> Result<ImportPreview, AppError> {
    let file_path = file_path.as_ref();

    // 读取文件内容
    let content = fs::read_to_string(file_path).map_err(|e| {
        AppError::Other(format!("无法读取文件: {}", e))
    })?;

    // 解析 JSON（支持多版本）
    let data = parse_export_data(&content)?;

    // 获取本地数据库版本
    let conn = get_connection()?;
    let local_db_version = get_db_version(&conn)?;
    let local_db_version_display = format_version(local_db_version);

    // 检查数据库版本兼容性
    let (can_import, error_message) = if data.db_version > local_db_version {
        (
            false,
            Some(format!(
                "导入文件的数据库版本（{}）高于本地版本（{}），请升级应用后再导入",
                data.db_version_display, local_db_version_display
            )),
        )
    } else {
        (true, None)
    };

    // 计算统计信息
    let stats = ExportStats {
        projects: data.tables.projects.len() as u32,
        cards: data.tables.cards.len() as u32,
        sf_senders: data.tables.sf_senders.len() as u32,
        sf_orders: data.tables.sf_orders.len() as u32,
    };

    Ok(ImportPreview {
        version: data.version,
        db_version: data.db_version,
        db_version_display: data.db_version_display,
        app_version: data.app_version,
        exported_at: data.exported_at,
        stats,
        can_import,
        error_message,
        local_db_version,
        local_db_version_display,
    })
}

/// 执行导入
///
/// 清空现有数据并导入新数据（事务保证原子性）
pub fn execute_import<P: AsRef<Path>>(file_path: P) -> Result<ExportStats, AppError> {
    let file_path = file_path.as_ref();

    // 读取文件内容
    let content = fs::read_to_string(file_path).map_err(|e| {
        AppError::Other(format!("无法读取文件: {}", e))
    })?;

    // 解析 JSON（支持多版本）
    let data = parse_export_data(&content)?;

    // 验证版本
    let conn = get_connection()?;
    let local_db_version = get_db_version(&conn)?;

    if data.db_version > local_db_version {
        return Err(AppError::Other(format!(
            "导入文件的数据库版本（{}）高于本地版本（{}），请升级应用后再导入",
            data.db_version_display, format_version(local_db_version)
        )));
    }

    // 开始事务
    let mut conn = get_connection()?;
    let tx = conn.transaction().map_err(|e| {
        AppError::Other(format!("无法开始事务: {}", e))
    })?;

    // 清空现有数据（按外键依赖顺序）
    tx.execute("DELETE FROM sf_orders", [])
        .map_err(|e| AppError::Other(format!("清空订单表失败: {}", e)))?;
    tx.execute("DELETE FROM sf_senders", [])
        .map_err(|e| AppError::Other(format!("清空寄件人表失败: {}", e)))?;
    tx.execute("DELETE FROM cards", [])
        .map_err(|e| AppError::Other(format!("清空卡片表失败: {}", e)))?;
    tx.execute("DELETE FROM projects", [])
        .map_err(|e| AppError::Other(format!("清空项目表失败: {}", e)))?;

    log::info!("🗑️  已清空现有数据");

    // 导入项目
    for project in &data.tables.projects {
        tx.execute(
            "INSERT INTO projects (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                &project.id,
                &project.name,
                &project.created_at,
                &project.updated_at,
            ],
        )
        .map_err(|e| AppError::Other(format!("导入项目失败 ({}): {}", project.id, e)))?;
    }
    log::info!("📦 导入 {} 个项目", data.tables.projects.len());

    // 导入卡片
    for card in &data.tables.cards {
        let metadata_json = card
            .metadata
            .as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default());

        tx.execute(
            "INSERT INTO cards (id, project_id, creator_id, callsign, qty, status, metadata, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                &card.id,
                &card.project_id,
                &card.creator_id,
                &card.callsign,
                card.qty,
                card.status.as_str(),
                metadata_json,
                &card.created_at,
                &card.updated_at,
            ],
        )
        .map_err(|e| AppError::Other(format!("导入卡片失败 ({}): {}", card.id, e)))?;
    }
    log::info!("📦 导入 {} 张卡片", data.tables.cards.len());

    // 导入寄件人
    for sender in &data.tables.sf_senders {
        tx.execute(
            "INSERT INTO sf_senders (id, name, phone, mobile, province, city, district, address, is_default, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                &sender.id,
                &sender.name,
                &sender.phone,
                &sender.mobile,
                &sender.province,
                &sender.city,
                &sender.district,
                &sender.address,
                sender.is_default as i32,
                &sender.created_at,
                &sender.updated_at,
            ],
        )
        .map_err(|e| AppError::Other(format!("导入寄件人失败 ({}): {}", sender.id, e)))?;
    }
    log::info!("📦 导入 {} 个寄件人", data.tables.sf_senders.len());

    // 导入订单
    for order in &data.tables.sf_orders {
        // 序列化为 JSON 字符串
        let sender_info_json = serde_json::to_string(&order.sender_info)
            .map_err(|e| AppError::Other(format!("序列化寄件人信息失败: {}", e)))?;
        let recipient_info_json = serde_json::to_string(&order.recipient_info)
            .map_err(|e| AppError::Other(format!("序列化收件人信息失败: {}", e)))?;

        tx.execute(
            "INSERT INTO sf_orders (id, order_id, waybill_no, card_id, status, pay_method, cargo_name, sender_info, recipient_info, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                &order.id,
                &order.order_id,
                &order.waybill_no,
                &order.card_id,
                &order.status,
                &order.pay_method,
                &order.cargo_name,
                &sender_info_json,
                &recipient_info_json,
                &order.created_at,
                &order.updated_at,
            ],
        )
        .map_err(|e| AppError::Other(format!("导入订单失败 ({}): {}", order.id, e)))?;
    }
    log::info!("📦 导入 {} 个订单", data.tables.sf_orders.len());

    // 提交事务
    tx.commit().map_err(|e| {
        AppError::Other(format!("提交事务失败: {}", e))
    })?;

    log::info!("✅ 数据导入完成");

    Ok(ExportStats {
        projects: data.tables.projects.len() as u32,
        cards: data.tables.cards.len() as u32,
        sf_senders: data.tables.sf_senders.len() as u32,
        sf_orders: data.tables.sf_orders.len() as u32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_preview_file_not_found() {
        let result = preview_import("/nonexistent/file.qslhub");
        assert!(result.is_err());
    }
}
