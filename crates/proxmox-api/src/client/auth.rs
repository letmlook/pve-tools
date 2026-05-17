//! Proxmox authentication (Ticket + CSRF, or API Token)

use serde::{Deserialize, Serialize};

/// Authentication context for PVE API requests
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// Cookie value for PVEAuthCookie
    pub ticket: String,
    /// CSRFPreventionToken header value
    pub csrf_token: String,
    /// Username (e.g. root@pam)
    pub username: String,
    /// Capabilities / permissions
    pub capabilities: serde_json::Value,
    /// Ticket expiration timestamp (seconds since epoch)
    pub expire: i64,
}

impl AuthContext {
    /// Build the PVEAuthCookie header value
    pub fn cookie_value(&self) -> String {
        format!("PVEAuthCookie={}", self.ticket)
    }
}

/// Ticket response from POST /api2/json/access/ticket
#[derive(Debug, Deserialize, Serialize)]
pub struct TicketResponse {
    pub username: String,
    pub ticket: String,
    #[serde(rename = "CSRFPreventionToken")]
    pub csrf_prevention_token: String,
    pub cap: serde_json::Value,
    pub expire: i64,
}

/// Token authentication (preferred over password)
#[derive(Debug, Clone)]
pub struct TokenAuth {
    pub user: String,
    pub token_id: String,
    pub secret: String,
}

impl TokenAuth {
    /// Build Authorization header value
    pub fn auth_header(&self) -> String {
        format!("PVEAPIToken={}={}", self.user, self.secret)
    }
}

/// Combined auth method
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// API Token (preferred)
    Token(String),
    /// Username + Password
    Password(String),
}

/// Build auth context from token or password
pub async fn login(
    http: &reqwest::Client,
    base_url: &str,
    user: &str,
    auth: AuthMethod,
) -> anyhow::Result<AuthContext> {
    // 1. 先尝试 token auth (如果提供了 token)
    if let AuthMethod::Token(ref token_str) = auth {
        if !token_str.is_empty() {
            if let Some(ctx) = try_token_auth(http, base_url, user, token_str).await {
                return Ok(ctx);
            }
            // token 无效，继续尝试密码
        }
    }

    // 2. 密码登录 (或 token 无效后的 fallback)
    if let AuthMethod::Password(ref password) = auth {
        if !password.is_empty() {
            return password_login(http, base_url, user, password).await;
        }
    }

    anyhow::bail!("no valid credentials provided (need token or password)")
}

/// Try token authentication, return None if token is invalid or not provided
async fn try_token_auth(
    http: &reqwest::Client,
    base_url: &str,
    user: &str,
    token_str: &str,
) -> Option<AuthContext> {
    // 构建 token value
    let auth_value = if token_str.contains('=') {
        format!("PVEAPIToken={}", token_str)
    } else {
        // 没有 =，把整个字符串当作 tokenid，前缀加 user
        format!("PVEAPIToken={}={}", user, token_str)
    };

    let resp = http
        .get(format!("{}/version", base_url))
        .header("Authorization", auth_value)
        .send()
        .await
        .ok()?;

    // 401/403 表示 token 无效
    if resp.status() == 401 || resp.status() == 403 {
        return None;
    }

    if resp.status().is_success() {
        return Some(AuthContext {
            ticket: token_str.to_string(),
            csrf_token: String::new(),
            username: user.to_string(),
            capabilities: serde_json::Value::Null,
            expire: i64::MAX,
        });
    }

    None
}

use std::collections::HashMap;

/// Extract error message from PVE error response JSON
fn extract_error_message(body: &str) -> Option<String> {
    // Try to parse as generic JSON and look for error fields
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        // Try "error" field first
        if let Some(msg) = json.get("error").and_then(|v| v.as_str()) {
            return Some(msg.to_string());
        }
        // Then "message" field
        if let Some(msg) = json.get("message").and_then(|v| v.as_str()) {
            return Some(msg.to_string());
        }
        // Try "response" field (some PVE endpoints use this)
        if let Some(msg) = json.get("response").and_then(|v| v.as_str()) {
            return Some(msg.to_string());
        }
    }
    None
}

/// Password login via PAM
async fn password_login(
    http: &reqwest::Client,
    base_url: &str,
    user: &str,
    password: &str,
) -> anyhow::Result<AuthContext> {
    // 确保用户名有 realm 后缀，默认使用 pam
    let user_with_realm = if user.contains('@') {
        user.to_string()
    } else {
        format!("{}@pam", user)
    };

    let url = format!("{}/access/ticket", base_url);
    println!("[DEBUG] login url = {}", url);
    println!("[DEBUG] login user = {}", user_with_realm);

    // 手动构造 form data (参考 Go 项目的做法)
    let mut form_data = HashMap::new();
    form_data.insert("username", user_with_realm.as_str());
    form_data.insert("password", password);

    let resp = http
        .post(url.as_str())
        .form(&form_data)
        .send()
        .await
        .map_err(|e| {
            println!("[DEBUG] reqwest error: {}", e);
            anyhow::anyhow!("request failed: {}", e)
        })?;

    println!("[DEBUG] response headers: {:?}", resp.headers());
    println!("[DEBUG] response status = {}", resp.status());
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();

    let body_preview = if body.len() > 300 { format!("{}...", &body[..300]) } else { body.clone() };

    if body.contains("<html") {
        println!("[DEBUG] response body (HTML): {}", body_preview);
    } else {
        println!("[DEBUG] response body (JSON/text): {}", body_preview);
    }

    if !status.is_success() {
        anyhow::bail!("login failed ({}): {}", status, body);
    }

    // First, try to parse as TicketResponse
    let ticket: Result<TicketResponse, _> = serde_json::from_str(&body);

    match ticket {
        Ok(t) => {
            Ok(AuthContext {
                ticket: t.ticket,
                csrf_token: t.csrf_prevention_token,
                username: t.username,
                capabilities: t.cap,
                expire: t.expire,
            })
        }
        Err(e) => {
            // JSON parse failed - check if server returned an error JSON
            println!("[DEBUG] JSON parse failed, checking for error response: {}", e);

            if let Some(error_msg) = extract_error_message(&body) {
                anyhow::bail!("authentication failed: {}", error_msg);
            }

            // No recognizable error message - show raw body for debugging
            let preview = if body.len() > 200 { &body[..200] } else { &body };
            anyhow::bail!("login failed: JSON parse error: {}; response: {}...", e, preview)
        }
    }
}
