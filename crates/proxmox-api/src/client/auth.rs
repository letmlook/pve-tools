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
    pub CSRFPreventionToken: String,
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
    let (token, password) = match auth {
        AuthMethod::Token(t) => (Some(t), None),
        AuthMethod::Password(p) => (None, Some(p)),
    };

    // If we have a token, try token auth
    if let Some(token_str) = token {
        if token_str.contains('=') {
            // Full token: userid=tokenid=secret
            let parts: Vec<&str> = token_str.splitn(2, '=').collect();
            if parts.len() == 2 {
                let auth_value = format!("PVEAPIToken={}", token_str);
                // Test the token by getting version
                let resp = http
                    .get(format!("{}/api2/json/version", base_url))
                    .header("Authorization", auth_value)
                    .send()
                    .await?;

                if resp.status() == 401 || resp.status() == 403 {
                    anyhow::bail!("invalid API token");
                }

                return Ok(AuthContext {
                    ticket: token_str.clone(),
                    csrf_token: String::new(),
                    username: user.to_string(),
                    capabilities: serde_json::Value::Null,
                    expire: i64::MAX,
                });
            }
        }
        anyhow::bail!("invalid token format, expected userid=tokenid=secret");
    }

    // Password login
    let password = password.ok_or_else(|| anyhow::anyhow!("password required"))?;

    let resp = http
        .post(format!("{}/api2/json/access/ticket", base_url))
        .form(&[
            ("username", user),
            ("password", &password),
        ])
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("login failed ({}): {}", status, body);
    }

    let ticket: TicketResponse = resp.json().await?;

    Ok(AuthContext {
        ticket: ticket.ticket,
        csrf_token: ticket.CSRFPreventionToken,
        username: ticket.username,
        capabilities: ticket.cap,
        expire: ticket.expire,
    })
}