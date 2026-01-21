// SQLite 数据库管理
//
// 提供数据库连接、初始化和迁移功能

use crate::error::AppError;
use once_cell::sync::OnceCell;
use rusqlite::Connection;
use std::path::PathBuf;

/// 数据库路径缓存
static DB_PATH: OnceCell<PathBuf> = OnceCell::new();

/// 当前数据库版本
const CURRENT_DB_VERSION: i32 = 2;

/// 获取数据库文件路径
pub fn get_db_path() -> Result<PathBuf, AppError> {
    // 开发模式：使用项目根目录的 data/
    #[cfg(debug_assertions)]
    {
        let data_dir = PathBuf::from("data");
        std::fs::create_dir_all(&data_dir).map_err(|e| {
            AppError::DirectoryCreationFailed(format!("无法创建数据目录: {}", e))
        })?;
        return Ok(data_dir.join("cards.db"));
    }

    // 生产模式：使用系统配置目录
    #[cfg(not(debug_assertions))]
    {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            AppError::DirectoryCreationFailed("无法获取用户主目录".to_string())
        })?;

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

        std::fs::create_dir_all(&config_dir).map_err(|e| {
            AppError::DirectoryCreationFailed(format!("无法创建配置目录: {}", e))
        })?;

        Ok(config_dir.join("cards.db"))
    }
}

/// 初始化数据库
///
/// 如果数据库文件不存在，创建新数据库并执行初始化脚本。
/// 如果数据库版本低于当前版本，执行迁移脚本。
pub fn init_database() -> Result<(), AppError> {
    let db_path = get_db_path()?;
    let db_exists = db_path.exists();

    log::info!("📁 数据库路径: {}", db_path.display());

    // 创建或打开数据库连接
    let conn = Connection::open(&db_path).map_err(|e| {
        AppError::Other(format!("无法打开数据库: {}", e))
    })?;

    // 启用外键支持
    conn.execute("PRAGMA foreign_keys = ON;", []).map_err(|e| {
        AppError::Other(format!("无法启用外键支持: {}", e))
    })?;

    if !db_exists {
        // 新数据库：执行初始化脚本
        log::info!("🔧 创建新数据库...");
        execute_migration(&conn, 0, CURRENT_DB_VERSION)?;
    } else {
        // 检查版本并执行迁移
        let current_version = get_db_version(&conn)?;
        log::info!("📊 当前数据库版本: {}", current_version);

        if current_version < CURRENT_DB_VERSION {
            log::info!(
                "🔄 执行数据库迁移: v{} -> v{}",
                current_version,
                CURRENT_DB_VERSION
            );
            execute_migration(&conn, current_version, CURRENT_DB_VERSION)?;
        }
    }

    // 缓存数据库路径
    let _ = DB_PATH.set(db_path);

    log::info!("✅ 数据库初始化完成");
    Ok(())
}

/// 获取数据库连接（每次创建新连接）
pub fn get_connection() -> Result<Connection, AppError> {
    let db_path = DB_PATH
        .get()
        .ok_or_else(|| AppError::Other("数据库未初始化".to_string()))?;

    let conn = Connection::open(db_path).map_err(|e| {
        AppError::Other(format!("无法打开数据库: {}", e))
    })?;

    // 启用外键支持
    conn.execute("PRAGMA foreign_keys = ON;", []).map_err(|e| {
        AppError::Other(format!("无法启用外键支持: {}", e))
    })?;

    Ok(conn)
}

/// 获取数据库版本
fn get_db_version(conn: &Connection) -> Result<i32, AppError> {
    let version: i32 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .map_err(|e| AppError::Other(format!("无法获取数据库版本: {}", e)))?;
    Ok(version)
}

/// 设置数据库版本
fn set_db_version(conn: &Connection, version: i32) -> Result<(), AppError> {
    conn.execute(&format!("PRAGMA user_version = {};", version), [])
        .map_err(|e| AppError::Other(format!("无法设置数据库版本: {}", e)))?;
    Ok(())
}

/// 执行数据库迁移
fn execute_migration(conn: &Connection, from_version: i32, to_version: i32) -> Result<(), AppError> {
    for version in (from_version + 1)..=to_version {
        log::info!("📦 执行迁移脚本 v{}...", version);

        let sql = get_migration_sql(version)?;
        conn.execute_batch(&sql).map_err(|e| {
            AppError::Other(format!("迁移脚本 v{} 执行失败: {}", version, e))
        })?;

        set_db_version(conn, version)?;
        log::info!("✅ 迁移脚本 v{} 执行成功", version);
    }
    Ok(())
}

/// 获取迁移脚本内容
fn get_migration_sql(version: i32) -> Result<String, AppError> {
    match version {
        1 => Ok(include_str!("../../migrations/001_init.sql").to_string()),
        2 => Ok(include_str!("../../migrations/002_add_cards.sql").to_string()),
        _ => Err(AppError::Other(format!("未知的迁移版本: {}", version))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_db_path() {
        let path = get_db_path().unwrap();
        assert!(path.to_string_lossy().contains("cards.db"));
    }
}
