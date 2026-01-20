// 条形码编码和渲染测试

use QSL_CardHub::printer::barcode_renderer::BarcodeRenderer;
use barcoders::sym::code128::Code128;
use image::{ImageBuffer, Rgb, RgbImage};
use std::path::PathBuf;

#[test]
fn test_barcode_encoder() {
    println!("\n========== 条形码编码器测试 ==========");

    // 测试不同的数据（使用 Start B 前缀，与实际渲染一致）
    let test_data = vec![
        ("BG7XXX", "标准呼号"),
        ("BD7ABC123", "长呼号"),
        ("VR2XYZ", "短呼号"),
        ("BA1AA", "短呼号"),
    ];

    for (data, desc) in test_data {
        println!("测试数据: '{}' ({})", data, desc);

        // 使用与 barcode_renderer 相同的前缀
        let prefixed_data = format!("\u{0181}{}", data);

        match Code128::new(&prefixed_data) {
            Ok(barcode) => {
                let encoded = barcode.encode();
                println!("  ✓ 编码成功，长度: {} bits", encoded.len());
            }
            Err(e) => {
                panic!("Code128 编码失败 '{}': {}", data, e);
            }
        }
    }

    println!("✅ 条形码编码器测试通过");
    println!("=====================================\n");
}

#[test]
fn test_barcode_rendering() {
    println!("\n========== 条形码渲染测试 ==========");

    let test_dir = PathBuf::from("test_output");
    if !test_dir.exists() {
        std::fs::create_dir_all(&test_dir).expect("创建测试目录失败");
    }

    // 创建白色画布
    let mut img: RgbImage = ImageBuffer::from_pixel(600, 200, Rgb([255u8, 255u8, 255u8]));

    // 创建条形码渲染器
    let renderer = BarcodeRenderer::new();

    println!("测试呼号: BG7XXX");

    // 渲染条形码
    match renderer.render_code128(&mut img, "BG7XXX", 50, 50, 400, 100) {
        Ok(_) => {
            println!("  ✓ 条形码渲染成功");

            // 保存图像
            let output_path = test_dir.join("barcode_test.png");
            match img.save(&output_path) {
                Ok(_) => println!("  ✓ 保存到: {}", output_path.display()),
                Err(e) => println!("  ⚠️  保存失败: {}", e),
            }
        }
        Err(e) => {
            panic!("条形码渲染失败: {}", e);
        }
    }

    println!("✅ 条形码渲染测试通过");
    println!("=====================================\n");
}
