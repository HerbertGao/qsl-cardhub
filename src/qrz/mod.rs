// QRZ.cn 模块
pub mod qrz_cn_client;
pub mod qrz_cn_parser;

// QRZ.com 模块
pub mod qrz_com_client;
pub mod qrz_com_parser;

// QRZ.herbertgao.me 模块
pub mod qrz_herbertgao_client;

// 导出 QRZ.cn 类型和函数
pub use qrz_cn_client::QRZCnClient;
pub use qrz_cn_parser::{parse_qrz_cn_page, QrzCnAddressInfo};

// 导出 QRZ.com 类型和函数
pub use qrz_com_client::QRZComClient;
pub use qrz_com_parser::{parse_qrz_com_page, QrzComAddressInfo};

// 导出 QRZ.herbertgao.me 类型和函数
pub use qrz_herbertgao_client::{query_callsign as query_herbertgao, HerbertgaoAddressInfo};