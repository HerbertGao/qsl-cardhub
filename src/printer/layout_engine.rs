// 布局引擎
//
// 负责计算每个元素的精确位置、字号和整体居中布局

use crate::config::template::{LayoutConfig, PageConfig, TemplateConfig};
use crate::printer::template_engine::ResolvedElement;
use crate::printer::text_renderer::TextRenderer;
use anyhow::{Context, Result};

/// 单位转换：mm转换为dots
fn mm_to_dots(mm: f32, dpi: u32) -> u32 {
    (mm * dpi as f32 / 25.4).ceil() as u32
}

/// 元素类型
#[derive(Debug, Clone, PartialEq)]
pub enum ElementType {
    Text,
    Barcode,
}

/// 条形码配置
#[derive(Debug, Clone)]
pub struct BarcodeConfig {
    pub barcode_type: String,
    pub height_dots: u32,
    pub quiet_zone_dots: u32,
    pub human_readable: bool,
}

/// 边框配置
#[derive(Debug, Clone)]
pub struct BorderConfig {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub thickness: u32,
}

/// 已布局的元素
#[derive(Debug, Clone)]
pub struct LayoutedElement {
    pub id: String,
    pub element_type: ElementType,
    pub content: String,
    /// 绝对x坐标(dots)
    pub x: u32,
    /// 绝对y坐标(dots)
    pub y: u32,
    /// 文本字号(pt) - 仅文本元素
    pub font_size: Option<f32>,
    /// 元素高度(dots)
    pub height: u32,
    /// 元素宽度(dots)
    pub width: u32,
    /// 条形码配置 - 仅条形码元素
    pub barcode_config: Option<BarcodeConfig>,
}

/// 布局结果
#[derive(Debug, Clone)]
pub struct LayoutResult {
    /// 画布宽度(dots)
    pub canvas_width: u32,
    /// 画布高度(dots)
    pub canvas_height: u32,
    /// 边框配置(可选)
    pub border: Option<BorderConfig>,
    /// 已布局的元素列表
    pub elements: Vec<LayoutedElement>,
}

/// 布局引擎
pub struct LayoutEngine {
    text_renderer: TextRenderer,
}

impl LayoutEngine {
    /// 创建新的布局引擎
    pub fn new() -> Result<Self> {
        Ok(Self {
            text_renderer: TextRenderer::new()?,
        })
    }

    /// 执行完整布局计算
    ///
    /// # 参数
    /// - `config`: v2模板配置
    /// - `resolved_elements`: 已解析的元素列表
    ///
    /// # 返回
    /// 完整的布局结果
    pub fn layout(
        &mut self,
        config: &TemplateConfig,
        resolved_elements: Vec<ResolvedElement>,
    ) -> Result<LayoutResult> {
        log::info!("开始布局计算: {}", config.metadata.name);

        // 1. 计算画布和可用区域
        let (canvas_width, canvas_height) = self.calculate_canvas_size(&config.page);
        let (
            content_left,
            _content_right,
            content_top,
            _content_bottom,
            available_width,
            available_height,
        ) = self.calculate_available_area(&config.page);

        log::debug!(
            "画布尺寸: {}x{} dots, 内容区域: 左{}右{}, 宽{} dots",
            canvas_width,
            canvas_height,
            content_left,
            _content_right,
            available_width
        );

        // 2. 为每个元素计算尺寸和字号
        let mut layouted_elements = Vec::new();
        let line_gap_dots = mm_to_dots(config.layout.line_gap_mm, config.page.dpi);

        for element in resolved_elements {
            let layouted = self.layout_element(&element, &config, available_width)?;
            layouted_elements.push(layouted);
        }

        // 3. 全局防溢出校验
        self.apply_overflow_protection(&mut layouted_elements, available_height, line_gap_dots)?;

        // 4. 计算垂直对齐偏移
        let total_content_height =
            self.calculate_total_content_height(&layouted_elements, line_gap_dots);
        let y_offset = if total_content_height < available_height {
            match config.layout.align_v.as_str() {
                "top" => 0,
                "bottom" => available_height - total_content_height,
                "center" | _ => (available_height - total_content_height) / 2,
            }
        } else {
            0
        };

        log::debug!(
            "内容总高度: {} dots, 垂直对齐: {}, 偏移: {} dots",
            total_content_height,
            config.layout.align_v,
            y_offset
        );

        // 5. 分配y坐标
        let gap_dots = mm_to_dots(config.layout.gap_mm, config.page.dpi);
        self.assign_y_positions(&mut layouted_elements, content_top + y_offset, line_gap_dots, gap_dots);

        // 6. 计算水平对齐x坐标
        for element in &mut layouted_elements {
            element.x = self.calculate_horizontal_position(
                element.width,
                available_width,
                content_left,
                &config.layout.align_h,
            );
            log::debug!(
                "元素 {} 水平对齐: x={}, width={}, 右边界={}",
                element.id,
                element.x,
                element.width,
                element.x + element.width
            );
        }

        // 7. 处理边框
        let border = if config.page.border {
            // 边框应该绘制在 margin 边界，而不是内容区域边界
            let border_x = mm_to_dots(config.page.margin_left_mm, config.page.dpi);
            let border_y = mm_to_dots(config.page.margin_top_mm, config.page.dpi);
            let border_width = canvas_width
                - mm_to_dots(config.page.margin_left_mm, config.page.dpi)
                - mm_to_dots(config.page.margin_right_mm, config.page.dpi);
            let border_height = canvas_height
                - mm_to_dots(config.page.margin_top_mm, config.page.dpi)
                - mm_to_dots(config.page.margin_bottom_mm, config.page.dpi);
            let border_thickness = mm_to_dots(config.page.border_thickness_mm, config.page.dpi);

            log::debug!(
                "边框配置: x={}, y={}, width={}, height={}, thickness={}",
                border_x,
                border_y,
                border_width,
                border_height,
                border_thickness
            );
            log::debug!(
                "内容区域应在边框内: 左边界={} (border_x + thickness), 右边界={} (border_x + width - thickness)",
                border_x + border_thickness,
                border_x + border_width - border_thickness
            );

            Some(BorderConfig {
                x: border_x,
                y: border_y,
                width: border_width,
                height: border_height,
                thickness: border_thickness,
            })
        } else {
            None
        };

        log::info!("✅ 布局计算完成，共 {} 个元素", layouted_elements.len());

        Ok(LayoutResult {
            canvas_width,
            canvas_height,
            border,
            elements: layouted_elements,
        })
    }

    /// 计算画布尺寸(dots)
    fn calculate_canvas_size(&self, page_config: &PageConfig) -> (u32, u32) {
        let width = mm_to_dots(page_config.width_mm, page_config.dpi);
        let height = mm_to_dots(page_config.height_mm, page_config.dpi);
        (width, height)
    }

    /// 计算可用区域
    ///
    /// # 返回
    /// (left, right, top, bottom, available_width, available_height) in dots
    fn calculate_available_area(&self, page_config: &PageConfig) -> (u32, u32, u32, u32, u32, u32) {
        let mut left = mm_to_dots(page_config.margin_left_mm, page_config.dpi);
        let right_margin = mm_to_dots(page_config.margin_right_mm, page_config.dpi);
        let mut top = mm_to_dots(page_config.margin_top_mm, page_config.dpi);
        let bottom_margin = mm_to_dots(page_config.margin_bottom_mm, page_config.dpi);

        let (canvas_width, canvas_height) = self.calculate_canvas_size(page_config);

        let mut right = canvas_width - right_margin;
        let mut bottom = canvas_height - bottom_margin;

        // 如果启用边框，内容区域需要再向内缩进边框宽度，避免内容被边框遮挡
        if page_config.border {
            let border_thickness = mm_to_dots(page_config.border_thickness_mm, page_config.dpi);
            // 边框绘制在 margin 区域的内侧，内容需要在边框内侧
            left += border_thickness;
            right -= border_thickness;
            top += border_thickness;
            bottom -= border_thickness;

            log::debug!(
                "边框已启用，厚度: {}mm ({}dots)，内容区域向内缩进",
                page_config.border_thickness_mm,
                border_thickness
            );
        }

        let available_width = right - left;
        let available_height = bottom - top;

        (left, right, top, bottom, available_width, available_height)
    }

    /// 为单个元素进行布局
    fn layout_element(
        &mut self,
        element: &ResolvedElement,
        config: &TemplateConfig,
        available_width: u32,
    ) -> Result<LayoutedElement> {
        match element.element_type.as_str() {
            "text" => self.layout_text_element(element, config, available_width),
            "barcode" => self.layout_barcode_element(element, config, available_width),
            _ => anyhow::bail!("未知的元素类型: {}", element.element_type),
        }
    }

    /// 布局文本元素
    fn layout_text_element(
        &mut self,
        element: &ResolvedElement,
        config: &TemplateConfig,
        available_width: u32,
    ) -> Result<LayoutedElement> {
        let max_height_mm = element
            .max_height_mm
            .ok_or_else(|| anyhow::anyhow!("文本元素缺少max_height_mm字段"))?;

        let max_height_dots = mm_to_dots(max_height_mm, config.page.dpi);

        // 为文字留出安全边距（左右各留出约1mm的空间）
        // 这样可以避免文字太靠近边框
        let safe_margin = mm_to_dots(1.0, config.page.dpi);  // 约 8 dots
        let safe_available_width = if available_width > safe_margin {
            available_width - safe_margin
        } else {
            available_width
        };

        // 使用二分搜索求最大字号
        let font_size =
            self.calculate_max_font_size(&element.content, max_height_dots, safe_available_width)?;

        // 测量实际尺寸
        let (width, height) = self
            .text_renderer
            .measure_text(&element.content, font_size)?;

        log::debug!(
            "文本元素 {}: \"{}\" -> {}pt, {}x{} dots",
            element.id,
            element.content,
            font_size,
            width,
            height
        );

        Ok(LayoutedElement {
            id: element.id.clone(),
            element_type: ElementType::Text,
            content: element.content.clone(),
            x: 0, // 稍后计算
            y: 0, // 稍后计算
            font_size: Some(font_size),
            height,
            width,
            barcode_config: None,
        })
    }

    /// 布局条形码元素
    fn layout_barcode_element(
        &mut self,
        element: &ResolvedElement,
        config: &TemplateConfig,
        available_width: u32,
    ) -> Result<LayoutedElement> {
        let barcode_type = element
            .barcode_type
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("条形码元素缺少barcode_type字段"))?
            .clone();

        let height_mm = element
            .height_mm
            .ok_or_else(|| anyhow::anyhow!("条形码元素缺少height_mm字段"))?;

        let height_dots = mm_to_dots(height_mm, config.page.dpi);

        let quiet_zone_mm = element.quiet_zone_mm.unwrap_or(2.0);
        let quiet_zone_dots = mm_to_dots(quiet_zone_mm, config.page.dpi);

        let human_readable = element.human_readable.unwrap_or(false);

        // 估算条形码宽度（简化：每字符12 dots）
        let estimated_width = element.content.len() as u32 * 12 + quiet_zone_dots * 2;

        log::debug!(
            "条形码元素 {}: \"{}\" -> {}mm高, 估算宽度{}",
            element.id,
            element.content,
            height_mm,
            estimated_width
        );

        Ok(LayoutedElement {
            id: element.id.clone(),
            element_type: ElementType::Barcode,
            content: element.content.clone(),
            x: 0,
            y: 0,
            font_size: None,
            height: height_dots,
            width: estimated_width,
            barcode_config: Some(BarcodeConfig {
                barcode_type,
                height_dots,
                quiet_zone_dots,
                human_readable,
            }),
        })
    }

    /// 使用二分搜索计算最大字号
    fn calculate_max_font_size(
        &mut self,
        content: &str,
        max_height_dots: u32,
        available_width_dots: u32,
    ) -> Result<f32> {
        let mut left = 8.0; // 最小字号
        let mut right = 120.0; // 最大字号
        let mut result = left;

        while right - left > 0.5 {
            let mid = (left + right) / 2.0;

            // 测量文本尺寸
            let (width, height) = self.text_renderer.measure_text(content, mid)?;

            if width <= available_width_dots && height <= max_height_dots {
                result = mid;
                left = mid;
            } else {
                right = mid;
            }
        }

        Ok(result)
    }

    /// 计算内容块总高度
    fn calculate_total_content_height(
        &self,
        elements: &[LayoutedElement],
        line_gap_dots: u32,
    ) -> u32 {
        if elements.is_empty() {
            return 0;
        }

        let total_height: u32 = elements.iter().map(|e| e.height).sum();
        let total_gaps = if elements.len() > 1 {
            (elements.len() - 1) as u32 * line_gap_dots
        } else {
            0
        };

        total_height + total_gaps
    }

    /// 分配y坐标
    fn assign_y_positions(
        &self,
        elements: &mut [LayoutedElement],
        start_y: u32,
        line_gap_dots: u32,
        extra_gap_dots: u32,
    ) {
        let mut current_y = start_y;
        let mut prev_element_type: Option<ElementType> = None;

        for element in elements {
            // 如果前一个元素和当前元素类型不同，且涉及条形码，添加额外间距
            if let Some(prev_type) = prev_element_type {
                if (prev_type == ElementType::Text && element.element_type == ElementType::Barcode)
                    || (prev_type == ElementType::Barcode && element.element_type == ElementType::Text)
                {
                    current_y += extra_gap_dots;
                }
            }

            element.y = current_y;
            current_y += element.height + line_gap_dots;
            prev_element_type = Some(element.element_type.clone());
        }
    }

    /// 计算水平居中x坐标
    /// 计算水平对齐的x坐标
    fn calculate_horizontal_position(
        &self,
        element_width: u32,
        available_width: u32,
        left_margin: u32,
        align: &str,
    ) -> u32 {
        match align {
            "left" => left_margin,
            "right" => left_margin + available_width - element_width,
            "center" | _ => left_margin + (available_width - element_width) / 2,
        }
    }

    /// 计算水平居中（保留用于测试兼容性）
    fn calculate_horizontal_center(
        &self,
        element_width: u32,
        available_width: u32,
        left_margin: u32,
    ) -> u32 {
        self.calculate_horizontal_position(element_width, available_width, left_margin, "center")
    }

    /// 全局防溢出校验和缩放
    fn apply_overflow_protection(
        &mut self,
        elements: &mut Vec<LayoutedElement>,
        available_height: u32,
        line_gap_dots: u32,
    ) -> Result<()> {
        let total_content_height = self.calculate_total_content_height(elements, line_gap_dots);

        if total_content_height <= available_height {
            log::debug!("内容未溢出，无需缩放");
            return Ok(());
        }

        // 计算缩放比例
        let scale_factor = available_height as f32 / total_content_height as f32;
        log::warn!(
            "内容溢出 ({} > {} dots), 全局缩放比例: {:.3}",
            total_content_height,
            available_height,
            scale_factor
        );

        // 缩放所有元素
        for element in elements.iter_mut() {
            // 缩放字号
            if let Some(font_size) = element.font_size {
                let new_font_size = font_size * scale_factor;
                element.font_size = Some(new_font_size);

                // 重新测量尺寸
                let (new_width, new_height) = self
                    .text_renderer
                    .measure_text(&element.content, new_font_size)?;
                element.width = new_width;
                element.height = new_height;

                log::debug!(
                    "  缩放元素 {}: {}pt -> {}pt, {}x{} dots",
                    element.id,
                    font_size,
                    new_font_size,
                    new_width,
                    new_height
                );
            } else {
                // 条形码元素缩放高度
                element.height = (element.height as f32 * scale_factor) as u32;
            }
        }

        log::info!("✅ 全局缩放完成");
        Ok(())
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new().expect("创建LayoutEngine失败")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mm_to_dots() {
        assert_eq!(mm_to_dots(76.0, 203), 608);
        assert_eq!(mm_to_dots(130.0, 203), 1039); // 130 * 203 / 25.4 = 1038.976, ceil = 1039
        assert_eq!(mm_to_dots(2.0, 203), 16);
        assert_eq!(mm_to_dots(3.0, 203), 24);
    }

    #[test]
    fn test_calculate_canvas_size() {
        let engine = LayoutEngine::new().unwrap();
        let page_config = PageConfig {
            dpi: 203,
            width_mm: 76.0,
            height_mm: 130.0,
            margin_left_mm: 2.0,
            margin_right_mm: 2.0,
            margin_top_mm: 3.0,
            margin_bottom_mm: 3.0,
            border: true,
            border_thickness_mm: 0.3,
        };

        let (width, height) = engine.calculate_canvas_size(&page_config);
        assert_eq!(width, 608);
        assert_eq!(height, 1039); // 精确计算值
    }

    #[test]
    fn test_calculate_available_area() {
        let engine = LayoutEngine::new().unwrap();
        let page_config = PageConfig {
            dpi: 203,
            width_mm: 76.0,
            height_mm: 130.0,
            margin_left_mm: 2.0,
            margin_right_mm: 2.0,
            margin_top_mm: 3.0,
            margin_bottom_mm: 3.0,
            border: true,
            border_thickness_mm: 0.3,
        };

        let (left, _right, top, _bottom, available_width, available_height) =
            engine.calculate_available_area(&page_config);

        assert_eq!(left, 16);
        assert_eq!(top, 24);
        assert_eq!(available_width, 576);
        assert_eq!(available_height, 991); // 1039 - 24 - 24 = 991
    }

    #[test]
    fn test_calculate_total_content_height() {
        let engine = LayoutEngine::new().unwrap();
        let elements = vec![
            LayoutedElement {
                id: "e1".to_string(),
                element_type: ElementType::Text,
                content: "test".to_string(),
                x: 0,
                y: 0,
                font_size: Some(72.0),
                height: 80,
                width: 100,
                barcode_config: None,
            },
            LayoutedElement {
                id: "e2".to_string(),
                element_type: ElementType::Text,
                content: "test2".to_string(),
                x: 0,
                y: 0,
                font_size: Some(48.0),
                height: 120,
                width: 100,
                barcode_config: None,
            },
        ];

        let line_gap = 16;
        let total_height = engine.calculate_total_content_height(&elements, line_gap);

        // 80 + 120 + 16 = 216
        assert_eq!(total_height, 216);
    }

    #[test]
    fn test_calculate_horizontal_center() {
        let engine = LayoutEngine::new().unwrap();

        let x = engine.calculate_horizontal_center(400, 576, 16);
        // 16 + (576 - 400) / 2 = 16 + 88 = 104
        assert_eq!(x, 104);
    }

    #[test]
    fn test_assign_y_positions() {
        let engine = LayoutEngine::new().unwrap();
        let mut elements = vec![
            LayoutedElement {
                id: "e1".to_string(),
                element_type: ElementType::Text,
                content: "test".to_string(),
                x: 0,
                y: 0,
                font_size: Some(72.0),
                height: 80,
                width: 100,
                barcode_config: None,
            },
            LayoutedElement {
                id: "e2".to_string(),
                element_type: ElementType::Text,
                content: "test2".to_string(),
                x: 0,
                y: 0,
                font_size: Some(48.0),
                height: 120,
                width: 100,
                barcode_config: None,
            },
        ];

        engine.assign_y_positions(&mut elements, 100, 16, 0); // extra_gap_dots = 0

        assert_eq!(elements[0].y, 100);
        assert_eq!(elements[1].y, 196); // 100 + 80 + 16
    }
}
