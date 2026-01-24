use anyhow::Result;
use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex};

use super::keyring_storage::KeyringStorage;
use super::encryption::EncryptedFileStorage;

/// 凭据存储 trait
pub trait CredentialStorage: Send + Sync {
    /// 保存凭据
    fn save(&self, key: &str, value: &str) -> Result<()>;

    /// 获取凭据
    fn get(&self, key: &str) -> Result<Option<String>>;

    /// 删除凭据
    fn delete(&self, key: &str) -> Result<()>;

    /// 检查是否可用
    fn is_available(&self) -> bool;
}

/// 凭据存储策略
enum StorageStrategy {
    Keyring(KeyringStorage),
    EncryptedFile(EncryptedFileStorage),
}

impl StorageStrategy {
    fn as_storage(&self) -> &dyn CredentialStorage {
        match self {
            StorageStrategy::Keyring(k) => k,
            StorageStrategy::EncryptedFile(e) => e,
        }
    }
}

/// 全局凭据存储实例
static CREDENTIAL_STORAGE: OnceCell<Arc<Mutex<StorageStrategy>>> = OnceCell::new();

/// 获取凭据存储实例
pub fn get_credential_storage() -> Arc<Mutex<StorageStrategy>> {
    CREDENTIAL_STORAGE.get_or_init(|| {
        // 开发模式下直接使用加密文件存储
        // 因为未签名的 macOS 应用无法正确使用钥匙串
        #[cfg(debug_assertions)]
        {
            log::info!("开发模式：使用本地加密文件存储凭据");
            let encrypted = EncryptedFileStorage::new().expect("无法初始化加密文件存储");
            return Arc::new(Mutex::new(StorageStrategy::EncryptedFile(encrypted)));
        }

        // 生产模式：优先尝试使用系统钥匙串
        #[cfg(not(debug_assertions))]
        {
            let keyring = KeyringStorage::new();
            log::info!("检查系统钥匙串可用性...");
            if keyring.is_available() {
                log::info!("使用系统钥匙串存储凭据");
                return Arc::new(Mutex::new(StorageStrategy::Keyring(keyring)));
            }

            // 降级使用加密文件
            log::warn!("系统钥匙串不可用，使用本地加密文件存储");
            let encrypted = EncryptedFileStorage::new().expect("无法初始化加密文件存储");
            Arc::new(Mutex::new(StorageStrategy::EncryptedFile(encrypted)))
        }
    }).clone()
}

/// 保存凭据的便捷函数
pub fn save_credential(key: &str, value: &str) -> Result<()> {
    log::info!("[凭据] 保存: key={}", key);
    let storage = get_credential_storage();
    let storage = storage.lock().unwrap();
    let result = storage.as_storage().save(key, value);
    match &result {
        Ok(_) => log::info!("[凭据] 保存成功: key={}", key),
        Err(e) => log::error!("[凭据] 保存失败: key={}, error={}", key, e),
    }
    result
}

/// 获取凭据的便捷函数
pub fn get_credential(key: &str) -> Result<Option<String>> {
    log::info!("[凭据] 获取: key={}", key);
    let storage = get_credential_storage();
    let storage = storage.lock().unwrap();
    let result = storage.as_storage().get(key);
    match &result {
        Ok(Some(v)) => log::info!("[凭据] 获取成功: key={}, value_len={}", key, v.len()),
        Ok(None) => log::info!("[凭据] 不存在: key={}", key),
        Err(e) => log::error!("[凭据] 获取失败: key={}, error={}", key, e),
    }
    result
}

/// 删除凭据的便捷函数
pub fn delete_credential(key: &str) -> Result<()> {
    let storage = get_credential_storage();
    let storage = storage.lock().unwrap();
    storage.as_storage().delete(key)
}

/// 检查钥匙串是否可用
pub fn is_keyring_available() -> bool {
    // 开发模式下总是返回 false（使用加密文件）
    #[cfg(debug_assertions)]
    {
        return false;
    }

    #[cfg(not(debug_assertions))]
    {
        let storage = get_credential_storage();
        let storage = storage.lock().unwrap();
        matches!(*storage, StorageStrategy::Keyring(_))
    }
}

/// 所有已知的凭据键
pub const ALL_CREDENTIAL_KEYS: &[&str] = &[
    // QRZ.cn
    "qsl-cardhub:qrz:username",
    "qsl-cardhub:qrz:password",
    "qsl-cardhub:qrz:session",
    // QRZ.com
    "qsl-cardhub:qrz.com:username",
    "qsl-cardhub:qrz.com:password",
    "qsl-cardhub:qrz.com:session",
    // 顺丰速运
    "qsl-cardhub:sf:partner_id",
    "qsl-cardhub:sf:checkword_prod",
    "qsl-cardhub:sf:checkword_sandbox",
    "qsl-cardhub:sf:environment",
    // 云同步
    "qsl-cardhub:sync:api_key",
];

/// 清除所有已知的凭据
/// 用于恢复出厂设置功能
pub fn clear_all_credentials() -> Result<()> {
    log::info!("[凭据] 开始清除所有凭据...");
    let mut errors = Vec::new();

    for key in ALL_CREDENTIAL_KEYS {
        if let Err(e) = delete_credential(key) {
            log::warn!("[凭据] 删除失败: key={}, error={}", key, e);
            errors.push(format!("{}: {}", key, e));
        } else {
            log::info!("[凭据] 已删除: key={}", key);
        }
    }

    if errors.is_empty() {
        log::info!("[凭据] 所有凭据已清除");
        Ok(())
    } else {
        // 即使有部分失败，也返回成功，因为我们希望继续重置流程
        log::warn!("[凭据] 部分凭据清除失败: {:?}", errors);
        Ok(())
    }
}
