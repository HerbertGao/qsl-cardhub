// 模板配置模块
//
// 定义 QSL 卡片打印模板的完整配置结构

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::{Context, Result};

/// 完整的模板配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// 模板元信息
    pub metadata: TemplateMetadata,
    /// 纸张配置
    pub paper: PaperConfig,
    /// 标题配置（主标题）
    pub title: TitleConfig,
    /// 副标题配置（任务名称）
    #[serde(default)]
    pub subtitle: SubtitleConfig,
    /// 呼号配置
    pub callsign: CallsignConfig,
    /// 条形码配置
    pub barcode: BarcodeConfig,
    /// 序列号配置
    pub serial: SerialConfig,
    /// 数量配置
    pub quantity: QuantityConfig,
}

/// 模板元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// 模板名称
    pub name: String,
    /// 模板版本
    pub version: String,
    /// 模板描述
    #[serde(default)]
    pub description: String,
}

/// 纸张配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperConfig {
    /// 纸张宽度（单位：mm）
    pub width_mm: u32,
    /// 纸张高度（单位：mm）
    pub height_mm: u32,
    /// 标签间隙（单位：mm）
    pub gap_mm: u32,
    /// 打印方向（0=正向, 1=90度, 2=180度, 3=270度）
    pub direction: u32,
}

/// 标题配置（固定文本，如"中国无线电协会业余分会-2区卡片局"）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleConfig {
    /// 标题文本
    pub text: String,
    /// X 坐标（dots）
    pub x: u32,
    /// Y 坐标（dots）
    pub y: u32,
    /// 字体编号
    pub font: String,
    /// 旋转角度
    pub rotation: u32,
    /// X 方向缩放
    pub x_scale: u32,
    /// Y 方向缩放
    pub y_scale: u32,
}

/// 副标题配置（动态文本，来自 task_name）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleConfig {
    /// X 坐标（dots）
    pub x: u32,
    /// Y 坐标（dots）
    pub y: u32,
    /// 字体编号
    pub font: String,
    /// 旋转角度
    pub rotation: u32,
    /// X 方向缩放
    pub x_scale: u32,
    /// Y 方向缩放
    pub y_scale: u32,
}

impl Default for SubtitleConfig {
    fn default() -> Self {
        Self {
            x: 304,
            y: 100,
            font: "5".to_string(),
            rotation: 0,
            x_scale: 2,
            y_scale: 2,
        }
    }
}

/// 呼号配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallsignConfig {
    /// X 坐标（dots，203 DPI）
    pub x: u32,
    /// Y 坐标（dots）
    pub y: u32,
    /// 字体编号（TSPL 字体：1-8）
    pub font: String,
    /// 旋转角度（0, 90, 180, 270）
    pub rotation: u32,
    /// X 方向缩放（1-10）
    pub x_scale: u32,
    /// Y 方向缩放（1-10）
    pub y_scale: u32,
    /// 对齐方式（可选，PDF 后端通过 x 坐标自动判断）
    #[serde(default)]
    pub align: Option<String>,
}

/// 条形码配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarcodeConfig {
    /// X 坐标（dots）
    pub x: u32,
    /// Y 坐标（dots）
    pub y: u32,
    /// 条形码类型（"128", "39", "EAN13" 等）
    #[serde(rename = "type")]
    pub barcode_type: String,
    /// 条形码高度（dots）
    pub height: u32,
    /// 人类可读（0=关闭, 1=开启）
    pub human_readable: u32,
    /// 旋转角度
    pub rotation: u32,
    /// 窄条宽度
    pub narrow_bar: u32,
    /// 宽条宽度
    pub wide_bar: u32,
}

/// 序列号配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialConfig {
    /// X 坐标（dots）
    pub x: u32,
    /// Y 坐标（dots）
    pub y: u32,
    /// 字体编号
    pub font: String,
    /// 旋转角度
    pub rotation: u32,
    /// X 方向缩放
    pub x_scale: u32,
    /// Y 方向缩放
    pub y_scale: u32,
    /// 前缀文本
    pub prefix: String,
    /// 格式化字符串（如 "{:03}" 表示补零到3位）
    pub format: String,
}

/// 数量配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantityConfig {
    /// X 坐标（dots）
    pub x: u32,
    /// Y 坐标（dots）
    pub y: u32,
    /// 字体编号
    pub font: String,
    /// 旋转角度
    pub rotation: u32,
    /// X 方向缩放
    pub x_scale: u32,
    /// Y 方向缩放
    pub y_scale: u32,
    /// 前缀文本
    pub prefix: String,
}

impl TemplateConfig {
    /// 从 TOML 文件加载模板
    pub fn load_from_file(path: PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(&path)
            .context(format!("读取模板文件失败: {}", path.display()))?;

        let config: TemplateConfig = toml::from_str(&content)
            .context(format!("解析模板文件失败: {}", path.display()))?;

        log::info!("✅ 加载模板: {} v{}", config.metadata.name, config.metadata.version);

        Ok(config)
    }

    /// 获取默认的 QSL Card v1 模板
    pub fn default_qsl_v1() -> Self {
        Self {
            metadata: TemplateMetadata {
                name: "QSL Card v1".to_string(),
                version: "1.0".to_string(),
                description: "标准 QSL 卡片模板，76mm x 130mm，包含中文标题".to_string(),
            },
            paper: PaperConfig {
                width_mm: 76,
                height_mm: 130,
                gap_mm: 2,
                direction: 0,
            },
            title: TitleConfig {
                text: "中国无线电协会业余分会-2区卡片局".to_string(),
                x: 304,
                y: 20,
                font: "5".to_string(),
                rotation: 0,
                x_scale: 2,
                y_scale: 2,
            },
            subtitle: SubtitleConfig {
                x: 304,
                y: 100,
                font: "5".to_string(),
                rotation: 0,
                x_scale: 2,
                y_scale: 2,
            },
            callsign: CallsignConfig {
                x: 304,
                y: 80,
                font: "5".to_string(),
                rotation: 0,
                x_scale: 3,
                y_scale: 3,
                align: Some("center".to_string()),
            },
            barcode: BarcodeConfig {
                x: 200,
                y: 300,
                barcode_type: "128".to_string(),
                height: 120,
                human_readable: 1,
                rotation: 0,
                narrow_bar: 3,
                wide_bar: 3,
            },
            serial: SerialConfig {
                x: 50,
                y: 520,
                font: "5".to_string(),
                rotation: 0,
                x_scale: 2,
                y_scale: 2,
                prefix: "SN: ".to_string(),
                format: "{:03}".to_string(),
            },
            quantity: QuantityConfig {
                x: 50,
                y: 720,
                font: "5".to_string(),
                rotation: 0,
                x_scale: 2,
                y_scale: 2,
                prefix: "QTY: ".to_string(),
            },
        }
    }

    /// 保存模板到 TOML 文件
    pub fn save_to_file(&self, path: PathBuf) -> Result<()> {
        let toml_str = toml::to_string_pretty(self)
            .context("序列化模板失败")?;

        std::fs::write(&path, toml_str)
            .context(format!("写入模板文件失败: {}", path.display()))?;

        log::info!("✅ 保存模板: {} 到 {}", self.metadata.name, path.display());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_template() {
        let template = TemplateConfig::default_qsl_v1();

        assert_eq!(template.metadata.name, "QSL Card v1");
        assert_eq!(template.metadata.version, "1.0");
        assert_eq!(template.paper.width_mm, 76);
        assert_eq!(template.paper.height_mm, 130);
        assert_eq!(template.callsign.font, "5");
        assert_eq!(template.barcode.barcode_type, "128");
    }

    #[test]
    fn test_template_serialization() {
        let template = TemplateConfig::default_qsl_v1();

        // 序列化到 TOML
        let toml_str = toml::to_string_pretty(&template).unwrap();
        assert!(toml_str.contains("QSL Card v1"));
        assert!(toml_str.contains("[paper]"));
        assert!(toml_str.contains("[callsign]"));

        // 反序列化回来
        let deserialized: TemplateConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.metadata.name, template.metadata.name);
        assert_eq!(deserialized.paper.width_mm, template.paper.width_mm);
    }
}
