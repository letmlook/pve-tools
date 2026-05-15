//! Proxmox API error types with structured error codes for agent use.

use serde::{Deserialize, Serialize};

/// Result type alias for Proxmox API operations
pub type PveResult<T> = Result<T, PveError>;

/// Structured error codes for agent use (exit codes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// 0: Success
    Success = 0,
    /// 1: General error
    General = 1,
    /// 2: Validation failed (invalid parameters)
    ValidationFailed = 2,
    /// 3: Resource not found
    NotFound = 3,
    /// 4: Authentication / permission denied
    AuthFailed = 4,
    /// 5: Conflict (resource already exists)
    Conflict = 5,
    /// 6: Timeout waiting for state change
    WaitTimeout = 6,
    /// 7: Server-side error (5xx)
    ServerError = 7,
    /// 10: Connection failed
    ConnectionFailed = 10,
    /// 11: Request timeout
    Timeout = 11,
}

impl ErrorCode {
    pub fn from_http_status(code: u16) -> Self {
        match code {
            401 | 403 => ErrorCode::AuthFailed,
            404 => ErrorCode::NotFound,
            409 => ErrorCode::Conflict,
            500..=599 => ErrorCode::ServerError,
            _ => ErrorCode::General,
        }
    }

    pub fn as_exit_code(&self) -> i32 {
        *self as i32
    }
}

/// Proxmox API error type
#[derive(Debug, thiserror::Error)]
pub enum PveError {
    #[error("connection failed: {0}")]
    Connection(String),

    #[error("request timeout: {0}")]
    Timeout(String),

    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("resource not found: {0}")]
    NotFound(String),

    #[error("validation failed: {0}")]
    ValidationFailed(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("server error {code}: {message}")]
    ServerError { code: u16, message: String },

    #[error("API error {code}: {message}")]
    ApiError { code: u32, message: String },

    #[error("timeout waiting for VM/container state change")]
    WaitTimeout,

    #[error("{context}")]
    WithContext { context: String, #[source] source: Box<PveError> },

    #[error("{message}")]
    Idempotent { message: String },
}

impl PveError {
    /// Exit code for agent use
    pub fn exit_code(&self) -> i32 {
        match self {
            PveError::Connection(_) => 10,
            PveError::Timeout(_) => 11,
            PveError::AuthenticationFailed(_) => 4,
            PveError::PermissionDenied(_) => 4,
            PveError::NotFound(_) => 3,
            PveError::ValidationFailed(_) => 2,
            PveError::Conflict(_) => 5,
            PveError::ServerError { .. } => 7,
            PveError::ApiError { .. } => 7,
            PveError::WaitTimeout => 6,
            PveError::WithContext { source, .. } => source.exit_code(),
            PveError::Idempotent { .. } => 0,
        }
    }

    /// Human-readable code name
    pub fn code_name(&self) -> &'static str {
        match self {
            PveError::Connection(_) => "CONNECTION_FAILED",
            PveError::Timeout(_) => "TIMEOUT",
            PveError::AuthenticationFailed(_) => "AUTH_FAILED",
            PveError::PermissionDenied(_) => "PERMISSION_DENIED",
            PveError::NotFound(_) => "NOT_FOUND",
            PveError::ValidationFailed(_) => "VALIDATION_FAILED",
            PveError::Conflict(_) => "CONFLICT",
            PveError::ServerError { .. } => "SERVER_ERROR",
            PveError::ApiError { .. } => "API_ERROR",
            PveError::WaitTimeout => "WAIT_TIMEOUT",
            PveError::WithContext { source, .. } => source.code_name(),
            PveError::Idempotent { .. } => "SUCCESS",
        }
    }

    /// Recovery hint for agent
    pub fn recovery_hint(&self) -> &'static str {
        match self {
            PveError::NotFound(_) => "List available resources: pve-agent vm list / pve-agent node list",
            PveError::Connection(_) => "Check PVE host is reachable and port 8006 is open",
            PveError::AuthenticationFailed(_) => "Verify credentials or token: pve-agent --token <token>",
            PveError::WaitTimeout => "Increase --timeout or use --dry-run to see the operation",
            _ => "Check command syntax: pve-agent --help",
        }
    }

    /// Convert to JSON for agent output
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "error": {
                "code": self.code_name(),
                "message": self.to_string(),
                "exit_code": self.exit_code(),
                "recovery": self.recovery_hint(),
            }
        })
    }

    /// Convert to JSON for human-readable output (TUI)
    pub fn to_user_json(&self) -> serde_json::Value {
        serde_json::json!({
            "error": {
                "code": self.code_name(),
                "message": self.to_string(),
                "hint": self.recovery_hint(),
            }
        })
    }
}

impl From<reqwest::Error> for PveError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            PveError::Timeout(e.to_string())
        } else if e.is_connect() {
            PveError::Connection(e.to_string())
        } else {
            PveError::ServerError { code: 0, message: e.to_string() }
        }
    }
}

impl From<url::ParseError> for PveError {
    fn from(e: url::ParseError) -> Self {
        PveError::ValidationFailed(format!("invalid URL: {e}"))
    }
}

impl From<serde_json::Error> for PveError {
    fn from(e: serde_json::Error) -> Self {
        PveError::ServerError { code: 0, message: format!("JSON error: {}", e) }
    }
}

