// QRZ.cn 客户端实现

use anyhow::{Context, Result};
use encoding_rs::GB18030;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::{redirect::Policy, Client};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

use super::qrz_cn_parser::{parse_qrz_cn_page, QrzCnAddressInfo as QrzCnAddressInfo};

// 用于递归 async 函数的 Future 类型
type BoxFuture<'a, T> = Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

/// QRZ.cn 会话信息（持久化存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRZCnSession {
    /// CFID Cookie
    cfid: Option<String>,
    /// CFTOKEN Cookie
    cftoken: Option<String>,
    /// 用户名
    username: Option<String>,
    /// 密码（URL 编码后）
    password: Option<String>,
    /// Cookie 过期时间（30天后，存储为 Unix 时间戳）
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_at_timestamp: Option<u64>,
}

impl QRZCnSession {
    /// 创建新的会话
    pub fn new() -> Self {
        Self {
            cfid: None,
            cftoken: None,
            username: None,
            password: None,
            expires_at_timestamp: None,
        }
    }

    /// 检查会话是否有效
    pub fn is_valid(&self) -> bool {
        if let (Some(cfid), Some(cftoken), Some(expires_timestamp)) =
            (&self.cfid, &self.cftoken, self.expires_at_timestamp)
        {
            // 检查是否过期
            let now_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let is_valid = now_timestamp < expires_timestamp;
            log::debug!(
                "QRZ.cn 会话有效性检查: CFID={}, CFTOKEN={}, 过期时间={}, 当前时间={}, 有效={}",
                &cfid[..cfid.len().min(8)],
                &cftoken[..cftoken.len().min(8)],
                expires_timestamp,
                now_timestamp,
                is_valid
            );
            return is_valid;
        }
        log::debug!("QRZ.cn 会话无效: 缺少 CFID 或 CFTOKEN");
        false
    }

    /// 更新会话信息
    fn update(&mut self, cfid: String, cftoken: String, username: String, password: String) {
        self.cfid = Some(cfid);
        self.cftoken = Some(cftoken);

        // URL 编码用户名和密码（使用 NON_ALPHANUMERIC 编码所有特殊字符，包括减号）
        let encoded_username = utf8_percent_encode(&username, NON_ALPHANUMERIC).to_string();
        let encoded_password = utf8_percent_encode(&password, NON_ALPHANUMERIC).to_string();

        self.username = Some(encoded_username);
        self.password = Some(encoded_password);

        // 设置过期时间为 30 天后
        let expires_at = SystemTime::now() + Duration::from_secs(30 * 24 * 60 * 60);
        self.expires_at_timestamp = expires_at
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .ok();
    }

    /// 获取 Cookie 字符串
    pub fn get_cookie_string(&self) -> Option<String> {
        if let (Some(cfid), Some(cftoken), Some(username), Some(password)) =
            (&self.cfid, &self.cftoken, &self.username, &self.password)
        {
            // 构造完整的 Cookie 字符串，包含所有必需的字段
            Some(format!(
                "CFID={}; CFTOKEN={}; USER={}; PASSWD={}; PASSWORD={}; HB_NICKNAME={}",
                cfid, cftoken, username, password, password, username
            ))
        } else {
            None
        }
    }
}

impl Default for QRZCnSession {
    fn default() -> Self {
        Self::new()
    }
}

/// QRZ.cn 客户端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRZCnConfig {
    /// 用户名
    pub username: String,
}

/// QRZ.cn 客户端
pub struct QRZCnClient {
    /// HTTP 客户端（默认）
    client: Client,
    /// HTTP 客户端（不自动跟随重定向，用于捕获 302/301 的 Set-Cookie）
    no_redirect_client: Client,
    /// 会话信息（仅内存）
    session: Arc<Mutex<QRZCnSession>>,
    /// 基础 URL
    base_url: String,
}

impl QRZCnClient {
    /// 创建新的客户端
    pub fn new() -> Result<Self> {
        let session = QRZCnSession::new();

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("无法创建 HTTP 客户端")?;

        let no_redirect_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .redirect(Policy::none())
            .build()
            .context("无法创建无重定向 HTTP 客户端")?;

        Ok(Self {
            client,
            no_redirect_client,
            session: Arc::new(Mutex::new(session)),
            base_url: "http://www.qrz.cn".to_string(),
        })
    }

    /// 登录 QRZ.cn
    ///
    /// # 参数
    /// * `username` - 用户名
    /// * `password` - 密码
    pub async fn login(&self, username: &str, password: &str) -> Result<()> {
        let login_url = format!("{}/my/", self.base_url);

        // 构建登录表单数据（按照规范使用 user 和 passwd）
        let form_data = [
            ("user", username),
            ("passwd", password),
        ];

        // 发送登录请求（按照规范添加必要的 HTTP 头部）
        log::info!("正在向 {} 发送登录请求...", login_url);
        let response = self
            .client
            .post(&login_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Origin", "http://www.qrz.cn")
            .header("Referer", "http://www.qrz.cn/home/")
            .form(&form_data)
            .send()
            .await
            .map_err(|e| {
                log::error!("HTTP 请求失败: {:?}", e);
                anyhow::anyhow!("登录请求失败: {}", e)
            })?;

        // 检查响应状态
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("登录失败: HTTP {}", response.status()));
        }

        // 从响应头中提取 Set-Cookie
        let mut cfid: Option<String> = None;
        let mut cftoken: Option<String> = None;

        let all_cookies: Vec<String> = response
            .headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .collect();

        log::debug!("登录响应中的所有 Set-Cookie 头部 ({} 个):", all_cookies.len());
        for (i, cookie) in all_cookies.iter().enumerate() {
            log::debug!("  Cookie {}: {}", i + 1, cookie);
        }

        for cookie_str in &all_cookies {
            // 解析 Cookie: "CFID=123; path=/; ..."
            if cookie_str.starts_with("CFID=") {
                if let Some(value) = cookie_str.split(';').next() {
                    if let Some(v) = value.strip_prefix("CFID=") {
                        cfid = Some(v.to_string());
                        log::info!("✓ 提取到 CFID: {}", v);
                    }
                }
            } else if cookie_str.starts_with("CFTOKEN=") {
                if let Some(value) = cookie_str.split(';').next() {
                    if let Some(v) = value.strip_prefix("CFTOKEN=") {
                        cftoken = Some(v.to_string());
                        log::info!("✓ 提取到 CFTOKEN: {}", v);
                    }
                }
            }
        }

        // 验证是否成功获取 Cookie
        match (cfid, cftoken) {
            (Some(cfid_value), Some(cftoken_value)) => {
                // 更新会话信息，包括用户名和密码
                let mut session = self.session.lock().await;
                session.update(
                    cfid_value,
                    cftoken_value,
                    username.to_string(),
                    password.to_string(),
                );
                log::info!("QRZ.cn 登录成功: {}", username);
                Ok(())
            }
            _ => {
                log::error!("登录失败: 未能获取 CFID 或 CFTOKEN");
                Err(anyhow::anyhow!("登录失败: 未能获取有效的 Cookie"))
            }
        }
    }

    /// 从响应头中的 Set-Cookie 更新会话字段（用于 302/301 场景：/call/ 首次会被踢到 ../home 并下发 USER/PASSWD 等）
    async fn apply_set_cookie_to_session(&self, response: &reqwest::Response) {
        let mut cfid: Option<String> = None;
        let mut cftoken: Option<String> = None;
        let mut user: Option<String> = None;
        let mut passwd: Option<String> = None;

        for cookie_str in response
            .headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|h| h.to_str().ok())
        {
            if let Some(kv) = cookie_str.split(';').next() {
                if let Some(v) = kv.strip_prefix("CFID=") {
                    cfid = Some(v.to_string());
                } else if let Some(v) = kv.strip_prefix("CFTOKEN=") {
                    cftoken = Some(v.to_string());
                } else if let Some(v) = kv.strip_prefix("USER=") {
                    user = Some(v.to_string());
                } else if let Some(v) = kv.strip_prefix("HB_NICKNAME=") {
                    // 有些响应只下发 HB_NICKNAME，这里复用到 USER（避免缺字段）
                    if user.is_none() {
                        user = Some(v.to_string());
                    }
                } else if let Some(v) = kv.strip_prefix("PASSWD=") {
                    passwd = Some(v.to_string());
                } else if let Some(v) = kv.strip_prefix("PASSWORD=") {
                    // 有些响应只下发 PASSWORD，这里复用到 PASSWD
                    if passwd.is_none() {
                        passwd = Some(v.to_string());
                    }
                }
            }
        }

        if cfid.is_none() && cftoken.is_none() && user.is_none() && passwd.is_none() {
            return;
        }

        let mut session = self.session.lock().await;
        if let Some(v) = cfid {
            session.cfid = Some(v);
        }
        if let Some(v) = cftoken {
            session.cftoken = Some(v);
        }
        if let Some(v) = user {
            session.username = Some(v);
        }
        if let Some(v) = passwd {
            session.password = Some(v);
        }
    }

    /// 查询呼号地址
    ///
    /// # 参数
    /// * `callsign` - 呼号
    ///
    /// # 返回
    /// 地址信息，如果未找到则返回 None
    pub async fn query_callsign(&self, callsign: &str) -> Result<Option<QrzCnAddressInfo>> {
        self.query_callsign_with_retry(callsign, 0).await
    }

    /// 查询呼号地址（带重试计数）
    ///
    /// # 参数
    /// * `callsign` - 呼号
    /// * `retry_count` - 当前重试次数
    ///
    /// # 返回
    /// 地址信息，如果未找到则返回 None
    fn query_callsign_with_retry<'a>(
        &'a self,
        callsign: &'a str,
        retry_count: u8,
    ) -> BoxFuture<'a, Result<Option<QrzCnAddressInfo>>> {
        Box::pin(async move {
        const MAX_RETRIES: u8 = 2;

        if retry_count > MAX_RETRIES {
            return Err(anyhow::anyhow!(
                "查询失败: 超过最大重试次数 ({} 次)",
                MAX_RETRIES
            ));
        }
        // 检查会话是否有效并获取 Cookie
        let cookie_string = {
            let session = self.session.lock().await;
            if !session.is_valid() {
                return Err(anyhow::anyhow!("会话已过期，请重新登录"));
            }
            session.get_cookie_string()
                .ok_or_else(|| anyhow::anyhow!("未找到有效的 Cookie"))?
        };

        // 构建查询 URL（按照规范使用 /call/ 而不是 /call/{callsign}）
        let query_url = format!("{}/call/", self.base_url);

        // 构建查询表单数据（按照规范使用 q 参数）
        let form_data = [("q", callsign)];

        // 第一次查询：使用 no_redirect_client 发送 POST，以便捕获 302/301 的 Set-Cookie（reqwest 自动跟随会吞掉中间响应头）
        let response = self
            .no_redirect_client
            .post(&query_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Origin", "http://www.qrz.cn")
            .header("Referer", "http://www.qrz.cn/call/")
            .header("Cookie", cookie_string.clone())
            .form(&form_data)
            .send()
            .await
            .context("查询请求失败")?;

        let status = response.status();
        log::debug!("响应状态码: {}", status);

        // 302/301：按 curl -L 的行为跟随跳转，并把 Set-Cookie 合并进 session，最后再重试一次原始 /call/ POST
        if status.is_redirection() {
            // 先吃下 /call/ 的 302 Set-Cookie（这里会下发 USER/PASSWD 等）
            self.apply_set_cookie_to_session(&response).await;

            let mut hops: u8 = 0;
            let mut current_url = response.url().clone();
            let mut next_location = response
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();

            while !next_location.is_empty() {
                hops += 1;
                if hops > 5 {
                    return Err(anyhow::anyhow!("重定向过多（>{}）", 5));
                }

                let next_url = current_url
                    .join(&next_location)
                    .context("无法解析重定向 URL")?;

                // 使用最新 session 生成 Cookie（因为重定向途中可能继续 Set-Cookie）
                let cookie_string = {
                    let session = self.session.lock().await;
                    session
                        .get_cookie_string()
                        .ok_or_else(|| anyhow::anyhow!("未找到有效的 Cookie"))?
                };

                log::warn!("⚠ 跟随重定向 hop={} -> {}", hops, next_url);

                let r = self
                    .no_redirect_client
                    // curl -L 在 302/301 后通常会改成 GET
                    .get(next_url.clone())
                    .header("Origin", "http://www.qrz.cn")
                    .header("Referer", "http://www.qrz.cn/call/")
                    .header("Cookie", cookie_string)
                    .send()
                    .await
                    .context("跟随重定向请求失败")?;

                self.apply_set_cookie_to_session(&r).await;

                if !r.status().is_redirection() {
                    break;
                }

                current_url = r.url().clone();
                next_location = r
                    .headers()
                    .get("location")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string();
            }

            log::warn!("重定向链处理完成（{} hops），重试原始查询一次...", hops);
            return self
                .query_callsign_with_retry(callsign, retry_count + 1)
                .await;
        }

        // 非重定向：继续处理正文
        let final_url = response.url().to_string();
        log::debug!("最终 URL: {}", final_url);

        // 检查响应状态
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("查询失败: HTTP {}", response.status()));
        }

        // 解码响应内容（GBK -> UTF-8）
        let body_bytes = response.bytes().await.context("无法读取响应内容")?;
        let (decoded, _, had_errors) = GB18030.decode(&body_bytes);

        if had_errors {
            log::warn!("解码响应内容时出现错误");
        }

        let body = decoded.to_string();
        log::debug!("响应内容长度: {} 字节", body.len());

        // 在 TRACE 模式下保存完整 HTML 以便调试
        if log::log_enabled!(log::Level::Trace) {
            log::trace!("HTML 响应:\n{}", &body);
        }

        // 解析地址信息
        parse_qrz_cn_page(&body, callsign)
    })
}

    /// 检查会话是否有效
    pub async fn is_session_valid(&self) -> bool {
        let session = self.session.lock().await;
        session.is_valid()
    }

    /// 清除会话
    pub async fn clear_session(&self) {
        let mut session = self.session.lock().await;
        *session = QRZCnSession::new();
        log::info!("QRZ.cn 会话已清除");
    }

    /// 保存会话到加密存储
    pub async fn save_session(&self) -> Result<()> {
        let session = self.session.lock().await;

        log::debug!("准备保存会话: 有效={}", session.is_valid());

        let session_json = serde_json::to_string(&*session)
            .context("序列化会话信息失败")?;

        log::debug!("会话 JSON 长度: {} 字节", session_json.len());

        crate::security::save_credential("qsl-cardhub:qrz:session", &session_json)
            .context("保存会话到加密存储失败")?;

        log::info!("✓ QRZ.cn 会话已成功保存到加密存储");
        Ok(())
    }

    /// 从加密存储加载会话
    pub async fn load_session(&self) -> Result<()> {
        log::debug!("正在尝试从加密存储加载会话...");

        match crate::security::get_credential("qsl-cardhub:qrz:session") {
            Ok(Some(session_json)) => {
                log::debug!("找到保存的会话，JSON 长度: {} 字节", session_json.len());

                let loaded_session: QRZCnSession = serde_json::from_str(&session_json)
                    .context("反序列化会话信息失败")?;

                log::debug!("会话反序列化成功");

                // 检查加载的会话是否仍然有效
                if loaded_session.is_valid() {
                    let mut session = self.session.lock().await;
                    *session = loaded_session;
                    log::info!("✓ QRZ.cn 会话已从存储成功恢复");
                    Ok(())
                } else {
                    log::warn!("存储的 QRZ.cn 会话已过期");
                    Err(anyhow::anyhow!("存储的会话已过期"))
                }
            }
            Ok(None) => {
                log::debug!("加密存储中未找到已保存的 QRZ.cn 会话");
                Err(anyhow::anyhow!("未找到已保存的会话"))
            }
            Err(e) => {
                log::warn!("从加密存储加载 QRZ.cn 会话失败: {}", e);
                Err(anyhow::anyhow!("加载会话失败: {}", e))
            }
        }
    }
}

impl Default for QRZCnClient {
    fn default() -> Self {
        Self::new().expect("无法创建 QRZ 客户端")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_validity() {
        let mut session = QRZCnSession::new();
        assert!(!session.is_valid());

        session.update(
            "123456".to_string(),
            "abcdef".to_string(),
            "testuser".to_string(),
            "testpass".to_string(),
        );
        assert!(session.is_valid());
    }

    #[test]
    fn test_client_creation() {
        let client = QRZCnClient::new();
        assert!(client.is_ok());
    }
}
