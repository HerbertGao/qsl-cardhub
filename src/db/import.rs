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
            client_id: None,
            tables: ExportTables {
                projects: self.tables.projects,
                cards: self.tables.cards,
                sf_senders: self.tables.sf_senders,
                sf_orders,
                app_settings: None,
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

/// `app_settings` 表的清空策略
///
/// 文件导入与「从云端恢复」对 `app_settings` 的清空语义不同。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppSettingsClearMode {
    /// 条件清空：仅当 `data.tables.app_settings` 含值时才清空并写入（文件导入沿用此语义）
    Conditional,
    /// 无条件清空：始终清空全部配置后再写入快照（从云端恢复必须用此，避免空配置时旧配置幽灵残留）
    Unconditional,
}

/// 共用导入内核：在单个 rusqlite 事务内清空 5 张业务表并按 `ExportData` 重建
///
/// 文件导入与「从云端恢复」共用本内核，避免维护两份并行的 DELETE/INSERT。
/// 注意：本内核**不**处理 `client_id` 恢复（该逻辑只属文件导入侧，见 `execute_import`）。
///
/// `app_settings` 的清空策略由 `app_settings_mode` 决定：
/// - `Conditional`：仅当快照含 `app_settings` 字段时才清空+写入（保留现有配置）
/// - `Unconditional`：无条件清空全部配置后写入快照（云端恢复语义，空快照也清空本地旧配置）
pub fn import_from_export_data(
    conn: &mut rusqlite::Connection,
    data: &ExportData,
    app_settings_mode: AppSettingsClearMode,
) -> Result<(), AppError> {
    // 开始事务
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
            "INSERT INTO cards (id, project_id, creator_id, callsign, qty, serial, status, metadata, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                &card.id,
                &card.project_id,
                &card.creator_id,
                &card.callsign,
                card.qty,
                card.serial,
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

    // 导入全局配置
    match app_settings_mode {
        AppSettingsClearMode::Conditional => {
            // 文件导入：仅当快照含 app_settings 字段时才清空并写入，否则保留现有配置
            if let Some(ref settings) = data.tables.app_settings {
                tx.execute("DELETE FROM app_settings", [])
                    .map_err(|e| AppError::Other(format!("清空配置表失败: {}", e)))?;

                for setting in settings {
                    tx.execute(
                        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)",
                        rusqlite::params![&setting.key, &setting.value],
                    )
                    .map_err(|e| AppError::Other(format!("导入配置失败 ({}): {}", setting.key, e)))?;
                }
                log::info!("📦 导入 {} 个配置项", settings.len());
            } else {
                log::info!("📦 导入文件不含配置项，保留现有配置");
            }
        }
        AppSettingsClearMode::Unconditional => {
            // 从云端恢复：无条件清空全部配置后写入快照（即便快照为空也清空本地旧配置）
            tx.execute("DELETE FROM app_settings", [])
                .map_err(|e| AppError::Other(format!("清空配置表失败: {}", e)))?;

            let settings = data.tables.app_settings.as_deref().unwrap_or(&[]);
            for setting in settings {
                tx.execute(
                    "INSERT INTO app_settings (key, value) VALUES (?1, ?2)",
                    rusqlite::params![&setting.key, &setting.value],
                )
                .map_err(|e| AppError::Other(format!("导入配置失败 ({}): {}", setting.key, e)))?;
            }
            log::info!("📦 无条件清空并导入 {} 个配置项", settings.len());
        }
    }

    // 提交事务
    tx.commit().map_err(|e| {
        AppError::Other(format!("提交事务失败: {}", e))
    })?;

    Ok(())
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

    // 复用共用导入内核（文件导入侧 app_settings 沿用条件清空语义）
    let mut conn = get_connection()?;
    import_from_export_data(&mut conn, &data, AppSettingsClearMode::Conditional)?;

    // 恢复 client_id 到同步配置
    if let Some(client_id) = data.client_id.as_deref().filter(|s| !s.is_empty()) {
        log::info!("🔄 恢复同步 client_id: {}", client_id);
        match crate::sync::config::load_sync_config() {
            Ok(Some(mut config)) => {
                config.client_id = client_id.to_string();
                if let Err(e) = crate::sync::config::save_sync_config(&config) {
                    log::warn!("恢复 client_id 失败: {}", e);
                }
            }
            Ok(None) => {
                // sync.toml 不存在，创建一个仅含 client_id 的默认配置
                let config = crate::sync::config::SyncConfig {
                    client_id: client_id.to_string(),
                    ..Default::default()
                };
                if let Err(e) = crate::sync::config::save_sync_config(&config) {
                    log::warn!("创建同步配置失败: {}", e);
                }
            }
            Err(e) => {
                log::warn!("加载同步配置失败，跳过 client_id 恢复: {}", e);
            }
        }
    } else {
        log::info!("📦 导入文件不含 client_id，跳过同步身份恢复");
    }

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
    use crate::db::models::{AppSetting, CardStatus};
    use rusqlite::Connection;

    #[test]
    fn test_import_preview_file_not_found() {
        let result = preview_import("/nonexistent/file.qslhub");
        assert!(result.is_err());
    }

    /// 创建一个含 5 张业务表的内存库（schema 与迁移一致）
    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE cards (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                creator_id TEXT,
                callsign TEXT NOT NULL,
                qty INTEGER NOT NULL,
                serial INTEGER,
                status TEXT NOT NULL DEFAULT 'pending',
                metadata TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE sf_senders (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                phone TEXT NOT NULL,
                mobile TEXT,
                province TEXT NOT NULL,
                city TEXT NOT NULL,
                district TEXT NOT NULL,
                address TEXT NOT NULL,
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE sf_orders (
                id TEXT PRIMARY KEY,
                order_id TEXT NOT NULL UNIQUE,
                waybill_no TEXT,
                card_id TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                pay_method INTEGER DEFAULT 1,
                cargo_name TEXT,
                sender_info TEXT NOT NULL,
                recipient_info TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )
        .unwrap();
        conn
    }

    /// 计算全部业务表的内容指纹（用于回滚不变量断言）
    fn fingerprint(conn: &Connection) -> String {
        let mut parts = Vec::new();
        for (table, sql) in [
            ("projects", "SELECT group_concat(id||'|'||name||'|'||created_at||'|'||updated_at, ';') FROM (SELECT * FROM projects ORDER BY id)"),
            ("cards", "SELECT group_concat(id||'|'||callsign||'|'||qty||'|'||IFNULL(serial,'')||'|'||status||'|'||IFNULL(metadata,''), ';') FROM (SELECT * FROM cards ORDER BY id)"),
            ("sf_senders", "SELECT group_concat(id||'|'||name||'|'||is_default||'|'||address, ';') FROM (SELECT * FROM sf_senders ORDER BY id)"),
            ("sf_orders", "SELECT group_concat(id||'|'||order_id||'|'||sender_info||'|'||recipient_info, ';') FROM (SELECT * FROM sf_orders ORDER BY id)"),
            ("app_settings", "SELECT group_concat(key||'|'||value, ';') FROM (SELECT * FROM app_settings ORDER BY key)"),
        ] {
            let v: Option<String> = conn.query_row(sql, [], |r| r.get(0)).unwrap();
            parts.push(format!("{}=[{}]", table, v.unwrap_or_default()));
        }
        parts.join("\n")
    }

    fn make_export_data(app_settings: Option<Vec<AppSetting>>) -> ExportData {
        ExportData {
            version: EXPORT_FORMAT_VERSION.to_string(),
            db_version: 0,
            db_version_display: String::new(),
            app_version: "test".to_string(),
            exported_at: "2026-01-01T00:00:00+08:00".to_string(),
            client_id: None,
            tables: ExportTables {
                projects: vec![Project {
                    id: "p1".to_string(),
                    name: "项目1".to_string(),
                    created_at: "2026-01-01T00:00:00+08:00".to_string(),
                    updated_at: "2026-01-01T00:00:00+08:00".to_string(),
                }],
                cards: vec![Card {
                    id: "c1".to_string(),
                    project_id: "p1".to_string(),
                    creator_id: None,
                    callsign: "BH2RO".to_string(),
                    qty: 1,
                    serial: Some(1),
                    status: CardStatus::Pending,
                    metadata: None,
                    created_at: "2026-01-01T00:00:00+08:00".to_string(),
                    updated_at: "2026-01-01T00:00:00+08:00".to_string(),
                }],
                sf_senders: vec![],
                sf_orders: vec![],
                app_settings,
            },
        }
    }

    /// 4.2：恢复路径（Unconditional）即使快照 app_settings 为空也必须清空本地旧配置
    #[test]
    fn test_unconditional_clears_app_settings_even_when_empty() {
        let mut conn = setup_test_db();
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES ('old_key', 'old_value')",
            [],
        )
        .unwrap();

        // 快照不含 app_settings（None） + Unconditional → 本地旧配置必须被清空
        let data = make_export_data(None);
        import_from_export_data(&mut conn, &data, AppSettingsClearMode::Unconditional).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM app_settings", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0, "Unconditional 应清空 app_settings，不留幽灵配置");
    }

    /// 4.2：文件导入路径（Conditional）快照不含 app_settings 时保留本地旧配置
    #[test]
    fn test_conditional_keeps_app_settings_when_absent() {
        let mut conn = setup_test_db();
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES ('old_key', 'old_value')",
            [],
        )
        .unwrap();

        let data = make_export_data(None);
        import_from_export_data(&mut conn, &data, AppSettingsClearMode::Conditional).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM app_settings", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "Conditional 在快照无 app_settings 时应保留现有配置");
    }

    /// 6.4：导入内核事务失败时必须回滚，回滚后本地表内容指纹 == 恢复前
    #[test]
    fn test_kernel_rollback_preserves_fingerprint_on_failure() {
        let mut conn = setup_test_db();
        // 预置原始数据
        conn.execute(
            "INSERT INTO projects (id, name, created_at, updated_at) VALUES ('orig_p', '原始项目', '2026-01-01T00:00:00+08:00', '2026-01-01T00:00:00+08:00')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES ('keep', 'me')",
            [],
        )
        .unwrap();

        let before = fingerprint(&conn);

        // 构造会触发 INSERT 失败的数据：两张卡片同 id（PRIMARY KEY 冲突）→ 第二条 INSERT 报错 → 事务回滚
        let mut data = make_export_data(Some(vec![AppSetting {
            key: "new".to_string(),
            value: "x".to_string(),
        }]));
        let dup = data.tables.cards[0].clone();
        data.tables.cards.push(dup); // 重复主键 c1

        let result = import_from_export_data(&mut conn, &data, AppSettingsClearMode::Unconditional);
        assert!(result.is_err(), "重复主键应导致导入失败");

        let after = fingerprint(&conn);
        assert_eq!(
            before, after,
            "事务失败回滚后本地表内容指纹必须与恢复前一致（不得留下已删未写中间态）"
        );
    }

    /// 多异构行逐字段相等（独立证伪绑定列错序）
    #[test]
    fn test_unconditional_roundtrip_multiple_heterogeneous_rows() {
        let mut conn = setup_test_db();
        let mut data = make_export_data(Some(vec![AppSetting {
            key: "theme".to_string(),
            value: "dark".to_string(),
        }]));
        // 追加第二张异构卡片
        data.tables.cards.push(Card {
            id: "c2".to_string(),
            project_id: "p1".to_string(),
            creator_id: None,
            callsign: "BG2ABC".to_string(),
            qty: 5,
            serial: Some(42),
            status: CardStatus::Distributed,
            metadata: None,
            created_at: "2026-01-02T00:00:00+08:00".to_string(),
            updated_at: "2026-01-02T00:00:00+08:00".to_string(),
        });

        import_from_export_data(&mut conn, &data, AppSettingsClearMode::Unconditional).unwrap();

        let (callsign, qty, serial): (String, i64, i64) = conn
            .query_row(
                "SELECT callsign, qty, serial FROM cards WHERE id = 'c2'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(callsign, "BG2ABC");
        assert_eq!(qty, 5);
        assert_eq!(serial, 42);
    }
}
