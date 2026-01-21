// QSL-CardHub - Rust + Tauri 版本
//
// 业余无线电 QSL 卡片打印工具

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod error;
mod logger;
mod printer;
mod utils;

use commands::{
    logger::{clear_logs, export_logs, get_log_file_path, get_logs},
    platform::get_platform_info,
    printer::{PrinterState, generate_tspl, load_template, preview_qsl, print_qsl, save_template},
    profile::{
        ProfileState, create_profile, delete_profile, export_profile, get_default_profile_id,
        get_profile, get_profiles, import_profile, set_default_profile, update_profile,
    },
};
use config::ProfileManager;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 获取配置目录
            let config_dir = get_config_dir()?;
            let output_dir = get_output_dir()?;
            let log_dir = get_log_dir()?;

            println!("📁 配置目录: {}", config_dir.display());
            println!("📁 输出目录: {}", output_dir.display());
            println!("📁 日志目录: {}", log_dir.display());

            // 初始化日志系统
            logger::init_logger(log_dir).map_err(|e| format!("无法初始化日志系统: {}", e))?;

            // 初始化 ProfileManager
            let profile_manager = ProfileManager::new(config_dir)
                .map_err(|e| format!("无法初始化配置管理器: {}", e))?;

            // 初始化 PrinterState
            let printer_state =
                PrinterState::new().map_err(|e| format!("无法初始化打印管理器: {}", e))?;

            // 管理应用状态
            app.manage(ProfileState {
                manager: Arc::new(Mutex::new(profile_manager)),
            });

            app.manage(printer_state);

            println!("✅ QSL-CardHub 初始化完成");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 平台信息
            get_platform_info,
            // Profile 管理
            get_profiles,
            get_profile,
            create_profile,
            update_profile,
            delete_profile,
            set_default_profile,
            get_default_profile_id,
            export_profile,
            import_profile,
            // 打印机管理
            preview_qsl,
            print_qsl,
            generate_tspl,
            load_template,
            save_template,
            // 日志管理
            get_logs,
            clear_logs,
            export_logs,
            get_log_file_path,
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}

/// 获取配置目录
fn get_config_dir() -> Result<PathBuf, String> {
    // 开发模式：使用项目根目录的 config/
    #[cfg(debug_assertions)]
    {
        let config_dir = PathBuf::from("config");
        return Ok(config_dir);
    }

    // 生产模式：使用系统配置目录
    #[cfg(not(debug_assertions))]
    {
        let home_dir = dirs::home_dir().ok_or("无法获取用户主目录")?;

        let config_dir = if cfg!(target_os = "windows") {
            // Windows: %APPDATA%/QSL-CardHub
            home_dir.join("AppData").join("Roaming").join("QSL-CardHub")
        } else if cfg!(target_os = "macos") {
            // macOS: ~/Library/Application Support/QSL-CardHub
            home_dir
                .join("Library")
                .join("Application Support")
                .join("QSL-CardHub")
        } else {
            // Linux: ~/.config/QSL-CardHub
            home_dir.join(".config").join("QSL-CardHub")
        };

        Ok(config_dir)
    }
}

/// 获取输出目录（Mock 打印）
fn get_output_dir() -> Result<PathBuf, String> {
    // 开发模式：使用项目根目录的 output/
    #[cfg(debug_assertions)]
    {
        let output_dir = PathBuf::from("output");
        return Ok(output_dir);
    }

    // 生产模式：使用配置目录下的 output/
    #[cfg(not(debug_assertions))]
    {
        let config_dir = get_config_dir()?;
        Ok(config_dir.join("output"))
    }
}

/// 获取日志目录
fn get_log_dir() -> Result<PathBuf, String> {
    // 开发模式：使用项目根目录的 logs/
    #[cfg(debug_assertions)]
    {
        let log_dir = PathBuf::from("logs");
        return Ok(log_dir);
    }

    // 生产模式：使用配置目录下的 logs/
    #[cfg(not(debug_assertions))]
    {
        let config_dir = get_config_dir()?;
        Ok(config_dir.join("logs"))
    }
}
