// qsl-cardhub
//
// 业余无线电 QSL 卡片打印工具

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod db;
mod error;
mod logger;
mod printer;
mod qrz;
mod security;
mod sf_express;
mod sync;
mod utils;

use commands::{
    cards::{
        create_card_cmd, delete_card_cmd, distribute_card_cmd, get_card_cmd, list_cards_cmd,
        return_card_cmd, save_card_address_cmd,
    },
    data_transfer::{export_data, import_data, preview_import_data},
    logger::{clear_logs, export_logs, get_log_file_path, get_logs, log_from_frontend},
    platform::get_platform_info,
    printer::{PrinterState, generate_tspl, get_printers, get_template_config, load_template, preview_qsl, print_qsl, save_template, save_template_config},
    profile::{
        ProfileState, create_profile, delete_profile, export_profile, get_default_profile_id,
        get_default_template_name, get_profile, get_profiles, import_profile, set_default_profile,
        update_profile,
    },
    projects::{
        create_project_cmd, delete_project_cmd, get_project_cmd, list_projects_cmd,
        update_project_cmd,
    },
    qrz_cn::{
        qrz_check_login_status, qrz_clear_credentials, qrz_load_credentials, qrz_query_callsign,
        qrz_save_and_login, qrz_test_connection,
    },
    qrz_com::{
        qrz_com_check_login_status, qrz_com_clear_credentials, qrz_com_load_credentials,
        qrz_com_query_callsign, qrz_com_save_and_login, qrz_com_test_connection,
    },
    qrz_herbertgao::qrz_herbertgao_query_callsign,
    security::{
        check_keyring_available, clear_credentials, load_credentials, save_credentials,
    },
    sf_express::{
        sf_clear_config, sf_fetch_waybill, sf_load_config, sf_print_waybill, sf_save_config,
        // 寄件人管理
        sf_create_sender, sf_update_sender, sf_delete_sender, sf_list_senders,
        sf_get_default_sender, sf_set_default_sender,
        // 下单管理
        sf_create_order, sf_confirm_order, sf_cancel_order, sf_search_order,
        // 订单列表
        sf_list_orders, sf_get_order, sf_get_order_by_order_id, sf_get_order_by_card_id,
        sf_delete_order, sf_mark_order_printed,
    },
    sync::{
        clear_sync_config_cmd, execute_sync_cmd, load_sync_config_cmd, save_sync_config_cmd,
        test_sync_connection_cmd,
    },
};
use config::ProfileManager;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // 获取配置目录
            let config_dir = get_config_dir()?;
            let output_dir = get_output_dir()?;
            let log_dir = get_log_dir()?;

            println!("📁 配置目录: {}", config_dir.display());
            println!("📁 输出目录: {}", output_dir.display());
            println!("📁 日志目录: {}", log_dir.display());

            // 生产模式：初始化用户配置目录，从应用资源复制默认配置
            #[cfg(not(debug_assertions))]
            {
                init_user_config(app, &config_dir)?;
            }

            // 初始化日志系统
            logger::init_logger(log_dir).map_err(|e| format!("无法初始化日志系统: {}", e))?;

            // 初始化数据库
            db::init_database().map_err(|e| format!("无法初始化数据库: {}", e))?;

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

            println!("✅ qsl-cardhub 初始化完成");

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
            get_default_template_name,
            export_profile,
            import_profile,
            // 打印机管理
            get_printers,
            preview_qsl,
            print_qsl,
            generate_tspl,
            load_template,
            save_template,
            get_template_config,
            save_template_config,
            // 日志管理
            get_logs,
            clear_logs,
            export_logs,
            get_log_file_path,
            log_from_frontend,
            // 项目管理
            create_project_cmd,
            list_projects_cmd,
            get_project_cmd,
            update_project_cmd,
            delete_project_cmd,
            // 卡片管理
            create_card_cmd,
            list_cards_cmd,
            get_card_cmd,
            distribute_card_cmd,
            return_card_cmd,
            delete_card_cmd,
            save_card_address_cmd,
            // 安全凭据管理
            save_credentials,
            load_credentials,
            clear_credentials,
            check_keyring_available,
            // QRZ.cn 集成
            qrz_save_and_login,
            qrz_load_credentials,
            qrz_clear_credentials,
            qrz_query_callsign,
            qrz_check_login_status,
            qrz_test_connection,
            // QRZ.com 集成
            qrz_com_save_and_login,
            qrz_com_load_credentials,
            qrz_com_clear_credentials,
            qrz_com_query_callsign,
            qrz_com_check_login_status,
            qrz_com_test_connection,
            // QRZ.herbertgao.me 集成
            qrz_herbertgao_query_callsign,
            // 顺丰速运集成
            sf_save_config,
            sf_load_config,
            sf_clear_config,
            sf_fetch_waybill,
            sf_print_waybill,
            // 顺丰寄件人管理
            sf_create_sender,
            sf_update_sender,
            sf_delete_sender,
            sf_list_senders,
            sf_get_default_sender,
            sf_set_default_sender,
            // 顺丰下单管理
            sf_create_order,
            sf_confirm_order,
            sf_cancel_order,
            sf_search_order,
            // 顺丰订单列表
            sf_list_orders,
            sf_get_order,
            sf_get_order_by_order_id,
            sf_get_order_by_card_id,
            sf_delete_order,
            sf_mark_order_printed,
            // 数据导出导入
            export_data,
            preview_import_data,
            import_data,
            // 云端同步
            save_sync_config_cmd,
            load_sync_config_cmd,
            clear_sync_config_cmd,
            test_sync_connection_cmd,
            execute_sync_cmd,
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
            // Windows: %APPDATA%/qsl-cardhub
            home_dir.join("AppData").join("Roaming").join("qsl-cardhub")
        } else if cfg!(target_os = "macos") {
            // macOS: ~/Library/Application Support/qsl-cardhub
            home_dir
                .join("Library")
                .join("Application Support")
                .join("qsl-cardhub")
        } else {
            // Linux: ~/.config/qsl-cardhub
            home_dir.join(".config").join("qsl-cardhub")
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

/// 初始化用户配置目录（仅在生产模式）
/// 从应用资源目录复制默认配置到用户配置目录
#[cfg(not(debug_assertions))]
fn init_user_config(app: &tauri::App, config_dir: &PathBuf) -> Result<(), String> {
    use std::io::Write;

    // 创建配置目录结构
    let templates_dir = config_dir.join("templates");
    fs::create_dir_all(&templates_dir)
        .map_err(|e| format!("无法创建模板目录: {}", e))?;

    // 获取应用资源目录路径
    let resource_path = app
        .path()
        .resource_dir()
        .map_err(|e| format!("无法获取资源目录: {}", e))?;

    // 复制默认模板文件（如果不存在）
    let default_template_src = resource_path.join("config/templates/default.toml");
    let default_template_dst = templates_dir.join("default.toml");

    if !default_template_dst.exists() {
        if default_template_src.exists() {
            fs::copy(&default_template_src, &default_template_dst)
                .map_err(|e| format!("无法复制默认模板: {}", e))?;
            println!("✅ 已复制默认模板到: {}", default_template_dst.display());
        } else {
            // 如果资源文件也不存在，创建一个基本的默认模板
            println!("⚠️  资源目录中未找到默认模板，创建基础模板");
            let basic_template = r#"[metadata]
template_version = "2.0"
name = "76mm × 130mm 标准模板"
description = "标准 QSL 卡片模板"

[page]
dpi = 203
width_mm = 76.0
height_mm = 130.0
margin_left_mm = 4.0
margin_right_mm = 4.0
margin_top_mm = 4.0
margin_bottom_mm = 4.0
border = true
border_thickness_mm = 0.3

[layout]
align_h = "center"
align_v = "top"
gap_mm = 5.0
line_gap_mm = 5.0

[fonts]
cn_bold = "SourceHanSansSC-Bold.otf"
en_bold = "LiberationSans-Bold.ttf"
fallback_bold = "SourceHanSansSC-Bold.otf"

[[elements]]
id = "title"
type = "text"
source = "fixed"
value = "中国无线电协会业余分会-2区卡片局"
max_height_mm = 10.0

[[elements]]
id = "callsign"
type = "text"
source = "input"
key = "callsign"
max_height_mm = 28.0

[output]
mode = "text_bitmap_plus_native_barcode"
threshold = 160
"#;
            let mut file = fs::File::create(&default_template_dst)
                .map_err(|e| format!("无法创建默认模板文件: {}", e))?;
            file.write_all(basic_template.as_bytes())
                .map_err(|e| format!("无法写入默认模板文件: {}", e))?;
            println!("✅ 已创建基础默认模板: {}", default_template_dst.display());
        }
    }

    // config.toml 由 ProfileManager 自动创建，不需要预先复制
    println!("📝 config.toml 将由 ProfileManager 自动创建");

    Ok(())
}
