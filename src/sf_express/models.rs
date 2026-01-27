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
}

impl Default for SFExpressConfig {
    fn default() -> Self {
        Self {
            environment: "sandbox".to_string(),
            partner_id: String::new(),
        }
    }
}

impl SFExpressConfig {
    /// 获取模板编码（根据顾客编码动态生成）
    pub fn template_code(&self) -> String {
        format!("fm_76130_standard_{}", self.partner_id)
    }

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
    /// 是否使用默认参数
    pub const USE_DEFAULT: &str = "qsl-cardhub:sf:use_default";
}

// ==================== 下单相关模型 ====================

/// 联系人类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactType {
    /// 寄件人
    Sender = 1,
    /// 收件人
    Recipient = 2,
}

/// 联系人信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactInfo {
    /// 联系人类型：1=寄件人，2=收件人
    pub contact_type: i32,
    /// 联系人姓名
    pub contact: String,
    /// 联系电话
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tel: Option<String>,
    /// 手机号码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile: Option<String>,
    /// 国家或地区代码
    pub country: String,
    /// 省份
    #[serde(skip_serializing_if = "Option::is_none")]
    pub province: Option<String>,
    /// 城市
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// 区县
    #[serde(skip_serializing_if = "Option::is_none")]
    pub county: Option<String>,
    /// 详细地址
    pub address: String,
    /// 公司名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
}

/// 托寄物信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CargoDetail {
    /// 物品名称
    pub name: String,
    /// 数量
    pub count: i32,
    /// 单位
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// 重量（kg）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    /// 金额
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    /// 币种
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}

/// 下订单请求 (EXP_RECE_CREATE_ORDER)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderRequest {
    /// 响应语言
    pub language: String,
    /// 客户订单号（唯一）
    pub order_id: String,
    /// 托寄物信息
    pub cargo_details: Vec<CargoDetail>,
    /// 联系人信息列表（寄件人和收件人）
    pub contact_info_list: Vec<ContactInfo>,
    /// 快件产品类别，2=顺丰标快
    pub express_type_id: i32,
    /// 付款方式：1=寄方付, 2=收方付, 3=第三方付
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pay_method: Option<i32>,
    /// 是否分配运单号，1=分配
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_gen_waybill_no: Option<i32>,
    /// 是否返回路由标签，1=返回
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_return_routelabel: Option<i32>,
    /// 月结卡号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_card: Option<String>,
}

impl CreateOrderRequest {
    /// 创建默认请求
    pub fn new(order_id: String) -> Self {
        Self {
            language: "zh-CN".to_string(),
            order_id,
            cargo_details: vec![],
            contact_info_list: vec![],
            express_type_id: 2,
            pay_method: Some(1),
            is_gen_waybill_no: Some(1),
            is_return_routelabel: Some(1),
            monthly_card: None,
        }
    }
}

/// 运单号信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaybillNoInfo {
    /// 运单号类型：1=母单, 2=子单, 3=签回单
    pub waybill_type: i32,
    /// 运单号
    pub waybill_no: String,
}

/// 下订单响应数据
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderResponseData {
    /// 客户订单号
    pub order_id: String,
    /// 原寄地区域代码
    #[serde(default)]
    pub origin_code: Option<String>,
    /// 目的地区域代码
    #[serde(default)]
    pub dest_code: Option<String>,
    /// 筛单结果：1=人工确认, 2=可收派, 3=不可收派
    #[serde(default)]
    pub filter_result: Option<i32>,
    /// 运单号列表
    #[serde(default)]
    pub waybill_no_info_list: Vec<WaybillNoInfo>,
}

/// 订单确认/取消请求 (EXP_RECE_UPDATE_ORDER)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrderRequest {
    /// 客户订单号
    pub order_id: String,
    /// 操作类型：1=确认, 2=取消
    pub deal_type: i32,
    /// 运单号信息列表（确认时必填）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waybill_no_info_list: Option<Vec<WaybillNoInfo>>,
}

/// 订单确认/取消响应数据
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrderResponseData {
    /// 客户订单号
    pub order_id: String,
    /// 运单号列表
    #[serde(default)]
    pub waybill_no_info_list: Vec<WaybillNoInfo>,
    /// 操作结果：1=订单号与运单不匹配, 2=操作成功
    #[serde(default)]
    pub res_status: Option<i32>,
}

/// 订单查询请求 (EXP_RECE_SEARCH_ORDER_RESP)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchOrderRequest {
    /// 客户订单号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    /// 运单号（15位或12位母单号）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_waybill_no: Option<String>,
    /// 查询类型：1=正向单, 2=退货单
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_type: Option<String>,
    /// 响应语言
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// 订单查询响应数据
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchOrderResponseData {
    /// 客户订单号
    pub order_id: String,
    /// 原寄地区域代码
    #[serde(default)]
    pub origincode: Option<String>,
    /// 目的地区域代码
    #[serde(default)]
    pub destcode: Option<String>,
    /// 筛单结果
    #[serde(default)]
    pub filter_result: Option<String>,
    /// 运单号列表
    #[serde(default)]
    pub waybill_no_info_list: Vec<WaybillNoInfo>,
}

/// 通用 API 业务响应
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiBusinessResponse<T> {
    /// 是否成功
    pub success: bool,
    /// 错误代码
    #[serde(default)]
    pub error_code: Option<String>,
    /// 错误信息
    #[serde(default)]
    pub error_msg: Option<String>,
    /// 业务数据
    #[serde(default)]
    pub msg_data: Option<T>,
}

// ==================== 寄件人信息模型 ====================

/// 寄件人信息（本地存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderInfo {
    /// ID
    pub id: String,
    /// 姓名
    pub name: String,
    /// 电话
    pub phone: String,
    /// 手机
    pub mobile: Option<String>,
    /// 省份
    pub province: String,
    /// 城市
    pub city: String,
    /// 区县
    pub district: String,
    /// 详细地址
    pub address: String,
    /// 是否默认
    pub is_default: bool,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

// ==================== 订单模型 ====================

/// 订单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    /// 待确认
    Pending,
    /// 已确认
    Confirmed,
    /// 已取消
    Cancelled,
    /// 已打印
    Printed,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderStatus::Pending => write!(f, "pending"),
            OrderStatus::Confirmed => write!(f, "confirmed"),
            OrderStatus::Cancelled => write!(f, "cancelled"),
            OrderStatus::Printed => write!(f, "printed"),
        }
    }
}

impl std::str::FromStr for OrderStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(OrderStatus::Pending),
            "confirmed" => Ok(OrderStatus::Confirmed),
            "cancelled" => Ok(OrderStatus::Cancelled),
            "printed" => Ok(OrderStatus::Printed),
            _ => Err(format!("未知订单状态: {}", s)),
        }
    }
}

/// 顺丰订单（本地存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SFOrder {
    /// ID
    pub id: String,
    /// 客户订单号
    pub order_id: String,
    /// 运单号（确认后获取）
    pub waybill_no: Option<String>,
    /// 关联的卡片 ID
    pub card_id: Option<String>,
    /// 订单状态
    pub status: String,
    /// 付款方式（1=寄方付, 2=收方付, 3=第三方付）
    pub pay_method: Option<i32>,
    /// 托寄物名称
    pub cargo_name: Option<String>,
    /// 寄件人信息（JSON）
    pub sender_info: String,
    /// 收件人信息（JSON）
    pub recipient_info: String,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

/// 顺丰订单（带卡片信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SFOrderWithCard {
    /// ID
    pub id: String,
    /// 客户订单号
    pub order_id: String,
    /// 运单号（确认后获取）
    pub waybill_no: Option<String>,
    /// 关联的卡片 ID
    pub card_id: Option<String>,
    /// 订单状态
    pub status: String,
    /// 付款方式（1=寄方付, 2=收方付, 3=第三方付）
    pub pay_method: Option<i32>,
    /// 托寄物名称
    pub cargo_name: Option<String>,
    /// 寄件人信息（JSON）
    pub sender_info: String,
    /// 收件人信息（JSON）
    pub recipient_info: String,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
    /// 关联卡片的呼号
    pub callsign: Option<String>,
    /// 关联卡片的项目名称
    pub project_name: Option<String>,
    /// 关联卡片的数量
    pub qty: Option<i32>,
}

// ==================== 错误码映射 ====================

/// 获取用户友好的错误信息
pub fn get_user_friendly_error(code: &str, msg: &str) -> String {
    match code {
        "A1001" => "请求参数不完整，请检查配置信息".to_string(),
        "A1002" => "请求时效已过期，请检查系统时间".to_string(),
        "A1003" => "IP 地址未授权，请联系顺丰开通".to_string(),
        "A1004" => "无服务权限，请检查 API 配置或联系顺丰".to_string(),
        "A1005" => "流量受控，请稍后重试".to_string(),
        "A1006" => "数字签名验证失败，请检查校验码是否正确".to_string(),
        "A1007" => "重复请求，请稍后重试".to_string(),
        "A1009" => "顺丰服务暂时不可用，请稍后重试".to_string(),
        "A1011" => "认证失败，请重新获取令牌".to_string(),
        "A1099" => "系统异常，请稍后重试".to_string(),
        "1010" | "1014" => "地址信息不完整，请补充详细地址".to_string(),
        "1011" | "1015" => "联系人姓名不能为空".to_string(),
        "1012" | "1016" => "联系电话不能为空".to_string(),
        "1023" => "物品名称不能为空".to_string(),
        "6126" => "月结卡号格式不正确".to_string(),
        "6135" => "未传入订单信息".to_string(),
        "6150" | "8018" => "订单不存在，请检查订单号".to_string(),
        "8016" => "订单号已存在，请勿重复下单".to_string(),
        "8017" => "订单号与运单号不匹配".to_string(),
        "8019" => "订单已确认或已取消".to_string(),
        "8037" | "8253" => "订单已取消".to_string(),
        "8114" | "8119" => "月结卡号无效或无权限".to_string(),
        "8196" => "联系电话信息异常".to_string(),
        "8252" => "订单已确认".to_string(),
        "20052" => "月结卡号不匹配，不允许操作该订单".to_string(),
        "S0000" => "操作成功".to_string(),
        _ => format!("操作失败：{}", msg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = SFExpressConfig::default();
        assert_eq!(config.environment, "sandbox");
        assert!(!config.is_production());
    }

    #[test]
    fn test_template_code() {
        let mut config = SFExpressConfig::default();
        config.partner_id = "TESTPARTNER".to_string();
        assert_eq!(config.template_code(), "fm_76130_standard_TESTPARTNER");
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

    #[test]
    fn test_order_status() {
        assert_eq!(OrderStatus::Pending.to_string(), "pending");
        assert_eq!("confirmed".parse::<OrderStatus>().unwrap(), OrderStatus::Confirmed);
    }
}
