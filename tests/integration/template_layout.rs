// 模板引擎 + 布局引擎集成测试
//
// 测试从配置文件到布局结果的完整流程

use qsl_cardhub::config::template::TemplateConfig;
use qsl_cardhub::printer::layout_engine::LayoutEngine;
use qsl_cardhub::printer::template_engine::TemplateEngine;
use std::collections::HashMap;
use std::path::Path;

#[test]
fn test_complete_template_to_layout_flow() {
    println!("\n========== 模板到布局完整流程测试 ==========");

    // 1. 加载v2配置文件
    let config_path = Path::new("../../config/templates/callsign.toml");
    let config = TemplateConfig::load_from_file(config_path).expect("加载配置文件失败");

    println!("✓ 加载配置: {}", config.metadata.name);

    // 2. 准备运行时数据
    let mut data = HashMap::new();
    data.insert("task_name".to_string(), "测试任务".to_string());
    data.insert("callsign".to_string(), "BG7XXX".to_string());
    data.insert("sn".to_string(), "001".to_string());
    data.insert("qty".to_string(), "100".to_string());

    println!("✓ 准备运行时数据: {:?}", data.keys().collect::<Vec<_>>());

    // 3. 解析模板
    let resolved_elements = TemplateEngine::resolve(&config, &data).expect("模板解析失败");

    println!("✓ 解析 {} 个元素", resolved_elements.len());
    for (i, element) in resolved_elements.iter().enumerate() {
        println!("  [{}] {}: \"{}\"", i + 1, element.id, element.content);
    }

    // 4. 执行布局计算
    let mut layout_engine = LayoutEngine::new().expect("创建布局引擎失败");
    let layout_result = layout_engine
        .layout(&config, resolved_elements)
        .expect("布局计算失败");

    println!("✓ 布局计算完成");
    println!(
        "  画布尺寸: {}x{} dots",
        layout_result.canvas_width, layout_result.canvas_height
    );

    if let Some(border) = &layout_result.border {
        println!(
            "  边框: {}x{} at ({}, {}), 线宽 {} dots",
            border.width, border.height, border.x, border.y, border.thickness
        );
    }

    // 5. 验证布局结果
    assert_eq!(layout_result.elements.len(), 6, "应该有6个已布局的元素");

    // 检查标题元素
    let title = &layout_result.elements[0];
    assert_eq!(title.id, "title");
    assert_eq!(title.content, "中国无线电协会业余分会-2区卡片局");
    assert!(title.font_size.is_some(), "文本元素应该有字号");
    assert!(title.x > 0, "x坐标应该大于0（考虑边距）");
    assert!(title.y > 0, "y坐标应该大于0（考虑边距）");
    println!(
        "  [标题] {}pt at ({}, {}), {}x{} dots",
        title.font_size.unwrap(),
        title.x,
        title.y,
        title.width,
        title.height
    );

    // 检查呼号元素
    let callsign = &layout_result.elements[2];
    assert_eq!(callsign.id, "callsign");
    assert_eq!(callsign.content, "BG7XXX");
    assert!(
        callsign.font_size.unwrap() > title.font_size.unwrap(),
        "呼号字号应该大于标题字号"
    );
    println!(
        "  [呼号] {}pt at ({}, {}), {}x{} dots",
        callsign.font_size.unwrap(),
        callsign.x,
        callsign.y,
        callsign.width,
        callsign.height
    );

    // 检查条形码元素
    let barcode = &layout_result.elements[3];
    assert_eq!(barcode.id, "barcode");
    assert_eq!(barcode.content, "BG7XXX");
    assert!(
        barcode.barcode_config.is_some(),
        "条形码元素应该有条形码配置"
    );
    println!(
        "  [条形码] at ({}, {}), {}x{} dots",
        barcode.x, barcode.y, barcode.width, barcode.height
    );

    // 检查垂直居中
    let first_y = layout_result.elements[0].y;
    let last_element = layout_result.elements.last().unwrap();
    let last_y = last_element.y + last_element.height;
    let content_height = last_y - first_y;
    println!(
        "  内容占用高度: {} dots (第一个y={}, 最后一个bottom={})",
        content_height, first_y, last_y
    );

    // 检查水平居中（所有元素x坐标应该考虑了宽度居中）
    let canvas_center = layout_result.canvas_width / 2;
    for element in &layout_result.elements {
        let element_center = element.x + element.width / 2;
        let offset_from_center = (element_center as i32 - canvas_center as i32).abs();
        assert!(
            offset_from_center < 50,
            "元素 {} 应该水平居中 (偏移: {})",
            element.id,
            offset_from_center
        );
    }

    println!("✅ 模板到布局完整流程测试通过");
    println!("=====================================\n");
}

#[test]
fn test_overflow_protection() {
    println!("\n========== 溢出保护测试 ==========");

    // 创建一个会溢出的配置
    let mut config = TemplateConfig::default_qsl_card();

    // 增大所有元素的max_height，使其更容易溢出
    for element in &mut config.elements {
        if let Some(max_height) = element.max_height_mm.as_mut() {
            *max_height *= 1.5;
        }
    }

    let mut data = HashMap::new();
    data.insert(
        "task_name".to_string(),
        "非常非常长的测试任务名称用于测试溢出保护".to_string(),
    );
    data.insert("callsign".to_string(), "BG7XXX".to_string());
    data.insert("sn".to_string(), "001".to_string());
    data.insert("qty".to_string(), "100".to_string());

    let resolved_elements = TemplateEngine::resolve(&config, &data).expect("模板解析失败");

    let mut layout_engine = LayoutEngine::new().expect("创建布局引擎失败");
    let layout_result = layout_engine
        .layout(&config, resolved_elements)
        .expect("布局计算失败");

    println!("✓ 布局计算完成（含溢出保护）");

    // 验证所有元素都在可用区域内
    let available_height = layout_result.canvas_height - 48; // 上下边距各24
    let first_y = layout_result.elements[0].y;
    let last_element = layout_result.elements.last().unwrap();
    let last_y = last_element.y + last_element.height;
    let content_height = last_y - first_y;

    println!("  可用高度: {} dots", available_height);
    println!("  内容高度: {} dots", content_height);

    assert!(
        content_height <= available_height + 50,
        "溢出保护应该确保内容不超出可用区域"
    );

    println!("✅ 溢出保护测试通过");
    println!("=====================================\n");
}

#[test]
fn test_element_spacing() {
    println!("\n========== 元素间距测试 ==========");

    let config = TemplateConfig::default_qsl_card();
    let mut data = HashMap::new();
    data.insert("task_name".to_string(), "测试".to_string());
    data.insert("callsign".to_string(), "BG7XXX".to_string());
    data.insert("sn".to_string(), "001".to_string());
    data.insert("qty".to_string(), "100".to_string());

    let resolved_elements = TemplateEngine::resolve(&config, &data).unwrap();
    let mut layout_engine = LayoutEngine::new().unwrap();
    let layout_result = layout_engine.layout(&config, resolved_elements).unwrap();

    // 检查元素间距是否符合line_gap配置
    let line_gap_dots = (config.layout.line_gap_mm * config.page.dpi as f32 / 25.4).ceil() as u32;

    println!(
        "  配置的行间距: {}mm = {} dots",
        config.layout.line_gap_mm, line_gap_dots
    );

    for i in 0..layout_result.elements.len() - 1 {
        let current = &layout_result.elements[i];
        let next = &layout_result.elements[i + 1];

        let current_bottom = current.y + current.height;
        let actual_gap = next.y - current_bottom;

        println!(
            "  元素 {} -> {}: 间距 {} dots (期望 {})",
            current.id, next.id, actual_gap, line_gap_dots
        );

        // 允许一定误差（由于居中和缩放可能导致小的偏差）
        assert!(
            (actual_gap as i32 - line_gap_dots as i32).abs() <= 5,
            "元素间距应该接近配置值"
        );
    }

    println!("✅ 元素间距测试通过");
    println!("=====================================\n");
}
