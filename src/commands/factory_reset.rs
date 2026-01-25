// 恢复出厂设置命令
//
// 清除所有用户数据并重置应用

use crate::security::clear_all_credentials;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// 清空数据库内容并删除数据库文件
///
/// 此函数先打开数据库连接，删除所有用户表和数据，
/// 然后关闭连接，最后删除数据库文件。
/// 这样可以避免 Windows 上的文件锁问题。
fn clear_and_delete_database(db_path: &Path) -> Result<(), String> {
    // 打开数据库连接
    let conn = Connection::open(db_path)
        .map_err(|e| format!("无法打开数据库: {}", e))?;

    // 查询所有用户表（排除 SQLite 系统表）
    let tables: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
            .map_err(|e| format!("无法查询表列表: {}", e))?;

        let tables: Result<Vec<String>, _> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| format!("无法读取表列表: {}", e))?
            .collect();

        tables.map_err(|e| format!("无法解析表名: {}", e))?
    };

    // 删除所有用户表
    for table in tables {
        conn.execute(&format!("DROP TABLE IF EXISTS {}", table), [])
            .map_err(|e| format!("无法删除表 {}: {}", table, e))?;
        log::debug!("已删除表: {}", table);
    }

    // 重置数据库版本号
    conn.execute("PRAGMA user_version = 0", [])
        .map_err(|e| format!("无法重置数据库版本: {}", e))?;

    // 显式关闭连接
    drop(conn);

    // 现在尝试删除数据库文件
    std::fs::remove_file(db_path)
        .map_err(|e| format!("无法删除数据库文件: {}", e))?;

    Ok(())
}

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
/// - 呼号模板 (templates/callsign.toml)
#[tauri::command]
pub async fn factory_reset() -> Result<(), String> {
    log::info!("开始执行恢复出厂设置...");

    let config_dir = get_config_dir()?;
    let mut errors = Vec::new();

    // 1. 清空并删除数据库文件
    #[cfg(debug_assertions)]
    let db_path = PathBuf::from("data").join("cards.db");
    #[cfg(not(debug_assertions))]
    let db_path = config_dir.join("cards.db");

    if db_path.exists() {
        // 先清空数据库内容，然后关闭连接再删除文件
        // 这样可以避免 Windows 上的文件锁问题
        if let Err(e) = clear_and_delete_database(&db_path) {
            log::error!("清空数据库失败: {}", e);
            errors.push(format!("清空数据库失败: {}", e));
        } else {
            log::info!("已清空并删除数据库文件: {}", db_path.display());
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
