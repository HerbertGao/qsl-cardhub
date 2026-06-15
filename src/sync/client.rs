// 云端同步 HTTP 客户端
//
// 与用户自建的云端 API 通信

use crate::db::export::{export_database, ExportStats};
use crate::db::models::{format_datetime, now_china};
use crate::sync::config::SyncConfig;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// `bool` 缺省（false）时跳过序列化的辅助函数
fn is_false(b: &bool) -> bool {
    !*b
}

/// 同步请求
#[derive(Debug, Clone, Serialize)]
pub struct SyncRequest {
    /// 客户端标识
    pub client_id: String,
    /// 同步时间戳
    pub sync_time: String,
    /// 本地持久化的云端基线版本
    ///
    /// `None` 时不序列化，保证旧服务端/首次同步的请求形态与旧版一致，
    /// 服务端据此走无条件覆盖路径。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_version: Option<i64>,
    /// 强制覆盖（逃生门）
    ///
    /// `false` 时不序列化，保证旧服务端兼容；`true` 时服务端跳过版本比较、
    /// 无条件覆盖。
    #[serde(skip_serializing_if = "is_false")]
    pub force: bool,
    /// 数据
    pub data: SyncData,
}

/// 同步数据
///
/// 同时用于 `/sync` 请求体（序列化）与 `/pull` 响应体（反序列化），
/// 故同时派生 `Serialize` + `Deserialize`。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncData {
    /// 项目列表
    pub projects: Vec<crate::db::models::Project>,
    /// 卡片列表
    pub cards: Vec<crate::db::models::Card>,
    /// 寄件人列表
    pub sf_senders: Vec<crate::sf_express::SenderInfo>,
    /// 订单列表
    pub sf_orders: Vec<crate::sf_express::SFOrder>,
    /// 全局配置项列表
    pub app_settings: Vec<crate::db::models::AppSetting>,
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
    /// 写入后的新云端版本（200 时回传，旧服务端缺省为 None）
    #[serde(default)]
    pub server_version: Option<i64>,
}

/// 409 版本冲突响应体（仅用于解析云端当前版本）
#[derive(Debug, Clone, Deserialize)]
struct ConflictBody {
    /// 云端当前版本（行缺失时为 null）
    #[serde(default)]
    server_version: Option<i64>,
}

/// 同步结果三态
///
/// 区分「成功 / 认证失败 / 版本冲突(409)」，供命令层转成前端可分辨结果。
#[derive(Debug, Clone)]
pub enum SyncOutcome {
    /// 同步成功（200），携带服务端响应、统计与新版本
    Success {
        response: SyncResponse,
        stats: ExportStats,
        server_version: Option<i64>,
    },
    /// 认证失败（401）
    AuthFailed,
    /// 版本冲突（409），携带云端当前版本（解析失败/行缺失时为 None）
    Conflict { server_version: Option<i64> },
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

/// 从云端拉取的全量快照响应（GET /pull）
///
/// `data` 各表字段形态与桌面端 `export_database()` 产出一致（对象/布尔形态），
/// 可直接复用 `SyncData` 的反序列化结构。
#[derive(Debug, Clone, Deserialize)]
pub struct PullResponse {
    /// 当前云端版本
    pub server_version: i64,
    /// 全量业务数据快照
    pub data: SyncData,
    /// 最近写入的客户端标识（可选）
    #[serde(default)]
    pub last_client_id: Option<String>,
    /// 最近同步时间（可选）
    #[serde(default)]
    pub sync_time: Option<String>,
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
///
/// 上传携带 `config.base_version` 与 `force`，返回类型化的三态结果：
/// - 200 → `SyncOutcome::Success`（携带新 `server_version`）
/// - 401 → `SyncOutcome::AuthFailed`
/// - 409 → `SyncOutcome::Conflict`（携带云端当前 `server_version`，解析失败/行缺失时为 None）
///
/// 其它非 2xx 与网络/解析错误仍返回 `Err(String)`。
pub async fn sync_data(
    config: &SyncConfig,
    api_key: &str,
    force: bool,
) -> Result<SyncOutcome, String> {
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
        base_version: config.base_version,
        force,
        data: SyncData {
            projects: export_data.tables.projects,
            cards: export_data.tables.cards,
            sf_senders: export_data.tables.sf_senders,
            sf_orders: export_data.tables.sf_orders,
            app_settings: export_data.tables.app_settings.unwrap_or_default(),
        },
    };

    log::info!(
        "📦 同步数据: {} 个项目, {} 张卡片, {} 个寄件人, {} 个订单 (base_version={:?}, force={})",
        stats.projects,
        stats.cards,
        stats.sf_senders,
        stats.sf_orders,
        config.base_version,
        force
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
        return Ok(SyncOutcome::AuthFailed);
    }

    // 409 版本冲突：解析为类型化结果，携带云端当前版本。
    // body 解析失败（非 JSON/缺字段）兜底为 server_version=None，禁 panic、禁回笼统 Err。
    if status == reqwest::StatusCode::CONFLICT {
        let body_text = response.text().await.unwrap_or_default();
        let server_version = serde_json::from_str::<ConflictBody>(&body_text)
            .ok()
            .and_then(|b| b.server_version);
        log::warn!("⚠️ 版本冲突 (409)，云端当前版本: {:?}", server_version);
        return Ok(SyncOutcome::Conflict { server_version });
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

    let server_version = sync_response.server_version;
    log::info!("✅ 数据同步成功 (server_version={:?})", server_version);
    Ok(SyncOutcome::Success {
        response: sync_response,
        stats,
        server_version,
    })
}

/// 从云端拉取全量快照（GET /pull）
///
/// 401 映射为「认证失败，请检查 API Key」；网络/解析错误返回对应 `Err`。
/// 调用方在收到 `Err` 时**必须**短路、禁进入本地导入，以保证本地库零改动。
pub async fn pull_data(api_url: &str, api_key: &str) -> Result<PullResponse, String> {
    let client = create_client()?;

    // 确保 URL 以 /pull 结尾
    let pull_url = if api_url.ends_with('/') {
        format!("{}pull", api_url)
    } else {
        format!("{}/pull", api_url)
    };

    log::info!("⬇️ 从云端拉取快照: {}", pull_url);

    let response = client
        .get(&pull_url)
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
        return Err("认证失败，请检查 API Key".to_string());
    }

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("拉取失败 ({}): {}", status, error_text));
    }

    let pull_response: PullResponse = response
        .json()
        .await
        .map_err(|e| format!("解析快照失败: {}", e))?;

    log::info!(
        "✅ 拉取成功 (server_version={}): {} 个项目, {} 张卡片, {} 个寄件人, {} 个订单",
        pull_response.server_version,
        pull_response.data.projects.len(),
        pull_response.data.cards.len(),
        pull_response.data.sf_senders.len(),
        pull_response.data.sf_orders.len()
    );

    Ok(pull_response)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_sync_data() -> SyncData {
        SyncData {
            projects: vec![],
            cards: vec![],
            sf_senders: vec![],
            sf_orders: vec![],
            app_settings: vec![],
        }
    }

    #[test]
    fn test_sync_request_serialization() {
        let request = SyncRequest {
            client_id: "test-id".to_string(),
            sync_time: "2026-01-23T14:30:00+08:00".to_string(),
            base_version: None,
            force: false,
            data: empty_sync_data(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("client_id"));
        assert!(json.contains("sync_time"));
    }

    #[test]
    fn test_sync_request_omits_base_version_and_force_when_default() {
        // base_version=None / force=false 时不序列化这两字段，保证旧服务端兼容、请求形态不变
        let request = SyncRequest {
            client_id: "test-id".to_string(),
            sync_time: "2026-01-23T14:30:00+08:00".to_string(),
            base_version: None,
            force: false,
            data: empty_sync_data(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(!json.contains("base_version"), "base_version 不应出现: {}", json);
        assert!(!json.contains("force"), "force 不应出现: {}", json);
    }

    #[test]
    fn test_sync_request_includes_base_version_and_force_when_set() {
        let request = SyncRequest {
            client_id: "test-id".to_string(),
            sync_time: "2026-01-23T14:30:00+08:00".to_string(),
            base_version: Some(7),
            force: true,
            data: empty_sync_data(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"base_version\":7"), "应含 base_version: {}", json);
        assert!(json.contains("\"force\":true"), "应含 force: {}", json);
    }

    #[test]
    fn test_sync_response_tolerates_missing_optional_fields() {
        // 旧服务端缺 server_version/stats/received_at 时仍能反序列化
        let json = r#"{"success":true,"message":"ok"}"#;
        let resp: SyncResponse = serde_json::from_str(json).unwrap();
        assert!(resp.success);
        assert!(resp.server_version.is_none());
        assert!(resp.stats.is_none());
    }

    #[test]
    fn test_sync_response_parses_server_version() {
        let json = r#"{"success":true,"message":"ok","server_version":12}"#;
        let resp: SyncResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.server_version, Some(12));
    }

    #[test]
    fn test_conflict_body_parses_and_tolerates_null() {
        let with_version: ConflictBody = serde_json::from_str(r#"{"server_version":9}"#).unwrap();
        assert_eq!(with_version.server_version, Some(9));

        let null_version: ConflictBody = serde_json::from_str(r#"{"server_version":null}"#).unwrap();
        assert!(null_version.server_version.is_none());

        let missing: ConflictBody = serde_json::from_str(r#"{}"#).unwrap();
        assert!(missing.server_version.is_none());
    }

    #[test]
    fn test_pull_response_deserializes_object_and_bool_form() {
        // 锚定 2.3 形态契约：metadata/sender_info 为对象、is_default 为布尔
        let json = r#"{
            "success": true,
            "server_version": 5,
            "last_client_id": "client-abc",
            "sync_time": "2026-01-23T14:30:00+08:00",
            "data": {
                "projects": [
                    {"id":"p1","name":"项目1","created_at":"2026-01-01T00:00:00+08:00","updated_at":"2026-01-01T00:00:00+08:00"}
                ],
                "cards": [
                    {"id":"c1","project_id":"p1","callsign":"BH2RO","qty":1,"status":"pending",
                     "metadata":{"note":"x"},"created_at":"2026-01-01T00:00:00+08:00","updated_at":"2026-01-01T00:00:00+08:00"}
                ],
                "sf_senders": [
                    {"id":"s1","name":"张三","phone":"010-1234","mobile":null,"province":"北京","city":"北京","district":"海淀","address":"中关村","is_default":true,"created_at":"2026-01-01T00:00:00+08:00","updated_at":"2026-01-01T00:00:00+08:00"}
                ],
                "sf_orders": [
                    {"id":"o1","order_id":"ORD1","waybill_no":null,"card_id":"c1","status":"pending","pay_method":1,"cargo_name":"卡片",
                     "sender_info":{"id":"s1","name":"张三","phone":"010-1234","mobile":null,"province":"北京","city":"北京","district":"海淀","address":"中关村","is_default":true,"created_at":"2026-01-01T00:00:00+08:00","updated_at":"2026-01-01T00:00:00+08:00"},
                     "recipient_info":{"name":"李四","phone":"020-5678","mobile":null,"province":"广东","city":"广州","district":"天河","address":"天河路"},
                     "created_at":"2026-01-01T00:00:00+08:00","updated_at":"2026-01-01T00:00:00+08:00"}
                ],
                "app_settings": [
                    {"key":"theme","value":"dark"}
                ]
            }
        }"#;

        let resp: PullResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.server_version, 5);
        assert_eq!(resp.last_client_id.as_deref(), Some("client-abc"));
        assert_eq!(resp.data.cards.len(), 1);
        assert!(resp.data.cards[0].metadata.is_some());
        assert_eq!(resp.data.sf_senders.len(), 1);
        assert!(resp.data.sf_senders[0].is_default);
        assert_eq!(resp.data.sf_orders.len(), 1);
        assert_eq!(resp.data.sf_orders[0].sender_info.name, "张三");
        assert_eq!(resp.data.sf_orders[0].recipient_info.name, "李四");
        assert_eq!(resp.data.app_settings.len(), 1);
    }

    #[test]
    fn test_pull_response_tolerates_empty_pii() {
        // RC-B：worker 兜底把缺/空 sender_info/recipient_info 存为 '{}'，/pull 读回 {}；
        // 恢复需容忍（缺字段→默认空串，不 abort）。
        let json = r#"{
            "success": true,
            "server_version": 5,
            "last_client_id": "client-abc",
            "sync_time": "2026-01-23T14:30:00+08:00",
            "data": {
                "projects": [],
                "cards": [],
                "sf_senders": [],
                "sf_orders": [
                    {"id":"o1","order_id":"ORD1","waybill_no":null,"card_id":"c1","status":"pending","pay_method":1,"cargo_name":"卡片",
                     "sender_info":{},
                     "recipient_info":{},
                     "created_at":"2026-01-01T00:00:00+08:00","updated_at":"2026-01-01T00:00:00+08:00"}
                ],
                "app_settings": []
            }
        }"#;

        let resp = serde_json::from_str::<PullResponse>(json);
        assert!(resp.is_ok(), "空 PII 应容忍反序列化、不 abort");
        let resp = resp.unwrap();
        assert_eq!(resp.data.sf_orders.len(), 1);
        assert_eq!(resp.data.sf_orders[0].sender_info.name, "");
        assert_eq!(resp.data.sf_orders[0].recipient_info.name, "");
    }
}
