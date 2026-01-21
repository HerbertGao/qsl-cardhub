// 字体加载测试

use qsl_cardhub::printer::font_loader::FontLoader;

#[test]
fn test_font_loading() {
    println!("\n========== 字体加载测试 ==========");

    let mut font_loader = FontLoader::new();

    // 测试英文字体加载
    println!("加载英文字体...");
    match font_loader.load_font("english") {
        Ok(_) => println!("  ✓ 英文字体加载成功"),
        Err(e) => panic!("英文字体加载失败: {}", e),
    }

    // 测试中文字体加载
    println!("加载中文字体...");
    match font_loader.load_font("chinese") {
        Ok(_) => println!("  ✓ 中文字体加载成功"),
        Err(e) => panic!("中文字体加载失败: {}", e),
    }

    println!("✅ 字体加载测试通过");
    println!("=====================================\n");
}
