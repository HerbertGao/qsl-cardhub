// QRZ.herbertgao.me JSON API 客户端

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// QRZ.herbertgao.me 地址信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HerbertgaoAddressInfo {
    /// 呼号
    pub callsign: String,
    /// 姓名
    pub name: String,
    /// 邮寄地址
    pub address: String,
    /// 邮寄方式
    pub mail_method: String,
    /// 创建时间（格式：YYYY-MM-DD，从 ISO 8601 提取日期部分）
    pub created_at: String,
    /// 数据来源（固定为 "QRZ卡片查询"）
    pub source: String,
}

/// API 响应结构（单条记录）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiRecord {
    call_sign: Option<String>,
    name: Option<String>,
    mail_address: Option<String>,
    mail_method: Option<String>,
    create_time: Option<String>,
    is_show: bool,
}

impl HerbertgaoAddressInfo {
    /// 创建新的地址信息
    pub fn new(
        callsign: String,
        name: String,
        address: String,
        mail_method: String,
        created_at: String,
    ) -> Self {
        Self {
            callsign,
            name,
            address,
            mail_method,
            created_at,
            source: "QRZ卡片查询".to_string(),
        }
    }
}

/// 查询呼号地址
///
/// # 参数
/// * `callsign` - 呼号
///
/// # 返回
/// 如果找到有效数据返回 Some(HerbertgaoAddressInfo)，否则返回 None
pub async fn query_callsign(callsign: &str) -> Result<Option<HerbertgaoAddressInfo>> {
    log::debug!("开始查询 QRZ.herbertgao.me 呼号: {}", callsign);

    let client = Client::new();
    let url = format!("https://qrz.herbertgao.me/qrz/callsign/{}", callsign);

    // 发送 GET 请求
    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .header("User-Agent", format!("qsl-cardhub/{}", env!("CARGO_PKG_VERSION")))
        .send()
        .await?;

    // 检查响应状态
    if !response.status().is_success() {
        log::warn!(
            "QRZ.herbertgao.me 查询失败，状态码: {}",
            response.status()
        );
        return Ok(None);
    }

    // 解析 JSON 响应
    let records: Vec<ApiRecord> = response.json().await?;

    // 如果响应为空数组，说明呼号不存在
    if records.is_empty() {
        log::info!("呼号 {} 在 QRZ.herbertgao.me 中未找到", callsign);
        return Ok(None);
    }

    // 查找第一个 isShow: true 的记录
    let valid_record = records.iter().find(|r| r.is_show);

    if let Some(record) = valid_record {
        // 检查必需字段是否存在
        if let (Some(call_sign), Some(name), Some(mail_address), Some(mail_method), Some(create_time)) = (
            &record.call_sign,
            &record.name,
            &record.mail_address,
            &record.mail_method,
            &record.create_time,
        ) {
            // 提取日期部分（YYYY-MM-DD）
            let date_part = if let Some(date) = create_time.split('T').next() {
                date.to_string()
            } else {
                // 如果无法解析，使用原始字符串
                create_time.clone()
            };

            log::info!("✓ 成功从 QRZ.herbertgao.me 查询到呼号 {} 的信息", callsign);

            Ok(Some(HerbertgaoAddressInfo::new(
                call_sign.clone(),
                name.clone(),
                mail_address.clone(),
                mail_method.clone(),
                date_part,
            )))
        } else {
            log::warn!("呼号 {} 的记录缺少必需字段", callsign);
            Ok(None)
        }
    } else {
        log::info!(
            "呼号 {} 在 QRZ.herbertgao.me 中所有记录的 isShow 都为 false",
            callsign
        );
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_callsign_bh2ro() {
        // 测试查询 BH2RO
        let result = query_callsign("BH2RO").await;
        assert!(result.is_ok());

        if let Ok(Some(info)) = result {
            assert_eq!(info.callsign, "BH2RO");
            assert_eq!(info.source, "QRZ卡片查询");
            assert!(!info.name.is_empty());
            assert!(!info.address.is_empty());
            assert!(!info.mail_method.is_empty());
        }
    }

    #[tokio::test]
    async fn test_query_nonexistent_callsign() {
        // 测试不存在的呼号
        let result = query_callsign("NONEXISTENT123").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
