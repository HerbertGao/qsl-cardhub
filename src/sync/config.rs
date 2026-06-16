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
    ///
    /// 注意：`client_id` 是**设备身份**（乐观并发护栏 / `sync_meta.last_client_id` 用），
    /// 与 `tenant`（申报归属）正交，**不是**租户/数据归属的依据。
    pub client_id: String,
    /// 上次同步时间（ISO 8601 格式）
    pub last_sync_at: Option<String>,
    /// 本地持久化的云端基线版本（乐观并发护栏 base_version）
    ///
    /// `None` 表示从未与新协议同步过，首次同步走无条件覆盖路径；
    /// 同步成功（200）后必须刷新为响应回传的 `server_version` 并落盘。
    #[serde(default)]
    pub base_version: Option<i64>,
    /// 申报的所属租户代码（slug）
    ///
    /// 仅用于在云端请求头 `X-Tenant-Id` 中**申报**租户身份，供服务端交叉校验
    /// （见 `crossCheckTenant`）。**红线**：租户归属的真源永远是写凭据
    /// （`key→tenant`），此字段**绝不**作为写入/读取的归属目标——只申报、不决定归属。
    /// `None`/空时不发送 `X-Tenant-Id`，行为与未引入本字段前逐字一致（软约束）。
    /// `#[serde(default)]` 兼容旧 `sync.toml`（无此字段时回退 `None`）。
    #[serde(default)]
    pub tenant: Option<String>,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            api_url: String::new(),
            client_id: Uuid::new_v4().to_string(),
            last_sync_at: None,
            base_version: None,
            tenant: None,
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
        assert!(config.base_version.is_none());
    }

    #[test]
    fn test_config_without_base_version_field_parses() {
        // 旧 sync.toml 无 base_version 字段时，#[serde(default)] 应回退为 None
        let toml_str = r#"
api_url = "https://example.com"
client_id = "test-client-id"
"#;
        let config: SyncConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.api_url, "https://example.com");
        assert_eq!(config.client_id, "test-client-id");
        assert!(config.base_version.is_none());
    }

    #[test]
    fn test_config_without_tenant_field_parses() {
        // 旧 sync.toml 无 tenant 字段时，#[serde(default)] 应回退为 None（向后兼容核心断言）
        let toml_str = r#"
api_url = "https://example.com"
client_id = "test-client-id"
base_version = 7
"#;
        let config: SyncConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.api_url, "https://example.com");
        assert_eq!(config.base_version, Some(7));
        assert!(config.tenant.is_none());
    }
}
