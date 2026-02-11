// 打印机管理 Commands
//
// 使用新的模板系统架构

use crate::config::template::{OutputConfig, TemplateConfig};
use crate::config::models::TsplPrintConfig;
use crate::commands::profile::ProfileState;
use crate::commands::tspl_config::normalize_tspl_print_config;
use crate::printer::backend::ImagePrintConfig;
use crate::printer::backend::PdfBackend;
use crate::printer::backend::PrinterBackend;
use crate::printer::backend::PDF_TEST_PRINTER_NAME;
use crate::printer::layout_engine::LayoutEngine;
use crate::printer::render_pipeline::RenderPipeline;
use crate::printer::template_engine::TemplateEngine;
use crate::printer::tspl::TSPLGenerator;
use image::GrayImage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::State;

#[cfg(target_os = "windows")]
use crate::printer::backend::WindowsBackend;

#[cfg(target_family = "unix")]
use crate::printer::backend::CupsBackend;

/// 加载模板配置
///
/// 优先级：
/// 1. 如果指定了 template_path，从该路径加载
/// 2. 否则尝试从 config/templates/callsign.toml 加载
/// 3. 如果都失败，使用硬编码的默认模板
fn load_template_config(template_path: Option<&String>) -> Result<TemplateConfig, String> {
    // 1. 如果指定了路径，从该路径加载
    if let Some(path) = template_path {
        log::info!("从指定路径加载模板: {}", path);
        return TemplateConfig::load_from_file(Path::new(path))
            .map_err(|e| format!("加载指定模板失败: {}", e));
    }

    // 2. 尝试从呼号模板文件加载
    let callsign_template_path = get_callsign_template_path();
    log::info!("尝试从呼号模板文件加载: {}", callsign_template_path.display());

    match TemplateConfig::load_from_file(&callsign_template_path) {
        Ok(config) => {
            log::info!("✓ 成功从文件加载模板: {}", callsign_template_path.display());
            Ok(config)
        }
        Err(e) => {
            log::warn!("从文件加载模板失败: {}, 使用硬编码的默认模板", e);
            Ok(TemplateConfig::default_qsl_card())
        }
    }
}

/// 获取呼号模板文件路径
fn get_callsign_template_path() -> PathBuf {
    // 开发模式：config/templates/callsign.toml
    #[cfg(debug_assertions)]
    {
        PathBuf::from("config/templates/callsign.toml")
    }

    // 生产模式：使用系统配置目录
    #[cfg(not(debug_assertions))]
    {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));
        config_dir.join("qsl-cardhub").join("templates").join("callsign.toml")
    }
}

/// 获取地址模板文件路径
fn get_address_template_path() -> PathBuf {
    // 开发模式：config/templates/address.toml
    #[cfg(debug_assertions)]
    {
        PathBuf::from("config/templates/address.toml")
    }

    // 生产模式：使用系统配置目录
    #[cfg(not(debug_assertions))]
    {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));
        config_dir.join("qsl-cardhub").join("templates").join("address.toml")
    }
}

/// 加载地址模板配置
fn load_address_template_config() -> Result<TemplateConfig, String> {
    let address_template_path = get_address_template_path();
    log::info!("加载地址模板: {}", address_template_path.display());

    TemplateConfig::load_from_file(&address_template_path)
        .map_err(|e| format!("加载地址模板失败: {}", e))
}

/// 读取并校验全局 TSPL 参数
fn load_tspl_print_config(profile_state: &State<'_, ProfileState>) -> Result<TsplPrintConfig, String> {
    let manager = profile_state
        .manager
        .lock()
        .map_err(|e| format!("锁定配置管理器失败: {}", e))?;
    let raw = manager
        .get_printer_config()
        .map_err(|e| format!("读取打印机配置失败: {}", e))?
        .tspl;
    let (normalized, warnings) = normalize_tspl_print_config(&raw);
    for warning in warnings {
        log::warn!("TSPL参数回退: {}", warning);
    }
    Ok(normalized)
}

/// 打印机管理器状态
pub struct PrinterState {
    /// 布局引擎
    pub layout_engine: Arc<Mutex<LayoutEngine>>,
    /// 渲染管道
    pub render_pipeline: Arc<Mutex<RenderPipeline>>,
    /// PDF 后端
    pub pdf_backend: Arc<Mutex<PdfBackend>>,
    /// TSPL 生成器
    pub tspl_generator: Arc<Mutex<TSPLGenerator>>,
    /// 系统打印机后端（Windows/CUPS）
    #[cfg(target_os = "windows")]
    pub system_backend: Arc<Mutex<WindowsBackend>>,
    #[cfg(target_family = "unix")]
    pub system_backend: Arc<Mutex<CupsBackend>>,
}

impl PrinterState {
    /// 创建新的打印机状态
    pub fn new() -> Result<Self, String> {
        let layout_engine = LayoutEngine::new().map_err(|e| format!("创建布局引擎失败: {}", e))?;
        let render_pipeline =
            RenderPipeline::new().map_err(|e| format!("创建渲染管道失败: {}", e))?;
        let pdf_backend =
            PdfBackend::with_downloads_dir().map_err(|e| format!("创建PDF后端失败: {}", e))?;
        let tspl_generator = TSPLGenerator::new();

        // 初始化系统打印机后端
        #[cfg(target_os = "windows")]
        let system_backend = WindowsBackend::new();

        #[cfg(target_family = "unix")]
        let system_backend = CupsBackend::new();

        Ok(Self {
            layout_engine: Arc::new(Mutex::new(layout_engine)),
            render_pipeline: Arc::new(Mutex::new(render_pipeline)),
            pdf_backend: Arc::new(Mutex::new(pdf_backend)),
            tspl_generator: Arc::new(Mutex::new(tspl_generator)),
            system_backend: Arc::new(Mutex::new(system_backend)),
        })
    }

    /// 统一的图像打印接口
    ///
    /// 根据打印机名称自动路由到正确的后端
    ///
    /// # 参数
    /// - `printer_name`: 打印机名称
    /// - `image`: 灰度图像
    /// - `config`: 打印配置
    ///
    /// # 返回
    /// 打印结果消息
    pub fn print_image_to_printer(
        &self,
        printer_name: &str,
        image: &GrayImage,
        config: &ImagePrintConfig,
    ) -> Result<String, String> {
        log::info!("打印图像到打印机: {}", printer_name);

        // 根据打印机名称选择后端
        if printer_name == PDF_TEST_PRINTER_NAME {
            // 使用 PDF 后端
            let pdf_backend = self
                .pdf_backend
                .lock()
                .map_err(|e| format!("锁定 PDF 后端失败: {}", e))?;

            let result = pdf_backend
                .print_image(printer_name, image, config)
                .map_err(|e| format!("打印失败: {}", e))?;

            Ok(result.message)
        } else {
            // 使用系统后端
            let system_backend = self
                .system_backend
                .lock()
                .map_err(|e| format!("锁定系统打印机后端失败: {}", e))?;

            let result = system_backend
                .print_image(printer_name, image, config)
                .map_err(|e| format!("打印失败: {}", e))?;

            if let Some(job_id) = result.job_id {
                Ok(format!("{} (作业ID: {})", result.message, job_id))
            } else {
                Ok(result.message)
            }
        }
    }
}

/// 打印请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintRequest {
    /// 模板配置路径（可选，不提供则使用默认模板）
    pub template_path: Option<String>,
    /// 运行时数据（替换模板中的占位符）
    pub data: HashMap<String, String>,
}

/// 预览响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewResponse {
    /// PNG 文件路径
    pub png_path: String,
    /// base64 编码的图片数据
    pub base64_data: String,
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
pub async fn preview_qsl(
    request: PrintRequest,
    state: State<'_, PrinterState>,
) -> Result<PreviewResponse, String> {
    log::info!("开始生成 QSL 卡片预览");
    log::debug!("请求参数: {:?}", request);

    // 1. 加载模板配置
    let config = load_template_config(request.template_path.as_ref())?;

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
        mode: config.output.mode.clone(),
        threshold: config.output.threshold,
    };
    let render_result = render_pipeline
        .render(layout_result, &output_config)
        .map_err(|e| format!("渲染失败: {}", e))?;
    log::info!("✓ 渲染完成");

    // 5. 保存为 PNG（使用临时目录）
    let mut pdf_backend = PdfBackend::with_temp_dir()
        .map_err(|e| format!("创建临时PDF后端失败: {}", e))?;
    let png_path = pdf_backend
        .render_with_prefix(render_result, "qsl_preview")
        .map_err(|e| format!("保存PNG失败: {}", e))?;

    // 6. 读取图像并转换为 base64
    let img = image::open(&png_path).map_err(|e| format!("读取图像失败: {}", e))?;
    let img_data = std::fs::read(&png_path).map_err(|e| format!("读取文件失败: {}", e))?;
    let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &img_data);

    log::info!("✅ QSL 预览生成成功: {}", png_path.display());

    Ok(PreviewResponse {
        png_path: png_path.to_string_lossy().to_string(),
        base64_data,
        width: img.width(),
        height: img.height(),
    })
}

/// 地址标签预览请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressPreviewRequest {
    /// 运行时数据
    pub data: HashMap<String, String>,
}

/// 生成地址标签预览（PNG）
///
/// # 参数
/// - `request`: 预览请求参数
///
/// # 返回
/// 预览响应（包含 base64 图片数据）
#[tauri::command]
pub async fn preview_address(
    request: AddressPreviewRequest,
    state: State<'_, PrinterState>,
) -> Result<PreviewResponse, String> {
    log::info!("开始生成地址标签预览");
    log::debug!("请求参数: {:?}", request);

    // 1. 加载地址模板配置
    let mut config = load_address_template_config()?;

    // 如果数据中没有 name 或为空，移除 name 元素（不打印姓名）
    let has_name = request
        .data
        .get("name")
        .map_or(false, |n| !n.trim().is_empty());
    if !has_name {
        config.elements.retain(|e| e.id != "name");
    }

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
        mode: config.output.mode.clone(),
        threshold: config.output.threshold,
    };
    let render_result = render_pipeline
        .render(layout_result, &output_config)
        .map_err(|e| format!("渲染失败: {}", e))?;
    log::info!("✓ 渲染完成");

    // 5. 保存为 PNG（使用临时目录）
    let mut pdf_backend = PdfBackend::with_temp_dir()
        .map_err(|e| format!("创建临时PDF后端失败: {}", e))?;
    let png_path = pdf_backend
        .render_with_prefix(render_result, "address_preview")
        .map_err(|e| format!("保存PNG失败: {}", e))?;

    // 6. 读取图像并转换为 base64
    let img = image::open(&png_path).map_err(|e| format!("读取图像失败: {}", e))?;
    let img_data = std::fs::read(&png_path).map_err(|e| format!("读取文件失败: {}", e))?;
    let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &img_data);

    log::info!("✅ 地址标签预览生成成功: {}", png_path.display());

    Ok(PreviewResponse {
        png_path: png_path.to_string_lossy().to_string(),
        base64_data,
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
pub async fn print_qsl(
    printer_name: String,
    request: PrintRequest,
    state: State<'_, PrinterState>,
    profile_state: State<'_, ProfileState>,
) -> Result<(), String> {
    log::info!("开始打印 QSL 卡片: 打印机={}", printer_name);
    log::debug!("请求参数: {:?}", request);

    // 1. 加载模板配置
    let config = load_template_config(request.template_path.as_ref())?;

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
    // 注意：打印时使用模板文件中保存的 output 配置，而不是前端传入的参数
    // 这样用户在模板配置页面修改后，打印会自动使用新配置
    let mut render_pipeline = state
        .render_pipeline
        .lock()
        .map_err(|e| format!("锁定渲染管道失败: {}", e))?;
    let output_config = OutputConfig {
        mode: config.output.mode.clone(),
        threshold: config.output.threshold,
    };
    log::info!("使用渲染模式: {}", output_config.mode);
    let render_result = render_pipeline
        .render(layout_result, &output_config)
        .map_err(|e| format!("渲染失败: {}", e))?;

    // 5. 判断打印机类型并执行相应操作
    if printer_name == PDF_TEST_PRINTER_NAME {
        // PDF 测试打印机：保存为 PNG 文件
        log::info!("使用 PDF 测试打印机，保存为 PNG 文件");
        let mut pdf_backend = state
            .pdf_backend
            .lock()
            .map_err(|e| format!("锁定PDF后端失败: {}", e))?;
        let png_path = pdf_backend
            .render_with_prefix(render_result, "qsl")
            .map_err(|e| format!("保存PNG失败: {}", e))?;

        log::info!("✅ 打印成功（已保存为PNG）: {}", png_path.display());
    } else {
        // 真实打印机：生成 TSPL 并发送
        log::info!("使用真实打印机: {}", printer_name);
        let tspl_config = load_tspl_print_config(&profile_state)?;
        log::info!(
            "QSL打印生效TSPL参数: GAP {} mm, {} mm; DIRECTION {}",
            tspl_config.gap_mm, tspl_config.gap_offset_mm, tspl_config.direction
        );

        // 生成 TSPL 指令
        let tspl_generator = state
            .tspl_generator
            .lock()
            .map_err(|e| format!("锁定TSPL生成器失败: {}", e))?;
        let tspl = tspl_generator
            .generate_with_options(
                render_result,
                config.page.width_mm,
                config.page.height_mm,
                tspl_config.gap_mm,
                tspl_config.gap_offset_mm,
                &tspl_config.direction,
            )
            .map_err(|e| format!("生成TSPL指令失败: {}", e))?;

        log::debug!("TSPL指令长度: {} 字节", tspl.len());

        // 在调试模式下保存 TSPL 到文件
        #[cfg(debug_assertions)]
        {
            let debug_path = std::path::Path::new("output/debug_tspl.txt");
            if let Some(parent) = debug_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(e) = std::fs::write(debug_path, &tspl) {
                log::warn!("无法保存调试TSPL文件: {}", e);
            } else {
                log::info!("调试: TSPL已保存到 {}", debug_path.display());
            }
        }

        // 发送到打印机
        let system_backend = state
            .system_backend
            .lock()
            .map_err(|e| format!("锁定系统打印机后端失败: {}", e))?;

        let print_result = system_backend
            .send_raw(&printer_name, &tspl)
            .map_err(|e| format!("发送到打印机失败: {}", e))?;

        // 记录详细的打印结果
        if let Some(job_id) = &print_result.job_id {
            log::info!("✅ TSPL指令已发送到打印机: {}, 作业ID: {}", printer_name, job_id);
        } else {
            log::info!("✅ TSPL指令已发送到打印机: {}", printer_name);
        }
        if let Some(details) = &print_result.details {
            log::debug!("打印详情: {}", details);
        }
    }

    Ok(())
}

/// 地址打印请求
#[derive(Debug, Deserialize, Serialize)]
pub struct AddressPrintRequest {
    /// 姓名（可选）
    pub name: Option<String>,
    /// 呼号
    pub callsign: String,
    /// 地址（中文或英文地址，前端负责选择）
    pub address: String,
}

/// 打印地址标签
///
/// 使用地址模板打印地址信息，支持双份打印（上下各一份）
///
/// # 参数
/// - `printer_name`: 打印机名称
/// - `request`: 地址打印请求
///
/// # 返回
/// 成功或错误信息
#[tauri::command]
pub async fn print_address(
    printer_name: String,
    request: AddressPrintRequest,
    state: State<'_, PrinterState>,
    profile_state: State<'_, ProfileState>,
) -> Result<(), String> {
    log::info!("开始打印地址标签: 打印机={}", printer_name);
    log::debug!("地址打印请求: {:?}", request);

    // 1. 加载地址模板配置
    let mut config = load_address_template_config()?;

    // 2. 构建数据映射
    let mut data: HashMap<String, String> = HashMap::new();
    data.insert("callsign".to_string(), request.callsign.clone());
    // 将地址中的逗号替换为换行，便于多行打印
    let address = request.address.replace("，", "\n").replace(", ", "\n");
    data.insert("address".to_string(), address);

    // 如果有姓名则加入数据，否则从模板中移除 name 元素（不打印姓名）
    let has_name = request
        .name
        .as_ref()
        .map_or(false, |n| !n.trim().is_empty());
    if has_name {
        data.insert("name".to_string(), request.name.clone().unwrap());
    } else {
        config.elements.retain(|e| e.id != "name");
        log::info!("姓名为空，跳过打印姓名元素");
    }

    // 3. 模板解析
    let resolved_elements = TemplateEngine::resolve(&config, &data)
        .map_err(|e| format!("模板解析失败: {}", e))?;

    // 4. 布局计算
    let mut layout_engine = state
        .layout_engine
        .lock()
        .map_err(|e| format!("锁定布局引擎失败: {}", e))?;
    let layout_result = layout_engine
        .layout(&config, resolved_elements)
        .map_err(|e| format!("布局计算失败: {}", e))?;

    // 5. 渲染
    let mut render_pipeline = state
        .render_pipeline
        .lock()
        .map_err(|e| format!("锁定渲染管道失败: {}", e))?;
    let output_config = OutputConfig {
        mode: config.output.mode.clone(),
        threshold: config.output.threshold,
    };
    let render_result = render_pipeline
        .render(layout_result, &output_config)
        .map_err(|e| format!("渲染失败: {}", e))?;

    // 6. 判断打印机类型并执行相应操作
    if printer_name == PDF_TEST_PRINTER_NAME {
        // PDF 测试打印机：保存为 PNG 文件
        log::info!("使用 PDF 测试打印机，保存为 PNG 文件");
        let mut pdf_backend = state
            .pdf_backend
            .lock()
            .map_err(|e| format!("锁定PDF后端失败: {}", e))?;
        let png_path = pdf_backend
            .render_with_prefix(render_result, "address")
            .map_err(|e| format!("保存PNG失败: {}", e))?;

        log::info!("✅ 地址标签打印成功（已保存为PNG）: {}", png_path.display());
    } else {
        // 真实打印机：生成 TSPL 并发送
        log::info!("使用真实打印机: {}", printer_name);
        let tspl_config = load_tspl_print_config(&profile_state)?;
        log::info!(
            "地址打印生效TSPL参数: GAP {} mm, {} mm; DIRECTION {}",
            tspl_config.gap_mm, tspl_config.gap_offset_mm, tspl_config.direction
        );

        // 生成 TSPL 指令
        let tspl_generator = state
            .tspl_generator
            .lock()
            .map_err(|e| format!("锁定TSPL生成器失败: {}", e))?;
        let tspl = tspl_generator
            .generate_with_options(
                render_result,
                config.page.width_mm,
                config.page.height_mm,
                tspl_config.gap_mm,
                tspl_config.gap_offset_mm,
                &tspl_config.direction,
            )
            .map_err(|e| format!("生成TSPL指令失败: {}", e))?;

        log::debug!("地址标签TSPL指令长度: {} 字节", tspl.len());

        // 在调试模式下保存 TSPL 到文件
        #[cfg(debug_assertions)]
        {
            let debug_path = std::path::Path::new("output/debug_address_tspl.txt");
            if let Some(parent) = debug_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(e) = std::fs::write(debug_path, &tspl) {
                log::warn!("无法保存调试地址TSPL文件: {}", e);
            } else {
                log::info!("调试: 地址TSPL已保存到 {}", debug_path.display());
            }
        }

        // 发送到打印机
        let system_backend = state
            .system_backend
            .lock()
            .map_err(|e| format!("锁定系统打印机后端失败: {}", e))?;

        let print_result = system_backend
            .send_raw(&printer_name, &tspl)
            .map_err(|e| format!("发送到打印机失败: {}", e))?;

        // 记录详细的打印结果
        if let Some(job_id) = &print_result.job_id {
            log::info!("✅ 地址标签TSPL指令已发送到打印机: {}, 作业ID: {}", printer_name, job_id);
        } else {
            log::info!("✅ 地址标签TSPL指令已发送到打印机: {}", printer_name);
        }
        if let Some(details) = &print_result.details {
            log::debug!("打印详情: {}", details);
        }
    }

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
pub async fn generate_tspl(
    request: PrintRequest,
    state: State<'_, PrinterState>,
) -> Result<String, String> {
    log::info!("生成 TSPL 指令");

    // 1. 加载模板配置（每次都重新从文件读取）
    let config = load_template_config(request.template_path.as_ref())?;

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
        mode: config.output.mode.clone(),
        threshold: config.output.threshold,
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

    // 转换为字符串用于调试显示（包含二进制数据，使用 lossy 转换）
    Ok(String::from_utf8_lossy(&tspl).into_owned())
}

/// 加载模板配置
///
/// # 参数
/// - `path`: 模板文件路径（可选，不提供则返回默认模板）
///
/// # 返回
/// 模板配置 JSON
#[tauri::command]
pub async fn load_template(path: Option<String>) -> Result<String, String> {
    log::info!("加载模板配置");

    let config = if let Some(path) = path {
        log::debug!("从文件加载: {}", path);
        TemplateConfig::load_from_file(std::path::Path::new(&path))
            .map_err(|e| format!("加载模板失败: {}", e))?
    } else {
        log::debug!("使用默认模板");
        TemplateConfig::default_qsl_card()
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
pub async fn save_template(path: String, config_json: String) -> Result<(), String> {
    log::info!("保存模板配置: {}", path);

    // 反序列化
    let config: TemplateConfig =
        serde_json::from_str(&config_json).map_err(|e| format!("解析模板配置失败: {}", e))?;

    // 保存到文件
    config
        .save_to_file(std::path::Path::new(&path))
        .map_err(|e| format!("保存模板失败: {}", e))?;

    log::info!("✅ 模板保存成功");
    Ok(())
}

/// 获取模板配置
///
/// 读取当前默认模板配置
///
/// # 返回
/// 模板配置结构
#[tauri::command]
pub async fn get_template_config() -> Result<TemplateConfig, String> {
    log::info!("获取模板配置");

    let template_path = get_callsign_template_path();
    log::debug!("模板路径: {}", template_path.display());

    TemplateConfig::load_from_file(&template_path)
        .map_err(|e| format!("读取模板配置失败: {}", e))
}

/// 保存模板配置
///
/// 将模板配置保存到默认模板文件
///
/// # 参数
/// - `config`: 模板配置结构
///
/// # 返回
/// 成功或错误信息
#[tauri::command]
pub async fn save_template_config(config: TemplateConfig) -> Result<(), String> {
    log::info!("保存模板配置");

    let template_path = get_callsign_template_path();
    log::debug!("保存路径: {}", template_path.display());

    // 确保目录存在
    if let Some(parent) = template_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }

    // 序列化为 TOML
    let toml_str = toml::to_string_pretty(&config)
        .map_err(|e| format!("序列化模板配置失败: {}", e))?;

    // 写入文件
    std::fs::write(&template_path, toml_str)
        .map_err(|e| format!("写入模板文件失败: {}", e))?;

    log::info!("✅ 模板配置保存成功: {}", template_path.display());
    Ok(())
}

/// 获取地址模板配置
///
/// 读取当前地址模板配置
///
/// # 返回
/// 模板配置结构
#[tauri::command]
pub async fn get_address_template_config() -> Result<TemplateConfig, String> {
    log::info!("获取地址模板配置");

    let template_path = get_address_template_path();
    log::debug!("地址模板路径: {}", template_path.display());

    TemplateConfig::load_from_file(&template_path)
        .map_err(|e| format!("读取地址模板配置失败: {}", e))
}

/// 保存地址模板配置
///
/// 将模板配置保存到地址模板文件
///
/// # 参数
/// - `config`: 模板配置结构
///
/// # 返回
/// 成功或错误信息
#[tauri::command]
pub async fn save_address_template_config(config: TemplateConfig) -> Result<(), String> {
    log::info!("保存地址模板配置");

    let template_path = get_address_template_path();
    log::debug!("保存路径: {}", template_path.display());

    // 确保目录存在
    if let Some(parent) = template_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }

    // 序列化为 TOML
    let toml_str = toml::to_string_pretty(&config)
        .map_err(|e| format!("序列化地址模板配置失败: {}", e))?;

    // 写入文件
    std::fs::write(&template_path, toml_str)
        .map_err(|e| format!("写入地址模板文件失败: {}", e))?;

    log::info!("✅ 地址模板配置保存成功: {}", template_path.display());
    Ok(())
}

/// 获取打印机列表
///
/// 返回系统打印机和 PDF 测试打印机的列表
#[tauri::command]
pub async fn get_printers(state: State<'_, PrinterState>) -> Result<Vec<String>, String> {
    log::info!("获取打印机列表");

    let mut printers = Vec::new();

    // 获取系统打印机
    let system_backend = state
        .system_backend
        .lock()
        .map_err(|e| format!("锁定系统打印机后端失败: {}", e))?;

    match system_backend.list_printers() {
        Ok(system_printers) => {
            log::info!("✓ 找到 {} 个系统打印机", system_printers.len());
            printers.extend(system_printers);
        }
        Err(e) => {
            log::warn!("获取系统打印机失败: {}", e);
        }
    }

    // 添加 PDF 测试打印机
    let pdf_backend = state
        .pdf_backend
        .lock()
        .map_err(|e| format!("锁定PDF后端失败: {}", e))?;

    match pdf_backend.list_printers() {
        Ok(pdf_printers) => {
            printers.extend(pdf_printers);
        }
        Err(e) => {
            log::warn!("获取PDF打印机失败: {}", e);
        }
    }

    log::info!("✅ 打印机列表获取成功: {} 个打印机", printers.len());

    Ok(printers)
}
