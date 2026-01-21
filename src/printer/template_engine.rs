// 模板引擎
//
// 负责解析v2模板配置，填充运行时数据，生成已解析的元素列表

use crate::config::template::{ElementConfig, TemplateConfig};
use anyhow::{Context, Result};
use std::collections::HashMap;

/// 已解析的元素
///
/// 包含填充后的内容和所有必要的渲染参数
#[derive(Debug, Clone)]
pub struct ResolvedElement {
    /// 元素ID
    pub id: String,
    /// 元素类型("text" | "barcode")
    pub element_type: String,
    /// 已解析的内容
    pub content: String,

    // 文本元素特有
    /// 文本元素最大高度预算(mm)
    pub max_height_mm: Option<f32>,

    // 条形码元素特有
    /// 条形码类型
    pub barcode_type: Option<String>,
    /// 条形码高度(mm)
    pub height_mm: Option<f32>,
    /// 条形码左右留白(mm)
    pub quiet_zone_mm: Option<f32>,
    /// 是否显示人类可读文本
    pub human_readable: Option<bool>,
}

/// 模板引擎
///
/// 负责解析模板配置和填充运行时数据
pub struct TemplateEngine;

impl TemplateEngine {
    /// 解析模板配置，填充运行时数据
    ///
    /// # 参数
    /// - `config`: v2版本模板配置
    /// - `data`: 运行时数据(key-value映射)
    ///
    /// # 返回
    /// - 已解析的元素列表
    ///
    /// # 错误
    /// - 如果input元素的key在data中不存在
    /// - 如果computed元素的format中的占位符在data中不存在
    pub fn resolve(
        config: &TemplateConfig,
        data: &HashMap<String, String>,
    ) -> Result<Vec<ResolvedElement>> {
        log::info!(
            "开始解析模板: {}, 共 {} 个元素",
            config.metadata.name,
            config.elements.len()
        );
        log::debug!("运行时数据键: {:?}", data.keys().collect::<Vec<_>>());

        let mut resolved_elements = Vec::new();

        for (index, element) in config.elements.iter().enumerate() {
            log::debug!(
                "[{}/{}] 开始解析元素: id={}, type={}, source={}",
                index + 1,
                config.elements.len(),
                element.id,
                element.element_type,
                element.source
            );

            let content = Self::resolve_content(element, data).with_context(|| {
                format!(
                    "解析元素 {} (type={}, source={}) 失败",
                    element.id, element.element_type, element.source
                )
            })?;

            log::debug!("  -> 解析结果: {:?}", content);

            resolved_elements.push(ResolvedElement {
                id: element.id.clone(),
                element_type: element.element_type.clone(),
                content,
                max_height_mm: element.max_height_mm,
                barcode_type: element.barcode_type.clone(),
                height_mm: element.height_mm,
                quiet_zone_mm: element.quiet_zone_mm,
                human_readable: element.human_readable,
            });
        }

        log::info!("✅ 模板解析完成，共解析 {} 个元素", resolved_elements.len());
        Ok(resolved_elements)
    }

    /// 解析元素内容
    ///
    /// 根据元素的source类型进行不同的处理：
    /// - fixed: 使用value字段
    /// - input: 从data中使用key获取
    /// - computed: 使用format进行占位符替换
    fn resolve_content(element: &ElementConfig, data: &HashMap<String, String>) -> Result<String> {
        match element.source.as_str() {
            "fixed" => Self::resolve_fixed(element),
            "input" => Self::resolve_input(element, data),
            "computed" => Self::resolve_computed(element, data),
            _ => anyhow::bail!("不支持的元素来源类型: {}", element.source),
        }
    }

    /// 处理固定值元素(source = "fixed")
    fn resolve_fixed(element: &ElementConfig) -> Result<String> {
        let value = element
            .value
            .clone()
            .ok_or_else(|| anyhow::anyhow!("fixed元素缺少value字段"))?;

        log::debug!("  fixed: 使用固定值");
        Ok(value)
    }

    /// 处理输入元素(source = "input")
    fn resolve_input(element: &ElementConfig, data: &HashMap<String, String>) -> Result<String> {
        let key = element
            .key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("input元素缺少key字段"))?;

        log::debug!("  input: 从运行时数据获取 key={}", key);

        data.get(key).cloned().ok_or_else(|| {
            log::warn!("运行时数据中缺少键: {}", key);
            anyhow::anyhow!("运行时数据中缺少键: {}", key)
        })
    }

    /// 处理计算元素(source = "computed")
    ///
    /// 使用简单模板引擎替换format字符串中的{field}占位符
    fn resolve_computed(element: &ElementConfig, data: &HashMap<String, String>) -> Result<String> {
        let format_str = element
            .format
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("computed元素缺少format字段"))?;

        log::debug!("  computed: 替换占位符 format={:?}", format_str);

        Self::replace_placeholders(format_str, data)
    }

    /// 简单模板引擎：替换{field}占位符
    ///
    /// # 参数
    /// - `format`: 格式化字符串，如 "SN: {sn}"
    /// - `data`: key-value数据映射
    ///
    /// # 返回
    /// 替换后的字符串
    ///
    /// # 错误
    /// 如果占位符对应的key在data中不存在
    fn replace_placeholders(format: &str, data: &HashMap<String, String>) -> Result<String> {
        let mut result = format.to_string();
        let mut missing_keys = Vec::new();

        // 查找所有{xxx}占位符
        let re = regex::Regex::new(r"\{(\w+)\}").unwrap();
        let placeholders: Vec<_> = re
            .captures_iter(format)
            .map(|cap| cap[1].to_string())
            .collect();

        if !placeholders.is_empty() {
            log::debug!("    发现占位符: {:?}", placeholders);
        }

        for cap in re.captures_iter(format) {
            let placeholder = &cap[0]; // 完整占位符，如 "{sn}"
            let key = &cap[1]; // 键名，如 "sn"

            if let Some(value) = data.get(key) {
                log::debug!("    替换 {} -> {}", placeholder, value);
                result = result.replace(placeholder, value);
            } else {
                missing_keys.push(key.to_string());
            }
        }

        if !missing_keys.is_empty() {
            log::error!(
                "format字符串中的占位符在运行时数据中不存在: {:?}",
                missing_keys
            );
            anyhow::bail!(
                "format字符串中的占位符在运行时数据中不存在: {:?}",
                missing_keys
            );
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::template::TemplateConfig;

    #[test]
    fn test_resolve_fixed_element() {
        let config = TemplateConfig::default_qsl_card();

        // 需要提供所有input和computed元素所需的数据
        let mut data = HashMap::new();
        data.insert("task_name".to_string(), "测试".to_string());
        data.insert("callsign".to_string(), "BG7XXX".to_string());
        data.insert("sn".to_string(), "001".to_string());
        data.insert("qty".to_string(), "100".to_string());

        let resolved = TemplateEngine::resolve(&config, &data).expect("应该成功解析");

        // 第一个元素是固定值标题
        let title = &resolved[0];
        assert_eq!(title.id, "title");
        assert_eq!(title.element_type, "text");
        assert_eq!(title.content, "中国无线电协会业余分会-2区卡片局");
    }

    #[test]
    fn test_resolve_input_element() {
        let config = TemplateConfig::default_qsl_card();
        let mut data = HashMap::new();
        data.insert("task_name".to_string(), "测试任务".to_string());
        data.insert("callsign".to_string(), "BG7XXX".to_string());
        data.insert("sn".to_string(), "001".to_string());
        data.insert("qty".to_string(), "100".to_string());

        let resolved = TemplateEngine::resolve(&config, &data).expect("应该成功解析");

        // 第二个元素是input类型的subtitle
        let subtitle = &resolved[1];
        assert_eq!(subtitle.id, "subtitle");
        assert_eq!(subtitle.content, "测试任务");

        // 第三个元素是input类型的callsign
        let callsign = &resolved[2];
        assert_eq!(callsign.id, "callsign");
        assert_eq!(callsign.content, "BG7XXX");
    }

    #[test]
    fn test_resolve_computed_element() {
        let config = TemplateConfig::default_qsl_card();
        let mut data = HashMap::new();
        data.insert("task_name".to_string(), "测试".to_string());
        data.insert("callsign".to_string(), "BG7XXX".to_string());
        data.insert("sn".to_string(), "001".to_string());
        data.insert("qty".to_string(), "100".to_string());

        let resolved = TemplateEngine::resolve(&config, &data).expect("应该成功解析");

        // 第四个元素是条形码(computed)
        let barcode = &resolved[3];
        assert_eq!(barcode.id, "barcode");
        assert_eq!(barcode.element_type, "barcode");
        assert_eq!(barcode.content, "BG7XXX"); // format = "{callsign}"

        // 第五个元素是SN(computed)
        let sn = &resolved[4];
        assert_eq!(sn.id, "sn");
        assert_eq!(sn.element_type, "text");
        assert_eq!(sn.content, "SN: 001"); // format = "SN: {sn}"

        // 第六个元素是QTY(computed)
        let qty = &resolved[5];
        assert_eq!(qty.id, "qty");
        assert_eq!(qty.element_type, "text");
        assert_eq!(qty.content, "QTY: 100"); // format = "QTY: {qty}"
    }

    #[test]
    fn test_resolve_input_element_missing_key() {
        let config = TemplateConfig::default_qsl_card();
        let data = HashMap::new(); // 空数据

        let result = TemplateEngine::resolve(&config, &data);
        assert!(result.is_err(), "缺少必需的input键应该返回错误");

        // 使用 {:#} 格式化来显示完整错误链
        let error_msg = format!("{:#}", result.unwrap_err());
        assert!(
            error_msg.contains("task_name") || error_msg.contains("缺少键"),
            "错误信息应包含缺失的键名，实际错误: {}",
            error_msg
        );
    }

    #[test]
    fn test_resolve_computed_element_missing_placeholder() {
        let mut data = HashMap::new();
        data.insert("callsign".to_string(), "BG7XXX".to_string());
        // 缺少 sn 和 qty

        let format_str = "SN: {sn}";
        let result = TemplateEngine::replace_placeholders(format_str, &data);

        assert!(result.is_err(), "缺少占位符对应的键应该返回错误");
        assert!(result.unwrap_err().to_string().contains("sn"));
    }

    #[test]
    fn test_replace_placeholders() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), "Alice".to_string());
        data.insert("age".to_string(), "30".to_string());

        let format = "Name: {name}, Age: {age}";
        let result = TemplateEngine::replace_placeholders(format, &data).unwrap();

        assert_eq!(result, "Name: Alice, Age: 30");
    }

    #[test]
    fn test_replace_placeholders_no_placeholders() {
        let data = HashMap::new();
        let format = "No placeholders here";
        let result = TemplateEngine::replace_placeholders(format, &data).unwrap();

        assert_eq!(result, "No placeholders here");
    }

    #[test]
    fn test_resolve_with_logging() {
        // 初始化日志系统（如果已经初始化会失败，忽略错误）
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();

        let config = TemplateConfig::default_qsl_card();
        let mut data = HashMap::new();
        data.insert("task_name".to_string(), "测试任务".to_string());
        data.insert("callsign".to_string(), "BG7XXX".to_string());
        data.insert("sn".to_string(), "001".to_string());
        data.insert("qty".to_string(), "100".to_string());

        let resolved = TemplateEngine::resolve(&config, &data).expect("应该成功解析");

        assert_eq!(resolved.len(), 6);
        println!("✅ 日志测试完成，解析了 {} 个元素", resolved.len());
    }
}
