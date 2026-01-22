// 顺丰速运数据模型
//
// 定义 API 请求/响应结构和配置模型

use serde::{Deserialize, Serialize};

/// 顺丰速运配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SFExpressConfig {
    /// 环境：production / sandbox
    pub environment: String,
    /// 顾客编码（合作伙伴编码）
    pub partner_id: String,
    /// 模板编码（固定值）
    pub template_code: String,
}

impl Default for SFExpressConfig {
    fn default() -> Self {
        Self {
            environment: "sandbox".to_string(),
            partner_id: String::new(),
            template_code: "fm_76130_standard_HBTRJT0FNP6E".to_string(),
        }
    }
}

impl SFExpressConfig {
    /// 获取 API 地址
    pub fn api_url(&self) -> &'static str {
        match self.environment.as_str() {
            "production" => "https://bspgw.sf-express.com/std/service",
            _ => "https://sfapi-sbox.sf-express.com/std/service",
        }
    }

    /// 是否为生产环境
    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }
}

/// 运单打印请求
#[derive(Debug, Serialize, Deserialize)]
pub struct WaybillPrintRequest {
    /// 运单号
    pub waybill_no: String,
}

/// 云打印 API 请求业务数据
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudPrintMsgData {
    /// 模板编码
    pub template_code: String,
    /// API 版本
    pub version: String,
    /// 文件类型
    pub file_type: String,
    /// 是否同步（直接返回 PDF 链接）
    pub sync: bool,
    /// 待打印的运单列表
    pub documents: Vec<PrintDocument>,
}

/// 打印文档
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintDocument {
    /// 主运单号
    pub master_waybill_no: String,
}

/// 云打印 API 响应
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudPrintResponse {
    /// API 结果码
    pub api_result_code: String,
    /// API 错误信息
    #[serde(default)]
    pub api_error_msg: Option<String>,
    /// API 结果数据（注意：这是一个 JSON 字符串，需要二次解析）
    #[serde(default)]
    pub api_result_data: Option<String>,
}

impl CloudPrintResponse {
    /// 是否成功
    pub fn is_success(&self) -> bool {
        self.api_result_code == "A1000"
    }

    /// 获取错误信息（中文）
    pub fn error_message(&self) -> String {
        match self.api_result_code.as_str() {
            "A1000" => "成功".to_string(),
            "A1001" => "必传参数为空，请检查配置".to_string(),
            "A1002" => "请求时效已过期，请检查系统时间".to_string(),
            "A1003" => "IP 无效，请联系顺丰开通 IP 白名单".to_string(),
            "A1004" => "无对应服务权限，请联系顺丰开通权限".to_string(),
            "A1006" => "数字签名验证失败，请检查校验码是否正确".to_string(),
            _ => self.api_error_msg.clone().unwrap_or_else(|| format!("未知错误: {}", self.api_result_code)),
        }
    }
}

/// API 结果数据
#[derive(Debug, Deserialize)]
pub struct ApiResultData {
    /// 是否成功
    pub success: bool,
    /// 错误代码
    #[serde(default)]
    pub error_code: Option<String>,
    /// 错误信息
    #[serde(default)]
    pub error_msg: Option<String>,
    /// 结果对象
    #[serde(default)]
    pub obj: Option<PrintResultObj>,
}

/// 打印结果对象
#[derive(Debug, Deserialize)]
pub struct PrintResultObj {
    /// 文件列表
    #[serde(default)]
    pub files: Vec<PrintFile>,
}

/// 打印文件信息
#[derive(Debug, Deserialize)]
pub struct PrintFile {
    /// PDF 下载地址
    pub url: String,
    /// 认证 Token
    pub token: String,
    /// 运单号
    #[serde(default)]
    pub waybill_no: Option<String>,
}

/// 凭据存储键名
pub mod credential_keys {
    /// 顾客编码
    pub const PARTNER_ID: &str = "qsl-cardhub:sf:partner_id";
    /// 生产环境校验码
    pub const CHECKWORD_PROD: &str = "qsl-cardhub:sf:checkword_prod";
    /// 沙箱环境校验码
    pub const CHECKWORD_SANDBOX: &str = "qsl-cardhub:sf:checkword_sandbox";
    /// 当前环境
    pub const ENVIRONMENT: &str = "qsl-cardhub:sf:environment";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = SFExpressConfig::default();
        assert_eq!(config.environment, "sandbox");
        assert_eq!(config.template_code, "fm_76130_standard_HBTRJT0FNP6E");
        assert!(!config.is_production());
    }

    #[test]
    fn test_api_url() {
        let mut config = SFExpressConfig::default();
        assert_eq!(config.api_url(), "https://sfapi-sbox.sf-express.com/std/service");

        config.environment = "production".to_string();
        assert_eq!(config.api_url(), "https://bspgw.sf-express.com/std/service");
    }

    #[test]
    fn test_response_error_messages() {
        let response = CloudPrintResponse {
            api_result_code: "A1006".to_string(),
            api_error_msg: None,
            api_result_data: None,
        };
        assert_eq!(response.error_message(), "数字签名验证失败，请检查校验码是否正确");
    }
}
