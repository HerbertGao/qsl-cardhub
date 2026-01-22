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
        // 优先尝试使用系统钥匙串
        let keyring = KeyringStorage::new();
        if keyring.is_available() {
            log::info!("使用系统钥匙串存储凭据");
            return Arc::new(Mutex::new(StorageStrategy::Keyring(keyring)));
        }

        // 降级使用加密文件
        log::warn!("系统钥匙串不可用，使用本地加密文件存储");
        let encrypted = EncryptedFileStorage::new().expect("无法初始化加密文件存储");
        Arc::new(Mutex::new(StorageStrategy::EncryptedFile(encrypted)))
    }).clone()
}

/// 保存凭据的便捷函数
pub fn save_credential(key: &str, value: &str) -> Result<()> {
    let storage = get_credential_storage();
    let storage = storage.lock().unwrap();
    storage.as_storage().save(key, value)
}

/// 获取凭据的便捷函数
pub fn get_credential(key: &str) -> Result<Option<String>> {
    let storage = get_credential_storage();
    let storage = storage.lock().unwrap();
    storage.as_storage().get(key)
}

/// 删除凭据的便捷函数
pub fn delete_credential(key: &str) -> Result<()> {
    let storage = get_credential_storage();
    let storage = storage.lock().unwrap();
    storage.as_storage().delete(key)
}

/// 检查钥匙串是否可用
pub fn is_keyring_available() -> bool {
    let storage = get_credential_storage();
    let storage = storage.lock().unwrap();
    matches!(*storage, StorageStrategy::Keyring(_))
}
