// 配置数据模型
//
// 定义所有配置相关的数据结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 打印配置 Profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// 唯一标识符（UUID v4）
    pub id: String,
    /// 配置名称
    pub name: String,
    /// 平台信息
    pub platform: Platform,
    /// 打印机配置
    pub printer: PrinterConfig,
    /// 纸张规格
    pub paper: PaperSpec,
    /// 模板配置
    pub template: Template,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
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
    /// 打印机型号（如 "Deli DL-888C"）
    pub model: String,
    /// 系统中的打印机名称
    pub name: String,
}

/// 纸张规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperSpec {
    /// 纸张宽度（单位：mm）
    pub width: u32,
    /// 纸张高度（单位：mm）
    pub height: u32,
}

/// 打印模板配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// 模板名称
    pub name: String,
    /// 模板版本
    pub version: String,
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
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            platform,
            printer: PrinterConfig {
                model: "Deli DL-888C".to_string(),
                name: printer_name,
            },
            paper: PaperSpec {
                width: 76,
                height: 130,
            },
            template: Template {
                name: "QSL Card v1".to_string(),
                version: "1.0".to_string(),
            },
            created_at: now,
            updated_at: now,
        }
    }

    /// 更新 Profile 的更新时间
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}
