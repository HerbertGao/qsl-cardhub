// SQLite 数据库管理
//
// 提供数据库连接、初始化和迁移功能

use crate::error::AppError;
use include_dir::{include_dir, Dir};
use once_cell::sync::OnceCell;
use rusqlite::Connection;
use std::path::PathBuf;

/// 数据库路径缓存
static DB_PATH: OnceCell<PathBuf> = OnceCell::new();

/// 编译时嵌入的迁移文件目录
static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

/// 迁移脚本信息
struct Migration {
    version: i32,
    name: String,
    sql: String,
}

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

    log::info!("📁 数据库路径: {}", db_path.display());

    // 创建或打开数据库连接
    let conn = Connection::open(&db_path).map_err(|e| {
        AppError::Other(format!("无法打开数据库: {}", e))
    })?;

    // 启用外键支持
    conn.execute("PRAGMA foreign_keys = ON;", []).map_err(|e| {
        AppError::Other(format!("无法启用外键支持: {}", e))
    })?;

    // 执行自动化迁移
    run_migrations(&conn)?;

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
pub fn get_db_version(conn: &Connection) -> Result<i32, AppError> {
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

/// 从文件名解析版本号
///
/// 支持两种格式：
/// - 旧格式：001_xxx.sql, 002_xxx.sql, ... (版本号 < 1000)
/// - 新格式：YYYY.M.D.NNN_xxx.sql (如 2026.1.23.001_init.sql)
///
/// 新格式版本号转换公式：(YYYY - 2020) * 10000000 + MMDD * 1000 + NNN
/// 例如：2026.1.23.001 -> 6 * 10000000 + 123 * 1000 + 1 = 60123001
fn parse_version(filename: &str) -> Option<i32> {
    if !filename.ends_with(".sql") {
        return None;
    }

    let parts: Vec<&str> = filename.split('_').collect();
    if parts.is_empty() {
        return None;
    }

    let version_part = parts[0];

    // 尝试解析新格式：YYYY.M.D.NNN
    if version_part.contains('.') {
        let segments: Vec<&str> = version_part.split('.').collect();
        if segments.len() == 4 {
            let year: i32 = segments[0].parse().ok()?;
            let month: i32 = segments[1].parse().ok()?;
            let day: i32 = segments[2].parse().ok()?;
            let sub: i32 = segments[3].parse().ok()?;

            // 验证范围
            if year < 2020 || year > 2099 {
                return None;
            }
            if month < 1 || month > 12 {
                return None;
            }
            if day < 1 || day > 31 {
                return None;
            }
            if sub < 1 || sub > 999 {
                return None;
            }

            // 转换为整数：(YYYY - 2020) * 10000000 + MMDD * 1000 + NNN
            let mmdd = month * 100 + day;
            let version = (year - 2020) * 10000000 + mmdd * 1000 + sub;
            return Some(version);
        }
        return None;
    }

    // 旧格式：直接解析数字
    version_part.parse::<i32>().ok()
}

/// 从嵌入的目录中解析迁移脚本
fn parse_migrations() -> Result<Vec<Migration>, AppError> {
    let mut migrations = Vec::new();

    for file in MIGRATIONS_DIR.files() {
        let filename = file
            .path()
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| AppError::Other("无效的迁移文件名".to_string()))?;

        // 解析版本号
        if let Some(version) = parse_version(filename) {
            let sql = file
                .contents_utf8()
                .ok_or_else(|| AppError::Other(format!("无法读取迁移文件: {}", filename)))?;

            migrations.push(Migration {
                version,
                name: filename.to_string(),
                sql: sql.to_string(),
            });
        } else {
            // 跳过不符合格式的文件
            log::debug!("跳过非迁移文件: {}", filename);
        }
    }

    // 按版本号排序
    migrations.sort_by_key(|m| m.version);
    Ok(migrations)
}

/// 旧版本号到新版本号的映射
/// 旧版本 1-5 对应新格式 2026.1.23.001-003
const OLD_VERSION_MAPPING: [(i32, i32); 5] = [
    (1, 60123001), // 001_init.sql -> 2026.1.23.001
    (2, 60123002), // 002_add_cards.sql -> 2026.1.23.002
    (3, 60123003), // 003_add_sf_express.sql -> 2026.1.23.003
    (4, 60123003), // 004_add_pay_method.sql -> 已整合到 2026.1.23.003
    (5, 60123003), // 005_add_cargo_name.sql -> 已整合到 2026.1.23.003
];

/// 将整数版本号格式化为可读字符串
///
/// - 新格式 (>= 1000): 60123001 -> "2026.1.23.001"
/// - 旧格式 (< 1000): 5 -> "5"
pub fn format_version(version: i32) -> String {
    if version < 1000 {
        return version.to_string();
    }

    // 逆向解析：version = (year - 2020) * 10000000 + mmdd * 1000 + sub
    let sub = version % 1000;
    let mmdd = (version / 1000) % 10000;
    let year_offset = version / 10000000;

    let year = year_offset + 2020;
    let month = mmdd / 100;
    let day = mmdd % 100;

    format!("{}.{}.{}.{:03}", year, month, day, sub)
}

/// 将旧版本号转换为新版本号
fn migrate_version_number(old_version: i32) -> i32 {
    // 如果已经是新格式（>= 1000），直接返回
    if old_version >= 1000 {
        return old_version;
    }

    // 查找映射
    for (old, new) in OLD_VERSION_MAPPING.iter() {
        if old_version == *old {
            return *new;
        }
    }

    // 未知的旧版本，保持不变
    old_version
}

/// 执行数据库迁移
fn run_migrations(conn: &Connection) -> Result<(), AppError> {
    let migrations = parse_migrations()?;
    let mut current_version = get_db_version(conn)?;

    log::info!("📊 当前数据库版本: {}", format_version(current_version));

    // 如果是旧版本号格式，先迁移版本号
    if current_version > 0 && current_version < 1000 {
        let new_version = migrate_version_number(current_version);
        log::info!("🔄 迁移版本号: {} -> {}", format_version(current_version), format_version(new_version));
        set_db_version(conn, new_version)?;
        current_version = new_version;
    }

    let pending: Vec<_> = migrations
        .iter()
        .filter(|m| m.version > current_version)
        .collect();

    if pending.is_empty() {
        log::info!("📊 数据库已是最新版本");
        return Ok(());
    }

    log::info!("🔄 发现 {} 个待执行迁移", pending.len());

    for migration in pending {
        log::info!("📦 执行迁移: {} (v{})", migration.name, format_version(migration.version));

        conn.execute_batch(&migration.sql).map_err(|e| {
            AppError::Other(format!("迁移 {} 执行失败: {}", migration.name, e))
        })?;

        set_db_version(conn, migration.version)?;
        log::info!("✅ 迁移完成: {}", migration.name);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_db_path() {
        let path = get_db_path().unwrap();
        assert!(path.to_string_lossy().contains("cards.db"));
    }

    #[test]
    fn test_parse_version_old_format() {
        // 旧格式：NNN_xxx.sql
        assert_eq!(parse_version("001_init.sql"), Some(1));
        assert_eq!(parse_version("002_add_cards.sql"), Some(2));
        assert_eq!(parse_version("003_add_sf_express.sql"), Some(3));
        assert_eq!(parse_version("100_future.sql"), Some(100));
    }

    #[test]
    fn test_parse_version_new_format() {
        // 新格式：YYYY.M.D.NNN_xxx.sql
        // 2026.1.23.001 -> (2026-2020) * 10000000 + 123 * 1000 + 1 = 60123001
        assert_eq!(parse_version("2026.1.23.001_init.sql"), Some(60123001));
        // 2026.1.23.002 -> 6 * 10000000 + 123 * 1000 + 2 = 60123002
        assert_eq!(parse_version("2026.1.23.002_add_cards.sql"), Some(60123002));
        // 2026.12.31.999 -> 6 * 10000000 + 1231 * 1000 + 999 = 61231999
        assert_eq!(parse_version("2026.12.31.999_future.sql"), Some(61231999));
        // 2020.1.1.001 -> 0 * 10000000 + 101 * 1000 + 1 = 101001
        assert_eq!(parse_version("2020.1.1.001_old.sql"), Some(101001));
    }

    #[test]
    fn test_parse_version_invalid() {
        // 非 .sql 文件
        assert_eq!(parse_version("readme.txt"), None);
        assert_eq!(parse_version("001_init.txt"), None);
        // 无版本号前缀
        assert_eq!(parse_version("init.sql"), None);
        assert_eq!(parse_version("abc_init.sql"), None);
        // 无效的新格式
        assert_eq!(parse_version("2026.1.23_init.sql"), None); // 缺少子版本号
        assert_eq!(parse_version("2019.1.1.001_old.sql"), None); // 年份过小
        assert_eq!(parse_version("2100.1.1.001_old.sql"), None); // 年份过大
        assert_eq!(parse_version("2026.13.1.001_old.sql"), None); // 月份无效
        assert_eq!(parse_version("2026.1.32.001_old.sql"), None); // 日期无效
    }

    #[test]
    fn test_parse_version_sorting() {
        // 旧格式版本号应小于新格式
        let old_v = parse_version("005_old.sql").unwrap();
        let new_v = parse_version("2026.1.23.001_new.sql").unwrap();
        assert!(old_v < new_v, "旧格式 {} 应小于新格式 {}", old_v, new_v);

        // 新格式按日期排序
        let v1 = parse_version("2026.1.23.001_a.sql").unwrap();
        let v2 = parse_version("2026.1.23.002_b.sql").unwrap();
        let v3 = parse_version("2026.1.24.001_c.sql").unwrap();
        assert!(v1 < v2, "同日版本 {} < {}", v1, v2);
        assert!(v2 < v3, "次日版本 {} < {}", v2, v3);
    }

    #[test]
    fn test_migrate_version_number() {
        // 旧版本号映射
        assert_eq!(migrate_version_number(1), 60123001);
        assert_eq!(migrate_version_number(2), 60123002);
        assert_eq!(migrate_version_number(3), 60123003);
        assert_eq!(migrate_version_number(4), 60123003);
        assert_eq!(migrate_version_number(5), 60123003);
        // 新格式版本号保持不变
        assert_eq!(migrate_version_number(60123001), 60123001);
        // 未知旧版本保持不变
        assert_eq!(migrate_version_number(6), 6);
    }

    #[test]
    fn test_format_version() {
        // 旧格式：直接显示数字
        assert_eq!(format_version(1), "1");
        assert_eq!(format_version(5), "5");
        assert_eq!(format_version(999), "999");

        // 新格式：转换为日期格式
        assert_eq!(format_version(60123001), "2026.1.23.001");
        assert_eq!(format_version(60123002), "2026.1.23.002");
        assert_eq!(format_version(60123003), "2026.1.23.003");
        assert_eq!(format_version(61231999), "2026.12.31.999");
        assert_eq!(format_version(101001), "2020.1.1.001");
    }

    #[test]
    fn test_parse_migrations() {
        let migrations = parse_migrations().unwrap();
        // 应该有迁移文件
        assert!(!migrations.is_empty());
        // 应该按版本号排序
        for i in 1..migrations.len() {
            assert!(migrations[i].version > migrations[i - 1].version);
        }
    }
}
