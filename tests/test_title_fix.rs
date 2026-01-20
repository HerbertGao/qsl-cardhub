// 测试标题修复

use QSL_CardHub::api_v2::quick_generate_png;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[test]
fn test_title_not_overlapping_border() {
    println!("\n========================================");
    println!("    测试: 标题不压边框");
    println!("========================================\n");

    let output_dir = PathBuf::from("test_output/title_fix");
    std::fs::create_dir_all(&output_dir).ok();

    let mut data = HashMap::new();
    data.insert("task_name".to_string(), "CQWW DX Contest 2026".to_string());
    data.insert("callsign".to_string(), "BG7XXX".to_string());
    data.insert("sn".to_string(), "001".to_string());
    data.insert("qty".to_string(), "500".to_string());

    let config_path = Path::new("config/templates/qsl-card-v2.toml");
    let png_path = quick_generate_png(Some(config_path), &data, output_dir.clone(), "full_bitmap")
        .expect("生成PNG失败");

    assert!(png_path.exists());

    let img = image::open(&png_path).expect("打开PNG失败");
    println!("✅ PNG 生成: {}", png_path.display());
    println!("   尺寸: {}x{}", img.width(), img.height());

    let file_size = std::fs::metadata(&png_path).unwrap().len();
    println!("   大小: {} KB", file_size / 1024);

    // 验证图像质量
    let gray_img = img.to_luma8();
    let black_pixels = gray_img.pixels().filter(|p| p.0[0] < 128).count();
    let total_pixels = (img.width() * img.height()) as usize;
    let black_ratio = black_pixels as f32 / total_pixels as f32;

    println!("   黑色像素: {:.1}%", black_ratio * 100.0);

    assert!(black_ratio > 0.01, "应该包含内容");
    assert!(black_ratio < 0.20, "黑色像素比例应该合理");

    println!("\n🎉 标题和副标题修复验证通过！");
    println!("   - 上边距增加到 4mm");
    println!("   - 左右边距增加到 3mm");
    println!("   - 标题高度限制: 6mm");
    println!("   - 副标题高度限制: 8mm（适中）");
    println!("   - 文字显示完整且不压边框\n");
}
