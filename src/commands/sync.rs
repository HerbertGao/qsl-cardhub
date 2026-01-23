// 云端同步命令
//
// 提供云端同步的 Tauri 命令

use crate::db::export::ExportStats;
use crate::db::models::{format_datetime, now_china};
use crate::security::{delete_credential, get_credential, save_credential};
use crate::sync::client::{sync_data, test_connection, PingResponse, SyncResponse};
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
#[command]
pub async fn execute_sync_cmd() -> Result<SyncResult, String> {
    log::info!("🔄 执行同步");

    // 加载配置
    let mut config = load_sync_config()?.ok_or("未配置同步服务")?;

    // 获取 API Key
    let api_key = get_credential(credential_keys::SYNC_API_KEY)
        .map_err(|e| format!("获取 API Key 失败: {}", e))?
        .ok_or("未配置 API Key")?;

    // 执行同步
    let (response, stats) = sync_data(&config, &api_key).await?;

    // 更新上次同步时间
    let sync_time = format_datetime(&now_china());
    config.last_sync_at = Some(sync_time.clone());
    save_sync_config(&config)?;

    Ok(SyncResult {
        response,
        stats,
        sync_time,
    })
}
