// 恢复出厂设置命令
//
// 清除所有用户数据并重置应用

use crate::security::clear_all_credentials;
use std::path::PathBuf;

/// 获取配置目录路径
fn get_config_dir() -> Result<PathBuf, String> {
    // 开发模式：使用项目根目录的 config/
    #[cfg(debug_assertions)]
    {
        return Ok(PathBuf::from("config"));
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

/// 恢复出厂设置
///
/// 清除以下数据：
/// - 数据库文件 (cards.db)
/// - 配置文件 (config.toml, template_config.toml, printer.toml)
/// - 所有钥匙串凭据
///
/// 保留：
/// - 默认模板 (templates/default.toml)
#[tauri::command]
pub async fn factory_reset() -> Result<(), String> {
    log::info!("开始执行恢复出厂设置...");

    let config_dir = get_config_dir()?;
    let mut errors = Vec::new();

    // 1. 删除数据库文件
    #[cfg(debug_assertions)]
    let db_path = PathBuf::from("data").join("cards.db");
    #[cfg(not(debug_assertions))]
    let db_path = config_dir.join("cards.db");

    if db_path.exists() {
        if let Err(e) = std::fs::remove_file(&db_path) {
            log::error!("删除数据库文件失败: {}", e);
            errors.push(format!("删除数据库失败: {}", e));
        } else {
            log::info!("已删除数据库文件: {}", db_path.display());
        }
    }

    // 2. 删除配置文件
    let config_files = ["config.toml", "template_config.toml", "printer.toml"];
    for file in &config_files {
        let file_path = config_dir.join(file);
        if file_path.exists() {
            if let Err(e) = std::fs::remove_file(&file_path) {
                log::error!("删除配置文件失败: {} - {}", file, e);
                errors.push(format!("删除 {} 失败: {}", file, e));
            } else {
                log::info!("已删除配置文件: {}", file_path.display());
            }
        }
    }

    // 3. 清除所有钥匙串凭据
    if let Err(e) = clear_all_credentials() {
        log::error!("清除凭据失败: {}", e);
        errors.push(format!("清除凭据失败: {}", e));
    } else {
        log::info!("已清除所有凭据");
    }

    // 即使有部分错误，也返回成功，让前端可以重启应用
    if !errors.is_empty() {
        log::warn!("恢复出厂设置完成，但有部分错误: {:?}", errors);
    } else {
        log::info!("恢复出厂设置完成");
    }

    Ok(())
}
