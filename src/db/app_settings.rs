// 全局配置管理
//
// 提供 app_settings 表的 CRUD 操作

use crate::db::models::AppSetting;
use crate::db::sqlite::get_connection;
use crate::error::AppError;
use rusqlite::OptionalExtension;

/// 获取单个配置项
pub fn get_setting(key: &str) -> Result<Option<String>, AppError> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare("SELECT value FROM app_settings WHERE key = ?1")
        .map_err(|e| AppError::Other(format!("准备查询失败: {}", e)))?;

    let result = stmt
        .query_row(rusqlite::params![key], |row| row.get(0))
        .optional()
        .map_err(|e| AppError::Other(format!("查询配置失败: {}", e)))?;

    Ok(result)
}

/// 设置配置项（存在则更新，不存在则插入）
pub fn set_setting(key: &str, value: &str) -> Result<(), AppError> {
    let conn = get_connection()?;
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value],
    )
    .map_err(|e| AppError::Other(format!("写入配置失败: {}", e)))?;

    Ok(())
}

/// 获取所有配置项
pub fn get_all_settings() -> Result<Vec<AppSetting>, AppError> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM app_settings ORDER BY key")
        .map_err(|e| AppError::Other(format!("准备查询失败: {}", e)))?;

    let settings = stmt
        .query_map([], |row| {
            Ok(AppSetting {
                key: row.get(0)?,
                value: row.get(1)?,
            })
        })
        .map_err(|e| AppError::Other(format!("查询配置失败: {}", e)))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Other(format!("读取配置数据失败: {}", e)))?;

    Ok(settings)
}
