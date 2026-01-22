// 顺丰速运集成模块
//
// 该模块负责：
// - 顺丰云打印 API 调用
// - 数字签名计算
// - PDF 面单渲染为 TSPL 打印指令

pub mod models;
pub mod client;
pub mod pdf_renderer;

pub use client::SFExpressClient;
pub use models::{SFExpressConfig, WaybillPrintRequest, CloudPrintResponse};
pub use pdf_renderer::PdfRenderer;
