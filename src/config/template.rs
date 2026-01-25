// 模板配置
//
// 定义模板配置结构,支持灵活的元素来源(fixed/input/computed)、
// 高度预算、布局约束等自适应布局功能

use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::{Context, Result};

/// v2版本完整模板配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// 模板元信息
    pub metadata: TemplateMetadata,
    /// 页面配置
    pub page: PageConfig,
    /// 布局配置
    pub layout: LayoutConfig,
    /// 字体配置
    pub fonts: FontConfig,
    /// 元素列表
    pub elements: Vec<ElementConfig>,
    /// 输出配置
    pub output: OutputConfig,
}

/// 模板元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// 模板版本(用于识别v2配置)
    #[serde(default = "default_template_version")]
    pub template_version: String,
    /// 模板名称
    pub name: String,
    /// 描述
    #[serde(default)]
    pub description: String,
}

fn default_template_version() -> String {
    "2.0".to_string()
}

/// 页面配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageConfig {
    /// DPI (常见: 203, 300)
    pub dpi: u32,
    /// 纸张宽度(mm)
    pub width_mm: f32,
    /// 纸张高度(mm)
    pub height_mm: f32,
    /// 左边距(mm)
    pub margin_left_mm: f32,
    /// 右边距(mm)
    pub margin_right_mm: f32,
    /// 上边距(mm)
    pub margin_top_mm: f32,
    /// 下边距(mm)
    pub margin_bottom_mm: f32,
    /// 是否显示边框
    #[serde(default)]
    pub border: bool,
    /// 边框线宽(mm)
    #[serde(default = "default_border_thickness")]
    pub border_thickness_mm: f32,
    /// 是否双份打印（上下各打印一份）
    #[serde(default)]
    pub duplicate_print: bool,
}

fn default_border_thickness() -> f32 {
    0.3
}

/// 布局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    /// 水平对齐方式("center", "left", "right")
    #[serde(default = "default_align_h")]
    pub align_h: String,
    /// 垂直对齐方式("center", "top", "bottom")
    #[serde(default = "default_align_v")]
    pub align_v: String,
    /// 文本块与条码块之间的额外间距(mm)
    #[serde(default = "default_gap")]
    pub gap_mm: f32,
    /// 文本行之间间距(mm)
    #[serde(default = "default_line_gap")]
    pub line_gap_mm: f32,
}

fn default_align_h() -> String {
    "center".to_string()
}

fn default_align_v() -> String {
    "center".to_string()
}

fn default_gap() -> f32 {
    2.0
}

fn default_line_gap() -> f32 {
    2.0
}

/// 字体配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    /// 中文粗体字体文件名
    pub cn_bold: String,
    /// 英文粗体字体文件名
    pub en_bold: String,
    /// fallback粗体字体文件名
    pub fallback_bold: String,
}

/// 元素配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementConfig {
    /// 元素ID
    pub id: String,
    /// 元素类型("text" | "barcode")
    #[serde(rename = "type")]
    pub element_type: String,
    /// 元素来源("fixed" | "input" | "computed")
    pub source: String,

    // 根据source不同,以下字段互斥:
    /// 固定值(source = "fixed")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// 输入键名(source = "input")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// 格式化字符串(source = "computed")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    // 文本元素特有:
    /// 文本元素最大高度预算(mm)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_height_mm: Option<f32>,

    // 条形码元素特有:
    /// 条形码类型("code128", "code39", "ean13"等)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub barcode_type: Option<String>,
    /// 条形码高度(mm)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_mm: Option<f32>,
    /// 条形码左右留白(quiet zone, mm)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quiet_zone_mm: Option<f32>,
    /// 条形码是否显示人类可读文本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub human_readable: Option<bool>,
}

/// 输出配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// 渲染模式:
    /// - "text_bitmap_plus_native_barcode": 文本位图 + TSPL原生条码(方案A)
    /// - "full_bitmap": 全部渲染为位图(方案B)
    pub mode: String,
    /// 灰度转黑白阈值(0-255)
    #[serde(default = "default_threshold")]
    pub threshold: u8,
}

fn default_threshold() -> u8 {
    160
}

impl TemplateConfig {
    /// 生成默认的QSL Card v2模板配置
    ///
    /// 符合 docs/template.v2.md 规范的完整模板配置
    pub fn default_qsl_card() -> Self {
        Self {
            metadata: TemplateMetadata {
                template_version: "2.0".to_string(),
                name: "QSL Card v2".to_string(),
                description: "标准 QSL 卡片模板，76mm × 130mm，包含中文标题".to_string(),
            },
            page: PageConfig {
                dpi: 203,
                width_mm: 76.0,
                height_mm: 130.0,
                margin_left_mm: 2.0,
                margin_right_mm: 2.0,
                margin_top_mm: 3.0,
                margin_bottom_mm: 3.0,
                border: true,
                border_thickness_mm: 0.3,
                duplicate_print: false,
            },
            layout: LayoutConfig {
                align_h: "center".to_string(),
                align_v: "center".to_string(),
                gap_mm: 2.0,
                line_gap_mm: 2.0,
            },
            fonts: FontConfig {
                cn_bold: "SourceHanSansSC-Bold.otf".to_string(),
                en_bold: "LiberationSans-Bold.ttf".to_string(),
                fallback_bold: "SourceHanSansSC-Bold.otf".to_string(),
            },
            elements: vec![
                // 标题(固定文本)
                ElementConfig {
                    id: "title".to_string(),
                    element_type: "text".to_string(),
                    source: "fixed".to_string(),
                    value: Some("中国无线电协会业余分会-2区卡片局".to_string()),
                    key: None,
                    format: None,
                    max_height_mm: Some(10.0),
                    barcode_type: None,
                    height_mm: None,
                    quiet_zone_mm: None,
                    human_readable: None,
                },
                // 副标题(输入)
                ElementConfig {
                    id: "subtitle".to_string(),
                    element_type: "text".to_string(),
                    source: "input".to_string(),
                    value: None,
                    key: Some("project_name".to_string()),
                    format: None,
                    max_height_mm: Some(16.0),
                    barcode_type: None,
                    height_mm: None,
                    quiet_zone_mm: None,
                    human_readable: None,
                },
                // 呼号(输入)
                ElementConfig {
                    id: "callsign".to_string(),
                    element_type: "text".to_string(),
                    source: "input".to_string(),
                    value: None,
                    key: Some("callsign".to_string()),
                    format: None,
                    max_height_mm: Some(28.0),
                    barcode_type: None,
                    height_mm: None,
                    quiet_zone_mm: None,
                    human_readable: None,
                },
                // 条形码(从呼号派生)
                ElementConfig {
                    id: "barcode".to_string(),
                    element_type: "barcode".to_string(),
                    source: "computed".to_string(),
                    value: None,
                    key: None,
                    format: Some("{callsign}".to_string()),
                    max_height_mm: None,
                    barcode_type: Some("code128".to_string()),
                    height_mm: Some(18.0),
                    quiet_zone_mm: Some(2.0),
                    human_readable: Some(false),
                },
                // 序列号(计算)
                ElementConfig {
                    id: "sn".to_string(),
                    element_type: "text".to_string(),
                    source: "computed".to_string(),
                    value: None,
                    key: None,
                    format: Some("SN: {sn}".to_string()),
                    max_height_mm: Some(22.0),
                    barcode_type: None,
                    height_mm: None,
                    quiet_zone_mm: None,
                    human_readable: None,
                },
                // 数量(计算)
                ElementConfig {
                    id: "qty".to_string(),
                    element_type: "text".to_string(),
                    source: "computed".to_string(),
                    value: None,
                    key: None,
                    format: Some("QTY: {qty}".to_string()),
                    max_height_mm: Some(22.0),
                    barcode_type: None,
                    height_mm: None,
                    quiet_zone_mm: None,
                    human_readable: None,
                },
            ],
            output: OutputConfig {
                mode: "text_bitmap_plus_native_barcode".to_string(),
                threshold: 160,
            },
        }
    }

    /// 从TOML文件加载配置
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context(format!("读取模板文件失败: {}", path.display()))?;

        let config: TemplateConfig = toml::from_str(&content)
            .context(format!("解析模板文件失败: {}", path.display()))?;

        // 基本验证
        config.validate()?;

        log::info!(
            "✅ 加载v2模板: {} (模板版本: {})",
            config.metadata.name,
            config.metadata.template_version
        );

        Ok(config)
    }

    /// 保存配置到TOML文件
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let toml_str = toml::to_string_pretty(self)
            .context("序列化模板配置失败")?;

        std::fs::write(path, toml_str)
            .context(format!("写入模板文件失败: {}", path.display()))?;

        log::info!("✅ 保存v2模板: {} 到 {}", self.metadata.name, path.display());

        Ok(())
    }

    /// 验证配置的有效性
    pub fn validate(&self) -> Result<()> {
        // 验证元素列表不为空
        if self.elements.is_empty() {
            anyhow::bail!("元素列表不能为空");
        }

        // 验证输出模式
        if self.output.mode != "text_bitmap_plus_native_barcode" && self.output.mode != "full_bitmap" {
            anyhow::bail!(
                "无效的输出模式: {}, 支持的模式: text_bitmap_plus_native_barcode, full_bitmap",
                self.output.mode
            );
        }

        // 验证每个元素
        for elem in &self.elements {
            self.validate_element(elem)?;
        }

        Ok(())
    }

    /// 验证单个元素配置
    fn validate_element(&self, elem: &ElementConfig) -> Result<()> {
        // 验证元素类型
        if elem.element_type != "text" && elem.element_type != "barcode" {
            anyhow::bail!(
                "元素 {} 的类型无效: {}, 支持的类型: text, barcode",
                elem.id, elem.element_type
            );
        }

        // 验证元素来源
        if elem.source != "fixed" && elem.source != "input" && elem.source != "computed" {
            anyhow::bail!(
                "元素 {} 的来源无效: {}, 支持的来源: fixed, input, computed",
                elem.id, elem.source
            );
        }

        // 验证来源对应的字段存在
        match elem.source.as_str() {
            "fixed" => {
                if elem.value.is_none() {
                    anyhow::bail!("元素 {} 的来源为 fixed, 但缺少 value 字段", elem.id);
                }
            }
            "input" => {
                if elem.key.is_none() {
                    anyhow::bail!("元素 {} 的来源为 input, 但缺少 key 字段", elem.id);
                }
            }
            "computed" => {
                if elem.format.is_none() {
                    anyhow::bail!("元素 {} 的来源为 computed, 但缺少 format 字段", elem.id);
                }
            }
            _ => {}
        }

        // 验证类型对应的字段存在
        match elem.element_type.as_str() {
            "text" => {
                if elem.max_height_mm.is_none() {
                    anyhow::bail!("文本元素 {} 缺少 max_height_mm 字段", elem.id);
                }
            }
            "barcode" => {
                if elem.barcode_type.is_none() {
                    anyhow::bail!("条形码元素 {} 缺少 barcode_type 字段", elem.id);
                }
                if elem.height_mm.is_none() {
                    anyhow::bail!("条形码元素 {} 缺少 height_mm 字段", elem.id);
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_config_serialization() {
        let config = TemplateConfig {
            metadata: TemplateMetadata {
                template_version: "2.0".to_string(),
                name: "Test Template".to_string(),
                description: "Test description".to_string(),
            },
            page: PageConfig {
                dpi: 203,
                width_mm: 76.0,
                height_mm: 130.0,
                margin_left_mm: 2.0,
                margin_right_mm: 2.0,
                margin_top_mm: 3.0,
                margin_bottom_mm: 3.0,
                border: true,
                border_thickness_mm: 0.3,
                duplicate_print: false,
            },
            layout: LayoutConfig {
                align_h: "center".to_string(),
                align_v: "center".to_string(),
                gap_mm: 2.0,
                line_gap_mm: 2.0,
            },
            fonts: FontConfig {
                cn_bold: "SourceHanSansSC-Bold.otf".to_string(),
                en_bold: "LiberationSans-Bold.ttf".to_string(),
                fallback_bold: "SourceHanSansSC-Bold.otf".to_string(),
            },
            elements: vec![
                ElementConfig {
                    id: "title".to_string(),
                    element_type: "text".to_string(),
                    source: "fixed".to_string(),
                    value: Some("Test Title".to_string()),
                    key: None,
                    format: None,
                    max_height_mm: Some(10.0),
                    barcode_type: None,
                    height_mm: None,
                    quiet_zone_mm: None,
                    human_readable: None,
                },
            ],
            output: OutputConfig {
                mode: "text_bitmap_plus_native_barcode".to_string(),
                threshold: 160,
            },
        };

        // 序列化
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("template_version = \"2.0\""));
        assert!(toml_str.contains("[page]"));
        assert!(toml_str.contains("[layout]"));
        assert!(toml_str.contains("[fonts]"));

        // 反序列化
        let deserialized: TemplateConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.metadata.template_version, "2.0");
        assert_eq!(deserialized.page.dpi, 203);
        assert_eq!(deserialized.elements.len(), 1);
    }

    #[test]
    fn test_element_validation() {
        let mut config = TemplateConfig {
            metadata: TemplateMetadata {
                template_version: "2.0".to_string(),
                name: "Test".to_string(),
                description: "".to_string(),
            },
            page: PageConfig {
                dpi: 203,
                width_mm: 76.0,
                height_mm: 130.0,
                margin_left_mm: 2.0,
                margin_right_mm: 2.0,
                margin_top_mm: 3.0,
                margin_bottom_mm: 3.0,
                border: false,
                border_thickness_mm: 0.3,
                duplicate_print: false,
            },
            layout: LayoutConfig {
                align_h: "center".to_string(),
                align_v: "center".to_string(),
                gap_mm: 2.0,
                line_gap_mm: 2.0,
            },
            fonts: FontConfig {
                cn_bold: "CN.otf".to_string(),
                en_bold: "EN.ttf".to_string(),
                fallback_bold: "CN.otf".to_string(),
            },
            elements: vec![],
            output: OutputConfig {
                mode: "text_bitmap_plus_native_barcode".to_string(),
                threshold: 160,
            },
        };

        // 测试空元素列表
        assert!(config.validate().is_err());

        // 测试无效的输出模式
        config.elements.push(ElementConfig {
            id: "test".to_string(),
            element_type: "text".to_string(),
            source: "fixed".to_string(),
            value: Some("test".to_string()),
            key: None,
            format: None,
            max_height_mm: Some(10.0),
            barcode_type: None,
            height_mm: None,
            quiet_zone_mm: None,
            human_readable: None,
        });
        config.output.mode = "invalid_mode".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_default_qsl_card() {
        let config = TemplateConfig::default_qsl_card();

        // 验证元信息
        assert_eq!(config.metadata.template_version, "2.0");
        assert_eq!(config.metadata.name, "QSL Card v2");

        // 验证页面配置
        assert_eq!(config.page.dpi, 203);
        assert_eq!(config.page.width_mm, 76.0);
        assert_eq!(config.page.height_mm, 130.0);

        // 验证元素顺序和数量
        assert_eq!(config.elements.len(), 6);
        assert_eq!(config.elements[0].id, "title");
        assert_eq!(config.elements[1].id, "subtitle");
        assert_eq!(config.elements[2].id, "callsign");
        assert_eq!(config.elements[3].id, "barcode");
        assert_eq!(config.elements[4].id, "sn");
        assert_eq!(config.elements[5].id, "qty");

        // 验证配置有效性
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_load_and_save() {
        use std::fs;

        let config = TemplateConfig::default_qsl_card();

        // 创建临时目录
        let temp_dir = std::env::temp_dir().join("qsl_test");
        fs::create_dir_all(&temp_dir).unwrap();

        let temp_file = temp_dir.join("test_template.toml");

        // 保存配置
        config.save_to_file(&temp_file).unwrap();

        // 加载配置
        let loaded_config = TemplateConfig::load_from_file(&temp_file).unwrap();

        // 验证加载的配置与原配置一致
        assert_eq!(loaded_config.metadata.name, config.metadata.name);
        assert_eq!(loaded_config.elements.len(), config.elements.len());
        assert_eq!(loaded_config.output.mode, config.output.mode);

        // 清理
        fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_load_qsl_card_toml() {
        let config_path = Path::new("../../config/templates/callsign.toml");
        let config = TemplateConfig::load_from_file(config_path)
            .expect("应该成功加载v2配置");

        // 验证基本信息
        assert_eq!(config.metadata.template_version, "2.0");
        assert_eq!(config.metadata.name, "QSL Card v2");

        // 验证页面配置
        assert_eq!(config.page.dpi, 203);
        assert_eq!(config.page.width_mm, 76.0);
        assert_eq!(config.page.height_mm, 130.0);

        // 验证元素数量和顺序
        assert_eq!(config.elements.len(), 6);
        assert_eq!(config.elements[0].id, "title");
        assert_eq!(config.elements[2].id, "callsign");
        assert_eq!(config.elements[3].id, "barcode");

        println!("✅ 成功加载并验证v2配置文件");
    }
}
