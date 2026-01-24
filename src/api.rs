// 高级 API
//
// 提供简洁的公共接口供外部调用

use crate::config::template::{OutputConfig, TemplateConfig};
use crate::printer::backend::PdfBackend;
use crate::printer::layout_engine::LayoutEngine;
use crate::printer::render_pipeline::{RenderPipeline, RenderResult};
use crate::printer::template_engine::TemplateEngine;
use crate::printer::tspl::TSPLGenerator;
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// QSL 卡片生成器
///
/// 提供完整的 QSL 卡片生成流程
pub struct QslCardGenerator {
    layout_engine: LayoutEngine,
    render_pipeline: RenderPipeline,
}

impl QslCardGenerator {
    /// 创建新的生成器
    pub fn new() -> Result<Self> {
        Ok(Self {
            layout_engine: LayoutEngine::new()?,
            render_pipeline: RenderPipeline::new()?,
        })
    }

    /// 生成 PNG 预览
    ///
    /// # 参数
    /// - `config`: 模板配置
    /// - `data`: 运行时数据
    /// - `output_dir`: 输出目录
    /// - `output_config`: 输出配置
    ///
    /// # 返回
    /// PNG 文件路径
    pub fn generate_png(
        &mut self,
        config: &TemplateConfig,
        data: &HashMap<String, String>,
        output_dir: PathBuf,
        output_config: &OutputConfig,
    ) -> Result<PathBuf> {
        // 1. 模板解析
        let resolved_elements = TemplateEngine::resolve(config, data)?;

        // 2. 布局计算
        let layout_result = self.layout_engine.layout(config, resolved_elements)?;

        // 3. 渲染
        let render_result = self.render_pipeline.render(layout_result, output_config)?;

        // 4. 保存为 PNG
        let mut pdf_backend = PdfBackend::new(output_dir)?;
        let png_path = pdf_backend.render(render_result)?;

        Ok(png_path)
    }

    /// 生成 TSPL 指令
    ///
    /// # 参数
    /// - `config`: 模板配置
    /// - `data`: 运行时数据
    /// - `output_config`: 输出配置
    ///
    /// # 返回
    /// TSPL 指令字节数组（包含二进制位图数据）
    pub fn generate_tspl(
        &mut self,
        config: &TemplateConfig,
        data: &HashMap<String, String>,
        output_config: &OutputConfig,
    ) -> Result<Vec<u8>> {
        // 1. 模板解析
        let resolved_elements = TemplateEngine::resolve(config, data)?;

        // 2. 布局计算
        let layout_result = self.layout_engine.layout(config, resolved_elements)?;

        // 3. 渲染
        let render_result = self.render_pipeline.render(layout_result, output_config)?;

        // 4. 生成 TSPL
        let tspl_generator = TSPLGenerator::new();
        let tspl =
            tspl_generator.generate(render_result, config.page.width_mm, config.page.height_mm)?;

        Ok(tspl)
    }

    /// 获取渲染结果（高级用法）
    ///
    /// # 参数
    /// - `config`: 模板配置
    /// - `data`: 运行时数据
    /// - `output_config`: 输出配置
    ///
    /// # 返回
    /// 渲染结果（可用于自定义后端）
    pub fn render(
        &mut self,
        config: &TemplateConfig,
        data: &HashMap<String, String>,
        output_config: &OutputConfig,
    ) -> Result<RenderResult> {
        let resolved_elements = TemplateEngine::resolve(config, data)?;
        let layout_result = self.layout_engine.layout(config, resolved_elements)?;
        let render_result = self.render_pipeline.render(layout_result, output_config)?;
        Ok(render_result)
    }
}

impl Default for QslCardGenerator {
    fn default() -> Self {
        Self::new().expect("创建 QslCardGenerator 失败")
    }
}

/// 便捷函数：快速生成 PNG 预览
///
/// # 参数
/// - `template_path`: 模板文件路径（None 使用默认模板）
/// - `data`: 运行时数据
/// - `output_dir`: 输出目录
/// - `mode`: 渲染模式（"text_bitmap_plus_native_barcode" 或 "full_bitmap"）
///
/// # 返回
/// PNG 文件路径
///
/// # 示例
/// ```no_run
/// use QSL_CardHub::api::quick_generate_png;
/// use std::collections::HashMap;
/// use std::path::PathBuf;
///
/// let mut data = HashMap::new();
/// data.insert("project_name".to_string(), "CQWW DX".to_string());
/// data.insert("callsign".to_string(), "BG7XXX".to_string());
/// data.insert("sn".to_string(), "001".to_string());
/// data.insert("qty".to_string(), "100".to_string());
///
/// let png_path = quick_generate_png(
///     None,
///     &data,
///     PathBuf::from("output"),
///     "full_bitmap",
/// ).unwrap();
///
/// println!("PNG: {}", png_path.display());
/// ```
pub fn quick_generate_png(
    template_path: Option<&Path>,
    data: &HashMap<String, String>,
    output_dir: PathBuf,
    mode: &str,
) -> Result<PathBuf> {
    let config = if let Some(path) = template_path {
        TemplateConfig::load_from_file(path)?
    } else {
        TemplateConfig::default_qsl_card()
    };

    let output_config = OutputConfig {
        mode: mode.to_string(),
        threshold: 160,
    };

    let mut generator = QslCardGenerator::new()?;
    generator.generate_png(&config, data, output_dir, &output_config)
}

/// 便捷函数：快速生成 TSPL 指令
///
/// # 参数
/// - `template_path`: 模板文件路径（None 使用默认模板）
/// - `data`: 运行时数据
/// - `mode`: 渲染模式（"text_bitmap_plus_native_barcode" 或 "full_bitmap"）
///
/// # 返回
/// TSPL 指令字符串
///
/// # 示例
/// ```no_run
/// use QSL_CardHub::api::quick_generate_tspl;
/// use std::collections::HashMap;
///
/// let mut data = HashMap::new();
/// data.insert("project_name".to_string(), "CQWW DX".to_string());
/// data.insert("callsign".to_string(), "BG7XXX".to_string());
/// data.insert("sn".to_string(), "001".to_string());
/// data.insert("qty".to_string(), "100".to_string());
///
/// let tspl = quick_generate_tspl(None, &data, "text_bitmap_plus_native_barcode").unwrap();
/// println!("TSPL: {} 字节", tspl.len());
/// ```
pub fn quick_generate_tspl(
    template_path: Option<&Path>,
    data: &HashMap<String, String>,
    mode: &str,
) -> Result<Vec<u8>> {
    let config = if let Some(path) = template_path {
        TemplateConfig::load_from_file(path)?
    } else {
        TemplateConfig::default_qsl_card()
    };

    let output_config = OutputConfig {
        mode: mode.to_string(),
        threshold: 160,
    };

    let mut generator = QslCardGenerator::new()?;
    generator.generate_tspl(&config, data, &output_config)
}
