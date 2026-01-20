// 端到端集成测试
//
// 测试完整流程: 配置 → 模板解析 → 布局计算 → 渲染输出

use QSL_CardHub::config::template_v2::{TemplateV2Config, OutputConfig};
use QSL_CardHub::printer::layout_engine::LayoutEngine;
use QSL_CardHub::printer::render_pipeline::{RenderPipeline, RenderResult};
use QSL_CardHub::printer::template_engine::TemplateEngine;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[test]
fn test_end_to_end_mixed_mode() {
    println!("\n========== 端到端测试: 混合模式 ==========");

    // 1. 加载配置
    let config_path = Path::new("config/templates/qsl-card-v2.toml");
    let config = TemplateV2Config::load_from_file(config_path)
        .expect("加载配置失败");

    println!("✓ 加载配置: {}", config.metadata.name);

    // 2. 准备运行时数据
    let mut data = HashMap::new();
    data.insert("task_name".to_string(), "CQWW DX Contest".to_string());
    data.insert("callsign".to_string(), "BG7XXX".to_string());
    data.insert("sn".to_string(), "001".to_string());
    data.insert("qty".to_string(), "500".to_string());

    println!("✓ 准备运行时数据");

    // 3. 模板解析
    let resolved_elements = TemplateEngine::resolve(&config, &data)
        .expect("模板解析失败");

    println!("✓ 解析 {} 个元素", resolved_elements.len());

    // 4. 布局计算
    let mut layout_engine = LayoutEngine::new().expect("创建布局引擎失败");
    let layout_result = layout_engine
        .layout(&config, resolved_elements)
        .expect("布局计算失败");

    println!("✓ 布局计算完成: {}x{} dots", layout_result.canvas_width, layout_result.canvas_height);

    // 5. 渲染管道
    let mut render_pipeline = RenderPipeline::new().expect("创建渲染管道失败");
    let output_config = OutputConfig {
        mode: "text_bitmap_plus_native_barcode".to_string(),
        threshold: 160,
    };

    let render_result = render_pipeline
        .render(layout_result, &output_config)
        .expect("渲染失败");

    println!("✓ 渲染完成");

    // 6. 验证渲染结果
    match render_result {
        RenderResult::MixedMode {
            bitmaps,
            native_barcodes,
            canvas_size,
            border,
        } => {
            println!("  模式: 混合模式");
            println!("  文本位图: {} 个", bitmaps.len());
            println!("  原生条码: {} 个", native_barcodes.len());
            println!("  画布尺寸: {}x{}", canvas_size.0, canvas_size.1);
            println!("  边框: {}", if border.is_some() { "启用" } else { "禁用" });

            assert_eq!(bitmaps.len(), 5, "应该有5个文本位图");
            assert_eq!(native_barcodes.len(), 1, "应该有1个原生条码");
            assert_eq!(canvas_size, (608, 1039));
            assert!(border.is_some());

            // 验证条形码信息
            let barcode = &native_barcodes[0];
            assert_eq!(barcode.content, "BG7XXX");
            assert_eq!(barcode.barcode_type, "code128");

            // 验证所有位图都是1bpp
            for (i, (x, y, bitmap)) in bitmaps.iter().enumerate() {
                println!("  位图[{}]: {}x{} at ({}, {})", i, bitmap.width(), bitmap.height(), x, y);

                // 验证所有像素都是0或255
                for pixel in bitmap.pixels() {
                    assert!(
                        pixel.0[0] == 0 || pixel.0[0] == 255,
                        "位图应该是1bpp (0或255)"
                    );
                }
            }
        }
        _ => panic!("应该返回MixedMode"),
    }

    println!("✅ 端到端测试: 混合模式 - 通过");
    println!("=====================================\n");
}

#[test]
fn test_end_to_end_full_bitmap() {
    println!("\n========== 端到端测试: 全位图模式 ==========");

    // 1. 加载配置并修改为全位图模式
    let config_path = Path::new("config/templates/qsl-card-v2.toml");
    let mut config = TemplateV2Config::load_from_file(config_path)
        .expect("加载配置失败");

    println!("✓ 加载配置: {}", config.metadata.name);

    // 2. 准备数据
    let mut data = HashMap::new();
    data.insert("task_name".to_string(), "IARU HF Championship".to_string());
    data.insert("callsign".to_string(), "BD7AA".to_string());
    data.insert("sn".to_string(), "100".to_string());
    data.insert("qty".to_string(), "250".to_string());

    // 3. 完整流程
    let resolved_elements = TemplateEngine::resolve(&config, &data).unwrap();
    let mut layout_engine = LayoutEngine::new().unwrap();
    let layout_result = layout_engine.layout(&config, resolved_elements).unwrap();

    let mut render_pipeline = RenderPipeline::new().unwrap();
    let output_config = OutputConfig {
        mode: "full_bitmap".to_string(),
        threshold: 160,
    };

    let render_result = render_pipeline.render(layout_result, &output_config).unwrap();

    // 4. 验证全位图结果
    match render_result {
        RenderResult::FullBitmap { canvas, canvas_size } => {
            println!("  模式: 全位图模式");
            println!("  画布尺寸: {}x{}", canvas.width(), canvas.height());

            assert_eq!(canvas.width(), 608);
            assert_eq!(canvas.height(), 1039);
            assert_eq!(canvas_size, (608, 1039));

            // 验证是1bpp
            for pixel in canvas.pixels() {
                assert!(pixel.0[0] == 0 || pixel.0[0] == 255);
            }

            // 验证包含内容（有黑色像素）
            let has_black = canvas.pixels().any(|p| p.0[0] == 0);
            assert!(has_black, "画布应该包含黑色像素");

            // 统计黑色像素比例
            let total_pixels = canvas.width() * canvas.height();
            let black_pixels = canvas.pixels().filter(|p| p.0[0] == 0).count();
            let black_ratio = black_pixels as f32 / total_pixels as f32;

            println!("  黑色像素: {} / {} ({:.1}%)", black_pixels, total_pixels, black_ratio * 100.0);

            // 合理的黑色像素比例应该在2%-15%之间（文字和条码）
            assert!(black_ratio > 0.01 && black_ratio < 0.20,
                "黑色像素比例应该合理");

            // 可选：保存用于视觉检查
            let test_dir = PathBuf::from("test_output");
            if !test_dir.exists() {
                std::fs::create_dir_all(&test_dir).ok();
            }
            let output_path = test_dir.join("end_to_end_full_bitmap.png");
            canvas.save(&output_path).ok();
            println!("  保存到: {}", output_path.display());
        }
        _ => panic!("应该返回FullBitmap"),
    }

    println!("✅ 端到端测试: 全位图模式 - 通过");
    println!("=====================================\n");
}

#[test]
fn test_different_content_variations() {
    println!("\n========== 不同内容变化测试 ==========");

    let config = TemplateV2Config::default_qsl_card_v2();

    let test_cases = vec![
        ("短呼号", "BH1AA", "001", "50"),
        ("长呼号", "BG7XXX/QRP", "999", "1000"),
        ("纯数字", "123456", "100", "200"),
    ];

    for (label, callsign, sn, qty) in test_cases {
        println!("测试场景: {}", label);

        let mut data = HashMap::new();
        data.insert("task_name".to_string(), format!("测试-{}", label));
        data.insert("callsign".to_string(), callsign.to_string());
        data.insert("sn".to_string(), sn.to_string());
        data.insert("qty".to_string(), qty.to_string());

        let resolved_elements = TemplateEngine::resolve(&config, &data).unwrap();
        let mut layout_engine = LayoutEngine::new().unwrap();
        let layout_result = layout_engine.layout(&config, resolved_elements).unwrap();

        let mut render_pipeline = RenderPipeline::new().unwrap();
        let output_config = OutputConfig {
            mode: "full_bitmap".to_string(),
            threshold: 160,
        };

        let result = render_pipeline.render(layout_result, &output_config);

        match result {
            Ok(RenderResult::FullBitmap { canvas, .. }) => {
                let has_content = canvas.pixels().any(|p| p.0[0] == 0);
                assert!(has_content, "场景 {} 应该有内容", label);
                println!("  ✓ {} 渲染成功: {}x{}", label, canvas.width(), canvas.height());
            }
            Err(e) => panic!("场景 {} 渲染失败: {}", label, e),
            _ => panic!("场景 {} 返回了错误的结果类型", label),
        }
    }

    println!("✅ 不同内容变化测试 - 通过");
    println!("=====================================\n");
}
