//! TypeScript 类型导出测试
//!
//! 运行此测试以生成 TypeScript 类型定义文件：
//! ```bash
//! cargo test export_bindings --features ts-rs
//! ```

#[cfg(feature = "ts-rs")]
mod export {
    use ts_rs::TS;

    // 导入需要导出的类型
    use qsl_cardhub::db::models::{
        AddressEntry, Card, CardMetadata, CardStatus, CardWithProject, DistributionInfo,
        PagedCards, Project, ProjectWithStats, ReturnInfo,
    };
    use qsl_cardhub::sf_express::models::{OrderStatus, SFOrder, SFOrderWithCard, SenderInfo};
    use qsl_cardhub::config::models::{Platform, PrinterConfig, Profile, Template};

    #[test]
    fn export_bindings() {
        // 数据库模型
        Project::export_all().expect("Failed to export Project");
        ProjectWithStats::export_all().expect("Failed to export ProjectWithStats");
        CardStatus::export_all().expect("Failed to export CardStatus");
        Card::export_all().expect("Failed to export Card");
        CardWithProject::export_all().expect("Failed to export CardWithProject");
        CardMetadata::export_all().expect("Failed to export CardMetadata");
        DistributionInfo::export_all().expect("Failed to export DistributionInfo");
        ReturnInfo::export_all().expect("Failed to export ReturnInfo");
        AddressEntry::export_all().expect("Failed to export AddressEntry");
        PagedCards::export_all().expect("Failed to export PagedCards");

        // 顺丰模型
        SenderInfo::export_all().expect("Failed to export SenderInfo");
        OrderStatus::export_all().expect("Failed to export OrderStatus");
        SFOrder::export_all().expect("Failed to export SFOrder");
        SFOrderWithCard::export_all().expect("Failed to export SFOrderWithCard");

        // 配置模型
        Profile::export_all().expect("Failed to export Profile");
        Platform::export_all().expect("Failed to export Platform");
        PrinterConfig::export_all().expect("Failed to export PrinterConfig");
        Template::export_all().expect("Failed to export Template");

        println!("TypeScript bindings exported successfully!");
    }
}
