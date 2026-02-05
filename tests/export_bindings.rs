//! TypeScript 类型导出测试
//!
//! 运行此测试以生成 TypeScript 类型定义文件：
//! ```bash
//! cargo test export_bindings --features ts-rs
//! ```
//!
//! 或通过设置环境变量指定输出目录：
//! ```bash
//! TS_RS_EXPORT_DIR=web/src/types/generated cargo test export_bindings --features ts-rs
//! ```

#[cfg(feature = "ts-rs")]
mod export {
    use std::path::PathBuf;
    use ts_rs::TS;

    // 导入需要导出的类型
    use qsl_cardhub::config::models::{Platform, PrinterConfig, Profile, Template};
    use qsl_cardhub::db::models::{
        AddressEntry, Card, CardMetadata, CardStatus, CardWithProject, DistributionInfo,
        PagedCards, Project, ProjectWithStats, ReturnInfo,
    };
    use qsl_cardhub::sf_express::models::{OrderStatus, SFOrder, SFOrderWithCard, SenderInfo};

    #[test]
    fn export_bindings() {
        // 设置输出目录环境变量（如果未设置）
        let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("web")
            .join("src")
            .join("types")
            .join("generated");

        // 确保输出目录存在
        std::fs::create_dir_all(&output_dir).expect("Failed to create output directory");

        // 设置环境变量（Rust 2024 edition 需要 unsafe）
        // SAFETY: 单线程测试环境，设置环境变量是安全的
        unsafe {
            std::env::set_var("TS_RS_EXPORT_DIR", &output_dir);
        }

        // 使用从环境变量读取的配置
        let config = ts_rs::Config::from_env();

        // 数据库模型
        Project::export_all(&config).expect("Failed to export Project");
        ProjectWithStats::export_all(&config).expect("Failed to export ProjectWithStats");
        CardStatus::export_all(&config).expect("Failed to export CardStatus");
        Card::export_all(&config).expect("Failed to export Card");
        CardWithProject::export_all(&config).expect("Failed to export CardWithProject");
        CardMetadata::export_all(&config).expect("Failed to export CardMetadata");
        DistributionInfo::export_all(&config).expect("Failed to export DistributionInfo");
        ReturnInfo::export_all(&config).expect("Failed to export ReturnInfo");
        AddressEntry::export_all(&config).expect("Failed to export AddressEntry");
        PagedCards::export_all(&config).expect("Failed to export PagedCards");

        // 顺丰模型
        SenderInfo::export_all(&config).expect("Failed to export SenderInfo");
        OrderStatus::export_all(&config).expect("Failed to export OrderStatus");
        SFOrder::export_all(&config).expect("Failed to export SFOrder");
        SFOrderWithCard::export_all(&config).expect("Failed to export SFOrderWithCard");

        // 配置模型
        Profile::export_all(&config).expect("Failed to export Profile");
        Platform::export_all(&config).expect("Failed to export Platform");
        PrinterConfig::export_all(&config).expect("Failed to export PrinterConfig");
        Template::export_all(&config).expect("Failed to export Template");

        println!("TypeScript bindings exported to: {:?}", output_dir);
    }
}
