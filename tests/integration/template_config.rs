// 模板配置集成测试
//
// 测试模板配置的加载、解析和使用

use QSL_CardHub::config::{TemplateConfig, TemplateManager};
use QSL_CardHub::printer::backend::PdfBackend;
use QSL_CardHub::printer::backend::PrinterBackend;
use QSL_CardHub::printer::tspl::TSPLGenerator;
use std::path::PathBuf;

#[test]
fn test_template_config_loading() {
    println!("\n========== 模板配置加载测试 ==========");

    // 测试默认模板
    let template = TemplateConfig::default_qsl_v1();

    println!("模板名称: {}", template.metadata.name);
    println!("模板版本: {}", template.metadata.version);
    println!(
        "纸张尺寸: {}mm x {}mm",
        template.paper.width_mm, template.paper.height_mm
    );

    assert_eq!(template.metadata.name, "QSL Card v1");
    assert_eq!(template.paper.width_mm, 76);
    assert_eq!(template.paper.height_mm, 130);
    assert_eq!(template.title.text, "中国无线电协会业余分会-2区卡片局");
    assert_eq!(template.title.x, 304);
    assert_eq!(template.subtitle.x, 304);

    println!("✅ 模板配置加载测试通过");
    println!("=====================================\n");
}

#[test]
fn test_template_serialization() {
    println!("\n========== 模板序列化测试 ==========");

    let template = TemplateConfig::default_qsl_v1();

    // 序列化到 TOML
    let toml_str = toml::to_string_pretty(&template).expect("序列化失败");

    println!("序列化的 TOML:\n{}", toml_str);

    // 验证包含关键字段
    assert!(toml_str.contains("[metadata]"));
    assert!(toml_str.contains("[paper]"));
    assert!(toml_str.contains("[title]"));
    assert!(toml_str.contains("[subtitle]"));
    assert!(toml_str.contains("[callsign]"));
    assert!(toml_str.contains("[barcode]"));
    assert!(toml_str.contains("[serial]"));
    assert!(toml_str.contains("[quantity]"));
    assert!(toml_str.contains("中国无线电协会业余分会-2区卡片局"));

    // 反序列化回来
    let deserialized: TemplateConfig = toml::from_str(&toml_str).expect("反序列化失败");

    assert_eq!(deserialized.metadata.name, template.metadata.name);
    assert_eq!(deserialized.title.text, template.title.text);
    assert_eq!(deserialized.paper.width_mm, template.paper.width_mm);

    println!("✅ 模板序列化测试通过");
    println!("=====================================\n");
}

#[test]
fn test_template_manager() {
    println!("\n========== 模板管理器测试 ==========");

    use tempfile::tempdir;

    // 创建临时模板目录
    let temp_dir = tempdir().expect("创建临时目录失败");
    let manager = TemplateManager::new(temp_dir.path().to_path_buf()).expect("创建管理器失败");

    // 测试列出模板
    let templates = manager.list_templates();
    println!("可用模板: {:?}", templates);
    assert!(!templates.is_empty());
    assert!(templates.contains(&"QSL Card v1".to_string()));

    // 测试获取默认模板
    let default_template = manager.get_default_template().expect("获取默认模板失败");
    assert_eq!(default_template.metadata.name, "QSL Card v1");

    println!("✅ 模板管理器测试通过");
    println!("=====================================\n");
}

#[test]
fn test_tspl_generation_with_template() {
    println!("\n========== TSPL 模板生成测试 ==========");

    let template = TemplateConfig::default_qsl_v1();
    let generator = TSPLGenerator::new();

    // 无任务名称
    let tspl_no_task = generator.generate_from_template(&template, "BG7XXX", 1, 10, None);

    println!("生成的 TSPL（无任务名称）:");
    println!("{}", tspl_no_task);

    assert!(tspl_no_task.contains("TEXT 304,20,\"5\",0,2,2,\"中国无线电协会业余分会-2区卡片局\""));
    assert!(!tspl_no_task.contains("春季活动")); // 无副标题
    assert!(tspl_no_task.contains("SIZE 76 mm, 130 mm"));
    assert!(tspl_no_task.contains("BG7XXX"));

    // 有任务名称
    let tspl_with_task =
        generator.generate_from_template(&template, "BD7ABC", 2, 5, Some("春季活动"));

    println!("\n生成的 TSPL（有任务名称）:");
    println!("{}", tspl_with_task);

    assert!(
        tspl_with_task.contains("TEXT 304,20,\"5\",0,2,2,\"中国无线电协会业余分会-2区卡片局\"")
    );
    assert!(tspl_with_task.contains("TEXT 304,100,\"5\",0,2,2,\"春季活动\""));
    assert!(tspl_with_task.contains("BD7ABC"));

    println!("✅ TSPL 模板生成测试通过");
    println!("=====================================\n");
}

#[test]
fn test_pdf_rendering_with_template() {
    println!("\n========== PDF 模板渲染测试 ==========");

    let test_dir = PathBuf::from("test_output");
    if !test_dir.exists() {
        std::fs::create_dir_all(&test_dir).expect("创建测试目录失败");
    }

    let template = TemplateConfig::default_qsl_v1();
    let generator = TSPLGenerator::new();
    let pdf_backend = PdfBackend::new(test_dir.clone()).expect("创建 PDF 后端失败");

    // 生成 TSPL
    let tspl = generator.generate_from_template(&template, "BG7XXX", 123, 50, Some("模板测试"));

    println!("测试 TSPL:");
    println!("{}", tspl);

    // 渲染 PDF
    let result = pdf_backend.send_raw("PDF 测试打印机", tspl.as_bytes());

    assert!(result.is_ok(), "PDF 渲染失败: {:?}", result.err());

    println!("✅ PDF 模板渲染测试通过");
    println!("📁 输出: {}", test_dir.display());
    println!("=====================================\n");
}

#[test]
fn test_template_file_loading() {
    println!("\n========== 模板文件加载测试 ==========");

    // 测试加载预设的模板文件
    let template_path = PathBuf::from("config/templates/qsl-card-v1.toml");

    if !template_path.exists() {
        println!("⚠️  模板文件不存在，跳过测试: {}", template_path.display());
        return;
    }

    let template = TemplateConfig::load_from_file(template_path.clone()).expect("加载模板文件失败");

    println!("从文件加载的模板:");
    println!("  名称: {}", template.metadata.name);
    println!("  版本: {}", template.metadata.version);
    println!("  标题: {}", template.title.text);

    assert_eq!(template.metadata.name, "76x130模板");
    assert_eq!(template.title.text, "中国无线电协会业余分会-2区卡片局");

    println!("✅ 模板文件加载测试通过");
    println!("=====================================\n");
}
