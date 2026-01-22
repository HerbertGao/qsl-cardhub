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
use crate::sf_express::{PdfRenderer, SFExpressClient, SFExpressConfig};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::commands::printer::PrinterState;
use crate::printer::backend::PrinterBackendTrait;

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

    let response = SFConfigResponse {
        environment,
        partner_id,
        template_code: "fm_76130_standard_HBTRJT0FNP6E".to_string(),
        has_prod_checkword,
        has_sandbox_checkword,
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

    log::info!("顺丰配置已清除");
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
        template_code: "fm_76130_standard_HBTRJT0FNP6E".to_string(),
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
/// 接收已获取的 PDF 数据（Base64 编码），转换为 TSPL 并发送到打印机。
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

    // 渲染 PDF 为 TSPL
    let renderer = PdfRenderer::new();
    let tspl = renderer.pdf_to_tspl(&pdf_bytes, Some(128))
        .map_err(|e| format!("渲染面单失败: {}", e))?;

    log::info!("TSPL 指令生成成功，长度: {} 字节", tspl.len());

    // 发送到打印机
    let system_backend = printer_state
        .system_backend
        .lock()
        .map_err(|e| format!("锁定打印机后端失败: {}", e))?;

    system_backend
        .send_raw(&printer_name, tspl.as_bytes())
        .map_err(|e| format!("发送到打印机失败: {}", e))?;

    log::info!("面单已发送到打印机: {}", printer_name);
    Ok(format!("面单已发送到打印机 {}", printer_name))
}
