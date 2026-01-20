// 打印机管理 Commands v2
//
// 使用新的模板系统 v2 架构

use crate::config::template_v2::{OutputConfig, TemplateV2Config};
use crate::printer::backend::{PdfBackendV2, PrinterBackend};
use crate::printer::layout_engine::LayoutEngine;
use crate::printer::render_pipeline::RenderPipeline;
use crate::printer::template_engine::TemplateEngine;
use crate::printer::tspl_v2::TSPLGeneratorV2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;

/// 打印机管理器状态 v2
pub struct PrinterStateV2 {
    /// 布局引擎
    pub layout_engine: Arc<Mutex<LayoutEngine>>,
    /// 渲染管道
    pub render_pipeline: Arc<Mutex<RenderPipeline>>,
    /// PDF 后端
    pub pdf_backend: Arc<Mutex<PdfBackendV2>>,
    /// TSPL 生成器
    pub tspl_generator: Arc<Mutex<TSPLGeneratorV2>>,
}

impl PrinterStateV2 {
    /// 创建新的打印机状态
    pub fn new() -> Result<Self, String> {
        let layout_engine = LayoutEngine::new().map_err(|e| format!("创建布局引擎失败: {}", e))?;
        let render_pipeline =
            RenderPipeline::new().map_err(|e| format!("创建渲染管道失败: {}", e))?;
        let pdf_backend =
            PdfBackendV2::with_downloads_dir().map_err(|e| format!("创建PDF后端失败: {}", e))?;
        let tspl_generator = TSPLGeneratorV2::new();

        Ok(Self {
            layout_engine: Arc::new(Mutex::new(layout_engine)),
            render_pipeline: Arc::new(Mutex::new(render_pipeline)),
            pdf_backend: Arc::new(Mutex::new(pdf_backend)),
            tspl_generator: Arc::new(Mutex::new(tspl_generator)),
        })
    }
}

/// 打印请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintRequest {
    /// 模板配置路径（可选，不提供则使用默认模板）
    pub template_path: Option<String>,
    /// 运行时数据（替换模板中的占位符）
    pub data: HashMap<String, String>,
    /// 输出配置
    pub output_config: OutputConfigRequest,
}

/// 输出配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfigRequest {
    /// 渲染模式: "text_bitmap_plus_native_barcode" 或 "full_bitmap"
    pub mode: String,
    /// 二值化阈值 (0-255)
    pub threshold: u8,
}

/// 预览响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewResponse {
    /// PNG 文件路径
    pub png_path: String,
    /// 画布宽度
    pub width: u32,
    /// 画布高度
    pub height: u32,
}

/// 生成 QSL 卡片预览（PDF/PNG）
///
/// # 参数
/// - `request`: 打印请求参数
///
/// # 返回
/// PNG 文件路径
#[tauri::command]
pub async fn preview_qsl_v2(
    request: PrintRequest,
    state: State<'_, PrinterStateV2>,
) -> Result<PreviewResponse, String> {
    log::info!("开始生成 QSL 卡片预览 (v2)");
    log::debug!("请求参数: {:?}", request);

    // 1. 加载模板配置
    let config = if let Some(path) = &request.template_path {
        log::info!("从文件加载模板: {}", path);
        TemplateV2Config::load_from_file(std::path::Path::new(path))
            .map_err(|e| format!("加载模板失败: {}", e))?
    } else {
        log::info!("使用默认模板");
        TemplateV2Config::default_qsl_card_v2()
    };

    // 2. 模板解析
    log::debug!("解析模板，数据: {:?}", request.data);
    let resolved_elements = TemplateEngine::resolve(&config, &request.data)
        .map_err(|e| format!("模板解析失败: {}", e))?;
    log::info!("✓ 解析 {} 个元素", resolved_elements.len());

    // 3. 布局计算
    let mut layout_engine = state
        .layout_engine
        .lock()
        .map_err(|e| format!("锁定布局引擎失败: {}", e))?;
    let layout_result = layout_engine
        .layout(&config, resolved_elements)
        .map_err(|e| format!("布局计算失败: {}", e))?;
    log::info!(
        "✓ 布局计算完成: {}x{} dots",
        layout_result.canvas_width,
        layout_result.canvas_height
    );

    // 4. 渲染
    let mut render_pipeline = state
        .render_pipeline
        .lock()
        .map_err(|e| format!("锁定渲染管道失败: {}", e))?;
    let output_config = OutputConfig {
        mode: request.output_config.mode.clone(),
        threshold: request.output_config.threshold,
    };
    let render_result = render_pipeline
        .render(layout_result, &output_config)
        .map_err(|e| format!("渲染失败: {}", e))?;
    log::info!("✓ 渲染完成");

    // 5. 保存为 PNG
    let mut pdf_backend = state
        .pdf_backend
        .lock()
        .map_err(|e| format!("锁定PDF后端失败: {}", e))?;
    let png_path = pdf_backend
        .render(render_result)
        .map_err(|e| format!("保存PNG失败: {}", e))?;

    // 6. 读取图像尺寸
    let img = image::open(&png_path).map_err(|e| format!("读取图像失败: {}", e))?;

    log::info!("✅ 预览生成成功: {}", png_path.display());

    Ok(PreviewResponse {
        png_path: png_path.to_string_lossy().to_string(),
        width: img.width(),
        height: img.height(),
    })
}

/// 打印 QSL 卡片（发送到打印机）
///
/// # 参数
/// - `printer_name`: 打印机名称
/// - `request`: 打印请求参数
///
/// # 返回
/// 成功或错误信息
#[tauri::command]
pub async fn print_qsl_v2(
    printer_name: String,
    request: PrintRequest,
    state: State<'_, PrinterStateV2>,
) -> Result<(), String> {
    log::info!("开始打印 QSL 卡片 (v2): 打印机={}", printer_name);
    log::debug!("请求参数: {:?}", request);

    // 1. 加载模板配置
    let config = if let Some(path) = &request.template_path {
        TemplateV2Config::load_from_file(std::path::Path::new(path))
            .map_err(|e| format!("加载模板失败: {}", e))?
    } else {
        TemplateV2Config::default_qsl_card_v2()
    };

    // 2. 模板解析
    let resolved_elements = TemplateEngine::resolve(&config, &request.data)
        .map_err(|e| format!("模板解析失败: {}", e))?;

    // 3. 布局计算
    let mut layout_engine = state
        .layout_engine
        .lock()
        .map_err(|e| format!("锁定布局引擎失败: {}", e))?;
    let layout_result = layout_engine
        .layout(&config, resolved_elements)
        .map_err(|e| format!("布局计算失败: {}", e))?;

    // 4. 渲染
    let mut render_pipeline = state
        .render_pipeline
        .lock()
        .map_err(|e| format!("锁定渲染管道失败: {}", e))?;
    let output_config = OutputConfig {
        mode: request.output_config.mode.clone(),
        threshold: request.output_config.threshold,
    };
    let render_result = render_pipeline
        .render(layout_result, &output_config)
        .map_err(|e| format!("渲染失败: {}", e))?;

    // 5. 生成 TSPL 指令
    let tspl_generator = state
        .tspl_generator
        .lock()
        .map_err(|e| format!("锁定TSPL生成器失败: {}", e))?;
    let tspl = tspl_generator
        .generate(render_result, config.page.width_mm, config.page.height_mm)
        .map_err(|e| format!("生成TSPL指令失败: {}", e))?;

    log::debug!("TSPL指令长度: {} 字节", tspl.len());

    // 6. 发送到打印机
    // TODO: 需要获取打印机后端实例
    // 暂时只记录日志
    log::info!("✅ TSPL指令已生成，准备发送到打印机: {}", printer_name);
    log::warn!("⚠️ 实际打印功能待实现（需要集成打印机后端）");

    // 临时返回成功
    // 在实际实现中，应该调用打印机后端的 send_raw 方法
    Ok(())
}

/// 生成 TSPL 指令（用于调试）
///
/// # 参数
/// - `request`: 打印请求参数
///
/// # 返回
/// TSPL 指令字符串
#[tauri::command]
pub async fn generate_tspl_v2(
    request: PrintRequest,
    state: State<'_, PrinterStateV2>,
) -> Result<String, String> {
    log::info!("生成 TSPL 指令 (v2)");

    // 1. 加载模板配置
    let config = if let Some(path) = &request.template_path {
        TemplateV2Config::load_from_file(std::path::Path::new(path))
            .map_err(|e| format!("加载模板失败: {}", e))?
    } else {
        TemplateV2Config::default_qsl_card_v2()
    };

    // 2. 模板解析 → 布局 → 渲染
    let resolved_elements = TemplateEngine::resolve(&config, &request.data)
        .map_err(|e| format!("模板解析失败: {}", e))?;

    let mut layout_engine = state
        .layout_engine
        .lock()
        .map_err(|e| format!("锁定布局引擎失败: {}", e))?;
    let layout_result = layout_engine
        .layout(&config, resolved_elements)
        .map_err(|e| format!("布局计算失败: {}", e))?;

    let mut render_pipeline = state
        .render_pipeline
        .lock()
        .map_err(|e| format!("锁定渲染管道失败: {}", e))?;
    let output_config = OutputConfig {
        mode: request.output_config.mode.clone(),
        threshold: request.output_config.threshold,
    };
    let render_result = render_pipeline
        .render(layout_result, &output_config)
        .map_err(|e| format!("渲染失败: {}", e))?;

    // 3. 生成 TSPL
    let tspl_generator = state
        .tspl_generator
        .lock()
        .map_err(|e| format!("锁定TSPL生成器失败: {}", e))?;
    let tspl = tspl_generator
        .generate(render_result, config.page.width_mm, config.page.height_mm)
        .map_err(|e| format!("生成TSPL指令失败: {}", e))?;

    log::info!("✅ TSPL指令生成成功: {} 字节", tspl.len());

    Ok(tspl)
}

/// 加载模板配置
///
/// # 参数
/// - `path`: 模板文件路径（可选，不提供则返回默认模板）
///
/// # 返回
/// 模板配置 JSON
#[tauri::command]
pub async fn load_template_v2(path: Option<String>) -> Result<String, String> {
    log::info!("加载模板配置 (v2)");

    let config = if let Some(path) = path {
        log::debug!("从文件加载: {}", path);
        TemplateV2Config::load_from_file(std::path::Path::new(&path))
            .map_err(|e| format!("加载模板失败: {}", e))?
    } else {
        log::debug!("使用默认模板");
        TemplateV2Config::default_qsl_card_v2()
    };

    // 序列化为 JSON
    serde_json::to_string_pretty(&config).map_err(|e| format!("序列化模板失败: {}", e))
}

/// 保存模板配置
///
/// # 参数
/// - `path`: 保存路径
/// - `config_json`: 模板配置 JSON
///
/// # 返回
/// 成功或错误信息
#[tauri::command]
pub async fn save_template_v2(path: String, config_json: String) -> Result<(), String> {
    log::info!("保存模板配置 (v2): {}", path);

    // 反序列化
    let config: TemplateV2Config =
        serde_json::from_str(&config_json).map_err(|e| format!("解析模板配置失败: {}", e))?;

    // 保存到文件
    config
        .save_to_file(std::path::Path::new(&path))
        .map_err(|e| format!("保存模板失败: {}", e))?;

    log::info!("✅ 模板保存成功");
    Ok(())
}
