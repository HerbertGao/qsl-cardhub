// 综合测试 - 模板系统 v2
//
// 测试从配置加载到 PNG/TSPL 生成的完整流程

use QSL_CardHub::api_v2::{QslCardGenerator, quick_generate_png, quick_generate_tspl};
use QSL_CardHub::config::template_v2::{OutputConfig, TemplateV2Config};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[test]
fn test_comprehensive_qsl_card_generation() {
    println!("\n========================================");
    println!("    综合测试: QSL 卡片生成系统 v2");
    println!("========================================\n");

    // 步骤 1: 加载模板配置
    println!("📋 步骤 1: 加载模板配置");
    let config_path = Path::new("config/templates/qsl-card-v2.toml");
    assert!(
        config_path.exists(),
        "配置文件不存在: {}",
        config_path.display()
    );

    let config = TemplateV2Config::load_from_file(config_path).expect("加载配置文件失败");

    println!("  ✓ 模板名称: {}", config.metadata.name);
    println!("  ✓ 模板版本: {}", config.metadata.version);
    println!(
        "  ✓ 纸张尺寸: {}x{} mm",
        config.page.width_mm, config.page.height_mm
    );
    println!("  ✓ DPI: {}", config.page.dpi);
    println!("  ✓ 元素数量: {}", config.elements.len());

    // 步骤 2: 准备运行时数据
    println!("\n📝 步骤 2: 准备运行时数据");
    let mut data = HashMap::new();
    data.insert("task_name".to_string(), "CQWW DX Contest 2026".to_string());
    data.insert("callsign".to_string(), "BG7XXX".to_string());
    data.insert("sn".to_string(), "001".to_string());
    data.insert("qty".to_string(), "500".to_string());

    println!("  ✓ 任务名称: {}", data.get("task_name").unwrap());
    println!("  ✓ 呼号: {}", data.get("callsign").unwrap());
    println!("  ✓ 序列号: {}", data.get("sn").unwrap());
    println!("  ✓ 数量: {}", data.get("qty").unwrap());

    // 步骤 3: 创建输出目录
    println!("\n📁 步骤 3: 准备输出目录");
    let output_dir = PathBuf::from("test_output/comprehensive");
    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir).expect("创建输出目录失败");
    }
    println!("  ✓ 输出目录: {}", output_dir.display());

    // 步骤 4: 测试混合模式渲染
    println!("\n🖨️  步骤 4: 测试混合模式渲染");
    let mixed_mode_config = OutputConfig {
        mode: "text_bitmap_plus_native_barcode".to_string(),
        threshold: 160,
    };

    let mut generator = QslCardGenerator::new().expect("创建生成器失败");

    let png_path_mixed = generator
        .generate_png(&config, &data, output_dir.clone(), &mixed_mode_config)
        .expect("生成混合模式PNG失败");

    assert!(png_path_mixed.exists(), "PNG文件不存在");

    let img_mixed = image::open(&png_path_mixed).expect("打开PNG失败");
    println!("  ✓ 混合模式PNG: {}", png_path_mixed.display());
    println!("  ✓ 图像尺寸: {}x{}", img_mixed.width(), img_mixed.height());
    println!(
        "  ✓ 文件大小: {} KB",
        std::fs::metadata(&png_path_mixed).unwrap().len() / 1024
    );

    // 步骤 5: 测试全位图模式渲染
    println!("\n🖼️  步骤 5: 测试全位图模式渲染");
    let full_bitmap_config = OutputConfig {
        mode: "full_bitmap".to_string(),
        threshold: 160,
    };

    let png_path_full = generator
        .generate_png(&config, &data, output_dir.clone(), &full_bitmap_config)
        .expect("生成全位图PNG失败");

    assert!(png_path_full.exists(), "PNG文件不存在");

    let img_full = image::open(&png_path_full).expect("打开PNG失败");
    println!("  ✓ 全位图PNG: {}", png_path_full.display());
    println!("  ✓ 图像尺寸: {}x{}", img_full.width(), img_full.height());
    println!(
        "  ✓ 文件大小: {} KB",
        std::fs::metadata(&png_path_full).unwrap().len() / 1024
    );

    // 步骤 6: 生成 TSPL 指令（混合模式）
    println!("\n📄 步骤 6: 生成TSPL指令（混合模式）");
    let tspl_mixed = generator
        .generate_tspl(&config, &data, &mixed_mode_config)
        .expect("生成TSPL失败");

    assert!(tspl_mixed.contains("SIZE"), "TSPL应包含SIZE指令");
    assert!(tspl_mixed.contains("BITMAP"), "TSPL应包含BITMAP指令");
    assert!(tspl_mixed.contains("BARCODE"), "TSPL应包含BARCODE指令");
    assert!(tspl_mixed.contains("BG7XXX"), "TSPL应包含呼号");

    let tspl_path_mixed = output_dir.join("mixed_mode.tspl");
    std::fs::write(&tspl_path_mixed, &tspl_mixed).expect("写入TSPL文件失败");

    println!("  ✓ TSPL文件: {}", tspl_path_mixed.display());
    println!("  ✓ TSPL大小: {} KB", tspl_mixed.len() / 1024);
    println!("  ✓ BITMAP指令数: {}", tspl_mixed.matches("BITMAP").count());
    println!(
        "  ✓ BARCODE指令数: {}",
        tspl_mixed.matches("BARCODE").count()
    );

    // 步骤 7: 生成 TSPL 指令（全位图模式）
    println!("\n📄 步骤 7: 生成TSPL指令（全位图模式）");
    let tspl_full = generator
        .generate_tspl(&config, &data, &full_bitmap_config)
        .expect("生成TSPL失败");

    assert!(tspl_full.contains("SIZE"), "TSPL应包含SIZE指令");
    assert!(tspl_full.contains("BITMAP"), "TSPL应包含BITMAP指令");
    assert!(
        !tspl_full.contains("BARCODE"),
        "全位图模式不应包含BARCODE指令"
    );

    let tspl_path_full = output_dir.join("full_bitmap.tspl");
    std::fs::write(&tspl_path_full, &tspl_full).expect("写入TSPL文件失败");

    println!("  ✓ TSPL文件: {}", tspl_path_full.display());
    println!("  ✓ TSPL大小: {} KB", tspl_full.len() / 1024);
    println!("  ✓ BITMAP指令数: {}", tspl_full.matches("BITMAP").count());

    // 步骤 8: 便捷API测试
    println!("\n🚀 步骤 8: 测试便捷API");
    let quick_png = quick_generate_png(Some(config_path), &data, output_dir.clone(), "full_bitmap")
        .expect("快速生成PNG失败");

    assert!(quick_png.exists());
    println!("  ✓ 快速生成PNG: {}", quick_png.display());

    let quick_tspl =
        quick_generate_tspl(Some(config_path), &data, "text_bitmap_plus_native_barcode")
            .expect("快速生成TSPL失败");

    println!("  ✓ 快速生成TSPL: {} KB", quick_tspl.len() / 1024);

    // 步骤 9: 批量生成测试
    println!("\n📦 步骤 9: 批量生成测试");
    let batch_dir = output_dir.join("batch");
    std::fs::create_dir_all(&batch_dir).ok();

    for i in 1..=5 {
        let mut batch_data = data.clone();
        batch_data.insert("sn".to_string(), format!("{:03}", i));

        let batch_png = generator
            .generate_png(&config, &batch_data, batch_dir.clone(), &full_bitmap_config)
            .expect("批量生成失败");

        assert!(batch_png.exists());
        println!(
            "  ✓ 批量 {}/5: {}",
            i,
            batch_png.file_name().unwrap().to_string_lossy()
        );
    }

    // 步骤 10: 不同内容变化测试
    println!("\n🔄 步骤 10: 不同内容变化测试");
    let test_cases = vec![
        ("短呼号", "BH1AA", "001", "50"),
        ("长呼号", "BG7XXX/QRP", "999", "1000"),
        ("纯数字", "123456", "100", "200"),
    ];

    for (label, callsign, sn, qty) in test_cases {
        let mut test_data = HashMap::new();
        test_data.insert("task_name".to_string(), format!("测试-{}", label));
        test_data.insert("callsign".to_string(), callsign.to_string());
        test_data.insert("sn".to_string(), sn.to_string());
        test_data.insert("qty".to_string(), qty.to_string());

        let test_png = generator
            .generate_png(&config, &test_data, output_dir.clone(), &full_bitmap_config)
            .expect(&format!("生成{}失败", label));

        assert!(test_png.exists());
        println!("  ✓ {}: {}", label, test_png.display());
    }

    // 步骤 11: 验证图像质量
    println!("\n✨ 步骤 11: 验证图像质量");
    let img = image::open(&png_path_full).unwrap();
    let (width, height) = (img.width(), img.height());

    // 验证尺寸
    assert_eq!(width, 608, "图像宽度应为608像素");
    assert_eq!(height, 1039, "图像高度应为1039像素");
    println!("  ✓ 尺寸验证通过: {}x{}", width, height);

    // 验证包含内容（应该有黑色像素）
    let gray_img = img.to_luma8();
    let black_pixels = gray_img.pixels().filter(|p| p.0[0] < 128).count();
    let total_pixels = (width * height) as usize;
    let black_ratio = black_pixels as f32 / total_pixels as f32;

    assert!(black_ratio > 0.01, "应该包含内容（黑色像素）");
    assert!(black_ratio < 0.30, "黑色像素比例应该合理");
    println!("  ✓ 内容验证通过: {:.1}% 黑色像素", black_ratio * 100.0);

    // 最终总结
    println!("\n========================================");
    println!("✅ 综合测试完成！");
    println!("========================================");
    println!("\n📊 测试统计:");
    println!("  • PNG文件生成: 9 个");
    println!("  • TSPL文件生成: 2 个");
    println!("  • 渲染模式: 2 种");
    println!("  • 内容变化: 3 种");
    println!("\n📁 输出目录: {}", output_dir.display());
    println!("\n🎉 所有测试通过！模板系统 v2 工作正常。\n");
}

#[test]
fn test_default_template_generation() {
    println!("\n========================================");
    println!("    测试: 默认模板快速生成");
    println!("========================================\n");

    let output_dir = PathBuf::from("test_output/default");
    std::fs::create_dir_all(&output_dir).ok();

    let mut data = HashMap::new();
    data.insert("task_name".to_string(), "默认模板测试".to_string());
    data.insert("callsign".to_string(), "BG7XXX".to_string());
    data.insert("sn".to_string(), "001".to_string());
    data.insert("qty".to_string(), "100".to_string());

    // 使用便捷API，不提供模板路径（使用默认模板）
    let png_path = quick_generate_png(
        None, // 使用默认模板
        &data,
        output_dir.clone(),
        "full_bitmap",
    )
    .expect("生成PNG失败");

    assert!(png_path.exists());
    println!("✅ 默认模板PNG: {}", png_path.display());

    let tspl =
        quick_generate_tspl(None, &data, "text_bitmap_plus_native_barcode").expect("生成TSPL失败");

    assert!(tspl.len() > 1000);
    println!("✅ 默认模板TSPL: {} KB", tspl.len() / 1024);
    println!("\n🎉 默认模板测试通过！\n");
}

#[test]
fn test_performance_batch_generation() {
    println!("\n========================================");
    println!("    性能测试: 批量生成");
    println!("========================================\n");

    let output_dir = PathBuf::from("test_output/performance");
    std::fs::create_dir_all(&output_dir).ok();

    let config = TemplateV2Config::default_qsl_card_v2();
    let output_config = OutputConfig {
        mode: "full_bitmap".to_string(),
        threshold: 160,
    };

    let mut generator = QslCardGenerator::new().unwrap();

    let start = std::time::Instant::now();
    let count = 20;

    for i in 1..=count {
        let mut data = HashMap::new();
        data.insert("task_name".to_string(), "性能测试".to_string());
        data.insert("callsign".to_string(), format!("BG7{:03}", i));
        data.insert("sn".to_string(), format!("{:03}", i));
        data.insert("qty".to_string(), "100".to_string());

        generator
            .generate_png(&config, &data, output_dir.clone(), &output_config)
            .expect("生成失败");
    }

    let elapsed = start.elapsed();
    let per_card = elapsed.as_millis() / count;

    println!("📊 性能统计:");
    println!("  • 总卡片数: {}", count);
    println!("  • 总耗时: {:.2}s", elapsed.as_secs_f32());
    println!("  • 平均每张: {}ms", per_card);
    println!("  • 每秒生成: {:.1} 张", 1000.0 / per_card as f32);

    // 性能基准：每张卡片应在 350ms 内完成（合理标准，包含文本渲染、布局计算、图像生成）
    // 注：首次加载字体会较慢，后续有缓存会加快
    assert!(
        per_card < 350,
        "性能不达标：每张耗时 {}ms > 350ms",
        per_card
    );

    println!("\n✅ 性能测试通过！\n");
}
