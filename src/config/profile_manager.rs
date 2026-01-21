// Profile 管理器
//
// 负责 Profile 的 CRUD 操作和持久化

use super::models::{AppConfig, Platform, Profile};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Profile 管理器
pub struct ProfileManager {
    /// 配置根目录
    config_dir: PathBuf,
    /// Profile 存储目录
    profiles_dir: PathBuf,
    /// 应用全局配置
    app_config: AppConfig,
}

impl ProfileManager {
    /// 创建新的 ProfileManager
    ///
    /// # 错误
    /// - 配置目录不存在且无法创建
    /// - 配置文件读取失败
    pub fn new(config_dir: PathBuf) -> Result<Self> {
        // 确保配置目录存在
        fs::create_dir_all(&config_dir).context("无法创建配置目录")?;

        let profiles_dir = config_dir.join("profiles");
        fs::create_dir_all(&profiles_dir).context("无法创建 profiles 目录")?;

        // 读取应用配置
        let config_path = config_dir.join("config.toml");
        let app_config = if config_path.exists() {
            let content = fs::read_to_string(&config_path).context("无法读取 config.toml")?;
            toml::from_str(&content).context("无法解析 config.toml")?
        } else {
            AppConfig::default()
        };

        Ok(Self {
            config_dir,
            profiles_dir,
            app_config,
        })
    }

    /// 获取所有 Profile
    ///
    /// 注意: 会自动过滤以 `.` 开头的隐藏配置文件（如 `.example.toml`）
    pub fn get_all(&self) -> Result<Vec<Profile>> {
        let mut profiles = Vec::new();

        // 扫描 profiles 目录
        if let Ok(entries) = fs::read_dir(&self.profiles_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                // 跳过以 `.` 开头的隐藏文件
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.starts_with('.') {
                        continue;
                    }
                }

                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    if let Ok(profile) = self.load_profile(&path) {
                        profiles.push(profile);
                    }
                }
            }
        }

        // 按创建时间排序
        profiles.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(profiles)
    }

    /// 根据 ID 获取 Profile
    pub fn get_by_id(&self, id: &str) -> Result<Option<Profile>> {
        let path = self.profiles_dir.join(format!("{}.toml", id));
        if path.exists() {
            Ok(Some(self.load_profile(&path)?))
        } else {
            Ok(None)
        }
    }

    /// 创建新的 Profile
    pub fn create(
        &mut self,
        name: String,
        printer_name: String,
        platform: Platform,
    ) -> Result<Profile> {
        let profile = Profile::new(name, printer_name, platform);
        self.save_profile(&profile)?;
        Ok(profile)
    }

    /// 更新 Profile
    pub fn update(&mut self, id: &str, mut profile: Profile) -> Result<()> {
        // 确保 ID 一致
        profile.id = id.to_string();
        profile.touch();
        self.save_profile(&profile)?;
        Ok(())
    }

    /// 删除 Profile
    pub fn delete(&mut self, id: &str) -> Result<()> {
        let path = self.profiles_dir.join(format!("{}.toml", id));
        if path.exists() {
            fs::remove_file(&path).context("无法删除 Profile 文件")?;
        }

        // 如果删除的是默认 Profile，清除默认设置
        if self.app_config.default_profile_id.as_deref() == Some(id) {
            self.app_config.default_profile_id = None;
            self.save_app_config()?;
        }

        Ok(())
    }

    /// 设置默认 Profile
    pub fn set_default(&mut self, id: &str) -> Result<()> {
        // 验证 Profile 存在
        if self.get_by_id(id)?.is_none() {
            anyhow::bail!("Profile 不存在: {}", id);
        }

        self.app_config.default_profile_id = Some(id.to_string());
        self.save_app_config()?;
        Ok(())
    }

    /// 获取默认 Profile ID
    pub fn get_default_id(&self) -> Option<&str> {
        self.app_config.default_profile_id.as_deref()
    }

    /// 导出 Profile 为 JSON
    pub fn export_profile(&self, id: &str) -> Result<String> {
        let profile = self.get_by_id(id)?.context("Profile 不存在")?;
        serde_json::to_string_pretty(&profile).context("无法序列化 Profile")
    }

    /// 从 JSON 导入 Profile
    pub fn import_profile(&mut self, json: &str) -> Result<Profile> {
        let mut profile: Profile = serde_json::from_str(json).context("无法解析 JSON")?;

        // 生成新的 ID 避免冲突
        profile.id = uuid::Uuid::new_v4().to_string();
        profile.touch();

        self.save_profile(&profile)?;
        Ok(profile)
    }

    // ========== 私有方法 ==========

    /// 从文件加载 Profile
    fn load_profile(&self, path: &PathBuf) -> Result<Profile> {
        let content = fs::read_to_string(path).context("无法读取 Profile 文件")?;
        toml::from_str(&content).context("无法解析 Profile 文件")
    }

    /// 保存 Profile 到文件
    fn save_profile(&self, profile: &Profile) -> Result<()> {
        let path = self.profiles_dir.join(format!("{}.toml", profile.id));
        let content = toml::to_string_pretty(profile).context("无法序列化 Profile")?;

        // 添加注释
        let content_with_comment = format!(
            "# qsl-cardhub 打印配置\n# 配置名称: {}\n# 创建时间: {}\n\n{}",
            profile.name,
            profile.created_at.format("%Y-%m-%d %H:%M:%S"),
            content
        );

        fs::write(&path, content_with_comment).context("无法写入 Profile 文件")?;
        Ok(())
    }

    /// 保存应用配置
    fn save_app_config(&self) -> Result<()> {
        let path = self.config_dir.join("config.toml");
        let content = toml::to_string_pretty(&self.app_config).context("无法序列化应用配置")?;

        let content_with_comment = format!("# qsl-cardhub 全局配置\n\n{}", content);

        fs::write(&path, content_with_comment).context("无法写入应用配置文件")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_get_all_filters_hidden_files() {
        // 创建临时目录
        let temp_dir = TempDir::new().unwrap();
        let profiles_dir = temp_dir.path().join("profiles");
        fs::create_dir_all(&profiles_dir).unwrap();

        // 创建一个正常配置文件
        let normal_profile = Profile::new(
            "Normal Profile".to_string(),
            "Test Printer".to_string(),
            Platform {
                os: "macOS".to_string(),
                arch: "arm64".to_string(),
            },
        );
        let normal_path = profiles_dir.join(format!("{}.toml", normal_profile.id));
        let content = toml::to_string_pretty(&normal_profile).unwrap();
        fs::write(&normal_path, content).unwrap();

        // 创建一个隐藏配置文件（以 `.` 开头）
        let hidden_profile = Profile::new(
            "Hidden Profile".to_string(),
            "Test Printer".to_string(),
            Platform {
                os: "macOS".to_string(),
                arch: "arm64".to_string(),
            },
        );
        let hidden_path = profiles_dir.join(".example.toml");
        let content = toml::to_string_pretty(&hidden_profile).unwrap();
        fs::write(&hidden_path, content).unwrap();

        // 创建 ProfileManager
        let manager = ProfileManager::new(temp_dir.path().to_path_buf()).unwrap();

        // 调用 get_all()
        let profiles = manager.get_all().unwrap();

        // 验证：应该只返回正常配置文件，不包含隐藏文件
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "Normal Profile");
    }

    #[test]
    fn test_get_all_with_only_hidden_files() {
        // 创建临时目录
        let temp_dir = TempDir::new().unwrap();
        let profiles_dir = temp_dir.path().join("profiles");
        fs::create_dir_all(&profiles_dir).unwrap();

        // 只创建隐藏配置文件
        let hidden_profile = Profile::new(
            "Hidden Profile".to_string(),
            "Test Printer".to_string(),
            Platform {
                os: "macOS".to_string(),
                arch: "arm64".to_string(),
            },
        );
        let hidden_path = profiles_dir.join(".example.toml");
        let content = toml::to_string_pretty(&hidden_profile).unwrap();
        fs::write(&hidden_path, content).unwrap();

        // 创建 ProfileManager
        let manager = ProfileManager::new(temp_dir.path().to_path_buf()).unwrap();

        // 调用 get_all()
        let profiles = manager.get_all().unwrap();

        // 验证：应该返回空列表
        assert_eq!(profiles.len(), 0);
    }

    #[test]
    fn test_get_all_with_empty_directory() {
        // 创建临时目录
        let temp_dir = TempDir::new().unwrap();

        // 创建 ProfileManager
        let manager = ProfileManager::new(temp_dir.path().to_path_buf()).unwrap();

        // 调用 get_all()
        let profiles = manager.get_all().unwrap();

        // 验证：应该返回空列表
        assert_eq!(profiles.len(), 0);
    }

    #[test]
    fn test_get_all_with_mixed_files() {
        // 创建临时目录
        let temp_dir = TempDir::new().unwrap();
        let profiles_dir = temp_dir.path().join("profiles");
        fs::create_dir_all(&profiles_dir).unwrap();

        // 创建2个正常配置文件
        for i in 1..=2 {
            let profile = Profile::new(
                format!("Profile {}", i),
                "Test Printer".to_string(),
                Platform {
                os: "macOS".to_string(),
                arch: "arm64".to_string(),
            },
            );
            let path = profiles_dir.join(format!("{}.toml", profile.id));
            let content = toml::to_string_pretty(&profile).unwrap();
            fs::write(&path, content).unwrap();
        }

        // 创建2个隐藏配置文件
        let hidden_files = vec![".example.toml", ".backup.toml"];
        for file_name in hidden_files {
            let profile = Profile::new(
                "Hidden Profile".to_string(),
                "Test Printer".to_string(),
                Platform {
                os: "macOS".to_string(),
                arch: "arm64".to_string(),
            },
            );
            let path = profiles_dir.join(file_name);
            let content = toml::to_string_pretty(&profile).unwrap();
            fs::write(&path, content).unwrap();
        }

        // 创建 ProfileManager
        let manager = ProfileManager::new(temp_dir.path().to_path_buf()).unwrap();

        // 调用 get_all()
        let profiles = manager.get_all().unwrap();

        // 验证：应该只返回2个正常配置文件
        assert_eq!(profiles.len(), 2);
        for profile in &profiles {
            assert!(!profile.name.contains("Hidden"));
        }
    }
}
