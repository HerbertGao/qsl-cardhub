// 后端集成测试
//
// 测试 PDF 和 TSPL 后端与渲染管道的集成

use QSL_CardHub::config::template_v2::{OutputConfig, TemplateV2Config};
use QSL_CardHub::printer::backend::PdfBackendV2;
use QSL_CardHub::printer::layout_engine::LayoutEngine;
use QSL_CardHub::printer::render_pipeline::RenderPipeline;
use QSL_CardHub::printer::template_engine::TemplateEngine;
use QSL_CardHub::printer::tspl_v2::TSPLGeneratorV2;
use std::collections::HashMap;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_pdf_backend_mixed_mode() {
    println!("\n========== PDF后端集成测试: 混合模式 ==========");

    // 1. 准备完整流程
    let config = TemplateV2Config::default_qsl_card_v2();
    let mut data = HashMap::new();
    data.insert("task_name".to_string(), "CQWW DX Contest".to_string());
    data.insert("callsign".to_string(), "BG7XXX".to_string());
    data.insert("sn".to_string(), "001".to_string());
    data.insert("qty".to_string(), "500".to_string());

    // 2. 模板解析 → 布局 → 渲染
    let resolved = TemplateEngine::resolve(&config, &data).unwrap();
    let mut layout_engine = LayoutEngine::new().unwrap();
    let layout_result = layout_engine.layout(&config, resolved).unwrap();

    let mut pipeline = RenderPipeline::new().unwrap();
    let output_config = OutputConfig {
        mode: "text_bitmap_plus_native_barcode".to_string(),
        threshold: 160,
    };
    let render_result = pipeline.render(layout_result, &output_config).unwrap();

    // 3. PDF 后端渲染
    let temp_dir = TempDir::new().unwrap();
    let mut pdf_backend = PdfBackendV2::new(temp_dir.path().to_path_buf()).unwrap();

    let png_path = pdf_backend.render(render_result).unwrap();

    // 4. 验证输出
    assert!(png_path.exists(), "PNG文件应该存在");
    assert!(png_path.extension().unwrap() == "png");

    let img = image::open(&png_path).unwrap();
    assert_eq!(img.width(), 608, "图像宽度应该是608");
    assert_eq!(img.height(), 1039, "图像高度应该是1039");

    println!("✓ PNG保存到: {}", png_path.display());
    println!("✓ 图像尺寸: {}x{}", img.width(), img.height());
    println!("✅ PDF后端混合模式测试通过");
    println!("=====================================\n");
}

#[test]
fn test_pdf_backend_full_bitmap() {
    println!("\n========== PDF后端集成测试: 全位图模式 ==========");

    let config = TemplateV2Config::default_qsl_card_v2();
    let mut data = HashMap::new();
    data.insert("task_name".to_string(), "IARU HF".to_string());
    data.insert("callsign".to_string(), "BD7AA".to_string());
    data.insert("sn".to_string(), "999".to_string());
    data.insert("qty".to_string(), "250".to_string());

    let resolved = TemplateEngine::resolve(&config, &data).unwrap();
    let mut layout_engine = LayoutEngine::new().unwrap();
    let layout_result = layout_engine.layout(&config, resolved).unwrap();

    let mut pipeline = RenderPipeline::new().unwrap();
    let output_config = OutputConfig {
        mode: "full_bitmap".to_string(),
        threshold: 160,
    };
    let render_result = pipeline.render(layout_result, &output_config).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let mut pdf_backend = PdfBackendV2::new(temp_dir.path().to_path_buf()).unwrap();

    let png_path = pdf_backend.render(render_result).unwrap();

    assert!(png_path.exists());
    let img = image::open(&png_path).unwrap();
    assert_eq!(img.width(), 608);
    assert_eq!(img.height(), 1039);

    println!("✓ PNG保存到: {}", png_path.display());
    println!("✅ PDF后端全位图模式测试通过");
    println!("=====================================\n");
}

#[test]
fn test_tspl_generator_mixed_mode() {
    println!("\n========== TSPL生成器集成测试: 混合模式 ==========");

    let config = TemplateV2Config::default_qsl_card_v2();
    let mut data = HashMap::new();
    data.insert("task_name".to_string(), "测试任务".to_string());
    data.insert("callsign".to_string(), "BH1ABC".to_string());
    data.insert("sn".to_string(), "042".to_string());
    data.insert("qty".to_string(), "100".to_string());

    let resolved = TemplateEngine::resolve(&config, &data).unwrap();
    let mut layout_engine = LayoutEngine::new().unwrap();
    let layout_result = layout_engine.layout(&config, resolved).unwrap();

    let mut pipeline = RenderPipeline::new().unwrap();
    let output_config = OutputConfig {
        mode: "text_bitmap_plus_native_barcode".to_string(),
        threshold: 160,
    };
    let render_result = pipeline.render(layout_result, &output_config).unwrap();

    // TSPL 生成
    let generator = TSPLGeneratorV2::new();
    let tspl = generator.generate(render_result, 76.0, 130.0).unwrap();

    // 验证 TSPL 内容
    assert!(tspl.contains("SIZE 76 mm, 130 mm"), "应该包含纸张尺寸");
    assert!(tspl.contains("CLS"), "应该包含清屏指令");
    assert!(tspl.contains("BITMAP"), "应该包含位图指令");
    assert!(tspl.contains("BARCODE"), "应该包含条码指令");
    assert!(tspl.contains("BOX"), "应该包含边框指令");
    assert!(tspl.contains("PRINT 1"), "应该包含打印指令");
    assert!(tspl.contains("\"BH1ABC\""), "应该包含呼号数据");

    // 统计指令数量
    let bitmap_count = tspl.matches("BITMAP").count();
    let barcode_count = tspl.matches("BARCODE").count();

    println!("✓ TSPL指令生成成功");
    println!("  - 位图指令: {} 个", bitmap_count);
    println!("  - 条码指令: {} 个", barcode_count);
    println!("  - 总长度: {} 字节", tspl.len());

    // 验证指令数量
    assert!(bitmap_count >= 5, "应该有至少5个位图指令（5个文本元素）");
    assert_eq!(barcode_count, 1, "应该有1个条码指令");

    println!("✅ TSPL生成器混合模式测试通过");
    println!("=====================================\n");
}

#[test]
fn test_tspl_generator_full_bitmap() {
    println!("\n========== TSPL生成器集成测试: 全位图模式 ==========");

    let config = TemplateV2Config::default_qsl_card_v2();
    let mut data = HashMap::new();
    data.insert("task_name".to_string(), "Full Test".to_string());
    data.insert("callsign".to_string(), "BA1CD".to_string());
    data.insert("sn".to_string(), "123".to_string());
    data.insert("qty".to_string(), "50".to_string());

    let resolved = TemplateEngine::resolve(&config, &data).unwrap();
    let mut layout_engine = LayoutEngine::new().unwrap();
    let layout_result = layout_engine.layout(&config, resolved).unwrap();

    let mut pipeline = RenderPipeline::new().unwrap();
    let output_config = OutputConfig {
        mode: "full_bitmap".to_string(),
        threshold: 160,
    };
    let render_result = pipeline.render(layout_result, &output_config).unwrap();

    let generator = TSPLGeneratorV2::new();
    let tspl = generator.generate(render_result, 76.0, 130.0).unwrap();

    // 验证 TSPL 内容
    assert!(tspl.contains("SIZE 76 mm, 130 mm"));
    assert!(tspl.contains("CLS"));
    assert!(tspl.contains("BITMAP"));
    assert!(tspl.contains("PRINT 1"));

    // 全位图模式不应该有独立的条码指令
    assert!(!tspl.contains("BARCODE"), "全位图模式不应该有BARCODE指令");

    // 应该只有一个 BITMAP 指令（整个画布）
    let bitmap_count = tspl.matches("BITMAP").count();
    assert_eq!(bitmap_count, 1, "应该只有1个位图指令（完整画布）");

    println!("✓ TSPL指令生成成功");
    println!("  - 位图指令: {} 个", bitmap_count);
    println!("  - 总长度: {} 字节", tspl.len());

    println!("✅ TSPL生成器全位图模式测试通过");
    println!("=====================================\n");
}

#[test]
fn test_backend_with_config_file() {
    println!("\n========== 后端集成测试: 使用配置文件 ==========");

    // 从配置文件加载
    let config_path = Path::new("config/templates/qsl-card-v2.toml");
    let config = TemplateV2Config::load_from_file(config_path).expect("加载配置文件失败");

    let mut data = HashMap::new();
    data.insert("task_name".to_string(), "From Config".to_string());
    data.insert("callsign".to_string(), "BG7XXX".to_string());
    data.insert("sn".to_string(), "001".to_string());
    data.insert("qty".to_string(), "100".to_string());

    // 完整流程
    let resolved = TemplateEngine::resolve(&config, &data).unwrap();
    let mut layout_engine = LayoutEngine::new().unwrap();
    let layout_result = layout_engine.layout(&config, resolved).unwrap();

    let mut pipeline = RenderPipeline::new().unwrap();
    let output_config = OutputConfig {
        mode: "text_bitmap_plus_native_barcode".to_string(),
        threshold: 160,
    };
    let render_result = pipeline.render(layout_result, &output_config).unwrap();

    // 测试 PDF 后端
    let temp_dir = TempDir::new().unwrap();
    let mut pdf_backend = PdfBackendV2::new(temp_dir.path().to_path_buf()).unwrap();
    let png_path = pdf_backend.render(render_result.clone()).unwrap();

    assert!(png_path.exists());
    println!("✓ PDF后端: {}", png_path.display());

    // 测试 TSPL 生成
    let generator = TSPLGeneratorV2::new();
    let tspl = generator.generate(render_result, 76.0, 130.0).unwrap();

    assert!(tspl.contains("BITMAP"));
    assert!(tspl.contains("BARCODE"));
    println!("✓ TSPL生成: {} 字节", tspl.len());

    println!("✅ 配置文件集成测试通过");
    println!("=====================================\n");
}

#[test]
fn test_multiple_cards_batch() {
    println!("\n========== 批量卡片生成测试 ==========");

    let config = TemplateV2Config::default_qsl_card_v2();
    let temp_dir = TempDir::new().unwrap();
    let mut pdf_backend = PdfBackendV2::new(temp_dir.path().to_path_buf()).unwrap();

    let test_cases = vec![
        ("BG7XXX", "001", "100"),
        ("BD7AA", "002", "200"),
        ("BH1ABC", "003", "300"),
    ];

    for (callsign, sn, qty) in test_cases {
        let mut data = HashMap::new();
        data.insert("task_name".to_string(), "Batch Test".to_string());
        data.insert("callsign".to_string(), callsign.to_string());
        data.insert("sn".to_string(), sn.to_string());
        data.insert("qty".to_string(), qty.to_string());

        let resolved = TemplateEngine::resolve(&config, &data).unwrap();
        let mut layout_engine = LayoutEngine::new().unwrap();
        let layout_result = layout_engine.layout(&config, resolved).unwrap();

        let mut pipeline = RenderPipeline::new().unwrap();
        let output_config = OutputConfig {
            mode: "full_bitmap".to_string(),
            threshold: 160,
        };
        let render_result = pipeline.render(layout_result, &output_config).unwrap();

        let png_path = pdf_backend.render(render_result).unwrap();
        assert!(png_path.exists());

        println!(
            "✓ 生成卡片: {} -> {}",
            callsign,
            png_path.file_name().unwrap().to_string_lossy()
        );
    }

    println!("✅ 批量生成测试通过");
    println!("=====================================\n");
}
