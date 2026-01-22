use anyhow::Result;
use keyring::Entry;
use super::credentials::CredentialStorage;

/// 系统钥匙串存储实现
pub struct KeyringStorage {
    service_name: String,
}

impl KeyringStorage {
    /// 创建新的钥匙串存储实例
    pub fn new() -> Self {
        Self {
            service_name: "qsl-cardhub".to_string(),
        }
    }

    /// 获取钥匙串条目
    fn get_entry(&self, key: &str) -> Result<Entry> {
        Entry::new(&self.service_name, key)
            .map_err(|e| anyhow::anyhow!("无法创建钥匙串条目: {}", e))
    }
}

impl Default for KeyringStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialStorage for KeyringStorage {
    fn save(&self, key: &str, value: &str) -> Result<()> {
        let entry = self.get_entry(key)?;
        entry.set_password(value)
            .map_err(|e| anyhow::anyhow!("无法保存到钥匙串: {}", e))
    }

    fn get(&self, key: &str) -> Result<Option<String>> {
        let entry = self.get_entry(key)?;
        match entry.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("无法从钥匙串读取: {}", e)),
        }
    }

    fn delete(&self, key: &str) -> Result<()> {
        let entry = self.get_entry(key)?;
        match entry.delete_credential() {
            Ok(_) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()), // 已经不存在，视为成功
            Err(e) => Err(anyhow::anyhow!("无法从钥匙串删除: {}", e)),
        }
    }

    fn is_available(&self) -> bool {
        // 测试钥匙串是否可用
        let test_key = "__qsl_cardhub_availability_test__";
        let test_value = "test";

        // 尝试保存、读取、删除测试条目
        if let Ok(entry) = self.get_entry(test_key) {
            let can_save = entry.set_password(test_value).is_ok();
            let can_read = entry.get_password().is_ok();
            let _ = entry.delete_credential(); // 清理测试条目

            can_save && can_read
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyring_operations() {
        let storage = KeyringStorage::new();

        // 如果钥匙串不可用，跳过测试
        if !storage.is_available() {
            println!("系统钥匙串不可用，跳过测试");
            return;
        }

        let test_key = "test:keyring_storage";
        let test_value = "test_password_123";

        // 测试保存
        assert!(storage.save(test_key, test_value).is_ok());

        // 测试读取
        let retrieved = storage.get(test_key).unwrap();
        assert_eq!(retrieved, Some(test_value.to_string()));

        // 测试删除
        assert!(storage.delete(test_key).is_ok());

        // 验证已删除
        let after_delete = storage.get(test_key).unwrap();
        assert_eq!(after_delete, None);
    }
}
