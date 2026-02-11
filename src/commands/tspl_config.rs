use crate::config::models::TsplPrintConfig;

/// 校验并归一化全局 TSPL 参数，返回（生效配置，回退告警）
pub fn normalize_tspl_print_config(raw: &TsplPrintConfig) -> (TsplPrintConfig, Vec<String>) {
    let defaults = TsplPrintConfig::default();
    let mut warnings: Vec<String> = Vec::new();

    let gap_mm = if (0.0..=10.0).contains(&raw.gap_mm) {
        raw.gap_mm
    } else {
        warnings.push(format!(
            "gap_mm={} 超出范围 [0,10]，回退为 {}",
            raw.gap_mm, defaults.gap_mm
        ));
        defaults.gap_mm
    };

    let gap_offset_mm = if (0.0..=10.0).contains(&raw.gap_offset_mm) {
        raw.gap_offset_mm
    } else {
        warnings.push(format!(
            "gap_offset_mm={} 超出范围 [0,10]，回退为 {}",
            raw.gap_offset_mm, defaults.gap_offset_mm
        ));
        defaults.gap_offset_mm
    };

    let direction = {
        let trimmed = raw.direction.trim();
        let valid = matches!(
            trimmed,
            "0" | "1" | "2" | "3" | "0,0" | "0,1" | "1,0" | "1,1" | "2,0" | "2,1" | "3,0"
                | "3,1"
        );
        if valid {
            trimmed.to_string()
        } else {
            warnings.push(format!(
                "direction=\"{}\" 非法，回退为 {}",
                raw.direction, defaults.direction
            ));
            defaults.direction
        }
    };

    (
        TsplPrintConfig {
            gap_mm,
            gap_offset_mm,
            direction,
        },
        warnings,
    )
}
