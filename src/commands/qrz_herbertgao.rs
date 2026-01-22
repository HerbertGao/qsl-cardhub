// QRZ.herbertgao.me Tauri 命令实现

use crate::qrz::qrz_herbertgao_client::{query_callsign, HerbertgaoAddressInfo};

/// 查询呼号地址
///
/// # 参数
/// * `callsign` - 呼号
///
/// # 返回
/// 查询到的地址信息，如果未找到则返回 None
///
/// # 错误处理
/// 查询失败时返回错误，但不应显示给用户（由前端静默处理）
#[tauri::command]
pub async fn qrz_herbertgao_query_callsign(
    callsign: String,
) -> Result<Option<HerbertgaoAddressInfo>, String> {
    log::debug!("前端请求查询 QRZ.herbertgao.me 呼号: {}", callsign);

    match query_callsign(&callsign).await {
        Ok(result) => {
            if result.is_some() {
                log::info!("✓ QRZ.herbertgao.me 查询成功: {}", callsign);
            } else {
                log::debug!("QRZ.herbertgao.me 未找到呼号: {}", callsign);
            }
            Ok(result)
        }
        Err(e) => {
            // 记录错误到日志，但返回 Err 供前端静默处理
            log::warn!("QRZ.herbertgao.me 查询失败: {}", e);
            Err(format!("查询失败: {}", e))
        }
    }
}
