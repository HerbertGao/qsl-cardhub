// 配置数据模型
//
// 定义所有配置相关的数据结构

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

/// 打印配置 Profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// 唯一标识符（UUID v4）
    pub id: String,
    /// 配置名称
    pub name: String,
    /// 任务名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_name: Option<String>,
    /// 平台信息
    pub platform: Platform,
    /// 打印机配置
    pub printer: PrinterConfig,
    /// 模板配置
    pub template: Template,
    /// 模板显示名称（运行时字段，从模板文件读取，保存时会被清空）
    #[serde(skip_deserializing)]
    pub template_display_name: Option<String>,
    /// 创建时间（东八区）
    pub created_at: DateTime<FixedOffset>,
    /// 更新时间（东八区）
    pub updated_at: DateTime<FixedOffset>,
}

/// 平台信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platform {
    /// 操作系统：Windows | macOS | Linux
    pub os: String,
    /// CPU 架构：x86_64 | arm64
    pub arch: String,
}

/// 打印机配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterConfig {
    /// 系统中的打印机名称
    pub name: String,
}

/// 打印模板配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// 模板文件路径（相对于 config/templates/）
    pub path: String,
}

/// 应用全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 默认配置 Profile ID
    pub default_profile_id: Option<String>,
    /// 窗口状态
    #[serde(default)]
    pub window_state: WindowState,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_profile_id: None,
            window_state: WindowState::default(),
        }
    }
}

/// 窗口状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    /// 窗口宽度
    pub width: u32,
    /// 窗口高度
    pub height: u32,
    /// 窗口 X 坐标
    pub x: i32,
    /// 窗口 Y 坐标
    pub y: i32,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
            x: 100,
            y: 100,
        }
    }
}

impl Profile {
    /// 创建新的 Profile
    pub fn new(name: String, printer_name: String, platform: Platform) -> Self {
        // 使用东八区时间
        let tz = FixedOffset::east_opt(8 * 3600).unwrap();
        let now = chrono::Utc::now().with_timezone(&tz);

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            task_name: None,
            platform,
            printer: PrinterConfig {
                name: printer_name,
            },
            template: Template {
                path: "default.toml".to_string(),
            },
            template_display_name: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 更新 Profile 的更新时间
    pub fn touch(&mut self) {
        // 使用东八区时间
        let tz = FixedOffset::east_opt(8 * 3600).unwrap();
        self.updated_at = chrono::Utc::now().with_timezone(&tz);
    }
}
