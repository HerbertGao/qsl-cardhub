// 配置管理模块
//
// 该模块负责：
// - Profile 数据模型定义
// - 配置文件的持久化（TOML 格式）
// - 配置 CRUD 操作
// - 默认配置管理
// - 模板配置管理

pub mod models;
pub mod profile_manager;
pub mod template;

pub use models::{Platform, Profile};
pub use profile_manager::ProfileManager;
pub use template::TemplateConfig;
