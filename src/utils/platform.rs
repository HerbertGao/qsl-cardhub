// 平台检测工具

use serde::Serialize;

/// 平台信息
#[derive(Debug, Clone, Serialize)]
pub struct PlatformInfo {
    /// 操作系统：Windows | macOS | Linux
    pub os: String,
    /// CPU 架构：x86_64 | arm64
    pub arch: String,
}

/// 检测当前平台信息
pub fn detect_platform() -> PlatformInfo {
    let os = if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        "Unknown"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        std::env::consts::ARCH
    };

    PlatformInfo {
        os: os.to_string(),
        arch: arch.to_string(),
    }
}
