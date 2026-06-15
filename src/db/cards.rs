// 卡片管理模块
//
// 提供卡片的 CRUD 操作

use crate::db::models::{
    format_datetime, now_china, Card, CardFilter, CardStatus, CardWithProject, DistributionInfo,
    PagedCards, Pagination, ReturnInfo,
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
pub fn create_card(project_id: String, callsign: String, qty: i32, serial: Option<i32>) -> Result<Card, AppError> {
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

    // 检查同项目下呼号是否已存在（大小写不敏感）
    let callsign_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM cards WHERE project_id = ?1 AND callsign = ?2 COLLATE NOCASE)",
            [&project_id, &callsign],
            |row| row.get(0),
        )
        .map_err(|e| AppError::Other(format!("查询呼号失败: {}", e)))?;

    if callsign_exists {
        return Err(AppError::InvalidParameter(
            "该呼号已在此项目中录入".to_string(),
        ));
    }

    // 创建卡片
    let card = Card::new(project_id, callsign, qty, serial);

    conn.execute(
        r#"
        INSERT INTO cards (id, project_id, creator_id, callsign, qty, serial, status, metadata, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#,
        rusqlite::params![
            &card.id,
            &card.project_id,
            &card.creator_id,
            &card.callsign,
            &card.qty,
            &card.serial,
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

/// 共享的卡片查询 SELECT 主体（SELECT 列 + FROM cards LEFT JOIN projects）。
///
/// 列序必须与 `map_card_row` 的读取顺序（0..=9）严格一致。
/// 结尾保留换行，保证与后续拼接的 `WHERE …`/`ORDER BY …`/`LIMIT/OFFSET` 之间留有空白。
const CARD_SELECT_BODY: &str = r#"
        SELECT
            c.id,
            c.project_id,
            p.name as project_name,
            c.callsign,
            c.qty,
            c.serial,
            c.status,
            c.metadata,
            c.created_at,
            c.updated_at
        FROM cards c
        LEFT JOIN projects p ON c.project_id = p.id
"#;

/// 构建 WHERE 子句与对应参数（crate-private）。
///
/// 返回的 WHERE 串不含前导/尾随空白：无条件时为空字符串，否则为 `WHERE …`。
/// 片段间空白由调用方共享主体（`CARD_SELECT_BODY`）的尾部换行保证。
fn build_card_where(filter: &CardFilter) -> (String, Vec<Box<dyn rusqlite::ToSql>>) {
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

    (where_clause, params)
}

/// 将查询行映射为 `CardWithProject`（crate-private）。
///
/// 列序须与 `CARD_SELECT_BODY` 的 SELECT 列顺序（0..=9）一致，零行为变更。
/// 第 2 列 `project_name` 读为 `String`（孤儿卡片 NULL 的既有行为不在本次改动范围）。
fn map_card_row(row: &rusqlite::Row) -> rusqlite::Result<CardWithProject> {
    let status_str: String = row.get(6)?;
    let metadata_str: Option<String> = row.get(7)?;

    Ok(CardWithProject {
        id: row.get(0)?,
        project_id: row.get(1)?,
        project_name: row.get(2)?,
        callsign: row.get(3)?,
        qty: row.get(4)?,
        serial: row.get(5)?,
        status: CardStatus::from_str(&status_str).unwrap_or(CardStatus::Pending),
        metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

/// 查询卡片列表（分页）
pub fn list_cards(filter: CardFilter, pagination: Pagination) -> Result<PagedCards, AppError> {
    let conn = get_connection()?;
    list_cards_conn(&conn, filter, pagination)
}

/// 分页查询主体（crate-private，接收连接以便测试）。
fn list_cards_conn(
    conn: &rusqlite::Connection,
    filter: CardFilter,
    pagination: Pagination,
) -> Result<PagedCards, AppError> {
    // 构建 WHERE 子句
    let (where_clause, mut params) = build_card_where(&filter);

    // 查询总数（不消费 LIMIT/OFFSET 参数）
    let count_sql = format!("SELECT COUNT(*) FROM cards c {}", where_clause);

    let total: u64 = {
        let mut stmt = conn
            .prepare(&count_sql)
            .map_err(|e| AppError::Other(format!("准备计数语句失败: {}", e)))?;

        let params_ref: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let count: i64 = stmt
            .query_row(params_ref.as_slice(), |row| row.get(0))
            .map_err(|e| AppError::Other(format!("查询计数失败: {}", e)))?;
        count as u64
    };

    // 计算分页
    let page = pagination.page.max(1);
    let page_size = pagination.page_size.min(100).max(1);
    let offset = (page - 1) * page_size;
    let total_pages = ((total as f64) / (page_size as f64)).ceil() as u32;

    // 查询数据：占位符编号随条件数（params.len()）动态计算
    let data_sql = format!(
        "{body}{where_clause}\n        ORDER BY c.created_at DESC\n        LIMIT ?{limit} OFFSET ?{offset}\n",
        body = CARD_SELECT_BODY,
        where_clause = where_clause,
        limit = params.len() + 1,
        offset = params.len() + 2,
    );

    params.push(Box::new(page_size as i64));
    params.push(Box::new(offset as i64));

    let mut stmt = conn
        .prepare(&data_sql)
        .map_err(|e| AppError::Other(format!("准备查询语句失败: {}", e)))?;

    // params_ref 在 push 完 LIMIT/OFFSET 之后、params 不再修改时构造
    let params_ref: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let items = stmt
        .query_map(params_ref.as_slice(), map_card_row)
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

/// 查询符合筛选条件的全部卡片（不分页）。
///
/// 与分页的 `list_cards` 解耦，用于导出等需要全量数据的场景，不施加 `page_size` 上限。
pub fn list_all_cards(filter: CardFilter) -> Result<Vec<CardWithProject>, AppError> {
    let conn = get_connection()?;
    list_all_cards_conn(&conn, filter)
}

/// 全量查询主体（crate-private，接收连接以便测试），无 LIMIT/OFFSET。
fn list_all_cards_conn(
    conn: &rusqlite::Connection,
    filter: CardFilter,
) -> Result<Vec<CardWithProject>, AppError> {
    let (where_clause, params) = build_card_where(&filter);

    let data_sql = format!(
        "{body}{where_clause}\n        ORDER BY c.created_at DESC\n",
        body = CARD_SELECT_BODY,
        where_clause = where_clause,
    );

    let mut stmt = conn
        .prepare(&data_sql)
        .map_err(|e| AppError::Other(format!("准备查询语句失败: {}", e)))?;

    let params_ref: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let cards = stmt
        .query_map(params_ref.as_slice(), map_card_row)
        .map_err(|e| AppError::Other(format!("查询卡片列表失败: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Other(format!("读取卡片数据失败: {}", e)))?;

    Ok(cards)
}

/// 获取单个卡片
pub fn get_card(id: &str) -> Result<Option<Card>, AppError> {
    let conn = get_connection()?;

    let result = conn.query_row(
        r#"
        SELECT id, project_id, creator_id, callsign, qty, serial, status, metadata, created_at, updated_at
        FROM cards WHERE id = ?1
        "#,
        [id],
        |row| {
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
    proxy_callsign: Option<String>,
) -> Result<Card, AppError> {
    let conn = get_connection()?;

    // 获取卡片
    let card = get_card(id)?.ok_or_else(|| AppError::ProfileNotFound(format!("卡片不存在: {}", id)))?;

    // 构建元数据（保留已有的退回信息）
    let distribution = DistributionInfo {
        method,
        address,
        remarks,
        proxy_callsign,
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

/// 获取项目的最大序列号
/// 返回该项目中最大的数字序列号，如果没有记录则返回 None
pub fn get_max_serial_by_project(project_id: &str) -> Result<Option<u32>, AppError> {
    let conn = get_connection()?;

    // 查询该项目下所有卡片的最大序列号
    let result: Result<Option<i32>, _> = conn.query_row(
        r#"
        SELECT MAX(serial) FROM cards
        WHERE project_id = ?1
          AND serial IS NOT NULL
        "#,
        [project_id],
        |row| row.get(0),
    );

    match result {
        Ok(Some(serial)) => Ok(Some(serial as u32)),
        Ok(None) => Ok(None),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Other(format!("查询最大序列号失败: {}", e))),
    }
}

/// 保存待处理运单号（不改变卡片状态）
///
/// 用于顺丰下单后暂存运单号，等待用户点击"确认分发"后再正式分发
pub fn save_pending_waybill(card_id: &str, waybill_no: String) -> Result<Card, AppError> {
    let conn = get_connection()?;

    // 获取卡片
    let card = get_card(card_id)?
        .ok_or_else(|| AppError::ProfileNotFound(format!("卡片不存在: {}", card_id)))?;

    // 更新 metadata 中的 pending_waybill_no
    let mut metadata = card.metadata.unwrap_or_default();
    metadata.pending_waybill_no = Some(waybill_no.clone());

    let metadata_json = serde_json::to_string(&metadata)
        .map_err(|e| AppError::Other(format!("序列化元数据失败: {}", e)))?;

    let updated_at = format_datetime(&now_china());

    // 更新卡片（不改变状态）
    conn.execute(
        "UPDATE cards SET metadata = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![metadata_json, updated_at, card_id],
    )
    .map_err(|e| AppError::Other(format!("更新卡片失败: {}", e)))?;

    log::info!("✅ 保存待处理运单号成功: {} -> {}", card_id, waybill_no);
    get_card(card_id)?.ok_or_else(|| AppError::Other("更新后无法获取卡片".to_string()))
}

/// 获取项目下的所有呼号（去重，统一大写）
pub fn get_project_callsigns(project_id: &str) -> Result<Vec<String>, AppError> {
    let conn = get_connection()?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT DISTINCT UPPER(callsign) FROM cards
            WHERE project_id = ?1
            ORDER BY UPPER(callsign) ASC
            "#,
        )
        .map_err(|e| AppError::Other(format!("准备查询语句失败: {}", e)))?;

    let callsigns = stmt
        .query_map([project_id], |row| row.get(0))
        .map_err(|e| AppError::Other(format!("查询呼号列表失败: {}", e)))?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| AppError::Other(format!("读取呼号数据失败: {}", e)))?;

    Ok(callsigns)
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    /// 创建内存测试库：建表语句取自迁移以与生产 schema 保真。
    ///
    /// - `projects` 取 `migrations/2026.1.24.001_init.sql`
    /// - `cards` 取 `migrations/2026.1.24.002_add_cards.sql`（含两条 CHECK 与 FK）
    ///
    /// 不开 `PRAGMA foreign_keys`：测试断言不依赖 FK 强制，仅靠先插 projects 命中 LEFT JOIN。
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
                qty INTEGER NOT NULL CHECK(qty > 0 AND qty <= 9999),
                serial INTEGER,
                status TEXT NOT NULL CHECK(status IN ('pending', 'distributed', 'returned')) DEFAULT 'pending',
                metadata TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );
            "#,
        )
        .unwrap();
        conn
    }

    /// 插入一个项目（含 name，供 LEFT JOIN 填充 project_name）。
    fn insert_project(conn: &Connection, id: &str, name: &str) {
        conn.execute(
            "INSERT INTO projects (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
            rusqlite::params![id, name, "2026-01-01T00:00:00+08:00"],
        )
        .unwrap();
    }

    /// 用直接 SQL 插入 `count` 条卡片（禁止 create_card 以避免同值 created_at）。
    ///
    /// - `created_at` 固定宽度递增（serial 越大越新），确保 DESC 排序可判定
    /// - `qty` 固定为 1，`serial` 递增 1..=count，使 qty != serial，能捕获列错位
    /// - `status='pending'`、`metadata` 为合法 JSON
    fn insert_cards(conn: &Connection, project_id: &str, count: i32) {
        for serial in 1..=count {
            let created_at = format!("2026-01-01 00:00:{:04}", serial);
            conn.execute(
                r#"
                INSERT INTO cards
                    (id, project_id, creator_id, callsign, qty, serial, status, metadata, created_at, updated_at)
                VALUES (?1, ?2, NULL, ?3, ?4, ?5, 'pending', ?6, ?7, ?7)
                "#,
                rusqlite::params![
                    format!("{}-card-{:04}", project_id, serial),
                    project_id,
                    format!("BH2T{:04}", serial),
                    1, // qty 固定为 1，与 serial 取不同值
                    serial,
                    "{}", // 合法 JSON 元数据
                    created_at,
                ],
            )
            .unwrap();
        }
    }

    /// 3.2 全量查询 + 列映射：单项目 150 条，验证不被 100 截断、project_name 填充、DESC 排序、逐列映射。
    #[test]
    fn test_list_all_cards_conn_full_and_mapping() {
        let conn = setup_test_db();
        insert_project(&conn, "p1", "项目一");
        insert_cards(&conn, "p1", 150);

        let filter = CardFilter {
            project_id: Some("p1".to_string()),
            ..Default::default()
        };
        let cards = list_all_cards_conn(&conn, filter).unwrap();

        // ① 返回 150 条（不被 100 截断）
        assert_eq!(cards.len(), 150);

        // ② project_name 已填充
        assert!(cards.iter().all(|c| c.project_name == "项目一"));

        // ③ 按 created_at DESC：首条 serial 最大、末条最小
        assert_eq!(cards.first().unwrap().serial, Some(150));
        assert_eq!(cards.last().unwrap().serial, Some(1));

        // ④ 逐列映射正确：取 serial != qty 的行（serial=150, qty=1）
        let top = cards.first().unwrap();
        assert_eq!(top.serial, Some(150));
        assert_eq!(top.qty, 1);
        assert_eq!(top.status, CardStatus::Pending);
        assert_eq!(top.callsign, "BH2T0150");
        // metadata 为合法空 JSON 对象，解析为 Some(默认 CardMetadata)
        assert!(top.metadata.is_some());
        let meta = top.metadata.as_ref().unwrap();
        assert!(meta.distribution.is_none());
        assert!(meta.return_info.is_none());
        assert!(meta.address_cache.is_none());
    }

    /// 3.3 分页上限：150 条传 page_size=100000，断言被钳到 100、total=150。
    #[test]
    fn test_list_cards_conn_page_size_cap() {
        let conn = setup_test_db();
        insert_project(&conn, "p1", "项目一");
        insert_cards(&conn, "p1", 150);

        let filter = CardFilter {
            project_id: Some("p1".to_string()),
            ..Default::default()
        };
        let paged = list_cards_conn(
            &conn,
            filter,
            Pagination {
                page: 1,
                page_size: 100000,
            },
        )
        .unwrap();

        assert_eq!(paged.items.len(), 100);
        assert_eq!(paged.page_size, 100);
        assert_eq!(paged.total, 150);
    }

    /// 3.4 项目过滤：两项目分别 120/30 条，按某项目筛选只返回该项目卡片。
    #[test]
    fn test_list_all_cards_conn_project_filter() {
        let conn = setup_test_db();
        insert_project(&conn, "p1", "项目一");
        insert_project(&conn, "p2", "项目二");
        insert_cards(&conn, "p1", 120);
        insert_cards(&conn, "p2", 30);

        let filter = CardFilter {
            project_id: Some("p1".to_string()),
            ..Default::default()
        };
        let cards = list_all_cards_conn(&conn, filter).unwrap();

        assert_eq!(cards.len(), 120);
        assert!(cards.iter().all(|c| c.project_id == "p1"));
    }

    /// 3.5 占位符编号等价性：验证 build_card_where 抽取后占位符随条件数动态编号。
    #[test]
    fn test_placeholder_numbering_equivalence() {
        let conn = setup_test_db();
        insert_project(&conn, "p1", "项目一");
        insert_cards(&conn, "p1", 150);

        // ① N=3：project_id + callsign + status 三条件，LIMIT/OFFSET 落 ?4/?5
        let filter_n3 = CardFilter {
            project_id: Some("p1".to_string()),
            callsign: Some("BH2T".to_string()),
            status: Some(CardStatus::Pending),
        };
        let paged_n3 = list_cards_conn(
            &conn,
            filter_n3,
            Pagination {
                page: 1,
                page_size: 100000,
            },
        )
        .unwrap();
        assert_eq!(paged_n3.items.len(), 100);
        assert!(paged_n3
            .items
            .iter()
            .all(|c| c.project_id == "p1"
                && c.callsign.contains("BH2T")
                && c.status == CardStatus::Pending));

        // ② N=0：同一 150 行库无 filter，LIMIT/OFFSET 落 ?1/?2
        let paged_n0 = list_cards_conn(
            &conn,
            CardFilter::default(),
            Pagination {
                page: 1,
                page_size: 100000,
            },
        )
        .unwrap();
        assert_eq!(paged_n0.items.len(), 100);
        assert_eq!(paged_n0.total, 150);
    }
}
