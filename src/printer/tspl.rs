// TSPL 指令生成器
//
// 负责生成 TSPL (TSC Printer Language) 打印指令

use crate::config::TemplateConfig;

/// TSPL 指令生成器
pub struct TSPLGenerator {
    /// 打印机 DPI（默认 203）
    #[allow(dead_code)]
    dpi: u32,
}

impl TSPLGenerator {
    /// 创建新的 TSPL 生成器
    pub fn new() -> Self {
        Self { dpi: 203 }
    }

    /// 创建指定 DPI 的 TSPL 生成器
    #[allow(dead_code)]
    pub fn with_dpi(dpi: u32) -> Self {
        Self { dpi }
    }

    /// 生成 QSL 卡片的 TSPL 指令
    ///
    /// # 参数
    /// - `callsign`: 呼号（如 "BG7XXX"）
    /// - `serial`: 序列号（如 1, 2, 3...）
    /// - `qty`: 打印数量
    /// - `task_name`: 任务名称（副标题，可选）
    ///
    /// # 返回
    /// TSPL 指令字符串
    pub fn generate_qsl_card(&self, callsign: &str, serial: u32, qty: u32, task_name: Option<&str>) -> String {
        let mut tspl = String::new();

        // 添加任务名称元数据（使用注释，PDF后端会解析）
        if let Some(name) = task_name {
            tspl.push_str(&format!("; TASK_NAME: {}\n", name));
        }

        // v0.1: 硬编码布局参数
        // 纸张尺寸：76mm x 130mm
        tspl.push_str("SIZE 76 mm, 130 mm\n");
        tspl.push_str("GAP 2 mm, 0 mm\n");
        tspl.push_str("DIRECTION 0\n");
        tspl.push_str("CLS\n");

        // 呼号（大字号，居中）
        // TEXT X, Y, "font", rotation, x_scale, y_scale, "content"
        tspl.push_str(&format!(
            "TEXT 304,80,\"5\",0,3,3,\"{}\"\n",
            callsign
        ));

        // 条形码（CODE128，居中）
        // BARCODE X, Y, "type", height, human_readable, rotation, narrow, wide, "content"
        tspl.push_str(&format!(
            "BARCODE 200,300,\"128\",120,1,0,3,3,\"{}\"\n",
            callsign
        ));

        // 序列号
        tspl.push_str(&format!(
            "TEXT 50,520,\"5\",0,2,2,\"SN: {:03}\"\n",
            serial
        ));

        // 数量
        tspl.push_str(&format!(
            "TEXT 50,720,\"5\",0,2,2,\"QTY: {}\"\n",
            qty
        ));

        // 打印命令
        tspl.push_str("PRINT 1\n");

        tspl
    }

    /// 使用模板配置生成 QSL 卡片的 TSPL 指令
    ///
    /// # 参数
    /// - `template`: 模板配置
    /// - `callsign`: 呼号
    /// - `serial`: 序列号
    /// - `qty`: 打印数量
    /// - `task_name`: 任务名称（可选）
    ///
    /// # 返回
    /// TSPL 指令字符串
    pub fn generate_from_template(
        &self,
        template: &TemplateConfig,
        callsign: &str,
        serial: u32,
        qty: u32,
        task_name: Option<&str>,
    ) -> String {
        let mut tspl = String::new();

        // 纸张配置
        tspl.push_str(&format!(
            "SIZE {} mm, {} mm\n",
            template.paper.width_mm, template.paper.height_mm
        ));
        tspl.push_str(&format!("GAP {} mm, 0 mm\n", template.paper.gap_mm));
        tspl.push_str(&format!("DIRECTION {}\n", template.paper.direction));
        tspl.push_str("CLS\n");

        // 标题
        let title_cfg = &template.title;
        tspl.push_str(&format!(
            "TEXT {},{},\"{}\",{},{},{},\"{}\"\n",
            title_cfg.x,
            title_cfg.y,
            title_cfg.font,
            title_cfg.rotation,
            title_cfg.x_scale,
            title_cfg.y_scale,
            title_cfg.text
        ));

        // 副标题（如果有 task_name）
        if let Some(name) = task_name {
            let subtitle_cfg = &template.subtitle;
            tspl.push_str(&format!(
                "TEXT {},{},\"{}\",{},{},{},\"{}\"\n",
                subtitle_cfg.x,
                subtitle_cfg.y,
                subtitle_cfg.font,
                subtitle_cfg.rotation,
                subtitle_cfg.x_scale,
                subtitle_cfg.y_scale,
                name
            ));
        }

        // 呼号
        let callsign_cfg = &template.callsign;
        tspl.push_str(&format!(
            "TEXT {},{},\"{}\",{},{},{},\"{}\"\n",
            callsign_cfg.x,
            callsign_cfg.y,
            callsign_cfg.font,
            callsign_cfg.rotation,
            callsign_cfg.x_scale,
            callsign_cfg.y_scale,
            callsign
        ));

        // 条形码
        let barcode_cfg = &template.barcode;
        tspl.push_str(&format!(
            "BARCODE {},{},\"{}\",{},{},{},{},{},\"{}\"\n",
            barcode_cfg.x,
            barcode_cfg.y,
            barcode_cfg.barcode_type,
            barcode_cfg.height,
            barcode_cfg.human_readable,
            barcode_cfg.rotation,
            barcode_cfg.narrow_bar,
            barcode_cfg.wide_bar,
            callsign
        ));

        // 序列号
        let serial_cfg = &template.serial;
        let serial_text = format!(
            "{}{}",
            serial_cfg.prefix,
            format!("{}", serial)
                .parse::<i32>()
                .map(|n| {
                    // 使用模板中的格式化字符串
                    if serial_cfg.format.contains("{:0") {
                        // 提取补零位数
                        let width = serial_cfg
                            .format
                            .chars()
                            .filter(|c| c.is_digit(10))
                            .collect::<String>()
                            .parse::<usize>()
                            .unwrap_or(3);
                        format!("{:0width$}", n, width = width)
                    } else {
                        format!("{}", n)
                    }
                })
                .unwrap_or_else(|_| format!("{}", serial))
        );

        tspl.push_str(&format!(
            "TEXT {},{},\"{}\",{},{},{},\"{}\"\n",
            serial_cfg.x,
            serial_cfg.y,
            serial_cfg.font,
            serial_cfg.rotation,
            serial_cfg.x_scale,
            serial_cfg.y_scale,
            serial_text
        ));

        // 数量
        let qty_cfg = &template.quantity;
        tspl.push_str(&format!(
            "TEXT {},{},\"{}\",{},{},{},\"{}{}\"\n",
            qty_cfg.x,
            qty_cfg.y,
            qty_cfg.font,
            qty_cfg.rotation,
            qty_cfg.x_scale,
            qty_cfg.y_scale,
            qty_cfg.prefix,
            qty
        ));

        // 打印命令
        tspl.push_str("PRINT 1\n");

        tspl
    }

    /// 生成校准页的 TSPL 指令
    ///
    /// 校准页包含：
    /// - 边框
    /// - 中心十字线
    /// - 四角标记
    /// - 尺寸信息
    pub fn generate_calibration_page(&self) -> String {
        let mut tspl = String::new();

        // 纸张设置
        tspl.push_str("SIZE 76 mm, 130 mm\n");
        tspl.push_str("GAP 2 mm, 0 mm\n");
        tspl.push_str("DIRECTION 0\n");
        tspl.push_str("CLS\n");

        // 外边框（使用 BOX 指令）
        // BOX X_start, Y_start, X_end, Y_end, line_thickness
        tspl.push_str("BOX 10,10,590,1010,3\n");

        // 中心十字线（水平线）
        tspl.push_str("BAR 10,505,580,2\n");
        // 中心十字线（垂直线）
        tspl.push_str("BAR 299,10,2,1000\n");

        // 四角标记
        // 左上角
        tspl.push_str("BOX 20,20,60,60,2\n");
        // 右上角
        tspl.push_str("BOX 540,20,580,60,2\n");
        // 左下角
        tspl.push_str("BOX 20,960,60,1000,2\n");
        // 右下角
        tspl.push_str("BOX 540,960,580,1000,2\n");

        // 标题
        tspl.push_str("TEXT 200,100,\"5\",0,2,2,\"CALIBRATION\"\n");
        tspl.push_str("TEXT 250,150,\"5\",0,2,2,\"PAGE\"\n");

        // 尺寸信息
        tspl.push_str("TEXT 180,400,\"3\",0,1,1,\"76mm x 130mm\"\n");
        tspl.push_str("TEXT 180,450,\"3\",0,1,1,\"203 DPI\"\n");

        // 打印命令
        tspl.push_str("PRINT 1\n");

        tspl
    }

    // v0.5: 基于模板生成
    // pub fn generate_from_template(
    //     &self,
    //     template: &TemplateConfig,
    //     callsign: &str,
    //     serial: u32,
    //     qty: u32,
    // ) -> String {
    //     // 根据模板配置生成 TSPL 指令
    // }
}

impl Default for TSPLGenerator {
    fn default() -> Self {
        Self::new()
    }
}
