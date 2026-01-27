// 顺丰速运 Tauri 命令
//
// 提供给前端调用的 API
//
// 打印流程分为两步：
// 1. sf_fetch_waybill - 获取 PDF 并返回预览图像
// 2. sf_print_waybill - 将 PDF 转换为 TSPL 并发送到打印机

use base64::{Engine as _, engine::general_purpose::STANDARD};
use crate::security::{delete_credential, get_credential, save_credential};
use crate::sf_express::models::credential_keys;
use crate::sf_express::{
    PdfRenderer, SFExpressClient, SFExpressConfig,
    CreateOrderRequest, UpdateOrderRequest, SearchOrderRequest,
    ContactInfo, CargoDetail, WaybillNoInfo, SenderInfo, SFOrder, SFOrderWithCard, OrderStatus,
};
use crate::db;
use crate::printer::backend::ImagePrintConfig;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use std::path::PathBuf;

use crate::commands::printer::PrinterState;

/// 顺丰配置响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SFConfigResponse {
    /// 环境（production/sandbox）
    pub environment: String,
    /// 顾客编码
    pub partner_id: String,
    /// 模板编码
    pub template_code: String,
    /// 是否已配置生产校验码
    pub has_prod_checkword: bool,
    /// 是否已配置沙箱校验码
    pub has_sandbox_checkword: bool,
    /// 是否使用默认参数
    pub use_default: bool,
}

/// 获取面单响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchWaybillResponse {
    /// 预览图像（Base64 编码的 PNG）
    pub preview_image: String,
    /// PDF 数据（Base64 编码）
    pub pdf_data: String,
    /// 运单号
    pub waybill_no: String,
}

/// 保存顺丰配置
#[tauri::command]
pub fn sf_save_config(
    environment: String,
    partner_id: String,
    checkword_prod: Option<String>,
    checkword_sandbox: Option<String>,
) -> Result<(), String> {
    log::info!("保存顺丰配置: environment={}, partner_id={}", environment, partner_id);

    // 保存环境配置
    save_credential(credential_keys::ENVIRONMENT, &environment)
        .map_err(|e| format!("保存环境配置失败: {}", e))?;

    // 保存顾客编码
    save_credential(credential_keys::PARTNER_ID, &partner_id)
        .map_err(|e| format!("保存顾客编码失败: {}", e))?;

    // 标记为使用自定义参数
    save_credential(credential_keys::USE_DEFAULT, "false")
        .map_err(|e| format!("保存配置模式失败: {}", e))?;

    // 保存生产校验码（如果提供）
    if let Some(checkword) = checkword_prod {
        if !checkword.is_empty() {
            save_credential(credential_keys::CHECKWORD_PROD, &checkword)
                .map_err(|e| format!("保存生产校验码失败: {}", e))?;
        }
    }

    // 保存沙箱校验码（如果提供）
    if let Some(checkword) = checkword_sandbox {
        if !checkword.is_empty() {
            save_credential(credential_keys::CHECKWORD_SANDBOX, &checkword)
                .map_err(|e| format!("保存沙箱校验码失败: {}", e))?;
        }
    }

    log::info!("顺丰配置保存成功");
    Ok(())
}

/// 加载顺丰配置
#[tauri::command]
pub fn sf_load_config() -> Result<SFConfigResponse, String> {
    log::info!("加载顺丰配置");

    let environment = get_credential(credential_keys::ENVIRONMENT)
        .map_err(|e| format!("加载环境配置失败: {}", e))?
        .unwrap_or_else(|| "sandbox".to_string());

    let partner_id = get_credential(credential_keys::PARTNER_ID)
        .map_err(|e| format!("加载顾客编码失败: {}", e))?
        .unwrap_or_default();

    let has_prod_checkword = get_credential(credential_keys::CHECKWORD_PROD)
        .map_err(|e| format!("检查生产校验码失败: {}", e))?
        .is_some();

    let has_sandbox_checkword = get_credential(credential_keys::CHECKWORD_SANDBOX)
        .map_err(|e| format!("检查沙箱校验码失败: {}", e))?
        .is_some();

    // 加载配置模式
    let use_default = get_credential(credential_keys::USE_DEFAULT)
        .map_err(|e| format!("加载配置模式失败: {}", e))?
        .map(|v| v == "true")
        .unwrap_or(false);

    // 动态生成模板编码
    let template_code = if partner_id.is_empty() {
        "fm_76130_standard_{partnerID}".to_string()
    } else if use_default {
        // 如果使用默认参数，脱敏模板编码
        let masked_id = if partner_id.chars().count() >= 6 {
            let start: String = partner_id.chars().take(3).collect();
            let end: String = partner_id.chars().skip(partner_id.chars().count() - 3).collect();
            format!("{}***{}", start, end)
        } else {
            "***".to_string()
        };
        format!("fm_76130_standard_{}", masked_id)
    } else {
        format!("fm_76130_standard_{}", partner_id)
    };

    let response = SFConfigResponse {
        environment,
        partner_id,
        template_code,
        has_prod_checkword,
        has_sandbox_checkword,
        use_default,
    };

    log::info!("顺丰配置加载成功: {:?}", response);
    Ok(response)
}

/// 清除顺丰配置
#[tauri::command]
pub fn sf_clear_config() -> Result<(), String> {
    log::info!("清除顺丰配置");

    // 删除所有凭据（忽略不存在的情况）
    let _ = delete_credential(credential_keys::ENVIRONMENT);
    let _ = delete_credential(credential_keys::PARTNER_ID);
    let _ = delete_credential(credential_keys::CHECKWORD_PROD);
    let _ = delete_credential(credential_keys::CHECKWORD_SANDBOX);
    let _ = delete_credential(credential_keys::USE_DEFAULT);

    log::info!("顺丰配置已清除");
    Ok(())
}

/// 默认 API 配置响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SFDefaultApiConfig {
    /// 是否启用默认参数
    pub enabled: bool,
    /// 顾客编码（脱敏显示，只显示前3位和后3位）
    pub partner_id_masked: String,
    /// 是否有沙箱校验码
    pub has_sandbox_checkword: bool,
    /// 是否有生产校验码
    pub has_prod_checkword: bool,
}

/// 默认 API 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SFDefaultApiConfigFile {
    enabled: bool,
    partner_id: String,
    checkword_sandbox: String,
    checkword_prod: String,
}

/// 获取默认 API 配置路径
fn get_default_api_config_path() -> PathBuf {
    // 开发模式：使用项目根目录的 config/sf_express_default.toml
    #[cfg(debug_assertions)]
    {
        PathBuf::from("config/sf_express_default.toml")
    }

    // 生产模式：使用系统配置目录
    #[cfg(not(debug_assertions))]
    {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        let config_dir = if cfg!(target_os = "windows") {
            // Windows: %APPDATA%/qsl-cardhub
            home_dir.join("AppData").join("Roaming").join("qsl-cardhub")
        } else if cfg!(target_os = "macos") {
            // macOS: ~/Library/Application Support/qsl-cardhub
            home_dir
                .join("Library")
                .join("Application Support")
                .join("qsl-cardhub")
        } else {
            // Linux: ~/.config/qsl-cardhub
            home_dir.join(".config").join("qsl-cardhub")
        };

        config_dir.join("sf_express_default.toml")
    }
}

/// 获取默认 API 配置（用于前端展示）
#[tauri::command]
pub fn sf_get_default_api_config() -> Result<SFDefaultApiConfig, String> {
    log::info!("获取默认 API 配置");

    let config_path = get_default_api_config_path();
    log::debug!("默认配置文件路径: {:?}", config_path);

    if !config_path.exists() {
        log::info!("默认配置文件不存在，返回禁用状态");
        return Ok(SFDefaultApiConfig {
            enabled: false,
            partner_id_masked: String::new(),
            has_sandbox_checkword: false,
            has_prod_checkword: false,
        });
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    let config: SFDefaultApiConfigFile = toml::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;

    // 脱敏处理顾客编码
    let partner_id_masked = if config.partner_id.chars().count() >= 6 {
        let start: String = config.partner_id.chars().take(3).collect();
        let end: String = config.partner_id.chars().skip(config.partner_id.chars().count() - 3).collect();
        format!("{}***{}", start, end)
    } else if !config.partner_id.is_empty() {
        "***".to_string()
    } else {
        String::new()
    };

    let result = SFDefaultApiConfig {
        enabled: config.enabled && !config.partner_id.is_empty(),
        partner_id_masked,
        has_sandbox_checkword: !config.checkword_sandbox.is_empty(),
        has_prod_checkword: !config.checkword_prod.is_empty(),
    };

    log::info!("默认配置加载成功: enabled={}, has_sandbox={}, has_prod={}",
        result.enabled, result.has_sandbox_checkword, result.has_prod_checkword);

    Ok(result)
}

/// 使用默认 API 配置保存到凭据存储
#[tauri::command]
pub fn sf_apply_default_api_config(environment: String) -> Result<(), String> {
    log::info!("应用默认 API 配置: environment={}", environment);

    let config_path = get_default_api_config_path();

    if !config_path.exists() {
        return Err("默认配置文件不存在".to_string());
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    let config: SFDefaultApiConfigFile = toml::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;

    if !config.enabled || config.partner_id.is_empty() {
        return Err("默认配置未启用或参数无效".to_string());
    }

    // 保存环境配置
    save_credential(credential_keys::ENVIRONMENT, &environment)
        .map_err(|e| format!("保存环境配置失败: {}", e))?;

    // 保存顾客编码
    save_credential(credential_keys::PARTNER_ID, &config.partner_id)
        .map_err(|e| format!("保存顾客编码失败: {}", e))?;

    // 标记为使用默认参数
    save_credential(credential_keys::USE_DEFAULT, "true")
        .map_err(|e| format!("保存配置模式失败: {}", e))?;

    // 根据环境保存对应的校验码
    if environment == "production" && !config.checkword_prod.is_empty() {
        save_credential(credential_keys::CHECKWORD_PROD, &config.checkword_prod)
            .map_err(|e| format!("保存生产校验码失败: {}", e))?;
    }

    if environment == "sandbox" && !config.checkword_sandbox.is_empty() {
        save_credential(credential_keys::CHECKWORD_SANDBOX, &config.checkword_sandbox)
            .map_err(|e| format!("保存沙箱校验码失败: {}", e))?;
    }

    log::info!("默认 API 配置应用成功");
    Ok(())
}

/// 获取面单（步骤1）
///
/// 调用顺丰 API 获取 PDF 并返回预览图像和 PDF 数据。
/// 用户确认预览后，可调用 sf_print_waybill 进行打印。
#[tauri::command]
pub fn sf_fetch_waybill(waybill_no: String) -> Result<FetchWaybillResponse, String> {
    log::info!("获取顺丰面单: {}", waybill_no);

    // 加载配置
    let environment = get_credential(credential_keys::ENVIRONMENT)
        .map_err(|e| format!("加载环境配置失败: {}", e))?
        .unwrap_or_else(|| "sandbox".to_string());

    let partner_id = get_credential(credential_keys::PARTNER_ID)
        .map_err(|e| format!("加载顾客编码失败: {}", e))?
        .ok_or("未配置顾客编码")?;

    if partner_id.is_empty() {
        return Err("未配置顾客编码".to_string());
    }

    // 根据环境获取校验码
    let checkword_key = if environment == "production" {
        credential_keys::CHECKWORD_PROD
    } else {
        credential_keys::CHECKWORD_SANDBOX
    };

    let checkword = get_credential(checkword_key)
        .map_err(|e| format!("加载校验码失败: {}", e))?
        .ok_or_else(|| format!("未配置{}校验码", if environment == "production" { "生产" } else { "沙箱" }))?;

    if checkword.is_empty() {
        return Err(format!("未配置{}校验码", if environment == "production" { "生产" } else { "沙箱" }));
    }

    // 创建配置
    let config = SFExpressConfig {
        environment: environment.clone(),
        partner_id,
    };

    // 创建客户端
    let client = SFExpressClient::new(config, checkword)
        .map_err(|e| format!("创建 API 客户端失败: {}", e))?;

    // 调用 API 获取 PDF
    let pdf_data = client.get_waybill_pdf(&waybill_no)
        .map_err(|e| format!("获取面单失败: {}", e))?;

    // 渲染 PDF 为预览图像
    let renderer = PdfRenderer::new();
    let preview_image = renderer.render_pdf_to_preview(&pdf_data)
        .map_err(|e| format!("渲染预览图像失败: {}", e))?;

    // 将 PDF 数据编码为 Base64
    let pdf_data_base64 = STANDARD.encode(&pdf_data);

    log::info!("面单获取成功，运单号: {}, PDF大小: {} 字节", waybill_no, pdf_data.len());

    Ok(FetchWaybillResponse {
        preview_image,
        pdf_data: pdf_data_base64,
        waybill_no,
    })
}

/// 打印面单（步骤2）
///
/// 接收已获取的 PDF 数据（Base64 编码），渲染为图像并通过统一的打印接口发送。
/// 需要先调用 sf_fetch_waybill 获取 PDF 数据。
#[tauri::command]
pub fn sf_print_waybill(
    pdf_data: String,
    printer_name: String,
    printer_state: State<'_, PrinterState>,
) -> Result<String, String> {
    log::info!("打印顺丰面单到打印机: {}", printer_name);

    // 解码 PDF 数据
    let pdf_bytes = STANDARD.decode(&pdf_data)
        .map_err(|e| format!("解码 PDF 数据失败: {}", e))?;

    log::info!("PDF 数据解码成功，大小: {} 字节", pdf_bytes.len());

    // 渲染 PDF 为灰度图像
    let renderer = PdfRenderer::new();
    let gray_image = renderer.render_pdf_to_grayscale(&pdf_bytes)
        .map_err(|e| format!("渲染面单图像失败: {}", e))?;

    log::info!("面单渲染完成，图像尺寸: {}x{}", gray_image.width(), gray_image.height());

    // 配置打印参数（顺丰面单规格：76mm x 130mm）
    let config = ImagePrintConfig {
        width_mm: 76.0,
        height_mm: 130.0,
        dpi: 203,
    };

    // 使用统一的打印接口
    printer_state.print_image_to_printer(&printer_name, &gray_image, &config)
}

// ==================== 寄件人管理命令 ====================

/// 创建寄件人
#[tauri::command]
pub fn sf_create_sender(
    name: String,
    phone: String,
    mobile: Option<String>,
    province: String,
    city: String,
    district: String,
    address: String,
    is_default: bool,
) -> Result<SenderInfo, String> {
    log::info!("创建寄件人: {}", name);
    db::create_sender(name, phone, mobile, province, city, district, address, is_default)
        .map_err(|e| format!("创建寄件人失败: {}", e))
}

/// 更新寄件人
#[tauri::command]
pub fn sf_update_sender(
    id: String,
    name: String,
    phone: String,
    mobile: Option<String>,
    province: String,
    city: String,
    district: String,
    address: String,
    is_default: bool,
) -> Result<SenderInfo, String> {
    log::info!("更新寄件人: {}", id);
    db::update_sender(&id, name, phone, mobile, province, city, district, address, is_default)
        .map_err(|e| format!("更新寄件人失败: {}", e))
}

/// 删除寄件人
#[tauri::command]
pub fn sf_delete_sender(id: String) -> Result<(), String> {
    log::info!("删除寄件人: {}", id);
    db::delete_sender(&id)
        .map_err(|e| format!("删除寄件人失败: {}", e))
}

/// 获取寄件人列表
#[tauri::command]
pub fn sf_list_senders() -> Result<Vec<SenderInfo>, String> {
    log::info!("获取寄件人列表");
    db::list_senders()
        .map_err(|e| format!("获取寄件人列表失败: {}", e))
}

/// 获取默认寄件人
#[tauri::command]
pub fn sf_get_default_sender() -> Result<Option<SenderInfo>, String> {
    log::info!("获取默认寄件人");
    db::get_default_sender()
        .map_err(|e| format!("获取默认寄件人失败: {}", e))
}

/// 设置默认寄件人
#[tauri::command]
pub fn sf_set_default_sender(id: String) -> Result<SenderInfo, String> {
    log::info!("设置默认寄件人: {}", id);
    db::set_default_sender(&id)
        .map_err(|e| format!("设置默认寄件人失败: {}", e))
}

// ==================== 下单命令 ====================

/// 下单请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderParams {
    /// 寄件人 ID（从数据库加载）
    pub sender_id: String,
    /// 收件人信息
    pub recipient: RecipientInfoParams,
    /// 托寄物名称（可选，默认"QSL卡片"）
    pub cargo_name: Option<String>,
    /// 关联的卡片 ID（可选）
    pub card_id: Option<String>,
    /// 付款方式：1=寄方付, 2=收方付（可选，默认寄方付）
    pub pay_method: Option<i32>,
}

/// 收件人参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipientInfoParams {
    pub name: String,
    pub phone: String,
    pub mobile: Option<String>,
    pub province: String,
    pub city: String,
    pub district: String,
    pub address: String,
}

/// 联系人展示信息（用于前端确认页面）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactDisplayInfo {
    /// 姓名
    pub name: String,
    /// 电话
    pub phone: String,
    /// 完整地址（省市区+详细地址）
    pub full_address: String,
}

/// 下单响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    /// 客户订单号
    pub order_id: String,
    /// 运单号列表
    pub waybill_no_list: Vec<String>,
    /// 筛单结果：1=人工确认, 2=可收派, 3=不可收派
    pub filter_result: Option<i32>,
    /// 本地订单记录
    pub local_order: SFOrder,
    /// 寄件人展示信息
    pub sender_info: ContactDisplayInfo,
    /// 收件人展示信息
    pub recipient_info: ContactDisplayInfo,
    /// 托寄物名称
    pub cargo_name: String,
    /// 付款方式：1=寄方付, 2=收方付, 3=第三方付
    pub pay_method: i32,
    /// 快件产品类别：2=顺丰标快
    pub express_type_id: i32,
    /// 原寄地区域代码
    pub origin_code: Option<String>,
    /// 目的地区域代码
    pub dest_code: Option<String>,
}

/// 辅助函数：获取 API 客户端
fn get_sf_client() -> Result<SFExpressClient, String> {
    let environment = get_credential(credential_keys::ENVIRONMENT)
        .map_err(|e| format!("加载环境配置失败: {}", e))?
        .unwrap_or_else(|| "sandbox".to_string());

    let partner_id = get_credential(credential_keys::PARTNER_ID)
        .map_err(|e| format!("加载顾客编码失败: {}", e))?
        .ok_or("未配置顾客编码")?;

    if partner_id.is_empty() {
        return Err("未配置顾客编码".to_string());
    }

    let checkword_key = if environment == "production" {
        credential_keys::CHECKWORD_PROD
    } else {
        credential_keys::CHECKWORD_SANDBOX
    };

    let checkword = get_credential(checkword_key)
        .map_err(|e| format!("加载校验码失败: {}", e))?
        .ok_or_else(|| format!("未配置{}校验码", if environment == "production" { "生产" } else { "沙箱" }))?;

    let config = SFExpressConfig {
        environment,
        partner_id,
    };

    SFExpressClient::new(config, checkword)
        .map_err(|e| format!("创建 API 客户端失败: {}", e))
}

/// 创建顺丰订单
#[tauri::command]
pub fn sf_create_order(params: CreateOrderParams) -> Result<CreateOrderResponse, String> {
    let cargo_name = params.cargo_name.unwrap_or_else(|| "QSL卡片".to_string());
    log::info!("创建顺丰订单: cargo={}, card_id={:?}", cargo_name, params.card_id);

    // 加载寄件人信息
    let sender = db::get_sender(&params.sender_id)
        .map_err(|e| format!("加载寄件人失败: {}", e))?
        .ok_or_else(|| "寄件人不存在".to_string())?;

    let client = get_sf_client()?;

    // 生成客户订单号
    let order_id = format!("QSL{}", Uuid::new_v4().to_string().replace("-", "")[..16].to_uppercase());

    // 构建下单请求
    let mut request = CreateOrderRequest::new(order_id.clone());
    request.pay_method = params.pay_method.or(Some(1));

    // 添加托寄物（国内件数量、单位、重量非必填）
    request.cargo_details.push(CargoDetail {
        name: cargo_name.clone(),
        count: 1,
        unit: None,
        weight: None,
        amount: None,
        currency: None,
    });

    // 添加寄件人
    request.contact_info_list.push(ContactInfo {
        contact_type: 1,
        contact: sender.name.clone(),
        tel: Some(sender.phone.clone()),
        mobile: sender.mobile.clone(),
        country: "CN".to_string(),
        province: Some(sender.province.clone()),
        city: Some(sender.city.clone()),
        county: Some(sender.district.clone()),
        address: sender.address.clone(),
        company: None,
    });

    // 添加收件人
    request.contact_info_list.push(ContactInfo {
        contact_type: 2,
        contact: params.recipient.name.clone(),
        tel: Some(params.recipient.phone.clone()),
        mobile: params.recipient.mobile.clone(),
        country: "CN".to_string(),
        province: Some(params.recipient.province.clone()),
        city: Some(params.recipient.city.clone()),
        county: Some(params.recipient.district.clone()),
        address: params.recipient.address.clone(),
        company: None,
    });

    // 调用 API
    let response = client.create_order(&request)
        .map_err(|e| format!("下单失败: {}", e))?;

    // 保存到本地数据库
    let sender_json = serde_json::to_string(&sender)
        .map_err(|e| format!("序列化寄件人失败: {}", e))?;
    let recipient_json = serde_json::to_string(&params.recipient)
        .map_err(|e| format!("序列化收件人失败: {}", e))?;

    let local_order = db::create_order(
        order_id.clone(),
        params.card_id,
        params.pay_method,
        Some(cargo_name.clone()),
        sender_json,
        recipient_json,
    ).map_err(|e| format!("保存订单失败: {}", e))?;

    // 提取运单号
    let waybill_no_list: Vec<String> = response.waybill_no_info_list
        .iter()
        .filter(|w| w.waybill_type == 1)
        .map(|w| w.waybill_no.clone())
        .collect();

    log::info!("订单创建成功: order_id={}, waybill_nos={:?}", order_id, waybill_no_list);

    // 构建展示信息
    let sender_info = ContactDisplayInfo {
        name: sender.name.clone(),
        phone: sender.phone.clone(),
        full_address: format!(
            "{}{}{}{}",
            sender.province, sender.city, sender.district, sender.address
        ),
    };

    let recipient_info = ContactDisplayInfo {
        name: params.recipient.name.clone(),
        phone: params.recipient.phone.clone(),
        full_address: format!(
            "{}{}{}{}",
            params.recipient.province,
            params.recipient.city,
            params.recipient.district,
            params.recipient.address
        ),
    };

    Ok(CreateOrderResponse {
        order_id: response.order_id,
        waybill_no_list,
        filter_result: response.filter_result,
        local_order,
        sender_info,
        recipient_info,
        cargo_name,
        pay_method: params.pay_method.unwrap_or(1),
        express_type_id: 2, // 顺丰标快
        origin_code: response.origin_code,
        dest_code: response.dest_code,
    })
}

/// 确认订单响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmOrderResponse {
    /// 客户订单号
    pub order_id: String,
    /// 运单号列表
    pub waybill_no_list: Vec<String>,
    /// 操作结果
    pub res_status: Option<i32>,
    /// 更新后的本地订单
    pub local_order: SFOrder,
}

/// 确认顺丰订单
#[tauri::command]
pub fn sf_confirm_order(order_id: String) -> Result<ConfirmOrderResponse, String> {
    log::info!("确认顺丰订单: {}", order_id);

    let client = get_sf_client()?;

    // 获取本地订单
    let local_order = db::get_order_by_order_id(&order_id)
        .map_err(|e| format!("获取订单失败: {}", e))?
        .ok_or_else(|| format!("订单不存在: {}", order_id))?;

    // 构建确认请求
    let request = UpdateOrderRequest {
        order_id: order_id.clone(),
        deal_type: 1, // 确认
        waybill_no_info_list: None, // 顺丰 API 默认自动确认，这里可以不传
    };

    // 调用 API
    let response = client.update_order(&request)
        .map_err(|e| format!("确认订单失败: {}", e))?;

    // 提取运单号
    let waybill_no_list: Vec<String> = response.waybill_no_info_list
        .iter()
        .filter(|w| w.waybill_type == 1)
        .map(|w| w.waybill_no.clone())
        .collect();

    let waybill_no = waybill_no_list.first().cloned();

    // 更新本地订单状态
    let updated_order = db::update_order_status(&order_id, OrderStatus::Confirmed, waybill_no.clone())
        .map_err(|e| format!("更新订单状态失败: {}", e))?;

    // 如果有关联卡片，回填运单号到卡片备注
    if let (Some(card_id), Some(waybill)) = (&updated_order.card_id, &waybill_no) {
        log::info!("回填运单号到卡片: card_id={}, waybill={}", card_id, waybill);
        // 这里可以调用卡片更新接口回填运单号
        // TODO: 实现卡片备注更新
    }

    log::info!("订单确认成功: order_id={}, waybill_nos={:?}", order_id, waybill_no_list);

    Ok(ConfirmOrderResponse {
        order_id: response.order_id,
        waybill_no_list,
        res_status: response.res_status,
        local_order: updated_order,
    })
}

/// 取消顺丰订单
#[tauri::command]
pub fn sf_cancel_order(order_id: String) -> Result<SFOrder, String> {
    log::info!("取消顺丰订单: {}", order_id);

    // 先获取本地订单信息，检查是否有运单号
    let local_order = db::get_order_by_order_id(&order_id)
        .map_err(|e| format!("查询订单失败: {}", e))?
        .ok_or_else(|| "订单不存在".to_string())?;

    let client = get_sf_client()?;

    // 构建取消请求，如果已有运单号则需要传递
    let waybill_no_info_list = if let Some(ref waybill_no) = local_order.waybill_no {
        Some(vec![WaybillNoInfo {
            waybill_type: 1,
            waybill_no: waybill_no.clone(),
        }])
    } else {
        Some(vec![])
    };

    let request = UpdateOrderRequest {
        order_id: order_id.clone(),
        deal_type: 2, // 取消
        waybill_no_info_list,
    };

    // 调用 API
    client.update_order(&request)
        .map_err(|e| format!("取消订单失败: {}", e))?;

    // 更新本地订单状态
    let updated_order = db::update_order_status(&order_id, OrderStatus::Cancelled, None)
        .map_err(|e| format!("更新订单状态失败: {}", e))?;

    log::info!("订单取消成功: {}", order_id);

    Ok(updated_order)
}

/// 订单查询响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOrderResponse {
    /// 客户订单号
    pub order_id: String,
    /// 运单号列表
    pub waybill_no_list: Vec<String>,
    /// 筛单结果
    pub filter_result: Option<String>,
    /// 原寄地代码
    pub origin_code: Option<String>,
    /// 目的地代码
    pub dest_code: Option<String>,
}

/// 查询顺丰订单
#[tauri::command]
pub fn sf_search_order(order_id: Option<String>, waybill_no: Option<String>) -> Result<SearchOrderResponse, String> {
    log::info!("查询顺丰订单: order_id={:?}, waybill_no={:?}", order_id, waybill_no);

    if order_id.is_none() && waybill_no.is_none() {
        return Err("订单号和运单号至少提供一个".to_string());
    }

    let client = get_sf_client()?;

    let request = SearchOrderRequest {
        order_id,
        main_waybill_no: waybill_no,
        search_type: Some("1".to_string()),
        language: Some("zh-CN".to_string()),
    };

    let response = client.search_order(&request)
        .map_err(|e| format!("查询订单失败: {}", e))?;

    let waybill_no_list: Vec<String> = response.waybill_no_info_list
        .iter()
        .map(|w| w.waybill_no.clone())
        .collect();

    Ok(SearchOrderResponse {
        order_id: response.order_id,
        waybill_no_list,
        filter_result: response.filter_result,
        origin_code: response.origincode,
        dest_code: response.destcode,
    })
}

// ==================== 订单列表命令 ====================

/// 订单列表查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListOrdersParams {
    pub status: Option<String>,
    pub card_id: Option<String>,
    pub page: u32,
    pub page_size: u32,
}

/// 订单列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListOrdersResponse {
    pub items: Vec<SFOrderWithCard>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

/// 获取订单列表
#[tauri::command]
pub fn sf_list_orders(params: ListOrdersParams) -> Result<ListOrdersResponse, String> {
    log::info!("获取订单列表: status={:?}, card_id={:?}, page={}", params.status, params.card_id, params.page);

    let filter = db::OrderFilter {
        status: params.status.and_then(|s| s.parse().ok()),
        card_id: params.card_id,
    };

    let pagination = db::OrderPagination {
        page: params.page,
        page_size: params.page_size,
    };

    let result = db::list_orders_with_cards(filter, pagination)
        .map_err(|e| format!("获取订单列表失败: {}", e))?;

    Ok(ListOrdersResponse {
        items: result.items,
        total: result.total,
        page: result.page,
        page_size: result.page_size,
        total_pages: result.total_pages,
    })
}

/// 获取单个订单
#[tauri::command]
pub fn sf_get_order(id: String) -> Result<Option<SFOrder>, String> {
    log::info!("获取订单: {}", id);
    db::get_order(&id)
        .map_err(|e| format!("获取订单失败: {}", e))
}

/// 根据订单号获取订单
#[tauri::command]
pub fn sf_get_order_by_order_id(order_id: String) -> Result<Option<SFOrder>, String> {
    log::info!("根据订单号获取订单: {}", order_id);
    db::get_order_by_order_id(&order_id)
        .map_err(|e| format!("获取订单失败: {}", e))
}

/// 根据卡片 ID 获取订单
#[tauri::command]
pub fn sf_get_order_by_card_id(card_id: String) -> Result<Option<SFOrder>, String> {
    log::info!("根据卡片 ID 获取订单: {}", card_id);
    db::get_order_by_card_id(&card_id)
        .map_err(|e| format!("获取订单失败: {}", e))
}

/// 删除订单
#[tauri::command]
pub fn sf_delete_order(id: String) -> Result<(), String> {
    log::info!("删除订单: {}", id);
    db::delete_order(&id)
        .map_err(|e| format!("删除订单失败: {}", e))
}

/// 更新订单状态为已打印
#[tauri::command]
pub fn sf_mark_order_printed(order_id: String) -> Result<SFOrder, String> {
    log::info!("标记订单为已打印: {}", order_id);

    let order = db::get_order_by_order_id(&order_id)
        .map_err(|e| format!("获取订单失败: {}", e))?
        .ok_or_else(|| format!("订单不存在: {}", order_id))?;

    db::update_order_status(&order_id, OrderStatus::Printed, order.waybill_no)
        .map_err(|e| format!("更新订单状态失败: {}", e))
}
