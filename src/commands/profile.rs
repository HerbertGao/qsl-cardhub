// Profile 配置管理 Commands

use crate::config::{Platform, Profile, ProfileManager};
use std::sync::{Arc, Mutex};
use tauri::State;

/// 应用状态（Profile 管理器部分）
pub struct ProfileState {
    pub manager: Arc<Mutex<ProfileManager>>,
}

/// 获取所有 Profile
#[tauri::command]
pub async fn get_profiles(state: State<'_, ProfileState>) -> Result<Vec<Profile>, String> {
    let manager = state
        .manager
        .lock()
        .map_err(|e| format!("锁定失败: {}", e))?;
    manager
        .get_all()
        .map_err(|e| format!("获取配置列表失败: {}", e))
}

/// 根据 ID 获取 Profile
#[tauri::command]
pub async fn get_profile(
    id: String,
    state: State<'_, ProfileState>,
) -> Result<Option<Profile>, String> {
    let manager = state
        .manager
        .lock()
        .map_err(|e| format!("锁定失败: {}", e))?;
    manager
        .get_by_id(&id)
        .map_err(|e| format!("获取配置失败: {}", e))
}

/// 创建新的 Profile
#[tauri::command]
pub async fn create_profile(
    name: String,
    task_name: Option<String>,
    printer_name: String,
    platform: Platform,
    state: State<'_, ProfileState>,
) -> Result<Profile, String> {
    let mut manager = state
        .manager
        .lock()
        .map_err(|e| format!("锁定失败: {}", e))?;
    manager
        .create(name, task_name, printer_name, platform)
        .map_err(|e| format!("创建配置失败: {}", e))
}

/// 更新 Profile
#[tauri::command]
pub async fn update_profile(
    id: String,
    profile: Profile,
    state: State<'_, ProfileState>,
) -> Result<(), String> {
    let mut manager = state
        .manager
        .lock()
        .map_err(|e| format!("锁定失败: {}", e))?;
    manager
        .update(&id, profile)
        .map_err(|e| format!("更新配置失败: {}", e))
}

/// 删除 Profile
#[tauri::command]
pub async fn delete_profile(id: String, state: State<'_, ProfileState>) -> Result<(), String> {
    let mut manager = state
        .manager
        .lock()
        .map_err(|e| format!("锁定失败: {}", e))?;
    manager
        .delete(&id)
        .map_err(|e| format!("删除配置失败: {}", e))
}

/// 设置默认 Profile
#[tauri::command]
pub async fn set_default_profile(id: String, state: State<'_, ProfileState>) -> Result<(), String> {
    let mut manager = state
        .manager
        .lock()
        .map_err(|e| format!("锁定失败: {}", e))?;
    manager
        .set_default(&id)
        .map_err(|e| format!("设置默认配置失败: {}", e))
}

/// 获取默认 Profile ID
#[tauri::command]
pub async fn get_default_profile_id(
    state: State<'_, ProfileState>,
) -> Result<Option<String>, String> {
    let manager = state
        .manager
        .lock()
        .map_err(|e| format!("锁定失败: {}", e))?;
    Ok(manager.get_default_id().map(|s| s.to_string()))
}

/// 导出 Profile 为 JSON
#[tauri::command]
pub async fn export_profile(id: String, state: State<'_, ProfileState>) -> Result<String, String> {
    let manager = state
        .manager
        .lock()
        .map_err(|e| format!("锁定失败: {}", e))?;
    manager
        .export_profile(&id)
        .map_err(|e| format!("导出配置失败: {}", e))
}

/// 从 JSON 导入 Profile
#[tauri::command]
pub async fn import_profile(
    json: String,
    state: State<'_, ProfileState>,
) -> Result<Profile, String> {
    let mut manager = state
        .manager
        .lock()
        .map_err(|e| format!("锁定失败: {}", e))?;
    manager
        .import_profile(&json)
        .map_err(|e| format!("导入配置失败: {}", e))
}

/// 获取默认模板的显示名称
#[tauri::command]
pub async fn get_default_template_name(state: State<'_, ProfileState>) -> Result<String, String> {
    let manager = state
        .manager
        .lock()
        .map_err(|e| format!("锁定失败: {}", e))?;
    Ok(manager.get_default_template_name())
}
