// 顺丰速运数据库操作模块
//
// 提供寄件人和订单的 CRUD 操作

use crate::db::models::{format_datetime, now_china};
use crate::db::sqlite::get_connection;
use crate::error::AppError;
use crate::sf_express::{OrderStatus, SFOrder, SFOrderWithCard, SenderInfo};
use uuid::Uuid;

// ==================== 寄件人操作 ====================

/// 创建寄件人
pub fn create_sender(
    name: String,
    phone: String,
    mobile: Option<String>,
    province: String,
    city: String,
    district: String,
    address: String,
    is_default: bool,
) -> Result<SenderInfo, AppError> {
    let conn = get_connection()?;

    // 如果设为默认，先清除其他默认
    if is_default {
        conn.execute("UPDATE sf_senders SET is_default = 0 WHERE is_default = 1", [])
            .map_err(|e| AppError::Other(format!("更新默认寄件人失败: {}", e)))?;
    }

    let now = format_datetime(&now_china());
    let sender = SenderInfo {
        id: Uuid::new_v4().to_string(),
        name,
        phone,
        mobile,
        province,
        city,
        district,
        address,
        is_default,
        created_at: now.clone(),
        updated_at: now,
    };

    conn.execute(
        r#"
        INSERT INTO sf_senders (id, name, phone, mobile, province, city, district, address, is_default, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        "#,
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
    .map_err(|e| AppError::Other(format!("创建寄件人失败: {}", e)))?;

    log::info!("✅ 创建寄件人成功: {} ({})", sender.name, sender.id);
    Ok(sender)
}

/// 更新寄件人
pub fn update_sender(
    id: &str,
    name: String,
    phone: String,
    mobile: Option<String>,
    province: String,
    city: String,
    district: String,
    address: String,
    is_default: bool,
) -> Result<SenderInfo, AppError> {
    let conn = get_connection()?;

    // 如果设为默认，先清除其他默认
    if is_default {
        conn.execute(
            "UPDATE sf_senders SET is_default = 0 WHERE is_default = 1 AND id != ?1",
            [id],
        )
        .map_err(|e| AppError::Other(format!("更新默认寄件人失败: {}", e)))?;
    }

    let updated_at = format_datetime(&now_china());

    conn.execute(
        r#"
        UPDATE sf_senders SET
            name = ?1, phone = ?2, mobile = ?3, province = ?4, city = ?5,
            district = ?6, address = ?7, is_default = ?8, updated_at = ?9
        WHERE id = ?10
        "#,
        rusqlite::params![
            name, phone, mobile, province, city, district, address,
            is_default as i32, updated_at, id
        ],
    )
    .map_err(|e| AppError::Other(format!("更新寄件人失败: {}", e)))?;

    log::info!("✅ 更新寄件人成功: {}", id);
    get_sender(id)?.ok_or_else(|| AppError::Other("更新后无法获取寄件人".to_string()))
}

/// 删除寄件人
pub fn delete_sender(id: &str) -> Result<(), AppError> {
    let conn = get_connection()?;

    // 检查是否为默认寄件人
    let sender = get_sender(id)?;
    let was_default = sender.map(|s| s.is_default).unwrap_or(false);

    conn.execute("DELETE FROM sf_senders WHERE id = ?1", [id])
        .map_err(|e| AppError::Other(format!("删除寄件人失败: {}", e)))?;

    // 如果删除的是默认寄件人，设置第一个为默认
    if was_default {
        conn.execute(
            "UPDATE sf_senders SET is_default = 1 WHERE id = (SELECT id FROM sf_senders ORDER BY created_at LIMIT 1)",
            [],
        )
        .ok(); // 忽略错误（可能没有其他寄件人）
    }

    log::info!("✅ 删除寄件人成功: {}", id);
    Ok(())
}

/// 获取单个寄件人
pub fn get_sender(id: &str) -> Result<Option<SenderInfo>, AppError> {
    let conn = get_connection()?;

    let result = conn.query_row(
        r#"
        SELECT id, name, phone, mobile, province, city, district, address, is_default, created_at, updated_at
        FROM sf_senders WHERE id = ?1
        "#,
        [id],
        |row| {
            Ok(SenderInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                phone: row.get(2)?,
                mobile: row.get(3)?,
                province: row.get(4)?,
                city: row.get(5)?,
                district: row.get(6)?,
                address: row.get(7)?,
                is_default: row.get::<_, i32>(8)? != 0,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        },
    );

    match result {
        Ok(sender) => Ok(Some(sender)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Other(format!("查询寄件人失败: {}", e))),
    }
}

/// 获取寄件人列表
pub fn list_senders() -> Result<Vec<SenderInfo>, AppError> {
    let conn = get_connection()?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, name, phone, mobile, province, city, district, address, is_default, created_at, updated_at
            FROM sf_senders ORDER BY is_default DESC, created_at DESC
            "#,
        )
        .map_err(|e| AppError::Other(format!("准备查询语句失败: {}", e)))?;

    let senders = stmt
        .query_map([], |row| {
            Ok(SenderInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                phone: row.get(2)?,
                mobile: row.get(3)?,
                province: row.get(4)?,
                city: row.get(5)?,
                district: row.get(6)?,
                address: row.get(7)?,
                is_default: row.get::<_, i32>(8)? != 0,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| AppError::Other(format!("查询寄件人列表失败: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Other(format!("读取寄件人数据失败: {}", e)))?;

    Ok(senders)
}

/// 获取默认寄件人
pub fn get_default_sender() -> Result<Option<SenderInfo>, AppError> {
    let conn = get_connection()?;

    let result = conn.query_row(
        r#"
        SELECT id, name, phone, mobile, province, city, district, address, is_default, created_at, updated_at
        FROM sf_senders WHERE is_default = 1 LIMIT 1
        "#,
        [],
        |row| {
            Ok(SenderInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                phone: row.get(2)?,
                mobile: row.get(3)?,
                province: row.get(4)?,
                city: row.get(5)?,
                district: row.get(6)?,
                address: row.get(7)?,
                is_default: row.get::<_, i32>(8)? != 0,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        },
    );

    match result {
        Ok(sender) => Ok(Some(sender)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Other(format!("查询默认寄件人失败: {}", e))),
    }
}

/// 设置默认寄件人
pub fn set_default_sender(id: &str) -> Result<SenderInfo, AppError> {
    let conn = get_connection()?;

    // 清除所有默认
    conn.execute("UPDATE sf_senders SET is_default = 0", [])
        .map_err(|e| AppError::Other(format!("清除默认寄件人失败: {}", e)))?;

    // 设置新默认
    let updated_at = format_datetime(&now_china());
    conn.execute(
        "UPDATE sf_senders SET is_default = 1, updated_at = ?1 WHERE id = ?2",
        rusqlite::params![updated_at, id],
    )
    .map_err(|e| AppError::Other(format!("设置默认寄件人失败: {}", e)))?;

    log::info!("✅ 设置默认寄件人成功: {}", id);
    get_sender(id)?.ok_or_else(|| AppError::Other("设置后无法获取寄件人".to_string()))
}

// ==================== 订单操作 ====================

/// 创建订单
pub fn create_order(
    order_id: String,
    card_id: Option<String>,
    pay_method: Option<i32>,
    cargo_name: Option<String>,
    sender_info: String,
    recipient_info: String,
) -> Result<SFOrder, AppError> {
    let conn = get_connection()?;
    let now = format_datetime(&now_china());

    let order = SFOrder {
        id: Uuid::new_v4().to_string(),
        order_id,
        waybill_no: None,
        card_id,
        status: OrderStatus::Pending.to_string(),
        pay_method,
        cargo_name,
        sender_info,
        recipient_info,
        created_at: now.clone(),
        updated_at: now,
    };

    conn.execute(
        r#"
        INSERT INTO sf_orders (id, order_id, waybill_no, card_id, status, pay_method, cargo_name, sender_info, recipient_info, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        "#,
        rusqlite::params![
            &order.id,
            &order.order_id,
            &order.waybill_no,
            &order.card_id,
            &order.status,
            &order.pay_method,
            &order.cargo_name,
            &order.sender_info,
            &order.recipient_info,
            &order.created_at,
            &order.updated_at,
        ],
    )
    .map_err(|e| AppError::Other(format!("创建订单失败: {}", e)))?;

    log::info!("✅ 创建订单成功: {} ({})", order.order_id, order.id);
    Ok(order)
}

/// 更新订单状态
pub fn update_order_status(
    order_id: &str,
    status: OrderStatus,
    waybill_no: Option<String>,
) -> Result<SFOrder, AppError> {
    let conn = get_connection()?;
    let updated_at = format_datetime(&now_china());

    conn.execute(
        "UPDATE sf_orders SET status = ?1, waybill_no = ?2, updated_at = ?3 WHERE order_id = ?4",
        rusqlite::params![status.to_string(), waybill_no, updated_at, order_id],
    )
    .map_err(|e| AppError::Other(format!("更新订单状态失败: {}", e)))?;

    log::info!("✅ 更新订单状态成功: {} -> {:?}", order_id, status);
    get_order_by_order_id(order_id)?
        .ok_or_else(|| AppError::Other("更新后无法获取订单".to_string()))
}

/// 获取订单（按内部 ID）
pub fn get_order(id: &str) -> Result<Option<SFOrder>, AppError> {
    let conn = get_connection()?;

    let result = conn.query_row(
        r#"
        SELECT id, order_id, waybill_no, card_id, status, pay_method, cargo_name, sender_info, recipient_info, created_at, updated_at
        FROM sf_orders WHERE id = ?1
        "#,
        [id],
        row_to_order,
    );

    match result {
        Ok(order) => Ok(Some(order)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Other(format!("查询订单失败: {}", e))),
    }
}

/// 获取订单（按顺丰订单号）
pub fn get_order_by_order_id(order_id: &str) -> Result<Option<SFOrder>, AppError> {
    let conn = get_connection()?;

    let result = conn.query_row(
        r#"
        SELECT id, order_id, waybill_no, card_id, status, pay_method, cargo_name, sender_info, recipient_info, created_at, updated_at
        FROM sf_orders WHERE order_id = ?1
        "#,
        [order_id],
        row_to_order,
    );

    match result {
        Ok(order) => Ok(Some(order)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Other(format!("查询订单失败: {}", e))),
    }
}

/// 获取订单（按运单号）
pub fn get_order_by_waybill_no(waybill_no: &str) -> Result<Option<SFOrder>, AppError> {
    let conn = get_connection()?;

    let result = conn.query_row(
        r#"
        SELECT id, order_id, waybill_no, card_id, status, pay_method, cargo_name, sender_info, recipient_info, created_at, updated_at
        FROM sf_orders WHERE waybill_no = ?1
        "#,
        [waybill_no],
        row_to_order,
    );

    match result {
        Ok(order) => Ok(Some(order)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Other(format!("查询订单失败: {}", e))),
    }
}

/// 获取卡片关联的订单
pub fn get_order_by_card_id(card_id: &str) -> Result<Option<SFOrder>, AppError> {
    let conn = get_connection()?;

    let result = conn.query_row(
        r#"
        SELECT id, order_id, waybill_no, card_id, status, pay_method, cargo_name, sender_info, recipient_info, created_at, updated_at
        FROM sf_orders WHERE card_id = ?1 ORDER BY created_at DESC LIMIT 1
        "#,
        [card_id],
        row_to_order,
    );

    match result {
        Ok(order) => Ok(Some(order)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Other(format!("查询订单失败: {}", e))),
    }
}

/// 订单列表查询参数
pub struct OrderFilter {
    pub status: Option<OrderStatus>,
    pub card_id: Option<String>,
}

/// 订单列表分页
pub struct OrderPagination {
    pub page: u32,
    pub page_size: u32,
}

/// 订单列表结果
pub struct PagedOrders {
    pub items: Vec<SFOrder>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

/// 查询订单列表
pub fn list_orders(filter: OrderFilter, pagination: OrderPagination) -> Result<PagedOrders, AppError> {
    let conn = get_connection()?;

    // 构建 WHERE 子句
    let mut conditions = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref status) = filter.status {
        conditions.push(format!("status = ?{}", params.len() + 1));
        params.push(Box::new(status.to_string()));
    }

    if let Some(ref card_id) = filter.card_id {
        conditions.push(format!("card_id = ?{}", params.len() + 1));
        params.push(Box::new(card_id.clone()));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // 查询总数
    let count_sql = format!("SELECT COUNT(*) FROM sf_orders {}", where_clause);
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
        SELECT id, order_id, waybill_no, card_id, status, pay_method, cargo_name, sender_info, recipient_info, created_at, updated_at
        FROM sf_orders {} ORDER BY created_at DESC LIMIT ?{} OFFSET ?{}
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
        .query_map(params_ref.as_slice(), row_to_order)
        .map_err(|e| AppError::Other(format!("查询订单列表失败: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Other(format!("读取订单数据失败: {}", e)))?;

    Ok(PagedOrders {
        items,
        total,
        page,
        page_size,
        total_pages,
    })
}

/// 订单列表结果（带卡片信息）
pub struct PagedOrdersWithCard {
    pub items: Vec<SFOrderWithCard>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

/// 查询订单列表（带卡片信息）
pub fn list_orders_with_cards(filter: OrderFilter, pagination: OrderPagination) -> Result<PagedOrdersWithCard, AppError> {
    let conn = get_connection()?;

    // 构建 WHERE 子句
    let mut conditions = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref status) = filter.status {
        conditions.push(format!("o.status = ?{}", params.len() + 1));
        params.push(Box::new(status.to_string()));
    }

    if let Some(ref card_id) = filter.card_id {
        conditions.push(format!("o.card_id = ?{}", params.len() + 1));
        params.push(Box::new(card_id.clone()));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // 查询总数
    let count_sql = format!("SELECT COUNT(*) FROM sf_orders o {}", where_clause);
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

    // 查询数据（LEFT JOIN cards 和 projects 表获取卡片信息）
    let data_sql = format!(
        r#"
        SELECT o.id, o.order_id, o.waybill_no, o.card_id, o.status, o.pay_method, o.cargo_name,
               o.sender_info, o.recipient_info, o.created_at, o.updated_at,
               c.callsign, p.name as project_name, c.qty
        FROM sf_orders o
        LEFT JOIN cards c ON o.card_id = c.id
        LEFT JOIN projects p ON c.project_id = p.id
        {} ORDER BY o.created_at DESC LIMIT ?{} OFFSET ?{}
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
        .query_map(params_ref.as_slice(), row_to_order_with_card)
        .map_err(|e| AppError::Other(format!("查询订单列表失败: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Other(format!("读取订单数据失败: {}", e)))?;

    Ok(PagedOrdersWithCard {
        items,
        total,
        page,
        page_size,
        total_pages,
    })
}

/// 删除订单
pub fn delete_order(id: &str) -> Result<(), AppError> {
    let conn = get_connection()?;

    conn.execute("DELETE FROM sf_orders WHERE id = ?1", [id])
        .map_err(|e| AppError::Other(format!("删除订单失败: {}", e)))?;

    log::info!("✅ 删除订单成功: {}", id);
    Ok(())
}

// 辅助函数：将数据库行转换为订单
fn row_to_order(row: &rusqlite::Row) -> rusqlite::Result<SFOrder> {
    Ok(SFOrder {
        id: row.get(0)?,
        order_id: row.get(1)?,
        waybill_no: row.get(2)?,
        card_id: row.get(3)?,
        status: row.get(4)?,
        pay_method: row.get(5)?,
        cargo_name: row.get(6)?,
        sender_info: row.get(7)?,
        recipient_info: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

// 辅助函数：将数据库行转换为带卡片信息的订单
fn row_to_order_with_card(row: &rusqlite::Row) -> rusqlite::Result<SFOrderWithCard> {
    Ok(SFOrderWithCard {
        id: row.get(0)?,
        order_id: row.get(1)?,
        waybill_no: row.get(2)?,
        card_id: row.get(3)?,
        status: row.get(4)?,
        pay_method: row.get(5)?,
        cargo_name: row.get(6)?,
        sender_info: row.get(7)?,
        recipient_info: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        callsign: row.get(11)?,
        project_name: row.get(12)?,
        qty: row.get(13)?,
    })
}
