// 卡片管理模块
//
// 提供卡片的 CRUD 操作

use crate::db::models::{
    format_datetime, now_china, Card, CardFilter, CardMetadata, CardStatus, CardWithProject,
    DistributionInfo, PagedCards, Pagination, ReturnInfo,
};
use crate::db::sqlite::get_connection;
use crate::error::AppError;
use regex::Regex;

/// 呼号验证正则（3-10 字符，仅字母、数字、斜杠）
fn validate_callsign(callsign: &str) -> Result<(), AppError> {
    let re = Regex::new(r"^[A-Za-z0-9/]{3,10}$").unwrap();
    if !re.is_match(callsign) {
        return Err(AppError::InvalidParameter(
            "呼号格式无效：必须为 3-10 个字符，仅包含字母、数字、斜杠".to_string(),
        ));
    }
    Ok(())
}

/// 数量验证（1-9999）
fn validate_qty(qty: i32) -> Result<(), AppError> {
    if qty < 1 || qty > 9999 {
        return Err(AppError::InvalidParameter(
            "数量无效：必须在 1-9999 之间".to_string(),
        ));
    }
    Ok(())
}

/// 创建卡片
pub fn create_card(project_id: String, callsign: String, qty: i32) -> Result<Card, AppError> {
    // 验证参数
    let callsign = callsign.trim().to_uppercase();
    validate_callsign(&callsign)?;
    validate_qty(qty)?;

    let conn = get_connection()?;

    // 检查项目是否存在
    let project_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1)",
            [&project_id],
            |row| row.get(0),
        )
        .map_err(|e| AppError::Other(format!("查询项目失败: {}", e)))?;

    if !project_exists {
        return Err(AppError::ProfileNotFound(format!(
            "项目不存在: {}",
            project_id
        )));
    }

    // 创建卡片
    let card = Card::new(project_id, callsign, qty);

    conn.execute(
        r#"
        INSERT INTO cards (id, project_id, creator_id, callsign, qty, status, metadata, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
        rusqlite::params![
            &card.id,
            &card.project_id,
            &card.creator_id,
            &card.callsign,
            &card.qty,
            card.status.as_str(),
            Option::<String>::None,
            &card.created_at,
            &card.updated_at,
        ],
    )
    .map_err(|e| AppError::Other(format!("创建卡片失败: {}", e)))?;

    log::info!(
        "✅ 创建卡片成功: {} x {} ({})",
        card.callsign,
        card.qty,
        card.id
    );
    Ok(card)
}

/// 查询卡片列表（分页）
pub fn list_cards(filter: CardFilter, pagination: Pagination) -> Result<PagedCards, AppError> {
    let conn = get_connection()?;

    // 构建 WHERE 子句
    let mut conditions = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref project_id) = filter.project_id {
        conditions.push(format!("c.project_id = ?{}", params.len() + 1));
        params.push(Box::new(project_id.clone()));
    }

    if let Some(ref callsign) = filter.callsign {
        conditions.push(format!("c.callsign LIKE ?{}", params.len() + 1));
        params.push(Box::new(format!("%{}%", callsign.to_uppercase())));
    }

    if let Some(ref status) = filter.status {
        conditions.push(format!("c.status = ?{}", params.len() + 1));
        params.push(Box::new(status.as_str().to_string()));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // 查询总数
    let count_sql = format!(
        "SELECT COUNT(*) FROM cards c {}",
        where_clause
    );

    let total: u64 = {
        let mut stmt = conn
            .prepare(&count_sql)
            .map_err(|e| AppError::Other(format!("准备计数语句失败: {}", e)))?;

        let params_ref: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        stmt.query_row(params_ref.as_slice(), |row| row.get(0))
            .map_err(|e| AppError::Other(format!("查询计数失败: {}", e)))?
    };

    // 计算分页
    let page = pagination.page.max(1);
    let page_size = pagination.page_size.min(100).max(1);
    let offset = (page - 1) * page_size;
    let total_pages = ((total as f64) / (page_size as f64)).ceil() as u32;

    // 查询数据
    let data_sql = format!(
        r#"
        SELECT
            c.id,
            c.project_id,
            p.name as project_name,
            c.callsign,
            c.qty,
            c.status,
            c.metadata,
            c.created_at,
            c.updated_at
        FROM cards c
        LEFT JOIN projects p ON c.project_id = p.id
        {}
        ORDER BY c.created_at DESC
        LIMIT ?{} OFFSET ?{}
        "#,
        where_clause,
        params.len() + 1,
        params.len() + 2
    );

    params.push(Box::new(page_size as i64));
    params.push(Box::new(offset as i64));

    let mut stmt = conn
        .prepare(&data_sql)
        .map_err(|e| AppError::Other(format!("准备查询语句失败: {}", e)))?;

    let params_ref: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let items = stmt
        .query_map(params_ref.as_slice(), |row| {
            let status_str: String = row.get(5)?;
            let metadata_str: Option<String> = row.get(6)?;

            Ok(CardWithProject {
                id: row.get(0)?,
                project_id: row.get(1)?,
                project_name: row.get(2)?,
                callsign: row.get(3)?,
                qty: row.get(4)?,
                status: CardStatus::from_str(&status_str).unwrap_or(CardStatus::Pending),
                metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| AppError::Other(format!("查询卡片列表失败: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Other(format!("读取卡片数据失败: {}", e)))?;

    Ok(PagedCards {
        items,
        total,
        page,
        page_size,
        total_pages,
    })
}

/// 获取单个卡片
pub fn get_card(id: &str) -> Result<Option<Card>, AppError> {
    let conn = get_connection()?;

    let result = conn.query_row(
        r#"
        SELECT id, project_id, creator_id, callsign, qty, status, metadata, created_at, updated_at
        FROM cards WHERE id = ?1
        "#,
        [id],
        |row| {
            let status_str: String = row.get(5)?;
            let metadata_str: Option<String> = row.get(6)?;

            Ok(Card {
                id: row.get(0)?,
                project_id: row.get(1)?,
                creator_id: row.get(2)?,
                callsign: row.get(3)?,
                qty: row.get(4)?,
                status: CardStatus::from_str(&status_str).unwrap_or(CardStatus::Pending),
                metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        },
    );

    match result {
        Ok(card) => Ok(Some(card)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Other(format!("查询卡片失败: {}", e))),
    }
}

/// 分发卡片
///
/// 允许对任意状态的卡片执行分发操作：
/// - 待分发(pending) → 已分发(distributed)：首次分发
/// - 已分发(distributed) → 已分发(distributed)：修改分发信息
/// - 已退回(returned) → 已分发(distributed)：重新分发
///
/// 分发信息和退回信息独立存储在 metadata 中，互不覆盖。
pub fn distribute_card(
    id: &str,
    method: String,
    address: Option<String>,
    remarks: Option<String>,
) -> Result<Card, AppError> {
    let conn = get_connection()?;

    // 获取卡片
    let card = get_card(id)?.ok_or_else(|| AppError::ProfileNotFound(format!("卡片不存在: {}", id)))?;

    // 构建元数据（保留已有的退回信息）
    let distribution = DistributionInfo {
        method,
        address,
        remarks,
        distributed_at: format_datetime(&now_china()),
    };

    let mut metadata = card.metadata.unwrap_or_default();
    metadata.distribution = Some(distribution);

    let metadata_json = serde_json::to_string(&metadata)
        .map_err(|e| AppError::Other(format!("序列化元数据失败: {}", e)))?;

    let updated_at = format_datetime(&now_china());

    // 更新卡片
    conn.execute(
        "UPDATE cards SET status = ?1, metadata = ?2, updated_at = ?3 WHERE id = ?4",
        rusqlite::params![CardStatus::Distributed.as_str(), metadata_json, updated_at, id],
    )
    .map_err(|e| AppError::Other(format!("更新卡片失败: {}", e)))?;

    log::info!("✅ 分发卡片成功: {}", id);
    get_card(id)?.ok_or_else(|| AppError::Other("更新后无法获取卡片".to_string()))
}

/// 退卡
///
/// 允许对任意状态的卡片执行退回操作：
/// - 待分发(pending) → 已退回(returned)：直接退回
/// - 已分发(distributed) → 已退回(returned)：分发后退回
/// - 已退回(returned) → 已退回(returned)：修改退回信息
///
/// 分发信息和退回信息独立存储在 metadata 中，互不覆盖。
pub fn return_card(id: &str, method: String, remarks: Option<String>) -> Result<Card, AppError> {
    let conn = get_connection()?;

    // 获取卡片
    let card = get_card(id)?.ok_or_else(|| AppError::ProfileNotFound(format!("卡片不存在: {}", id)))?;

    // 构建元数据（保留已有的分发信息）
    let return_info = ReturnInfo {
        method,
        remarks,
        returned_at: format_datetime(&now_china()),
    };

    let mut metadata = card.metadata.unwrap_or_default();
    metadata.return_info = Some(return_info);

    let metadata_json = serde_json::to_string(&metadata)
        .map_err(|e| AppError::Other(format!("序列化元数据失败: {}", e)))?;

    let updated_at = format_datetime(&now_china());

    // 更新卡片
    conn.execute(
        "UPDATE cards SET status = ?1, metadata = ?2, updated_at = ?3 WHERE id = ?4",
        rusqlite::params![CardStatus::Returned.as_str(), metadata_json, updated_at, id],
    )
    .map_err(|e| AppError::Other(format!("更新卡片失败: {}", e)))?;

    log::info!("✅ 退卡成功: {}", id);
    get_card(id)?.ok_or_else(|| AppError::Other("更新后无法获取卡片".to_string()))
}

/// 删除卡片
pub fn delete_card(id: &str) -> Result<(), AppError> {
    let conn = get_connection()?;

    // 检查卡片是否存在
    let card = get_card(id)?;
    if card.is_none() {
        return Err(AppError::ProfileNotFound(format!("卡片不存在: {}", id)));
    }

    conn.execute("DELETE FROM cards WHERE id = ?1", [id])
        .map_err(|e| AppError::Other(format!("删除卡片失败: {}", e)))?;

    log::info!("✅ 删除卡片成功: {}", id);
    Ok(())
}

/// 保存地址到卡片
pub fn save_card_address(
    card_id: &str,
    source: String,
    chinese_address: Option<String>,
    english_address: Option<String>,
    name: Option<String>,
    mail_method: Option<String>,
    updated_at: Option<String>,
) -> Result<Card, AppError> {
    let conn = get_connection()?;

    // 获取卡片
    let mut card = get_card(card_id)?
        .ok_or_else(|| AppError::ProfileNotFound(format!("卡片不存在: {}", card_id)))?;

    // 使用 Card 的方法添加或更新地址
    card.add_or_update_address(
        source,
        chinese_address,
        english_address,
        name,
        mail_method,
        updated_at,
    );

    // 序列化 metadata
    let metadata_json = if let Some(ref metadata) = card.metadata {
        Some(serde_json::to_string(metadata)
            .map_err(|e| AppError::Other(format!("序列化元数据失败: {}", e)))?)
    } else {
        None
    };

    let card_updated_at = format_datetime(&now_china());

    // 更新数据库
    conn.execute(
        "UPDATE cards SET metadata = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![metadata_json, card_updated_at, card_id],
    )
    .map_err(|e| AppError::Other(format!("更新卡片失败: {}", e)))?;

    log::info!("✅ 保存地址到卡片成功: {}", card_id);

    // 返回更新后的卡片
    get_card(card_id)?.ok_or_else(|| AppError::Other("更新后无法获取卡片".to_string()))
}
