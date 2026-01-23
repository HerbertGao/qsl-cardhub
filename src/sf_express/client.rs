// 顺丰速运 API 客户端
//
// 负责：
// - 数字签名计算
// - API 请求发送
// - PDF 文件下载

use anyhow::{Context, Result, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::blocking::Client;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use super::models::{
    CloudPrintMsgData, CloudPrintResponse, PrintDocument, PrintFile, SFExpressConfig,
    CreateOrderRequest, CreateOrderResponseData, UpdateOrderRequest, UpdateOrderResponseData,
    SearchOrderRequest, SearchOrderResponseData, ApiBusinessResponse, get_user_friendly_error,
};

/// 顺丰速运 API 客户端
pub struct SFExpressClient {
    config: SFExpressConfig,
    check_word: String,
    http_client: Client,
}

impl SFExpressClient {
    /// 创建新的客户端实例
    pub fn new(config: SFExpressConfig, check_word: String) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("创建 HTTP 客户端失败")?;

        Ok(Self {
            config,
            check_word,
            http_client,
        })
    }

    /// 计算数字签名
    ///
    /// 签名算法：msgDigest = Base64(MD5(msgData + timestamp + checkWord))
    fn calculate_signature(&self, msg_data: &str, timestamp: i64) -> String {
        let to_sign = format!("{}{}{}", msg_data, timestamp, self.check_word);
        let digest = md5::compute(to_sign.as_bytes());
        STANDARD.encode(digest.0)
    }

    /// 获取当前时间戳（毫秒）
    fn get_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }

    /// 调用云打印 API 获取面单 PDF
    pub fn print_waybill(&self, waybill_no: &str) -> Result<PrintFile> {
        log::info!("调用顺丰云打印 API，运单号: {}", waybill_no);

        // 构造业务数据
        let msg_data = CloudPrintMsgData {
            template_code: self.config.template_code(),
            version: "2.0".to_string(),
            file_type: "pdf".to_string(),
            sync: true,
            documents: vec![PrintDocument {
                master_waybill_no: waybill_no.to_string(),
            }],
        };

        let msg_data_json = serde_json::to_string(&msg_data)
            .context("序列化业务数据失败")?;

        // 生成请求参数
        let timestamp = Self::get_timestamp();
        let request_id = Uuid::new_v4().to_string();
        let msg_digest = self.calculate_signature(&msg_data_json, timestamp);

        log::info!("API 请求: url={}, partnerID={}, requestID={}",
            self.config.api_url(), self.config.partner_id, request_id);

        // 构造 form 请求体
        let form_data = [
            ("partnerID", self.config.partner_id.as_str()),
            ("requestID", &request_id),
            ("serviceCode", "COM_RECE_CLOUD_PRINT_WAYBILLS"),
            ("timestamp", &timestamp.to_string()),
            ("msgDigest", &msg_digest),
            ("msgData", &msg_data_json),
        ];

        // 发送请求
        let response = self.http_client
            .post(self.config.api_url())
            .header("Content-Type", "application/x-www-form-urlencoded;charset=UTF-8")
            .form(&form_data)
            .send()
            .context("发送 API 请求失败")?;

        let response_text = response.text().context("读取响应失败")?;
        log::info!("API 响应: {}", response_text);

        // 解析响应
        let api_response: CloudPrintResponse = serde_json::from_str(&response_text)
            .context("解析 API 响应失败")?;

        // 检查 API 结果码
        if !api_response.is_success() {
            log::error!("API 错误: code={}, msg={}",
                api_response.api_result_code,
                api_response.error_message());
            bail!("{}", api_response.error_message());
        }

        // 获取结果数据字符串（顺丰 API 返回的是 JSON 字符串，需要二次解析）
        let result_data_str = api_response.api_result_data
            .context("API 响应中缺少结果数据")?;

        log::info!("解析结果数据: {}", result_data_str);

        // 二次解析 JSON 字符串
        let result_data: super::models::ApiResultData = serde_json::from_str(&result_data_str)
            .context("解析结果数据失败")?;

        if !result_data.success {
            let error_msg = result_data.error_msg
                .unwrap_or_else(|| "未知业务错误".to_string());
            bail!("业务错误: {}", error_msg);
        }

        // 获取文件信息
        let obj = result_data.obj
            .context("API 响应中缺少结果对象")?;

        let file = obj.files
            .into_iter()
            .next()
            .context("API 响应中没有文件信息")?;

        log::info!("获取到 PDF 下载地址: {}", file.url);

        Ok(file)
    }

    /// 下载 PDF 文件
    pub fn download_pdf(&self, file: &PrintFile) -> Result<Vec<u8>> {
        log::info!("下载 PDF 文件: {}", file.url);

        let response = self.http_client
            .get(&file.url)
            .header("X-Auth-token", &file.token)
            .send()
            .context("下载 PDF 失败")?;

        if !response.status().is_success() {
            bail!("下载 PDF 失败: HTTP {}", response.status());
        }

        let pdf_data = response.bytes()
            .context("读取 PDF 数据失败")?
            .to_vec();

        log::info!("PDF 下载完成，大小: {} 字节", pdf_data.len());

        Ok(pdf_data)
    }

    /// 获取面单 PDF 数据（调用 API + 下载）
    pub fn get_waybill_pdf(&self, waybill_no: &str) -> Result<Vec<u8>> {
        let file = self.print_waybill(waybill_no)?;
        self.download_pdf(&file)
    }

    // ==================== 下单 API ====================

    /// 发送通用 API 请求
    fn send_api_request(&self, service_code: &str, msg_data: &str) -> Result<String> {
        let timestamp = Self::get_timestamp();
        let request_id = Uuid::new_v4().to_string();
        let msg_digest = self.calculate_signature(msg_data, timestamp);

        log::info!("API 请求: service={}, partnerID={}, requestID={}",
            service_code, self.config.partner_id, request_id);

        let form_data = [
            ("partnerID", self.config.partner_id.as_str()),
            ("requestID", &request_id),
            ("serviceCode", service_code),
            ("timestamp", &timestamp.to_string()),
            ("msgDigest", &msg_digest),
            ("msgData", msg_data),
        ];

        let response = self.http_client
            .post(self.config.api_url())
            .header("Content-Type", "application/x-www-form-urlencoded;charset=UTF-8")
            .form(&form_data)
            .send()
            .context("发送 API 请求失败")?;

        let response_text = response.text().context("读取响应失败")?;
        log::info!("API 响应: {}", response_text);

        // 解析外层响应
        let api_response: CloudPrintResponse = serde_json::from_str(&response_text)
            .context("解析 API 响应失败")?;

        if !api_response.is_success() {
            log::error!("API 错误: code={}, msg={}",
                api_response.api_result_code,
                api_response.error_message());
            bail!("{}", api_response.error_message());
        }

        api_response.api_result_data
            .context("API 响应中缺少结果数据")
    }

    /// 创建订单 (EXP_RECE_CREATE_ORDER)
    pub fn create_order(&self, request: &CreateOrderRequest) -> Result<CreateOrderResponseData> {
        log::info!("创建顺丰订单: order_id={}", request.order_id);

        let msg_data = serde_json::to_string(request)
            .context("序列化下单请求失败")?;

        let result_str = self.send_api_request("EXP_RECE_CREATE_ORDER", &msg_data)?;

        let result: ApiBusinessResponse<CreateOrderResponseData> = serde_json::from_str(&result_str)
            .context("解析下单响应失败")?;

        if !result.success {
            let error_code = result.error_code.unwrap_or_default();
            let error_msg = result.error_msg.unwrap_or_else(|| "未知错误".to_string());
            log::error!("下单失败: code={}, msg={}", error_code, error_msg);
            bail!("{}", get_user_friendly_error(&error_code, &error_msg));
        }

        result.msg_data.context("下单响应中缺少业务数据")
    }

    /// 确认/取消订单 (EXP_RECE_UPDATE_ORDER)
    pub fn update_order(&self, request: &UpdateOrderRequest) -> Result<UpdateOrderResponseData> {
        let action = if request.deal_type == 1 { "确认" } else { "取消" };
        log::info!("{}顺丰订单: order_id={}", action, request.order_id);

        let msg_data = serde_json::to_string(request)
            .context("序列化订单更新请求失败")?;

        let result_str = self.send_api_request("EXP_RECE_UPDATE_ORDER", &msg_data)?;

        let result: ApiBusinessResponse<UpdateOrderResponseData> = serde_json::from_str(&result_str)
            .context("解析订单更新响应失败")?;

        if !result.success {
            let error_code = result.error_code.unwrap_or_default();
            let error_msg = result.error_msg.unwrap_or_else(|| "未知错误".to_string());
            log::error!("订单{}失败: code={}, msg={}", action, error_code, error_msg);
            bail!("{}", get_user_friendly_error(&error_code, &error_msg));
        }

        result.msg_data.context("订单更新响应中缺少业务数据")
    }

    /// 查询订单 (EXP_RECE_SEARCH_ORDER_RESP)
    pub fn search_order(&self, request: &SearchOrderRequest) -> Result<SearchOrderResponseData> {
        log::info!("查询顺丰订单: order_id={:?}, waybill_no={:?}",
            request.order_id, request.main_waybill_no);

        let msg_data = serde_json::to_string(request)
            .context("序列化订单查询请求失败")?;

        let result_str = self.send_api_request("EXP_RECE_SEARCH_ORDER_RESP", &msg_data)?;

        let result: ApiBusinessResponse<SearchOrderResponseData> = serde_json::from_str(&result_str)
            .context("解析订单查询响应失败")?;

        if !result.success {
            let error_code = result.error_code.unwrap_or_default();
            let error_msg = result.error_msg.unwrap_or_else(|| "未知错误".to_string());
            log::error!("订单查询失败: code={}, msg={}", error_code, error_msg);
            bail!("{}", get_user_friendly_error(&error_code, &error_msg));
        }

        result.msg_data.context("订单查询响应中缺少业务数据")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_calculation() {
        let config = SFExpressConfig {
            environment: "sandbox".to_string(),
            partner_id: "test_partner".to_string(),
        };

        let client = SFExpressClient::new(config, "test_checkword".to_string()).unwrap();

        // 使用固定的测试数据验证签名计算
        let msg_data = r#"{"test":"data"}"#;
        let timestamp = 1234567890000i64;
        let signature = client.calculate_signature(msg_data, timestamp);

        // 验证签名是 Base64 编码的 MD5
        assert!(!signature.is_empty());
        // Base64 解码后应该是 16 字节（128 位 MD5）
        let decoded = STANDARD.decode(&signature).unwrap();
        assert_eq!(decoded.len(), 16);
    }
}
