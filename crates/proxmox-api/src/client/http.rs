//! Proxmox HTTP client with auth, retry, and API methods

use std::time::Duration;

use reqwest::{Client, header};
use tokio::sync::Mutex;

use super::auth::{login, AuthContext, AuthMethod};
use super::config::ClientConfig as Config;
use crate::error::{PveError, PveResult};

/// Proxmox API client
pub struct PveClient {
    host: String,
    port: u16,
    auth: Mutex<Option<AuthContext>>,
    http: Client,
    api_base: String,
}

impl std::fmt::Debug for PveClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PveClient")
            .field("host", &self.host)
            .field("port", &self.port)
            .finish()
    }
}

impl PveClient {
    /// Create a new PveClient from config
    pub async fn new(config: &Config) -> PveResult<Self> {
        let http = Self::build_http_client(config.verify_ssl, config.timeout_secs)?;

        let api_base = format!("https://{}:{}/api2/json", config.host, config.port);

        let auth = if let Some(ref token) = config.token {
            if token.contains('=') {
                AuthMethod::Token(token.clone())
            } else {
                AuthMethod::Token(format!("{}={}", config.user, token))
            }
        } else {
            AuthMethod::Password(config.password.clone().unwrap_or_default())
        };

        let auth_context = login(&http, &api_base, &config.user, auth)
            .await
            .map_err(|e| PveError::AuthenticationFailed(e.to_string()))?;

        Ok(Self {
            host: config.host.clone(),
            port: config.port,
            auth: Mutex::new(Some(auth_context)),
            http,
            api_base,
        })
    }

    pub async fn from_env() -> PveResult<Self> {
        let config = Config::from_env();
        Self::new(&config).await
    }

    fn build_http_client(verify_ssl: bool, timeout_secs: u64) -> PveResult<Client> {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .pool_max_idle_per_host(4);

        if !verify_ssl {
            // Build a dangerous verifier for self-signed certs
            let danger_mod = DangerousVerifier {};
            builder = builder.danger_accept_invalid_certs(true);
        }

        builder.build().map_err(|e| PveError::Connection(e.to_string()))
    }

    async fn auth_context(&self) -> PveResult<AuthContext> {
        let guard = self.auth.lock().await;
        guard.clone().ok_or_else(|| PveError::AuthenticationFailed("not logged in".to_string()))
    }

    async fn build_request(&self, method: reqwest::Method, path: &str) -> PveResult<reqwest::Request> {
        let auth = self.auth_context().await?;
        let url = format!("{}{}", self.api_base, path);
        let mut request = reqwest::Request::new(method, url.parse()?);

        let headers = request.headers_mut();
        headers.insert(header::COOKIE, header::HeaderValue::from_str(&auth.cookie_value()).map_err(|e| PveError::ValidationFailed(format!("invalid cookie: {}", e)))?);

        if !auth.csrf_token.is_empty() {
            headers.insert(
                header::HeaderName::from_static("CSRFPreventionToken"),
                header::HeaderValue::from_str(&auth.csrf_token).map_err(|e| PveError::ValidationFailed(format!("invalid csrf: {}", e)))?,
            );
        }

        Ok(request)
    }

    /// GET request
    pub async fn get(&self, path: &str) -> PveResult<serde_json::Value> {
        let mut req = self.build_request(reqwest::Method::GET, path).await?;
        req.headers_mut().insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );

        let resp = self.http.execute(req).await?;
        self.handle_response(resp).await
    }

    /// POST request with form params
    pub async fn post_form(&self, path: &str, params: Option<&[(String, String)]>) -> PveResult<serde_json::Value> {
        let mut req = self.build_request(reqwest::Method::POST, path).await?;

        if let Some(params) = params {
            let form = reqwest::Body::from(
                params.iter()
                    .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
                    .collect::<Vec<_>>()
                    .join("&"),
            );
            *req.body_mut() = Some(form);
            req.headers_mut().insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("application/x-www-form-urlencoded"),
            );
        }

        let resp = self.http.execute(req).await?;
        self.handle_response(resp).await
    }

    /// PUT request
    pub async fn put(&self, path: &str, params: Option<&[(String, String)]>) -> PveResult<serde_json::Value> {
        let mut req = self.build_request(reqwest::Method::PUT, path).await?;

        if let Some(params) = params {
            let form = reqwest::Body::from(
                params.iter()
                    .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
                    .collect::<Vec<_>>()
                    .join("&"),
            );
            *req.body_mut() = Some(form);
            req.headers_mut().insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("application/x-www-form-urlencoded"),
            );
        }

        let resp = self.http.execute(req).await?;
        self.handle_response(resp).await
    }

    /// DELETE request
    pub async fn delete(&self, path: &str) -> PveResult<serde_json::Value> {
        let mut req = self.build_request(reqwest::Method::DELETE, path).await?;
        let resp = self.http.execute(req).await?;
        self.handle_response(resp).await
    }

    async fn handle_response(&self, resp: reqwest::Response) -> PveResult<serde_json::Value> {
        let status = resp.status();

        if status.is_success() {
            let body = resp.text().await?;
            if body.is_empty() {
                return Ok(serde_json::json!({"data": null}));
            }
            serde_json::from_str(&body).map_err(|e| {
                PveError::ServerError {
                    code: status.as_u16(),
                    message: format!("JSON parse error: {e}, body: {}", &body[..body.len().min(200)]),
                }
            })
        } else {
            let body = resp.text().await.unwrap_or_default();

            if let Ok(err) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(msg) = err.get("message").or(err.get("error")) {
                    let message = msg.as_str().unwrap_or("unknown").to_string();

                    match status.as_u16() {
                        401 => return Err(PveError::AuthenticationFailed(message)),
                        403 => return Err(PveError::PermissionDenied(message)),
                        404 => return Err(PveError::NotFound(message)),
                        400 => return Err(PveError::ValidationFailed(message)),
                        409 => return Err(PveError::Conflict(message)),
                        500..=599 => return Err(PveError::ServerError { code: status.as_u16(), message }),
                        _ => {
                            if let Some(code) = err.get("code").and_then(|c| c.as_i64()) {
                                return Err(PveError::ApiError { code: code as u32, message });
                            }
                        }
                    }
                }
            }

            Err(PveError::ServerError { code: status.as_u16(), message: body })
        }
    }

    /// Wait for VM/Container state change
    pub async fn wait_for_vm_state(
        &self, node: &str, vmid: u32, vm_type: &str, expected: &str, timeout_secs: u32,
    ) -> PveResult<String> {
        let start = std::time::Instant::now();
        let path = format!("/nodes/{}/{}/{}/status/current", node, vm_type, vmid);

        loop {
            let resp: serde_json::Value = self.get(&path).await?;
            let current = resp.pointer("/data/status").and_then(|v| v.as_str()).unwrap_or("unknown");

            if current == expected {
                return Ok(current.to_string());
            }

            if start.elapsed().as_secs() > timeout_secs as u64 {
                return Err(PveError::WaitTimeout);
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }
}

fn encode(s: &str) -> String {
    let mut result = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

struct DangerousVerifier;
