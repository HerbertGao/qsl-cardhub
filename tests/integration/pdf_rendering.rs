// PDF 渲染集成测试
//
// 测试完整的 PDF 后端渲染流程

use QSL_CardHub::printer::PdfBackend;
use QSL_CardHub::printer::backend::PrinterBackend;
use std::path::PathBuf;

/// 确保测试输出目录存在
fn ensure_test_dir() -> PathBuf {
    let test_dir = PathBuf::from("test_output");
    if !test_dir.exists() {
        std::fs::create_dir_all(&test_dir).expect("创建测试目录失败");
    }
    test_dir
}

/// 生成标准 QSL 卡片 TSPL（无任务名称）
fn generate_qsl_tspl(callsign: &str, serial: u32, qty: u32) -> String {
    format!(
        r#"SIZE 76 mm, 130 mm
GAP 2 mm, 0 mm
DIRECTION 0
CLS
TEXT 304,80,"5",0,3,3,"{}"
BARCODE 179,170,"128",80,1,0,2,2,"{}"
TEXT 304,440,"5",0,2,2,"SN:{}"
TEXT 304,480,"5",0,2,2,"QTY:{}"
BOX 10,10,598,1030,2
PRINT 1
"#,
        callsign, callsign, serial, qty
    )
}

/// 生成带任务名称的 QSL 卡片 TSPL
fn generate_qsl_tspl_with_task(callsign: &str, serial: u32, qty: u32, task_name: &str) -> String {
    format!(
        r#"; TASK_NAME: {}
SIZE 76 mm, 130 mm
GAP 2 mm, 0 mm
DIRECTION 0
CLS
TEXT 304,80,"5",0,3,3,"{}"
BARCODE 179,170,"128",80,1,0,2,2,"{}"
TEXT 304,440,"5",0,2,2,"SN:{}"
TEXT 304,480,"5",0,2,2,"QTY:{}"
BOX 10,10,598,1030,2
PRINT 1
"#,
        task_name, callsign, callsign, serial, qty
    )
}

#[test]
fn test_pdf_backend_basic() {
    println!("\n========== PDF 基础渲染测试 ==========");

    let test_dir = ensure_test_dir();
    let pdf_backend = PdfBackend::new(test_dir.clone()).expect("创建 PDF 后端失败");

    // 测试基本的 QSL 卡片渲染
    let tspl = generate_qsl_tspl("BG7XXX", 1, 10);

    println!("测试呼号: BG7XXX, SN:1, QTY:10");
    let result = pdf_backend.send_raw("PDF 测试打印机", tspl.as_bytes());

    assert!(result.is_ok(), "PDF 渲染失败: {:?}", result.err());

    println!("✅ PDF 基础渲染测试通过");
    println!("📁 输出: {}", test_dir.display());
    println!("=====================================\n");
}

#[test]
fn test_pdf_multiple_callsigns() {
    println!("\n========== 多呼号渲染测试 ==========");

    let test_dir = ensure_test_dir();
    let pdf_backend = PdfBackend::new(test_dir.clone()).expect("创建 PDF 后端失败");

    // 测试不同长度和数值的呼号
    let test_cases = vec![
        ("BG7XXX", 1, 5, "短呼号，小数值"),
        ("BD7ABC123", 10, 1, "长呼号，中等序列号"),
        ("VR2XYZ", 100, 50, "中等呼号，大数值"),
        ("BA1AA", 999, 100, "短呼号，最大数值"),
    ];

    for (callsign, serial, qty, desc) in test_cases {
        println!("测试: {} - {}", desc, callsign);

        let tspl = generate_qsl_tspl(callsign, serial, qty);
        let result = pdf_backend.send_raw("PDF 测试打印机", tspl.as_bytes());

        assert!(
            result.is_ok(),
            "呼号 {} 渲染失败: {:?}",
            callsign,
            result.err()
        );

        println!("  ✓ 渲染成功 (SN:{}, QTY:{})", serial, qty);
    }

    println!("✅ 多呼号渲染测试通过");
    println!("=====================================\n");
}

#[test]
fn test_pdf_chinese_title() {
    println!("\n========== 中文标题渲染测试 ==========");

    let test_dir = ensure_test_dir();
    let pdf_backend = PdfBackend::new(test_dir.clone()).expect("创建 PDF 后端失败");

    // 最小化的 TSPL，只渲染标题
    let minimal_tspl = r#"SIZE 76 mm, 130 mm
GAP 2 mm, 0 mm
DIRECTION 0
CLS
PRINT 1
"#;

    println!("测试中文标题单独渲染");
    let result = pdf_backend.send_raw("PDF 测试打印机", minimal_tspl.as_bytes());

    assert!(result.is_ok(), "中文标题渲染失败: {:?}", result.err());

    println!("✅ 中文标题渲染测试通过");
    println!("请检查: 中国无线电协会业余分会-2区卡片局");
    println!("=====================================\n");
}

#[test]
fn test_pdf_text_centering() {
    println!("\n========== 文本居中测试 ==========");

    let test_dir = ensure_test_dir();
    let pdf_backend = PdfBackend::new(test_dir.clone()).expect("创建 PDF 后端失败");

    // 测试较长的文本是否正确居中
    let tspl = r#"SIZE 76 mm, 130 mm
GAP 2 mm, 0 mm
DIRECTION 0
CLS
TEXT 304,440,"5",0,2,2,"SN:12345"
TEXT 304,480,"5",0,2,2,"QTY:100"
PRINT 1
"#;

    println!("测试 SN 和 QTY 文本居中");
    let result = pdf_backend.send_raw("PDF 测试打印机", tspl.as_bytes());

    assert!(result.is_ok(), "文本居中测试失败: {:?}", result.err());

    println!("✅ 文本居中测试通过");
    println!("请检查: SN:12345 和 QTY:100 是否居中");
    println!("=====================================\n");
}

#[test]
fn test_task_name_rendering() {
    println!("\n========== 任务名称渲染测试 ==========");

    let test_dir = ensure_test_dir();
    let pdf_backend = PdfBackend::new(test_dir.clone()).expect("创建 PDF 后端失败");

    // 测试不同的任务名称场景
    let test_cases = vec![
        ("2026年度第一次转卡", "BG7XXX", 1, 10),
        ("春季联谊活动", "BD7ABC", 50, 20),
        ("国庆特别活动", "VR2XYZ", 100, 5),
        ("日常QSO卡片", "BA1AA", 200, 1),
    ];

    for (task_name, callsign, serial, qty) in test_cases {
        println!("\n任务: '{}' | 呼号: {}", task_name, callsign);

        let tspl = generate_qsl_tspl_with_task(callsign, serial, qty, task_name);
        let result = pdf_backend.send_raw("PDF 测试打印机", tspl.as_bytes());

        assert!(
            result.is_ok(),
            "任务名称渲染失败 '{}': {:?}",
            task_name,
            result.err()
        );

        println!("  ✓ 渲染成功");
    }

    println!("\n✅ 任务名称渲染测试通过");
    println!("=====================================\n");
}

#[test]
fn test_task_name_optional() {
    println!("\n========== 可选任务名称测试 ==========");

    let test_dir = ensure_test_dir();
    let pdf_backend = PdfBackend::new(test_dir.clone()).expect("创建 PDF 后端失败");

    // 测试无任务名称（应该只显示标题，无副标题）
    println!("测试无任务名称");
    let tspl_no_task = generate_qsl_tspl("BG7XXX", 1, 10);
    let result = pdf_backend.send_raw("PDF 测试打印机", tspl_no_task.as_bytes());
    assert!(result.is_ok(), "无任务名称渲染失败: {:?}", result.err());
    println!("  ✓ 无任务名称渲染成功");

    // 测试有任务名称
    println!("测试有任务名称");
    let tspl_with_task = generate_qsl_tspl_with_task("BD7ABC", 2, 5, "测试任务");
    let result = pdf_backend.send_raw("PDF 测试打印机", tspl_with_task.as_bytes());
    assert!(result.is_ok(), "有任务名称渲染失败: {:?}", result.err());
    println!("  ✓ 有任务名称渲染成功");

    println!("\n✅ 可选任务名称测试通过");
    println!("请检查: 有/无副标题的显示效果");
    println!("=====================================\n");
}

#[test]
fn test_full_rendering_pipeline() {
    println!("\n========== 完整渲染流程测试 ==========");

    let test_dir = ensure_test_dir();
    let pdf_backend = PdfBackend::new(test_dir.clone()).expect("创建 PDF 后端失败");

    // 测试完整的 QSL 卡片（包含所有元素）
    let full_tspl = generate_qsl_tspl_with_task("BG7XXX", 123, 50, "2026年春季活动");

    println!("测试完整渲染流程:");
    println!("  - 中文标题: 中国无线电协会业余分会-2区卡片局");
    println!("  - 中文副标题: 2026年春季活动");
    println!("  - 呼号: BG7XXX (居中，加粗，最大字号)");
    println!("  - 条形码: Code128 (居中)");
    println!("  - 序列号: SN:123 (居中)");
    println!("  - 数量: QTY:50 (居中，靠下)");
    println!("  - 边框: 矩形边框");

    let result = pdf_backend.send_raw("PDF 测试打印机", full_tspl.as_bytes());
    assert!(result.is_ok(), "完整渲染失败: {:?}", result.err());

    println!("\n✅ 完整渲染流程测试通过");
    println!("📁 输出: {}", test_dir.display());

    // 列出生成的文件
    if let Ok(entries) = std::fs::read_dir(&test_dir) {
        println!("\n生成的文件:");
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "png" || ext == "pdf" {
                        let metadata = std::fs::metadata(&path).ok();
                        let size = metadata.map(|m| m.len()).unwrap_or(0);
                        println!(
                            "  - {} ({} KB)",
                            path.file_name().unwrap().to_string_lossy(),
                            size / 1024
                        );
                    }
                }
            }
        }
    }

    println!("=====================================\n");
}
