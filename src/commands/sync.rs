// 云端同步命令
//
// 提供云端同步的 Tauri 命令

use crate::db::export::ExportStats;
use crate::db::import::{import_from_export_data, AppSettingsClearMode};
use crate::db::models::{format_datetime, now_china};
use crate::security::{delete_credential, get_credential, save_credential};
use crate::sync::client::{pull_data, sync_data, test_connection, PingResponse, SyncOutcome, SyncResponse};
use crate::sync::config::{
    clear_sync_config, credential_keys, load_sync_config, save_sync_config, SyncConfig,
};
use serde::{Deserialize, Serialize};
use tauri::command;

/// 同步配置响应（不含敏感信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfigResponse {
    /// API 地址
    pub api_url: String,
    /// 客户端标识
    pub client_id: String,
    /// 上次同步时间
    pub last_sync_at: Option<String>,
    /// 是否已配置 API Key
    pub has_api_key: bool,
    /// 本地持久化的云端基线版本（只读展示）
    pub base_version: Option<i64>,
}

/// 同步结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// 服务器响应
    pub response: SyncResponse,
    /// 本地统计
    pub stats: ExportStats,
    /// 同步时间
    pub sync_time: String,
    /// 写入后的新云端版本
    pub server_version: Option<i64>,
}

/// 同步命令三态结果（供前端分流）
///
/// 序列化为带 `status` 标签的对象：
/// - `{"status":"success", ...}`
/// - `{"status":"auth_failed"}`
/// - `{"status":"conflict","server_version":N}`（云端当前版本，解析失败时为 null）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SyncCmdResult {
    /// 同步成功
    Success {
        /// 服务器响应
        response: SyncResponse,
        /// 本地统计
        stats: ExportStats,
        /// 同步时间
        sync_time: String,
        /// 写入后的新云端版本
        server_version: Option<i64>,
    },
    /// 认证失败（401）
    AuthFailed,
    /// 版本冲突（409）
    Conflict {
        /// 云端当前版本（解析失败/行缺失时为 null）
        server_version: Option<i64>,
    },
}

/// 从云端恢复结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    /// 恢复后对齐的云端版本
    pub server_version: i64,
    /// 恢复的数据统计
    pub stats: ExportStats,
}

/// 保存同步配置
#[command]
pub async fn save_sync_config_cmd(api_url: String, api_key: Option<String>) -> Result<SyncConfigResponse, String> {
    log::info!("💾 保存同步配置");

    // 加载现有配置或创建新配置
    let mut config = load_sync_config()?.unwrap_or_default();

    // 更新 API URL
    config.api_url = api_url;

    // 保存配置
    save_sync_config(&config)?;

    // 如果提供了 API Key，保存到凭据存储
    let has_api_key = if let Some(key) = api_key {
        if !key.is_empty() {
            save_credential(credential_keys::SYNC_API_KEY, &key)
                .map_err(|e| format!("保存 API Key 失败: {}", e))?;
            log::info!("✅ API Key 已保存");
            true
        } else {
            // 检查是否已有 API Key
            get_credential(credential_keys::SYNC_API_KEY)
                .ok()
                .flatten()
                .is_some()
        }
    } else {
        // 检查是否已有 API Key
        get_credential(credential_keys::SYNC_API_KEY)
            .ok()
            .flatten()
            .is_some()
    };

    Ok(SyncConfigResponse {
        api_url: config.api_url,
        client_id: config.client_id,
        last_sync_at: config.last_sync_at,
        has_api_key,
        base_version: config.base_version,
    })
}

/// 加载同步配置
#[command]
pub async fn load_sync_config_cmd() -> Result<Option<SyncConfigResponse>, String> {
    log::info!("📂 加载同步配置");

    match load_sync_config()? {
        Some(config) => {
            // 检查是否有 API Key
            let has_api_key = get_credential(credential_keys::SYNC_API_KEY)
                .ok()
                .flatten()
                .is_some();

            Ok(Some(SyncConfigResponse {
                api_url: config.api_url,
                client_id: config.client_id,
                last_sync_at: config.last_sync_at,
                has_api_key,
                base_version: config.base_version,
            }))
        }
        None => Ok(None),
    }
}

/// 清除同步配置
#[command]
pub async fn clear_sync_config_cmd() -> Result<(), String> {
    log::info!("🗑️  清除同步配置");

    // 清除配置文件
    clear_sync_config()?;

    // 清除 API Key
    let _ = delete_credential(credential_keys::SYNC_API_KEY);

    log::info!("✅ 同步配置已清除");
    Ok(())
}

/// 测试同步连接
#[command]
pub async fn test_sync_connection_cmd() -> Result<PingResponse, String> {
    log::info!("🔗 测试同步连接");

    // 加载配置
    let config = load_sync_config()?.ok_or("未配置同步服务")?;

    // 获取 API Key
    let api_key = get_credential(credential_keys::SYNC_API_KEY)
        .map_err(|e| format!("获取 API Key 失败: {}", e))?
        .ok_or("未配置 API Key")?;

    // 测试连接
    test_connection(&config.api_url, &api_key).await
}

/// 执行同步
///
/// 返回三态结果（成功 / 认证失败 / 版本冲突），供前端分流：
/// - 成功（200）：刷新本地 `base_version` 为响应回传的 `server_version` 并落盘（load-bearing，否则下次必 409）
/// - 认证失败（401）：本地数据与基线均不动
/// - 版本冲突（409）：本地数据与基线均不动，携带云端当前版本供前端引导
///
/// `force=Some(true)` 走强制覆盖逃生门（无条件覆盖云端）。
#[command]
pub async fn execute_sync_cmd(force: Option<bool>) -> Result<SyncCmdResult, String> {
    let force = force.unwrap_or(false);
    log::info!("🔄 执行同步 (force={})", force);

    // 加载配置
    let mut config = load_sync_config()?.ok_or("未配置同步服务")?;

    // 获取 API Key
    let api_key = get_credential(credential_keys::SYNC_API_KEY)
        .map_err(|e| format!("获取 API Key 失败: {}", e))?
        .ok_or("未配置 API Key")?;

    // 执行同步
    match sync_data(&config, &api_key, force).await? {
        SyncOutcome::Success {
            response,
            stats,
            server_version,
        } => {
            // 更新上次同步时间
            let sync_time = format_datetime(&now_china());
            config.last_sync_at = Some(sync_time.clone());
            // load-bearing：把本地基线刷新为云端回传的新版本，否则下次上传必 409
            config.base_version = server_version;
            save_sync_config(&config)?;

            Ok(SyncCmdResult::Success {
                response,
                stats,
                sync_time,
                server_version,
            })
        }
        SyncOutcome::AuthFailed => Ok(SyncCmdResult::AuthFailed),
        SyncOutcome::Conflict { server_version } => {
            Ok(SyncCmdResult::Conflict { server_version })
        }
    }
}

/// 从云端恢复
///
/// 调用 `GET /pull` 拉回全量快照 → 在单个本地事务内无条件重建 5 张业务表（含 app_settings）
/// → 成功后把本地 `base_version` 对齐为快照的 `server_version` 并落盘。
///
/// **警告**：本操作会**销毁本地未上传的改动**，前端必须前置二次确认。
/// `pull_data` 失败（401/网络/坏 body）时短路返回 `Err`，**不进入导入**，保证本地库零改动。
#[command]
pub async fn restore_from_cloud() -> Result<RestoreResult, String> {
    log::info!("⬇️ 从云端恢复（将销毁本地未上传改动）");

    // 加载配置
    let mut config = load_sync_config()?.ok_or("未配置同步服务")?;

    // 获取 API Key
    let api_key = get_credential(credential_keys::SYNC_API_KEY)
        .map_err(|e| format!("获取 API Key 失败: {}", e))?
        .ok_or("未配置 API Key")?;

    // 拉取云端快照（失败必须短路，禁进入导入）
    let pulled = pull_data(&config.api_url, &api_key).await?;

    let server_version = pulled.server_version;
    let stats = ExportStats {
        projects: pulled.data.projects.len() as u32,
        cards: pulled.data.cards.len() as u32,
        sf_senders: pulled.data.sf_senders.len() as u32,
        sf_orders: pulled.data.sf_orders.len() as u32,
    };

    // 用快照构造等价 ExportData 调共用导入内核（无条件清空全部 5 张业务表，含 app_settings）
    let export_data = sync_data_to_export_data(pulled);

    let mut conn = crate::db::sqlite::get_connection().map_err(|e| e.to_string())?;
    import_from_export_data(&mut conn, &export_data, AppSettingsClearMode::Unconditional)
        .map_err(|e| format!("从云端恢复失败: {}", e))?;

    // 对齐本地基线为快照版本并落盘
    config.base_version = Some(server_version);
    save_sync_config(&config)?;

    log::info!("✅ 从云端恢复完成，base_version 对齐至 {}", server_version);

    Ok(RestoreResult {
        server_version,
        stats,
    })
}

/// 把 `/pull` 返回的快照转为共用导入内核所需的 `ExportData`
fn sync_data_to_export_data(pulled: crate::sync::client::PullResponse) -> crate::db::export::ExportData {
    use crate::db::export::{ExportData, ExportTables, EXPORT_FORMAT_VERSION};

    ExportData {
        version: EXPORT_FORMAT_VERSION.to_string(),
        // 恢复路径不参与本地数据库版本校验（execute_import 才校验文件版本），
        // 这里填本地版本占位即可：内核只用 tables，不读 db_version。
        db_version: 0,
        db_version_display: String::new(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        exported_at: format_datetime(&now_china()),
        client_id: None,
        tables: ExportTables {
            projects: pulled.data.projects,
            cards: pulled.data.cards,
            sf_senders: pulled.data.sf_senders,
            sf_orders: pulled.data.sf_orders,
            app_settings: Some(pulled.data.app_settings),
        },
    }
}
