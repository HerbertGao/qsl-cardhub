use anyhow::{Context, Result};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::credentials::CredentialStorage;

/// 加密存储条目
#[derive(Serialize, Deserialize)]
struct EncryptedEntry {
    /// 随机盐值（用于密钥派生）
    salt: Vec<u8>,
    /// 随机 nonce（用于 AES-GCM）
    nonce: Vec<u8>,
    /// 加密后的值
    ciphertext: Vec<u8>,
}

/// 加密存储数据结构
#[derive(Serialize, Deserialize, Default)]
struct EncryptedStore {
    /// 所有加密条目
    entries: HashMap<String, EncryptedEntry>,
}

/// 加密文件存储实现
pub struct EncryptedFileStorage {
    store_path: PathBuf,
    machine_id: String,
}

impl EncryptedFileStorage {
    /// 创建新的加密文件存储实例
    pub fn new() -> Result<Self> {
        let store_path = Self::get_store_path()?;
        let machine_id = Self::get_machine_id()?;

        Ok(Self {
            store_path,
            machine_id,
        })
    }

    /// 获取存储文件路径
    fn get_store_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("无法获取配置目录")?;

        let store_dir = config_dir.join("qsl-cardhub");
        fs::create_dir_all(&store_dir)
            .context("无法创建存储目录")?;

        Ok(store_dir.join("credentials.enc"))
    }

    /// 获取机器唯一标识符
    fn get_machine_id() -> Result<String> {
        // 使用 hostname 作为机器标识符
        // 在生产环境中，可以考虑使用更稳定的机器 ID
        let hostname = hostname::get()
            .context("无法获取主机名")?
            .to_string_lossy()
            .to_string();

        Ok(hostname)
    }

    /// 派生加密密钥
    fn derive_key(&self, salt: &[u8]) -> [u8; 32] {
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(
            self.machine_id.as_bytes(),
            salt,
            100_000,
            &mut key,
        );
        key
    }

    /// 读取存储文件
    fn read_store(&self) -> Result<EncryptedStore> {
        if !self.store_path.exists() {
            return Ok(EncryptedStore::default());
        }

        let data = fs::read(&self.store_path)
            .context("无法读取存储文件")?;

        let store: EncryptedStore = serde_json::from_slice(&data)
            .context("无法解析存储文件")?;

        Ok(store)
    }

    /// 写入存储文件
    fn write_store(&self, store: &EncryptedStore) -> Result<()> {
        let data = serde_json::to_vec_pretty(store)
            .context("无法序列化存储数据")?;

        fs::write(&self.store_path, data)
            .context("无法写入存储文件")?;

        // 设置文件权限（仅所有者可读写）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&self.store_path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&self.store_path, perms)?;
        }

        Ok(())
    }

    /// 加密值
    fn encrypt(&self, value: &str) -> Result<EncryptedEntry> {
        // 生成随机盐值和 nonce
        let mut salt = vec![0u8; 32];
        let mut nonce_bytes = vec![0u8; 12];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce_bytes);

        // 派生加密密钥
        let key = self.derive_key(&salt);

        // 创建加密器
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| anyhow::anyhow!("无法创建加密器: {}", e))?;

        let nonce = Nonce::from_slice(&nonce_bytes);

        // 加密数据
        let ciphertext = cipher
            .encrypt(nonce, value.as_bytes())
            .map_err(|e| anyhow::anyhow!("加密失败: {}", e))?;

        Ok(EncryptedEntry {
            salt,
            nonce: nonce_bytes,
            ciphertext,
        })
    }

    /// 解密值
    fn decrypt(&self, entry: &EncryptedEntry) -> Result<String> {
        // 派生解密密钥
        let key = self.derive_key(&entry.salt);

        // 创建解密器
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| anyhow::anyhow!("无法创建解密器: {}", e))?;

        let nonce = Nonce::from_slice(&entry.nonce);

        // 解密数据
        let plaintext = cipher
            .decrypt(nonce, entry.ciphertext.as_ref())
            .map_err(|e| anyhow::anyhow!("解密失败: {}", e))?;

        String::from_utf8(plaintext)
            .context("解密后的数据不是有效的 UTF-8")
    }
}

impl CredentialStorage for EncryptedFileStorage {
    fn save(&self, key: &str, value: &str) -> Result<()> {
        let mut store = self.read_store()?;
        let entry = self.encrypt(value)?;
        store.entries.insert(key.to_string(), entry);
        self.write_store(&store)?;
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<String>> {
        let store = self.read_store()?;

        if let Some(entry) = store.entries.get(key) {
            let value = self.decrypt(entry)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn delete(&self, key: &str) -> Result<()> {
        let mut store = self.read_store()?;
        store.entries.remove(key);
        self.write_store(&store)?;
        Ok(())
    }

    fn is_available(&self) -> bool {
        // 检查存储目录是否可访问
        self.store_path.parent()
            .map(|dir| dir.exists() || fs::create_dir_all(dir).is_ok())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypted_storage_operations() -> Result<()> {
        let storage = EncryptedFileStorage::new()?;

        let test_key = "test:encryption";
        let test_value = "secret_password_456";

        // 测试保存
        storage.save(test_key, test_value)?;

        // 测试读取
        let retrieved = storage.get(test_key)?;
        assert_eq!(retrieved, Some(test_value.to_string()));

        // 测试删除
        storage.delete(test_key)?;

        // 验证已删除
        let after_delete = storage.get(test_key)?;
        assert_eq!(after_delete, None);

        Ok(())
    }

    #[test]
    fn test_encryption_roundtrip() -> Result<()> {
        let storage = EncryptedFileStorage::new()?;
        let original = "test_value_123";

        let encrypted = storage.encrypt(original)?;
        let decrypted = storage.decrypt(&encrypted)?;

        assert_eq!(original, decrypted);
        Ok(())
    }
}
