// 平台信息 Commands

use crate::utils::platform::{detect_platform, PlatformInfo};

/// 获取平台信息
///
/// 返回当前操作系统和 CPU 架构信息
#[tauri::command]
pub fn get_platform_info() -> PlatformInfo {
    detect_platform()
}
