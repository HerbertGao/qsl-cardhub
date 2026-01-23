// 卡片管理 Tauri 命令
//
// 提供前端调用的卡片管理 API

use crate::db::{self, Card, CardFilter, CardStatus, PagedCards, Pagination};

/// 创建卡片
#[tauri::command]
pub async fn create_card_cmd(
    project_id: String,
    callsign: String,
    qty: i32,
    serial: Option<i32>,
) -> Result<Card, String> {
    tokio::task::spawn_blocking(move || {
        db::create_card(project_id, callsign, qty, serial).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 查询卡片列表（分页）
#[tauri::command]
pub async fn list_cards_cmd(
    project_id: Option<String>,
    callsign: Option<String>,
    status: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<PagedCards, String> {
    tokio::task::spawn_blocking(move || {
        let filter = CardFilter {
            project_id,
            callsign,
            status: status.and_then(|s| CardStatus::from_str(&s)),
        };

        let pagination = Pagination {
            page: page.unwrap_or(1),
            page_size: page_size.unwrap_or(20),
        };

        db::list_cards(filter, pagination).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 获取单个卡片
#[tauri::command]
pub async fn get_card_cmd(id: String) -> Result<Option<Card>, String> {
    tokio::task::spawn_blocking(move || db::get_card(&id).map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

/// 分发卡片
#[tauri::command]
pub async fn distribute_card_cmd(
    id: String,
    method: String,
    address: Option<String>,
    remarks: Option<String>,
) -> Result<Card, String> {
    tokio::task::spawn_blocking(move || {
        db::distribute_card(&id, method, address, remarks).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 退回卡片
#[tauri::command]
pub async fn return_card_cmd(
    id: String,
    method: String,
    remarks: Option<String>,
) -> Result<Card, String> {
    tokio::task::spawn_blocking(move || {
        db::return_card(&id, method, remarks).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 删除卡片
#[tauri::command]
pub async fn delete_card_cmd(id: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || db::delete_card(&id).map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

/// 获取项目的最大序列号
#[tauri::command]
pub async fn get_max_serial_cmd(project_id: String) -> Result<Option<u32>, String> {
    tokio::task::spawn_blocking(move || {
        db::get_max_serial_by_project(&project_id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 保存地址到卡片
#[tauri::command]
pub async fn save_card_address_cmd(
    card_id: String,
    source: String,
    chinese_address: Option<String>,
    english_address: Option<String>,
    name: Option<String>,
    mail_method: Option<String>,
    updated_at: Option<String>,
) -> Result<Card, String> {
    tokio::task::spawn_blocking(move || {
        db::save_card_address(
            &card_id,
            source,
            chinese_address,
            english_address,
            name,
            mail_method,
            updated_at,
        )
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
