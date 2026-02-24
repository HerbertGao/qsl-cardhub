// 全局配置 Commands
//
// 提供 app_settings 的 Tauri 命令

use crate::db::app_settings;
use crate::db::models::AppSetting;
use tauri::command;

/// 获取单个配置项
#[command]
pub async fn get_app_setting_cmd(key: String) -> Result<Option<String>, String> {
    app_settings::get_setting(&key).map_err(|e| e.to_string())
}

/// 设置配置项
#[command]
pub async fn set_app_setting_cmd(key: String, value: String) -> Result<(), String> {
    app_settings::set_setting(&key, &value).map_err(|e| e.to_string())
}

/// 获取所有配置项
#[command]
pub async fn get_all_app_settings_cmd() -> Result<Vec<AppSetting>, String> {
    app_settings::get_all_settings().map_err(|e| e.to_string())
}
