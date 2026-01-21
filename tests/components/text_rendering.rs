// 文本渲染组件测试
//
// 测试TextRenderer的完整功能，包括：
// - 多字体自动选择
// - 中英文混排
// - 1bpp位图渲染
// - 字体度量缓存

use qsl_cardhub::printer::text_renderer::TextRenderer;
use image::{ImageBuffer, Luma, Rgb, RgbImage};
use std::path::PathBuf;

#[test]
fn test_text_renderer() {
    println!("\n========== 文本渲染器测试 ==========");

    let test_dir = PathBuf::from("test_output");
    if !test_dir.exists() {
        std::fs::create_dir_all(&test_dir).expect("创建测试目录失败");
    }

    let mut text_renderer = TextRenderer::new().expect("创建TextRenderer失败");

    // 创建测试画布
    let mut img: RgbImage = ImageBuffer::from_pixel(608, 1040, Rgb([255u8, 255u8, 255u8]));

    // 测试英文文本渲染
    println!("测试英文文本渲染...");
    match text_renderer.draw_text(&mut img, "TEST TEXT", 100, 100, 24.0, false) {
        Ok(_) => println!("  ✓ 英文文本渲染成功"),
        Err(e) => println!("  ⚠️  英文文本渲染失败: {}", e),
    }

    // 测试中文标题渲染
    println!("测试中文标题渲染...");
    match text_renderer.render_chinese_headers(
        &mut img,
        "中国无线电协会业余分会-2区卡片局",
        "测试任务",
        608,
    ) {
        Ok(_) => println!("  ✓ 中文标题渲染成功"),
        Err(e) => println!("  ⚠️  中文标题渲染失败: {}", e),
    }

    // 测试字号计算
    println!("测试自动字号计算...");
    match text_renderer.calculate_max_font_size("测试文本", 300, true) {
        Ok(size) => println!("  ✓ 计算字号: {:.1}px", size),
        Err(e) => println!("  ⚠️  字号计算失败: {}", e),
    }

    // 保存测试图像
    let test_img_path = test_dir.join("text_renderer_test.png");
    match img.save(&test_img_path) {
        Ok(_) => println!("  ✓ 测试图像: {}", test_img_path.display()),
        Err(e) => println!("  ⚠️  保存图像失败: {}", e),
    }

    println!("✅ 文本渲染器测试完成");
    println!("=====================================\n");
}

#[test]
fn test_render_mixed_text_to_1bpp() {
    println!("\n========== 中英文混排1bpp渲染测试 ==========");

    let test_dir = PathBuf::from("test_output");
    if !test_dir.exists() {
        std::fs::create_dir_all(&test_dir).expect("创建测试目录失败");
    }

    let mut text_renderer = TextRenderer::new().expect("创建TextRenderer失败");

    // 测试数据
    let test_cases = vec![
        ("BG7XXX", 72.0, "english"),
        ("中国无线电协会", 48.0, "chinese"),
        ("BG7XXX 中国", 60.0, "mixed"),
        ("SN: 001", 60.0, "alphanumeric"),
        ("QTY: 100", 60.0, "alphanumeric"),
    ];

    for (text, font_size, label) in test_cases {
        println!("渲染: \"{}\" ({}pt, {})", text, font_size, label);

        // 渲染为1bpp位图
        match text_renderer.render_text(text, font_size) {
            Ok(bitmap) => {
                println!("  ✓ 位图尺寸: {}x{}", bitmap.width(), bitmap.height());

                // 验证1bpp (只有0或255)
                let mut has_black = false;
                let mut has_white = false;
                let mut invalid_pixels = 0;

                for pixel in bitmap.pixels() {
                    match pixel.0[0] {
                        0 => has_black = true,
                        255 => has_white = true,
                        _ => invalid_pixels += 1,
                    }
                }

                assert_eq!(invalid_pixels, 0, "发现非1bpp像素");
                assert!(has_black, "位图应包含黑色像素（文字）");
                assert!(has_white, "位图应包含白色像素（背景）");

                println!("  ✓ 1bpp验证通过");

                // 保存位图
                let filename = format!("text_{}_{}pt.png", label, font_size as u32);
                let filepath = test_dir.join(filename);
                if let Err(e) = bitmap.save(&filepath) {
                    println!("  ⚠️  保存位图失败: {}", e);
                } else {
                    println!("  ✓ 保存: {}", filepath.display());
                }
            }
            Err(e) => panic!("渲染 \"{}\" 失败: {}", text, e),
        }
    }

    println!("✅ 中英文混排1bpp渲染测试完成");
    println!("=====================================\n");
}

#[test]
fn test_text_measurement_accuracy() {
    println!("\n========== 文本测量精度测试 ==========");

    let mut text_renderer = TextRenderer::new().expect("创建TextRenderer失败");

    let test_cases = vec![
        ("A", 72.0),
        ("ABC", 72.0),
        ("BG7XXX", 72.0),
        ("中", 48.0),
        ("中国", 48.0),
        ("中国无线电", 48.0),
        ("BG7XXX 中国", 60.0),
        ("SN: 001", 60.0),
    ];

    for (text, font_size) in test_cases {
        // 测量文本
        let (measured_width, measured_height) = text_renderer
            .measure_text(text, font_size)
            .expect("测量失败");

        // 渲染文本
        let bitmap = text_renderer
            .render_text(text, font_size)
            .expect("渲染失败");

        // 比较测量值和实际位图尺寸
        println!(
            "\"{}\" ({}pt): 测量={}x{}, 实际={}x{}",
            text,
            font_size,
            measured_width,
            measured_height,
            bitmap.width(),
            bitmap.height()
        );

        // 允许微小误差（±5像素）
        let width_diff = (measured_width as i32 - bitmap.width() as i32).abs();
        let height_diff = (measured_height as i32 - bitmap.height() as i32).abs();

        assert!(
            width_diff <= 5,
            "宽度误差过大: {} (测量={}, 实际={})",
            width_diff,
            measured_width,
            bitmap.width()
        );
        assert!(
            height_diff <= 5,
            "高度误差过大: {} (测量={}, 实际={})",
            height_diff,
            measured_height,
            bitmap.height()
        );
    }

    println!("✅ 文本测量精度测试通过");
    println!("=====================================\n");
}

#[test]
fn test_font_metrics_cache_performance() {
    println!("\n========== 字体度量缓存性能测试 ==========");

    let mut text_renderer = TextRenderer::new().expect("创建TextRenderer失败");

    // 第一次测量（填充缓存）
    let text = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let font_size = 72.0;

    let start = std::time::Instant::now();
    text_renderer
        .measure_text(text, font_size)
        .expect("第一次测量失败");
    let first_duration = start.elapsed();

    println!("第一次测量: {:?}", first_duration);

    // 第二次测量（使用缓存）
    let start = std::time::Instant::now();
    text_renderer
        .measure_text(text, font_size)
        .expect("第二次测量失败");
    let second_duration = start.elapsed();

    println!("第二次测量: {:?} (使用缓存)", second_duration);

    // 第二次应该更快
    if second_duration < first_duration {
        let speedup = first_duration.as_secs_f64() / second_duration.as_secs_f64();
        println!("  ✓ 缓存加速: {:.1}x", speedup);
    } else {
        println!("  ℹ️  缓存未显著加速（可能环境因素）");
    }

    // 测试大量不同字符
    let chinese_text = "中国无线电协会业余分会";
    let start = std::time::Instant::now();
    for _ in 0..100 {
        text_renderer
            .measure_text(chinese_text, 48.0)
            .expect("测量失败");
    }
    let batch_duration = start.elapsed();

    println!("批量测量100次 \"{}\": {:?}", chinese_text, batch_duration);
    println!("  平均每次: {:?}", batch_duration / 100);

    println!("✅ 字体度量缓存性能测试完成");
    println!("=====================================\n");
}

#[test]
fn test_baseline_alignment() {
    println!("\n========== 基线对齐测试 ==========");

    let test_dir = PathBuf::from("test_output");
    if !test_dir.exists() {
        std::fs::create_dir_all(&test_dir).expect("创建测试目录失败");
    }

    let mut text_renderer = TextRenderer::new().expect("创建TextRenderer失败");

    // 渲染中英文混排，检查基线是否对齐
    let mixed_text = "BG7XXX 中国 123";
    let font_size = 72.0;

    let bitmap = text_renderer
        .render_text(mixed_text, font_size)
        .expect("渲染失败");

    println!(
        "混排文本 \"{}\": {}x{}",
        mixed_text,
        bitmap.width(),
        bitmap.height()
    );

    // 保存用于视觉检查
    let filepath = test_dir.join("baseline_alignment_test.png");
    bitmap.save(&filepath).expect("保存失败");
    println!("  ✓ 保存到: {}", filepath.display());

    // 检查是否有黑色像素（文字）
    let has_content = bitmap.pixels().any(|p| p.0[0] == 0);
    assert!(has_content, "位图应包含渲染的文字");

    println!("  ✓ 基线对齐测试完成（需要视觉检查）");
    println!("=====================================\n");
}
