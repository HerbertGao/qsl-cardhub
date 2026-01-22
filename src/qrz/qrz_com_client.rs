// QRZ.com 客户端实现

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

/// QRZ.com 会话信息（持久化存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRZComSession {
    /// xf_session Cookie
    xf_session: Option<String>,
    /// Cookie 过期时间（30天后，存储为 Unix 时间戳）
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_at_timestamp: Option<u64>,
}

impl QRZComSession {
    /// 创建新的会话
    pub fn new() -> Self {
        Self {
            xf_session: None,
            expires_at_timestamp: None,
        }
    }

    /// 检查会话是否有效
    pub fn is_valid(&self) -> bool {
        if let (Some(session), Some(expires_timestamp)) =
            (&self.xf_session, self.expires_at_timestamp)
        {
            // 检查是否过期
            let now_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let is_valid = now_timestamp < expires_timestamp;
            log::debug!(
                "QRZ.com 会话有效性检查: xf_session={}, 过期时间={}, 当前时间={}, 有效={}",
                &session[..session.len().min(8)],
                expires_timestamp,
                now_timestamp,
                is_valid
            );
            return is_valid;
        }
        log::debug!("QRZ.com 会话无效: 缺少 xf_session");
        false
    }

    /// 更新会话信息
    fn update(&mut self, xf_session: String) {
        self.xf_session = Some(xf_session);

        // 设置过期时间为 30 天后
        let expires_at = SystemTime::now() + Duration::from_secs(30 * 24 * 60 * 60);
        self.expires_at_timestamp = expires_at
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .ok();
    }

    /// 获取 Cookie 字符串
    pub fn get_cookie_string(&self) -> Option<String> {
        self.xf_session
            .as_ref()
            .map(|session| format!("xf_session={}", session))
    }
}

impl Default for QRZComSession {
    fn default() -> Self {
        Self::new()
    }
}

/// QRZ.com 客户端
pub struct QRZComClient {
    /// HTTP 客户端
    client: Client,
    /// 会话信息（仅内存）
    session: Arc<Mutex<QRZComSession>>,
    /// 基础 URL
    base_url: String,
}

impl QRZComClient {
    /// 创建新的客户端
    pub fn new() -> Result<Self> {
        let session = QRZComSession::new();

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("无法创建 HTTP 客户端")?;

        Ok(Self {
            client,
            session: Arc::new(Mutex::new(session)),
            base_url: "https://www.qrz.com".to_string(),
        })
    }

    /// 登录 QRZ.com
    ///
    /// # 参数
    /// * `username` - 用户名
    /// * `password` - 密码
    pub async fn login(&self, username: &str, password: &str) -> Result<()> {
        let login_url = format!("{}/login", self.base_url);

        // 构建登录表单数据（按照规范）
        let form_data = [
            ("username", username),
            ("password", password),
            ("login_ref", "https%3A%2F%2Fwww.qrz.com%2F"),
            ("target", "%2F"),
            ("flush", "1"),
            ("2fcode", ""),
        ];

        // 发送登录请求（按照规范添加必要的 HTTP 头部）
        log::info!("正在向 {} 发送登录请求...", login_url);
        let response = self
            .client
            .post(&login_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("User-Agent", "Mozilla/5.0")
            .header("Referer", "https://www.qrz.com/")
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

        // 从响应头中提取 Set-Cookie: xf_session
        let mut xf_session: Option<String> = None;

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
            // 解析 Cookie: "xf_session=xxx; path=/; ..."
            if cookie_str.starts_with("xf_session=") {
                if let Some(value) = cookie_str.split(';').next() {
                    if let Some(v) = value.strip_prefix("xf_session=") {
                        xf_session = Some(v.to_string());
                        log::info!("✓ 提取到 xf_session: {}...", &v[..v.len().min(16)]);
                    }
                }
            }
        }

        // 验证是否成功获取 Cookie
        if let Some(session_value) = xf_session {
            // 更新会话信息
            let mut session = self.session.lock().await;
            session.update(session_value);
            log::info!("QRZ.com 登录成功: {}", username);
            Ok(())
        } else {
            log::error!("登录失败: 未能获取 xf_session");
            Err(anyhow::anyhow!("登录失败: 未能获取有效的 Cookie"))
        }
    }

    /// 查询呼号信息
    ///
    /// # 参数
    /// * `callsign` - 呼号
    ///
    /// # 返回
    /// HTML 响应内容
    pub async fn query_callsign(&self, callsign: &str) -> Result<String> {
        // 检查会话是否有效并获取 Cookie
        let cookie_string = {
            let session = self.session.lock().await;
            if !session.is_valid() {
                return Err(anyhow::anyhow!("会话已过期，请重新登录"));
            }
            session
                .get_cookie_string()
                .ok_or_else(|| anyhow::anyhow!("未找到有效的 Cookie"))?
        };

        // 构建查询 URL
        let query_url = format!("{}/db/{}", self.base_url, callsign);

        // 发送查询请求
        let response = self
            .client
            .get(&query_url)
            .header("User-Agent", "Mozilla/5.0")
            .header("Referer", "https://www.qrz.com/")
            .header("Cookie", cookie_string)
            .send()
            .await
            .context("查询请求失败")?;

        // 检查响应状态
        let status = response.status();
        if status.as_u16() == 404 {
            return Err(anyhow::anyhow!("呼号未找到"));
        }
        if !status.is_success() {
            return Err(anyhow::anyhow!("查询失败: HTTP {}", status));
        }

        // 获取响应文本（UTF-8 编码）
        let html = response.text().await.context("读取响应内容失败")?;

        Ok(html)
    }

    /// 检查登录状态
    pub async fn check_login_status(&self) -> bool {
        let session = self.session.lock().await;
        session.is_valid()
    }

    /// 清除会话
    pub async fn clear_session(&self) {
        let mut session = self.session.lock().await;
        *session = QRZComSession::new();
        log::info!("QRZ.com 会话已清除");
    }

    /// 保存会话到加密存储
    pub async fn save_session(&self) -> Result<()> {
        let session = self.session.lock().await;

        log::debug!("准备保存 QRZ.com 会话: 有效={}", session.is_valid());

        let session_json = serde_json::to_string(&*session)
            .context("序列化会话信息失败")?;

        log::debug!("QRZ.com 会话 JSON 长度: {} 字节", session_json.len());

        crate::security::save_credential("qsl-cardhub:qrz.com:session", &session_json)
            .context("保存会话到加密存储失败")?;

        log::info!("✓ QRZ.com 会话已成功保存到加密存储");
        Ok(())
    }

    /// 从加密存储加载会话
    pub async fn load_session(&self) -> Result<()> {
        log::debug!("正在尝试从加密存储加载 QRZ.com 会话...");

        match crate::security::get_credential("qsl-cardhub:qrz.com:session") {
            Ok(Some(session_json)) => {
                log::debug!("找到保存的 QRZ.com 会话，JSON 长度: {} 字节", session_json.len());

                let loaded_session: QRZComSession = serde_json::from_str(&session_json)
                    .context("反序列化会话信息失败")?;

                log::debug!("QRZ.com 会话反序列化成功");

                // 检查加载的会话是否仍然有效
                if loaded_session.is_valid() {
                    let mut session = self.session.lock().await;
                    *session = loaded_session;
                    log::info!("✓ QRZ.com 会话已从存储成功恢复");
                    Ok(())
                } else {
                    log::warn!("存储的 QRZ.com 会话已过期");
                    Err(anyhow::anyhow!("存储的会话已过期"))
                }
            }
            Ok(None) => {
                log::debug!("加密存储中未找到已保存的 QRZ.com 会话");
                Err(anyhow::anyhow!("未找到已保存的会话"))
            }
            Err(e) => {
                log::warn!("从加密存储加载 QRZ.com 会话失败: {}", e);
                Err(anyhow::anyhow!("加载会话失败: {}", e))
            }
        }
    }
}

impl Default for QRZComClient {
    fn default() -> Self {
        Self::new().expect("无法创建 QRZ.com 客户端")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qrz_com_session_validity() {
        let mut session = QRZComSession::new();
        assert!(!session.is_valid());

        session.update("test_session_token".to_string());
        assert!(session.is_valid());
    }

    #[test]
    fn test_qrz_com_client_creation() {
        let client = QRZComClient::new();
        assert!(client.is_ok());
    }
}
