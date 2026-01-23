// 云端同步 HTTP 客户端
//
// 与用户自建的云端 API 通信

use crate::db::export::{export_database, ExportStats};
use crate::db::models::{format_datetime, now_china};
use crate::sync::config::SyncConfig;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 同步请求
#[derive(Debug, Clone, Serialize)]
pub struct SyncRequest {
    /// 客户端标识
    pub client_id: String,
    /// 同步时间戳
    pub sync_time: String,
    /// 数据
    pub data: SyncData,
}

/// 同步数据
#[derive(Debug, Clone, Serialize)]
pub struct SyncData {
    /// 项目列表
    pub projects: Vec<crate::db::models::Project>,
    /// 卡片列表
    pub cards: Vec<crate::db::models::Card>,
    /// 寄件人列表
    pub sf_senders: Vec<crate::sf_express::SenderInfo>,
    /// 订单列表
    pub sf_orders: Vec<crate::sf_express::SFOrder>,
}

/// 同步响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    /// 是否成功
    pub success: bool,
    /// 消息
    pub message: String,
    /// 接收时间
    #[serde(default)]
    pub received_at: Option<String>,
    /// 统计信息
    #[serde(default)]
    pub stats: Option<SyncStats>,
}

/// 同步统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    /// 项目数
    pub projects: u32,
    /// 卡片数
    pub cards: u32,
    /// 寄件人数
    pub sf_senders: u32,
    /// 订单数
    pub sf_orders: u32,
}

/// Ping 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResponse {
    /// 是否成功
    pub success: bool,
    /// 消息
    pub message: String,
    /// 服务器时间
    #[serde(default)]
    pub server_time: Option<String>,
}

/// 创建 HTTP 客户端
fn create_client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))
}

/// 测试云端连接
pub async fn test_connection(api_url: &str, api_key: &str) -> Result<PingResponse, String> {
    let client = create_client()?;

    // 确保 URL 以 /ping 结尾
    let ping_url = if api_url.ends_with('/') {
        format!("{}ping", api_url)
    } else {
        format!("{}/ping", api_url)
    };

    log::info!("🔗 测试连接: {}", ping_url);

    let response = client
        .get(&ping_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() {
                "无法连接到服务器，请检查网络".to_string()
            } else if e.is_timeout() {
                "连接超时，请稍后重试".to_string()
            } else {
                format!("网络请求失败: {}", e)
            }
        })?;

    let status = response.status();

    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err("API Key 无效，请检查配置".to_string());
    }

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("服务器返回错误 ({}): {}", status, error_text));
    }

    let ping_response: PingResponse = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if !ping_response.success {
        return Err(format!("连接测试失败: {}", ping_response.message));
    }

    log::info!("✅ 连接测试成功");
    Ok(ping_response)
}

/// 执行数据同步
pub async fn sync_data(config: &SyncConfig, api_key: &str) -> Result<(SyncResponse, ExportStats), String> {
    let client = create_client()?;

    // 确保 URL 以 /sync 结尾
    let sync_url = if config.api_url.ends_with('/') {
        format!("{}sync", config.api_url)
    } else {
        format!("{}/sync", config.api_url)
    };

    log::info!("🔄 开始同步数据到: {}", sync_url);

    // 导出数据
    let export_data = export_database()
        .map_err(|e| format!("导出数据失败: {}", e))?;

    let stats = ExportStats {
        projects: export_data.tables.projects.len() as u32,
        cards: export_data.tables.cards.len() as u32,
        sf_senders: export_data.tables.sf_senders.len() as u32,
        sf_orders: export_data.tables.sf_orders.len() as u32,
    };

    // 构建同步请求
    let sync_request = SyncRequest {
        client_id: config.client_id.clone(),
        sync_time: format_datetime(&now_china()),
        data: SyncData {
            projects: export_data.tables.projects,
            cards: export_data.tables.cards,
            sf_senders: export_data.tables.sf_senders,
            sf_orders: export_data.tables.sf_orders,
        },
    };

    log::info!(
        "📦 同步数据: {} 个项目, {} 张卡片, {} 个寄件人, {} 个订单",
        stats.projects,
        stats.cards,
        stats.sf_senders,
        stats.sf_orders
    );

    // 发送同步请求
    let response = client
        .post(&sync_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&sync_request)
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() {
                "无法连接到服务器，请检查网络".to_string()
            } else if e.is_timeout() {
                "连接超时，请稍后重试".to_string()
            } else {
                format!("网络请求失败: {}", e)
            }
        })?;

    let status = response.status();

    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err("认证失败，请检查 API Key".to_string());
    }

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("同步失败 ({}): {}", status, error_text));
    }

    let sync_response: SyncResponse = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if !sync_response.success {
        return Err(format!("同步失败: {}", sync_response.message));
    }

    log::info!("✅ 数据同步成功");
    Ok((sync_response, stats))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_request_serialization() {
        let request = SyncRequest {
            client_id: "test-id".to_string(),
            sync_time: "2026-01-23T14:30:00+08:00".to_string(),
            data: SyncData {
                projects: vec![],
                cards: vec![],
                sf_senders: vec![],
                sf_orders: vec![],
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("client_id"));
        assert!(json.contains("sync_time"));
    }
}
