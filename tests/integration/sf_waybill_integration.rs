// 顺丰面单打印链路集成测试
//
// 目标：
// 1. 覆盖顺丰两步打印流程的离线路径（获取阶段的 Base64 交接 -> 打印阶段解码渲染）
// 2. 验证全局 TSPL 参数（GAP / DIRECTION）能够生效到最终 TSPL 头部

use base64::{Engine as _, engine::general_purpose::STANDARD};
use image::DynamicImage;
use qsl_cardhub::printer::tspl::TSPLGenerator;
use qsl_cardhub::sf_express::pdf_renderer::WaybillSize;
use qsl_cardhub::sf_express::PdfRenderer;
use std::path::Path;

fn build_minimal_test_pdf() -> Vec<u8> {
    let stream_content = b"BT /F1 18 Tf 36 300 Td (SF WAYBILL TEST) Tj ET";

    let mut objects = vec![
        "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".to_string(),
        "2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n".to_string(),
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 216 360] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n".to_string(),
        format!(
            "4 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
            stream_content.len(),
            String::from_utf8_lossy(stream_content)
        ),
        "5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n".to_string(),
    ];

    let mut pdf = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.4\n");

    let mut offsets = Vec::new();
    for object in &objects {
        offsets.push(pdf.len());
        pdf.extend_from_slice(object.as_bytes());
    }

    let xref_offset = pdf.len();
    let object_count = objects.len() + 1;
    pdf.extend_from_slice(format!("xref\n0 {}\n", object_count).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets {
        pdf.extend_from_slice(format!("{:010} 00000 n \n", offset).as_bytes());
    }

    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            object_count, xref_offset
        )
        .as_bytes(),
    );

    pdf
}

#[test]
fn test_sf_waybill_two_step_handoff_and_render() {
    let pdf_bytes = build_minimal_test_pdf();

    // 模拟 Step 1：后端返回 Base64 的 PDF 数据，前端持有并用于 Step 2 打印
    let encoded_pdf = STANDARD.encode(&pdf_bytes);

    // 模拟 Step 2：按 sf_print_waybill 逻辑解码后进行渲染
    let decoded_pdf = STANDARD.decode(&encoded_pdf).expect("PDF Base64 解码失败");
    assert_eq!(decoded_pdf, pdf_bytes, "两步流程中 PDF 数据不应发生变化");

    let size = WaybillSize {
        width_mm: 76.0,
        height_mm: 130.0,
        dpi: 203,
    };
    let expected_width = size.width_pixels();
    let expected_height = size.height_pixels();
    let renderer = PdfRenderer::with_size(size);
    let gray = renderer
        .render_pdf_to_grayscale(&decoded_pdf)
        .expect("顺丰面单灰度渲染失败");

    assert_eq!(gray.width(), expected_width);
    assert_eq!(gray.height(), expected_height);
}

#[test]
fn test_sf_waybill_two_step_applies_global_tspl_options() {
    let pdf_bytes = build_minimal_test_pdf();
    let encoded_pdf = STANDARD.encode(&pdf_bytes);
    let decoded_pdf = STANDARD.decode(&encoded_pdf).expect("PDF Base64 解码失败");

    let renderer = PdfRenderer::with_size(WaybillSize {
        width_mm: 76.0,
        height_mm: 130.0,
        dpi: 203,
    });
    let gray = renderer
        .render_pdf_to_grayscale(&decoded_pdf)
        .expect("顺丰面单灰度渲染失败");

    let generator = TSPLGenerator::new();
    let tspl = generator
        .generate_from_image_with_options(&gray, 76.0, 130.0, 2.0, 0.0, "1,0")
        .expect("生成 TSPL 失败");
    let tspl_text = String::from_utf8_lossy(&tspl);

    assert!(tspl_text.contains("GAP 2 mm, 0 mm"));
    assert!(tspl_text.contains("DIRECTION 1,0"));

    let tspl_custom = generator
        .generate_from_image_with_options(&gray, 76.0, 130.0, 3.0, 1.0, "0,0")
        .expect("生成自定义 TSPL 失败");
    let tspl_custom_text = String::from_utf8_lossy(&tspl_custom);

    assert!(tspl_custom_text.contains("GAP 3 mm, 1 mm"));
    assert!(tspl_custom_text.contains("DIRECTION 0,0"));
}

#[test]
fn test_local_png_waybill_tspl_options() {
    let local_png = Path::new("/Users/herbertgao/Downloads/print_20260125_221736.png");
    if !local_png.exists() {
        eprintln!(
            "跳过本机 PNG 测试：未找到文件 {}",
            local_png.display()
        );
        return;
    }

    let image = image::open(local_png).expect("加载本机 PNG 失败");
    let gray = match image {
        DynamicImage::ImageLuma8(img) => img,
        _ => image.to_luma8(),
    };

    let generator = TSPLGenerator::new();
    let tspl = generator
        .generate_from_image_with_options(&gray, 76.0, 130.0, 2.0, 0.0, "1,0")
        .expect("从本机 PNG 生成 TSPL 失败");
    let tspl_text = String::from_utf8_lossy(&tspl);

    assert!(tspl_text.contains("GAP 2 mm, 0 mm"));
    assert!(tspl_text.contains("DIRECTION 1,0"));
}
