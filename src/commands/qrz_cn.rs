// QRZ.cn Tauri 命令实现

use crate::qrz::{QRZCnClient, QrzCnAddressInfo};
use crate::security::{get_credential, save_credential};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

/// 全局 QRZ.cn 客户端实例
static QRZ_CN_CLIENT: OnceCell<Mutex<QRZCnClient>> = OnceCell::new();

/// 获取或创建 QRZ.cn 客户端
fn get_qrz_cn_client() -> Result<&'static Mutex<QRZCnClient>, String> {
    QRZ_CN_CLIENT.get_or_try_init(|| {
        QRZCnClient::new()
            .map(Mutex::new)
            .map_err(|e| format!("无法创建 QRZ.cn 客户端: {}", e))
    })
}

/// 保存并登录 QRZ.cn
#[tauri::command]
pub async fn qrz_save_and_login(username: String, password: String) -> Result<String, String> {
    // 保存用户名到加密存储
    save_credential("qsl-cardhub:qrz:username", &username)
        .map_err(|e| format!("保存用户名失败: {}", e))?;

    // 保存密码到加密存储
    save_credential("qsl-cardhub:qrz:password", &password)
        .map_err(|e| format!("保存密码失败: {}", e))?;

    // 登录
    let client = get_qrz_cn_client()?;
    let client = client.lock().await;
    client
        .login(&username, &password)
        .await
        .map_err(|e| format!("登录失败: {}", e))?;

    // 保存会话
    client
        .save_session()
        .await
        .map_err(|e| format!("保存会话失败: {}", e))?;

    Ok("登录成功".to_string())
}

/// 加载 QRZ.cn 凭据（仅返回用户名，密码从加密存储读取）
#[tauri::command]
pub fn qrz_load_credentials() -> Result<Option<String>, String> {
    let key = "qsl-cardhub:qrz:password";
    get_credential(key).map_err(|e| format!("加载凭据失败: {}", e))
}

/// 清除 QRZ.cn 凭据
#[tauri::command]
pub async fn qrz_clear_credentials() -> Result<String, String> {
    // 清除用户名
    let _ = crate::security::delete_credential("qsl-cardhub:qrz:username");

    // 清除密码
    let _ = crate::security::delete_credential("qsl-cardhub:qrz:password");

    // 清除会话
    let _ = crate::security::delete_credential("qsl-cardhub:qrz:session");

    if let Ok(client) = get_qrz_cn_client() {
        let client = client.lock().await;
        client.clear_session().await;
    }

    Ok("凭据已清除".to_string())
}

/// 查询呼号地址
#[tauri::command]
pub async fn qrz_query_callsign(callsign: String) -> Result<Option<QrzCnAddressInfo>, String> {
    let client = get_qrz_cn_client()?;
    let client = client.lock().await;

    // 检查会话是否有效
    if !client.is_session_valid().await {
        return Err("会话已过期，请重新登录".to_string());
    }

    // 查询地址
    client
        .query_callsign(&callsign)
        .await
        .map_err(|e| format!("查询失败: {}", e))
}

/// 检查 QRZ.cn 登录状态
#[tauri::command]
pub async fn qrz_check_login_status() -> bool {
    log::debug!("前端请求检查 QRZ.cn 登录状态");

    if let Ok(client) = get_qrz_cn_client() {
        let client = client.lock().await;

        // 先检查内存中的会话是否有效
        let is_valid = client.is_session_valid().await;
        log::debug!("内存中的会话有效性: {}", is_valid);

        // 如果会话无效，尝试从存储恢复
        if !is_valid {
            log::debug!("尝试从加密存储恢复会话...");
            if let Ok(()) = client.load_session().await {
                // 成功恢复会话，再次检查有效性
                let restored_valid = client.is_session_valid().await;
                log::info!("会话恢复成功，有效性: {}", restored_valid);
                return restored_valid;
            } else {
                log::debug!("无法从存储恢复会话");
            }
        }

        is_valid
    } else {
        log::error!("无法获取 QRZ 客户端实例");
        false
    }
}

/// 测试 QRZ.cn 连接（从系统获取登录信息后执行查询）
#[tauri::command]
pub async fn qrz_test_connection() -> Result<String, String> {
    // 从加密存储读取用户名和密码
    let username = get_credential("qsl-cardhub:qrz:username")
        .map_err(|e| format!("读取用户名失败: {}", e))?
        .ok_or_else(|| "未找到保存的用户名".to_string())?;

    let password = get_credential("qsl-cardhub:qrz:password")
        .map_err(|e| format!("读取密码失败: {}", e))?
        .ok_or_else(|| "未找到保存的密码".to_string())?;

    // 获取客户端
    let client = get_qrz_cn_client()?;
    let client = client.lock().await;

    // 检查会话是否有效，如果无效则重新登录
    if !client.is_session_valid().await {
        log::info!("会话已过期，使用保存的凭据重新登录");
        client
            .login(&username, &password)
            .await
            .map_err(|e| format!("重新登录失败: {}", e))?;

        // 保存新会话
        client
            .save_session()
            .await
            .map_err(|e| format!("保存会话失败: {}", e))?;
    }

    // 查询测试呼号 BY1CRA
    let test_callsign = "BY1CRA";
    log::info!("正在查询测试呼号: {}", test_callsign);

    match client.query_callsign(test_callsign).await {
        Ok(Some(address_info)) => {
            // 格式化地址信息
            let mut result = format!("测试成功！查询到呼号 {} 的地址信息：\n", test_callsign);
            result.push_str(&format!("呼号: {} (qrz.cn)\n", address_info.callsign));

            if let Some(chinese) = &address_info.chinese_address {
                result.push_str(&format!("中文地址: {}\n", chinese));
            }

            if let Some(english) = &address_info.english_address {
                result.push_str(&format!("英文地址: {}\n", english));
            }

            if let Some(updated) = &address_info.updated_at {
                result.push_str(&format!("更新时间: {}", updated));
            }

            Ok(result)
        }
        Ok(None) => Err(format!(
            "连接正常，但未找到测试呼号 {} 的地址信息",
            test_callsign
        )),
        Err(e) => Err(format!("查询失败: {}", e)),
    }
}
