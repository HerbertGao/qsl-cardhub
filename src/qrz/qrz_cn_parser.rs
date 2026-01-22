// QRZ.cn HTML 解析器

use anyhow::Result;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

/// QRZ.cn 地址信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrzCnAddressInfo {
    /// 呼号
    pub callsign: String,
    /// 中文地址
    pub chinese_address: Option<String>,
    /// 英文地址
    pub english_address: Option<String>,
    /// 更新日期（格式：YYYY-MM-DD）
    pub updated_at: Option<String>,
    /// 数据来源（固定为 "qrz.cn"）
    pub source: String,
}

impl QrzCnAddressInfo {
    /// 创建新的地址信息
    pub fn new(callsign: String) -> Self {
        Self {
            callsign,
            chinese_address: None,
            english_address: None,
            updated_at: None,
            source: "qrz.cn".to_string(),
        }
    }
}

/// 解析 QRZ.cn 地址查询页面
///
/// # 参数
/// * `html` - HTML 页面内容
/// * `callsign` - 呼号
///
/// # 返回
/// 解析后的地址信息，如果未找到地址则返回 None
pub fn parse_qrz_cn_page(html: &str, callsign: &str) -> Result<Option<QrzCnAddressInfo>> {
    log::debug!("开始解析呼号 {} 的页面内容", callsign);

    // 检查呼号是否存在（通过特殊注释标记）
    if !html.contains("<!-- callsign is exist-->") {
        log::info!("呼号 {} 不存在或未找到", callsign);
        return Ok(None);
    }

    log::debug!("找到呼号存在标记");

    // 找到 <!-- callsign is exist--> 注释后的 table 元素
    // 提取包含地址信息的 table HTML
    let table_html = extract_qrz_cn_table(html)?;
    log::debug!("提取到的 table HTML 长度: {} 字节", table_html.len());

    // 解析这个 table
    let document = Html::parse_fragment(&table_html);

    // 提取呼号（从页面验证）
    let parsed_callsign = extract_callsign(&document);
    log::debug!("提取的呼号: {:?}", parsed_callsign);

    let parsed_callsign = parsed_callsign.unwrap_or_else(|| callsign.to_string());

    // 提取更新日期
    let updated_at = extract_updated_date(&document);
    log::debug!("更新日期: {:?}", updated_at);

    // 提取中文地址
    let chinese_address = extract_chinese_address(&document);
    log::debug!("中文地址: {:?}", chinese_address);

    // 提取英文地址
    let english_address = extract_english_address(&document);
    log::debug!("英文地址: {:?}", english_address);

    // 创建地址信息对象
    let mut info = QrzCnAddressInfo::new(parsed_callsign);
    info.chinese_address = chinese_address;
    info.english_address = english_address;
    info.updated_at = updated_at;

    log::info!("✓ 成功解析呼号 {} 的信息", callsign);
    Ok(Some(info))
}

/// 提取包含呼号信息的 table（在 <!-- callsign is exist--> 注释之后）
fn extract_qrz_cn_table(html: &str) -> Result<String> {
    // 找到注释的位置
    let comment_marker = "<!-- callsign is exist-->";
    let start_pos = html.find(comment_marker)
        .ok_or_else(|| anyhow::anyhow!("未找到呼号存在标记"))?;

    // 从注释位置开始，找到下一个 <table> 标签
    let after_comment = &html[start_pos + comment_marker.len()..];
    let table_start_pos = after_comment.find("<table")
        .ok_or_else(|| anyhow::anyhow!("未找到 table 标签"))?;

    // 从 <table> 开始，找到对应的 </table> 结束标签
    let table_content = &after_comment[table_start_pos..];

    // 简单的 table 标签匹配（计数 <table> 和 </table>）
    let mut table_count = 0;
    let mut end_pos = 0;
    let mut in_tag = false;
    let mut tag_name = String::new();

    for (i, c) in table_content.char_indices() {
        if c == '<' {
            in_tag = true;
            tag_name.clear();
        } else if c == '>' && in_tag {
            in_tag = false;
            if tag_name.starts_with("table") {
                table_count += 1;
            } else if tag_name.starts_with("/table") {
                table_count -= 1;
                if table_count == 0 {
                    end_pos = i + 1;
                    break;
                }
            }
        } else if in_tag && !c.is_whitespace() {
            tag_name.push(c);
        }
    }

    if end_pos > 0 {
        Ok(table_content[..end_pos].to_string())
    } else {
        Err(anyhow::anyhow!("未找到 table 结束标签"))
    }
}

/// 提取呼号（从大字号红色显示的部分）
/// 格式：<font color="#FF0000" size="6"><strong>BH2RO</strong></font>
fn extract_callsign(document: &Html) -> Option<String> {
    // 查找红色大字号的 font 标签 - 使用属性选择器
    if let Ok(selector) = Selector::parse("font[size='6'] strong") {
        if let Some(element) = document.select(&selector).next() {
            let callsign = element.text().collect::<String>().trim().to_string();
            if !callsign.is_empty() {
                return Some(callsign);
            }
        }
    }

    // 备用方案：查找任何 font 标签中的 strong
    if let Ok(selector) = Selector::parse("font strong") {
        if let Some(element) = document.select(&selector).next() {
            let callsign = element.text().collect::<String>().trim().to_string();
            if !callsign.is_empty() {
                return Some(callsign);
            }
        }
    }

    None
}

/// 提取更新日期
/// 格式：更新日期: <font color="#000000">2018-06-11
fn extract_updated_date(document: &Html) -> Option<String> {
    let text = document.root_element().text().collect::<String>();

    // 查找"更新日期："后面的日期
    if let Some(pos) = text.find("更新日期:") {
        let start = pos + "更新日期:".len();
        let remaining = &text[start..];

        // 提取日期（格式：YYYY-MM-DD）
        let date_str = remaining
            .trim()
            .split_whitespace()
            .next()?
            .trim()
            .to_string();

        if !date_str.is_empty() {
            return Some(date_str);
        }
    }

    None
}

/// 提取中文地址
/// 格式：
/// <tr><td class="title">中文地址</td></tr>
/// <tr><td height="10"></td></tr>  <!-- 空行 -->
/// <tr><td>中国无线电集体业余电台, 北京西城区北礼士路80号, 100037</td></tr>
fn extract_chinese_address(document: &Html) -> Option<String> {
    let tr_selector = Selector::parse("tr").ok()?;
    let td_selector = Selector::parse("td").ok()?;

    // 收集所有的 tr 元素
    let all_trs: Vec<_> = document.select(&tr_selector).collect();

    // 查找包含 "中文地址" 的 tr
    for (index, tr) in all_trs.iter().enumerate() {
        let tr_text = tr.text().collect::<String>();

        if tr_text.contains("中文地址") {
            log::debug!("在 tr[{}] 找到中文地址标记: {}", index, tr_text.trim());

            // 跳过下一个 tr（空行），取第二个 tr
            if let Some(address_tr) = all_trs.get(index + 2) {
                // 获取这个 tr 里的第一个 td 的 HTML 内容
                if let Some(td) = address_tr.select(&td_selector).next() {
                    let html = td.inner_html();
                    // 将 <br> 标签转换为换行符
                    let address = html
                        .replace("<br>", "\n")
                        .replace("<br/>", "\n")
                        .replace("<br />", "\n")
                        .replace("<BR>", "\n")
                        .replace("<BR/>", "\n")
                        .replace("<BR />", "\n");
                    // 移除其他 HTML 标签，保留纯文本
                    let address = strip_html_tags(&address).trim().to_string();
                    log::debug!("提取到的中文地址: {}", address);
                    if !address.is_empty() {
                        return Some(address);
                    }
                }
            }
            break;
        }
    }

    None
}

/// 提取英文地址
/// 格式：
/// <tr><td class="title">英文地址</td></tr>
/// <tr><td height="10"></td></tr>  <!-- 空行 -->
/// <tr><td class="font">No.80 Beilishi Road,Xicheng district,Beijing 100037,P.R.China<br>China Amateur Radio Club Station</td></tr>
fn extract_english_address(document: &Html) -> Option<String> {
    let tr_selector = Selector::parse("tr").ok()?;
    let td_selector = Selector::parse("td").ok()?;

    // 收集所有的 tr 元素
    let all_trs: Vec<_> = document.select(&tr_selector).collect();

    // 查找包含 "英文地址" 的 tr
    for (index, tr) in all_trs.iter().enumerate() {
        let tr_text = tr.text().collect::<String>();

        if tr_text.contains("英文地址") {
            log::debug!("在 tr[{}] 找到英文地址标记: {}", index, tr_text.trim());
            // 跳过下一个 tr（空行），取第二个 tr
            if let Some(address_tr) = all_trs.get(index + 2) {
                // 获取这个 tr 里的第一个 td 的原始 HTML 内容
                if let Some(td) = address_tr.select(&td_selector).next() {
                    let html = td.inner_html();
                    // 将 <br> 标签转换为换行符
                    let address = html
                        .replace("<br>", "\n")
                        .replace("<br/>", "\n")
                        .replace("<br />", "\n")
                        .replace("<BR>", "\n")
                        .replace("<BR/>", "\n")
                        .replace("<BR />", "\n");
                    // 移除其他 HTML 标签，保留纯文本和换行符
                    let address = strip_html_tags(&address).trim().to_string();
                    log::debug!("提取到的英文地址: {}", address);
                    if !address.is_empty() {
                        return Some(address);
                    }
                }
            }
            break;
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
    fn test_parse_qrz_cn_format() {
        let html = r##"
            <!-- callsign is exist-->
            <table width="95%" border="0" cellspacing="0" cellpadding="0" align="center">
              <tr>
                <td><div align="center" class="font"><font color="#FF0000" size="6"><strong>BY1CRA</strong></font></div></td>
              </tr>
              <tr>
                <td class="blue">查询次数: <font color="#000000">187</font><br>
                  资料更新: <font color="#000000">BY1CRA</font><br>
                  更新日期: <font color="#000000">2008-02-20<br>
                    <br>
                    </font> <img src="../images/web/arrow_right.gif" width="12" height="7"> 点<a href="updatecall.cfm?callid=BY1CRA">这里</a>修改呼号信息和卡片<br>
                </td>
              </tr>
              <tr>
                <td height="10"></td>


              <tr>
                <td class="title">中文地址</td>
              </tr>
              <tr>
                <td height="10"></td>
              </tr>
              <tr>
                <td>
                     中国无线电集体业余电台, 北京西城区北礼士路80号, 100037
                                      </td>
              </tr>
              <tr>
                <td height="10"></td>
              </tr>
              <tr>
                <td class="title">英文地址</td>
              </tr>
              <tr>
                <td height="10"></td>
              </tr>
              <tr>
                <td class="font">No.80 Beilishi Road,Xicheng district,Beijing 100037,P.R.China<br>China Amateur Radio Club Station </td>
              </tr>
            </table>
        "##;

        let result = parse_qrz_cn_page(html, "BY1CRA").unwrap();
        assert!(result.is_some());

        let info = result.unwrap();
        assert_eq!(info.callsign, "BY1CRA");
        assert_eq!(info.chinese_address, Some("中国无线电集体业余电台, 北京西城区北礼士路80号, 100037".to_string()));
        assert_eq!(info.english_address, Some("No.80 Beilishi Road,Xicheng district,Beijing 100037,P.R.China\nChina Amateur Radio Club Station".to_string()));
        assert_eq!(info.updated_at, Some("2008-02-20".to_string()));
        assert_eq!(info.source, "qrz.cn");
    }

    #[test]
    fn test_parse_qrz_cn_no_address() {
        let html = r##"
            <html>
            <body>
                <p>呼号不存在</p>
            </body>
            </html>
        "##;

        let result = parse_qrz_cn_page(html, "TEST").unwrap();
        assert!(result.is_none());
    }
}
