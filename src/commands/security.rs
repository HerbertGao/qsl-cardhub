use crate::security::{delete_credential, get_credential, is_keyring_available, save_credential};
use serde::{Deserialize, Serialize};

/// 凭据信息（用于前端交互）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialInfo {
    /// 用户名
    pub username: String,
}

/// 保存凭据
#[tauri::command]
pub fn save_credentials(
    category: String,
    item: String,
    username: String,
    password: String,
) -> Result<String, String> {
    // 保存用户名到配置文件（由前端处理，这里不处理）
    // 保存密码到加密存储
    let key = format!("qsl-cardhub:{}:{}", category, item);
    save_credential(&key, &password).map_err(|e| format!("保存凭据失败: {}", e))?;

    Ok("凭据保存成功".to_string())
}

/// 加载凭据
#[tauri::command]
pub fn load_credentials(category: String, item: String) -> Result<Option<String>, String> {
    let key = format!("qsl-cardhub:{}:{}", category, item);
    get_credential(&key).map_err(|e| format!("加载凭据失败: {}", e))
}

/// 清除凭据
#[tauri::command]
pub fn clear_credentials(category: String, item: String) -> Result<String, String> {
    let key = format!("qsl-cardhub:{}:{}", category, item);
    delete_credential(&key).map_err(|e| format!("清除凭据失败: {}", e))?;

    Ok("凭据清除成功".to_string())
}

/// 检查钥匙串是否可用
#[tauri::command]
pub fn check_keyring_available() -> bool {
    is_keyring_available()
}
