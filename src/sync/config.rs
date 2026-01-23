// 同步配置模块
//
// 管理云端同步的配置信息

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// 凭据存储键名
pub mod credential_keys {
    /// 云端同步 API Key
    pub const SYNC_API_KEY: &str = "qsl-cardhub:sync:api_key";
}

/// 同步配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// API 地址
    pub api_url: String,
    /// 客户端标识（UUID 格式）
    pub client_id: String,
    /// 上次同步时间（ISO 8601 格式）
    pub last_sync_at: Option<String>,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            api_url: String::new(),
            client_id: Uuid::new_v4().to_string(),
            last_sync_at: None,
        }
    }
}

/// 获取同步配置文件路径
fn get_sync_config_path() -> Result<PathBuf, String> {
    #[cfg(debug_assertions)]
    {
        let config_dir = PathBuf::from("config");
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("无法创建配置目录: {}", e))?;
        return Ok(config_dir.join("sync.toml"));
    }

    #[cfg(not(debug_assertions))]
    {
        let home_dir = dirs::home_dir().ok_or("无法获取用户主目录")?;

        let config_dir = if cfg!(target_os = "windows") {
            home_dir.join("AppData").join("Roaming").join("qsl-cardhub")
        } else if cfg!(target_os = "macos") {
            home_dir
                .join("Library")
                .join("Application Support")
                .join("qsl-cardhub")
        } else {
            home_dir.join(".config").join("qsl-cardhub")
        };

        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("无法创建配置目录: {}", e))?;

        Ok(config_dir.join("sync.toml"))
    }
}

/// 保存同步配置
pub fn save_sync_config(config: &SyncConfig) -> Result<(), String> {
    let path = get_sync_config_path()?;
    let content = toml::to_string_pretty(config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&path, content)
        .map_err(|e| format!("保存配置文件失败: {}", e))?;
    log::info!("✅ 同步配置已保存到: {}", path.display());
    Ok(())
}

/// 加载同步配置
pub fn load_sync_config() -> Result<Option<SyncConfig>, String> {
    let path = get_sync_config_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
    let config: SyncConfig = toml::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;
    Ok(Some(config))
}

/// 清除同步配置
pub fn clear_sync_config() -> Result<(), String> {
    let path = get_sync_config_path()?;
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("删除配置文件失败: {}", e))?;
        log::info!("✅ 同步配置已清除");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SyncConfig::default();
        assert!(config.api_url.is_empty());
        assert!(!config.client_id.is_empty());
        assert!(config.last_sync_at.is_none());
    }
}
