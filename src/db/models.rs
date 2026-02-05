// 数据库模型定义
//
// 定义项目和卡片的数据结构

use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "ts-rs")]
use ts_rs::TS;

/// 东八区偏移量（秒）
const CHINA_TIMEZONE_OFFSET: i32 = 8 * 3600;

/// 获取东八区时间
pub fn now_china() -> DateTime<FixedOffset> {
    let offset = FixedOffset::east_opt(CHINA_TIMEZONE_OFFSET).unwrap();
    Utc::now().with_timezone(&offset)
}

/// 格式化时间为 ISO 8601 字符串
pub fn format_datetime(dt: &DateTime<FixedOffset>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

/// 解析 ISO 8601 时间字符串
pub fn parse_datetime(s: &str) -> Result<DateTime<FixedOffset>, chrono::ParseError> {
    DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%:z")
}

/// 转卡项目
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub struct Project {
    /// 项目 ID（UUID 格式）
    pub id: String,
    /// 项目名称
    pub name: String,
    /// 创建时间（ISO 8601 格式，东八区）
    pub created_at: String,
    /// 更新时间（ISO 8601 格式，东八区）
    pub updated_at: String,
}

impl Project {
    /// 创建新项目
    pub fn new(name: String) -> Self {
        let now = format_datetime(&now_china());
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

/// 带统计信息的项目
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub struct ProjectWithStats {
    /// 项目 ID
    pub id: String,
    /// 项目名称
    pub name: String,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
    /// 卡片总数
    pub total_cards: i64,
    /// 待分发卡片数
    pub pending_cards: i64,
    /// 已分发卡片数
    pub distributed_cards: i64,
    /// 已退卡卡片数
    pub returned_cards: i64,
}

impl From<Project> for ProjectWithStats {
    fn from(project: Project) -> Self {
        Self {
            id: project.id,
            name: project.name,
            created_at: project.created_at,
            updated_at: project.updated_at,
            total_cards: 0,
            pending_cards: 0,
            distributed_cards: 0,
            returned_cards: 0,
        }
    }
}

/// 卡片状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[serde(rename_all = "lowercase")]
pub enum CardStatus {
    /// 已录入（待分发）
    Pending,
    /// 已分发
    Distributed,
    /// 已退卡
    Returned,
}

impl CardStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CardStatus::Pending => "pending",
            CardStatus::Distributed => "distributed",
            CardStatus::Returned => "returned",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(CardStatus::Pending),
            "distributed" => Some(CardStatus::Distributed),
            "returned" => Some(CardStatus::Returned),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            CardStatus::Pending => "已录入",
            CardStatus::Distributed => "已分发",
            CardStatus::Returned => "已退卡",
        }
    }
}

/// 分发信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub struct DistributionInfo {
    /// 处理方式：直接分发、邮寄、自取
    pub method: String,
    /// 分发地址
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// 备注
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
    /// 代领人呼号（代领方式时使用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_callsign: Option<String>,
    /// 分发时间
    pub distributed_at: String,
}

/// 退卡信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub struct ReturnInfo {
    /// 处理方式：NOT FOUND、CALLSIGN INVALID、REFUSED、OTHER
    pub method: String,
    /// 备注
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
    /// 退卡时间
    pub returned_at: String,
}

/// 地址缓存记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub struct AddressEntry {
    /// 数据来源（如 "qrz.cn", "qrz.com", "QRZ卡片查询"）
    pub source: String,
    /// 中文地址（QRZ.cn 使用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chinese_address: Option<String>,
    /// 英文地址（QRZ.cn 和 QRZ.com 使用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub english_address: Option<String>,
    /// 姓名（QRZ.herbertgao.me 使用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 邮寄方式（QRZ.herbertgao.me 使用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mail_method: Option<String>,
    /// 更新时间（数据最后更新时间）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// 缓存时间（数据获取时间）
    pub cached_at: String,
}

/// 卡片元数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "ts-rs", derive(TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub struct CardMetadata {
    /// 分发信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distribution: Option<DistributionInfo>,
    /// 退卡信息
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "return")]
    pub return_info: Option<ReturnInfo>,
    /// 地址缓存（每个来源只保留1条最新记录）
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "address_history")]
    pub address_cache: Option<Vec<AddressEntry>>,
    /// 待处理运单号（顺丰下单后暂存，确认分发后会移到 distribution.remarks）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_waybill_no: Option<String>,
}

/// 卡片
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub struct Card {
    /// 卡片 ID（UUID 格式）
    pub id: String,
    /// 所属项目 ID
    pub project_id: String,
    /// 创建者 ID（预留字段）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_id: Option<String>,
    /// 呼号
    pub callsign: String,
    /// 数量
    pub qty: i32,
    /// 序列号（数字，前端显示时格式化为三位数如 "001"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<i32>,
    /// 状态
    pub status: CardStatus,
    /// 元数据（分发/退卡信息）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<CardMetadata>,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

impl Card {
    /// 创建新卡片
    pub fn new(project_id: String, callsign: String, qty: i32, serial: Option<i32>) -> Self {
        let now = format_datetime(&now_china());
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            project_id,
            creator_id: None,
            callsign: callsign.to_uppercase(),
            qty,
            serial,
            status: CardStatus::Pending,
            metadata: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// 添加或更新地址缓存（智能去重，单版本）
    ///
    /// 规则：
    /// - 如果数据与现有记录完全一致，只更新时间戳
    /// - 否则直接覆盖原有记录（每个来源只保留1条）
    /// - 缓存有效期365天
    pub fn add_or_update_address(
        &mut self,
        source: String,
        chinese_address: Option<String>,
        english_address: Option<String>,
        name: Option<String>,
        mail_method: Option<String>,
        updated_at: Option<String>,
    ) {
        let now = format_datetime(&now_china());

        // 确保 metadata 存在
        if self.metadata.is_none() {
            self.metadata = Some(CardMetadata::default());
        }

        let metadata = self.metadata.as_mut().unwrap();

        // 确保 address_cache 存在
        if metadata.address_cache.is_none() {
            metadata.address_cache = Some(Vec::new());
        }

        let cache = metadata.address_cache.as_mut().unwrap();

        // 查找同一来源的现有记录
        let existing = cache
            .iter_mut()
            .find(|h| h.source == source);

        if let Some(entry) = existing {
            // 根据数据源比较不同字段
            let is_same = match source.as_str() {
                "qrz.cn" => {
                    entry.chinese_address == chinese_address
                        && entry.english_address == english_address
                }
                "qrz.com" => entry.english_address == english_address,
                "QRZ卡片查询" => {
                    entry.english_address == english_address
                        && entry.name == name
                        && entry.mail_method == mail_method
                }
                _ => {
                    // 未知数据源，比较所有字段
                    entry.chinese_address == chinese_address
                        && entry.english_address == english_address
                        && entry.name == name
                        && entry.mail_method == mail_method
                }
            };

            if is_same {
                // 数据完全一致，只更新时间戳
                entry.updated_at = updated_at.clone();
                entry.cached_at = now;
                return;
            }
        }

        // 创建新记录
        let new_record = AddressEntry {
            source: source.clone(),
            chinese_address,
            english_address,
            name,
            mail_method,
            updated_at,
            cached_at: now,
        };

        // 单版本：删除同一来源的所有旧记录，然后添加新记录
        cache.retain(|h| h.source != source);
        cache.push(new_record);
    }

    /// 获取有效的地址缓存（365天内）
    pub fn get_valid_addresses(&self) -> Vec<AddressEntry> {
        if let Some(metadata) = &self.metadata {
            if let Some(cache) = &metadata.address_cache {
                let now = now_china();
                return cache
                    .iter()
                    .filter(|h| {
                        // 检查缓存是否在365天内
                        if let Ok(cached_time) = parse_datetime(&h.cached_at) {
                            let duration = now.signed_duration_since(cached_time);
                            duration.num_days() <= 365
                        } else {
                            false
                        }
                    })
                    .cloned()
                    .collect();
            }
        }
        Vec::new()
    }
}

/// 带项目名称的卡片（用于列表显示）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub struct CardWithProject {
    /// 卡片 ID
    pub id: String,
    /// 所属项目 ID
    pub project_id: String,
    /// 项目名称
    pub project_name: String,
    /// 呼号
    pub callsign: String,
    /// 数量
    pub qty: i32,
    /// 序列号（数字，前端显示时格式化为三位数如 "001"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<i32>,
    /// 状态
    pub status: CardStatus,
    /// 元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<CardMetadata>,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

/// 卡片查询过滤器
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CardFilter {
    /// 项目 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    /// 呼号关键词（模糊匹配）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callsign: Option<String>,
    /// 状态筛选
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<CardStatus>,
}

/// 分页参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// 当前页码（从 1 开始）
    pub page: u32,
    /// 每页条数
    pub page_size: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}

/// 分页结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub struct PagedCards {
    /// 卡片列表
    pub items: Vec<CardWithProject>,
    /// 总记录数
    #[cfg_attr(feature = "ts-rs", ts(type = "number"))]
    pub total: u64,
    /// 当前页码
    pub page: u32,
    /// 每页条数
    pub page_size: u32,
    /// 总页数
    pub total_pages: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_new() {
        let project = Project::new("测试项目".to_string());
        assert!(!project.id.is_empty());
        assert_eq!(project.name, "测试项目");
        assert!(!project.created_at.is_empty());
        assert!(!project.updated_at.is_empty());
    }

    #[test]
    fn test_datetime_format() {
        let now = now_china();
        let formatted = format_datetime(&now);
        let parsed = parse_datetime(&formatted).unwrap();
        assert_eq!(now.timestamp(), parsed.timestamp());
    }
}
