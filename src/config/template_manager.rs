// 模板管理器
//
// 负责模板的加载、缓存和管理

use super::template::TemplateConfig;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;

/// 模板管理器
pub struct TemplateManager {
    /// 模板目录
    template_dir: PathBuf,
    /// 模板缓存 (name -> TemplateConfig)
    templates: HashMap<String, TemplateConfig>,
}

impl TemplateManager {
    /// 创建模板管理器
    pub fn new(template_dir: PathBuf) -> Result<Self> {
        // 确保模板目录存在
        if !template_dir.exists() {
            std::fs::create_dir_all(&template_dir).context("创建模板目录失败")?;
            log::info!("📁 创建模板目录: {}", template_dir.display());
        }

        let mut manager = Self {
            template_dir,
            templates: HashMap::new(),
        };

        // 加载所有模板
        manager.load_all_templates()?;

        Ok(manager)
    }

    /// 加载所有模板文件
    fn load_all_templates(&mut self) -> Result<()> {
        let entries = std::fs::read_dir(&self.template_dir).context("读取模板目录失败")?;

        for entry in entries {
            let entry = entry.context("读取目录项失败")?;
            let path = entry.path();

            // 只处理 .toml 文件
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                match TemplateConfig::load_from_file(path.clone()) {
                    Ok(template) => {
                        let name = template.metadata.name.clone();
                        self.templates.insert(name, template);
                    }
                    Err(e) => {
                        log::warn!("⚠️  加载模板失败 {}: {}", path.display(), e);
                    }
                }
            }
        }

        log::info!("📋 加载了 {} 个模板", self.templates.len());

        // 如果没有模板，创建默认模板
        if self.templates.is_empty() {
            self.create_default_template()?;
        }

        Ok(())
    }

    /// 创建默认模板
    fn create_default_template(&mut self) -> Result<()> {
        let template = TemplateConfig::default_qsl_v1();
        let name = template.metadata.name.clone();

        // 保存到文件
        let path = self.template_dir.join("qsl-card-v1.toml");
        template.save_to_file(path)?;

        // 添加到缓存
        self.templates.insert(name, template);

        log::info!("✅ 创建默认模板: QSL Card v1");

        Ok(())
    }

    /// 获取所有模板名称列表
    pub fn list_templates(&self) -> Vec<String> {
        let mut names: Vec<String> = self.templates.keys().cloned().collect();
        names.sort();
        names
    }

    /// 根据名称获取模板
    pub fn get_template(&self, name: &str) -> Result<&TemplateConfig> {
        self.templates
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("模板不存在: {}", name))
    }

    /// 获取默认模板（QSL Card v1）
    pub fn get_default_template(&self) -> Result<&TemplateConfig> {
        self.get_template("QSL Card v1")
    }

    /// 添加新模板
    pub fn add_template(&mut self, template: TemplateConfig) -> Result<()> {
        let name = template.metadata.name.clone();

        // 保存到文件
        let filename = name.to_lowercase().replace(" ", "-") + ".toml";
        let path = self.template_dir.join(filename);
        template.save_to_file(path)?;

        // 添加到缓存
        self.templates.insert(name, template);

        Ok(())
    }

    /// 删除模板
    pub fn remove_template(&mut self, name: &str) -> Result<()> {
        // 不能删除默认模板
        if name == "QSL Card v1" {
            anyhow::bail!("不能删除默认模板");
        }

        // 从缓存中移除
        self.templates
            .remove(name)
            .ok_or_else(|| anyhow::anyhow!("模板不存在: {}", name))?;

        // 删除文件
        let filename = name.to_lowercase().replace(" ", "-") + ".toml";
        let path = self.template_dir.join(filename);
        if path.exists() {
            std::fs::remove_file(&path).context(format!("删除模板文件失败: {}", path.display()))?;
        }

        log::info!("🗑️  删除模板: {}", name);

        Ok(())
    }

    /// 重新加载所有模板
    pub fn reload(&mut self) -> Result<()> {
        self.templates.clear();
        self.load_all_templates()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_template_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let manager = TemplateManager::new(temp_dir.path().to_path_buf());

        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert!(!manager.list_templates().is_empty());
    }

    #[test]
    fn test_get_default_template() {
        let temp_dir = tempdir().unwrap();
        let manager = TemplateManager::new(temp_dir.path().to_path_buf()).unwrap();

        let template = manager.get_default_template();
        assert!(template.is_ok());

        let template = template.unwrap();
        assert_eq!(template.metadata.name, "QSL Card v1");
    }

    #[test]
    fn test_list_templates() {
        let temp_dir = tempdir().unwrap();
        let manager = TemplateManager::new(temp_dir.path().to_path_buf()).unwrap();

        let templates = manager.list_templates();
        assert!(templates.contains(&"QSL Card v1".to_string()));
    }
}
