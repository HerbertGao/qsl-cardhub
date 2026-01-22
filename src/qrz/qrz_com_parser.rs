// QRZ.com HTML 解析器

use anyhow::Result;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

/// QRZ.com 地址信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrzComAddressInfo {
    /// 呼号
    pub callsign: String,
    /// 姓名或组织名称
    pub name: Option<String>,
    /// 地址（多行，用 \n 分隔）
    pub address: Option<String>,
    /// 最后更新时间（格式：YYYY-MM-DD）
    pub updated_at: Option<String>,
    /// 数据来源（固定为 "qrz.com"）
    pub source: String,
}

impl QrzComAddressInfo {
    /// 创建新的地址信息
    pub fn new(callsign: String) -> Self {
        Self {
            callsign,
            name: None,
            address: None,
            updated_at: None,
            source: "qrz.com".to_string(),
        }
    }
}

/// 解析 QRZ.com 呼号查询页面
///
/// # 参数
/// * `html` - HTML 页面内容
/// * `callsign` - 呼号
///
/// # 返回
/// 解析后的地址信息，如果未找到地址则返回 None
pub fn parse_qrz_com_page(html: &str, callsign: &str) -> Result<Option<QrzComAddressInfo>> {
    log::debug!("开始解析 QRZ.com 呼号 {} 的页面内容", callsign);

    let document = Html::parse_document(html);

    // 提取呼号（从 <span class="csignm hamcall"> 标签）
    let parsed_callsign = extract_qrz_com_callsign(&document)
        .unwrap_or_else(|| callsign.to_string());
    log::debug!("提取的呼号: {}", parsed_callsign);

    // 提取姓名和地址（从 <p class="m0" style="..."> 标签）
    let (name, address) = extract_qrz_com_name_and_address(&document);
    log::debug!("姓名: {:?}", name);
    log::debug!("地址: {:?}", address);

    // 提取最后更新时间（从 Detail 标签页）
    let updated_at = extract_qrz_com_updated_date(&document);
    log::debug!("更新时间: {:?}", updated_at);

    // 如果姓名和地址都为空，说明未找到数据
    if name.is_none() && address.is_none() {
        log::info!("呼号 {} 未找到数据", callsign);
        return Ok(None);
    }

    // 创建地址信息对象
    let mut info = QrzComAddressInfo::new(parsed_callsign);
    info.name = name;
    info.address = address;
    info.updated_at = updated_at;

    log::info!("✓ 成功解析 QRZ.com 呼号 {} 的信息", callsign);
    Ok(Some(info))
}

/// 提取呼号（从 <span class="csignm hamcall"> 标签）
/// 格式：<span class="csignm hamcall">BY1CRA</span>
fn extract_qrz_com_callsign(document: &Html) -> Option<String> {
    let selector = Selector::parse("span.csignm.hamcall").ok()?;
    let callsign = document
        .select(&selector)
        .next()?
        .text()
        .collect::<String>()
        .trim()
        .to_string();

    if !callsign.is_empty() {
        Some(callsign)
    } else {
        None
    }
}

/// 提取姓名和地址
/// 格式：
/// <p class="m0" style="color: #666; font-weight: normal; font-size: 17px">
///   <span style="color: black; font-weight: bold">Chinese Amateur Radio Club Station</span>
///   <span class="csgnl none">, BY1CRA</span>
///   <br />Box 100029-73
///   <br/>Beijing 100029
///   <br/>China
/// </p>
fn extract_qrz_com_name_and_address(document: &Html) -> (Option<String>, Option<String>) {
    // 查找 <p class="m0" style="color: #666; font-weight: normal; font-size: 17px">
    let selector = match Selector::parse(r#"p.m0[style*="color: #666"]"#).ok() {
        Some(s) => s,
        None => return (None, None),
    };
    let p_element = match document.select(&selector).next() {
        Some(e) => e,
        None => return (None, None),
    };

    // 提取第一个 <span style="color: black; font-weight: bold"> 作为姓名
    let name_selector = match Selector::parse(r#"span[style*="color: black"][style*="font-weight: bold"]"#).ok() {
        Some(s) => s,
        None => return (None, None),
    };
    let name = p_element
        .select(&name_selector)
        .next()
        .map(|elem| elem.text().collect::<String>().trim().to_string());

    // 提取地址：获取 p 元素的 HTML，移除姓名部分，提取 <br/> 分隔的行
    let html = p_element.inner_html();

    // 使用更精确的方式提取地址行
    let mut address_lines = Vec::new();

    // 按 <br> 或 <br/> 或 <br /> 分割
    let lines: Vec<&str> = html
        .split("<br")
        .skip(1) // 跳过第一部分（包含姓名）
        .collect();

    for line in lines {
        // 移除 /> 或 > 之后的内容直到下一个标签
        if let Some(start) = line.find('>') {
            let content = &line[start + 1..];
            // 移除 HTML 标签
            let clean_content = strip_html_tags(content).trim().to_string();
            if !clean_content.is_empty() {
                address_lines.push(clean_content);
            }
        }
    }

    let address = if address_lines.is_empty() {
        None
    } else {
        Some(address_lines.join("\n"))
    };

    (name, address)
}

/// 提取最后更新时间
/// 格式：<tr><td class="dh">Last Update</td><td class="di">2020-04-26 12:33:04</td></tr>
fn extract_qrz_com_updated_date(document: &Html) -> Option<String> {
    let tr_selector = Selector::parse("tr").ok()?;
    let td_selector = Selector::parse("td").ok()?;

    for tr in document.select(&tr_selector) {
        let tds: Vec<_> = tr.select(&td_selector).collect();

        if tds.len() >= 2 {
            let first_td_text = tds[0].text().collect::<String>().trim().to_string();

            if first_td_text == "Last Update" {
                let date_text = tds[1].text().collect::<String>().trim().to_string();
                // 提取日期部分（YYYY-MM-DD）
                if let Some(date) = date_text.split_whitespace().next() {
                    return Some(date.to_string());
                }
            }
        }
    }

    None
}

/// 移除 HTML 标签，保留纯文本和换行符
fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ => {
                if !in_tag {
                    result.push(c);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_qrz_com_format() {
        let html = r##"
            <span class="csignm hamcall">BY1CRA</span>
            <p class="m0" style="color: #666; font-weight: normal; font-size: 17px">
                <span style="color: black; font-weight: bold">Chinese Amateur Radio Club Station</span>
                <span class="csgnl none">, BY1CRA</span>
                <br />Box 100029-73
                <br/>Beijing 100029
                <br/>China
            </p>
            <tr><td class="dh">Last Update</td><td class="di">2020-04-26 12:33:04</td></tr>
        "##;

        let result = parse_qrz_com_page(html, "BY1CRA").unwrap();
        assert!(result.is_some());

        let info = result.unwrap();
        assert_eq!(info.callsign, "BY1CRA");
        assert_eq!(info.name, Some("Chinese Amateur Radio Club Station".to_string()));
        assert_eq!(info.address, Some("Box 100029-73\nBeijing 100029\nChina".to_string()));
        assert_eq!(info.updated_at, Some("2020-04-26".to_string()));
        assert_eq!(info.source, "qrz.com");
    }

    #[test]
    fn test_parse_qrz_com_no_data() {
        let html = r##"
            <html>
            <body>
                <p>Not Found</p>
            </body>
            </html>
        "##;

        let result = parse_qrz_com_page(html, "TEST").unwrap();
        assert!(result.is_none());
    }
}
