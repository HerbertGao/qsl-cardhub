// 顺丰速运集成模块
//
// 该模块负责：
// - 顺丰云打印 API 调用
// - 顺丰下单 API 调用（创建、确认、取消、查询）
// - 数字签名计算
// - PDF 面单渲染为 TSPL 打印指令

pub mod models;
pub mod client;
pub mod pdf_renderer;

pub use client::SFExpressClient;
pub use models::{
    SFExpressConfig, WaybillPrintRequest, CloudPrintResponse,
    // 下单相关
    CreateOrderRequest, CreateOrderResponseData,
    UpdateOrderRequest, UpdateOrderResponseData,
    SearchOrderRequest, SearchOrderResponseData,
    ContactInfo, CargoDetail, WaybillNoInfo,
    // 本地存储模型
    SenderInfo, SFOrder, SFOrderWithCard, OrderStatus,
    // 工具函数
    get_user_friendly_error,
};
pub use pdf_renderer::PdfRenderer;
