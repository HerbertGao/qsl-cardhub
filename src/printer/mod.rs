// 打印模块
//
// 该模块负责：
// - TSPL 指令生成
// - 打印机后端抽象
// - 跨平台打印支持

pub mod backend;
pub mod barcode_renderer;
pub mod font_loader;
pub mod layout_engine;
pub mod render_pipeline;
pub mod template_engine;
pub mod text_renderer;
pub mod tspl;

pub use backend::PdfBackend;
