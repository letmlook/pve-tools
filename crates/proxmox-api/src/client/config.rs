//! Client configuration from file / env / CLI arguments

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Client configuration (per profile)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ClientConfig {
    /// PVE host (IP or hostname)
    pub host: String,

    /// API port (default: 8006)
    #[serde(default = "default_port")]
    pub port: u16,

    /// Username (e.g. root@pam, root@pam@realm)
    pub user: String,

    /// API token (recommended, alternative to password)
    #[serde(default)]
    pub token: Option<String>,

    /// Password (not recommended, use token instead)
    #[serde(default)]
    pub password: Option<String>,

    /// Skip SSL verification (for self-signed certs)
    #[serde(default = "default_false")]
    pub verify_ssl: bool,

    /// Timeout for HTTP requests in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_port() -> u16 { 8006 }
fn default_false() -> bool { false }
fn default_timeout() -> u64 { 60 }

impl ClientConfig {
    /// Build base URL
    pub fn base_url(&self) -> String {
        format!("https://{}:{}", self.host, self.port)
    }

    /// Build API base URL
    pub fn api_base(&self) -> String {
        format!("{}/api2/json", self.base_url())
    }
}

/// Multi-profile configuration file (~/.config/pve/config.toml)
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigFile {
    /// Default profile
    #[serde(default)]
    pub default: Option<ClientConfig>,

    /// Named profiles
    #[serde(default)]
    pub profiles: HashMap<String, ClientConfig>,
}

impl ConfigFile {
    /// Load configuration from standard paths
    ///
    /// Searches: ~/.config/pve/config.toml, /etc/pve/config.toml
    pub fn load() -> anyhow::Result<Self> {
        // Try user config first
        if let Some(path) = Self::user_config_path() {
            if path.exists() {
                let contents = std::fs::read_to_string(&path)?;
                return Self::parse(&contents);
            }
        }

        // Fallback to system config
        let system = PathBuf::from("/etc/pve/config.toml");
        if system.exists() {
            let contents = std::fs::read_to_string(&system)?;
            return Self::parse(&contents);
        }

        Ok(ConfigFile::default())
    }

    fn user_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("pve").join("config.toml"))
    }

    fn parse(contents: &str) -> anyhow::Result<Self> {
        let config: ConfigFile = toml::from_str(contents)?;
        Ok(config)
    }

    /// Get profile by name, or default, or first available
    pub fn resolve_profile(&self, name: Option<&str>) -> Option<&ClientConfig> {
        name.and_then(|n| self.profiles.get(n))
            .or(self.default.as_ref())
            .or(self.profiles.values().next())
    }

    /// Save config to user config file (~/.config/pve/config.toml)
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::user_config_path()
            .ok_or_else(|| anyhow::anyhow!("could not determine config directory"))?;

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&path, contents)?;
        Ok(())
    }
}

impl ClientConfig {
    /// Load from env vars + config file + defaults
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("PVE_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("PVE_PORT")
                .unwrap_or_else(|_| "8006".to_string())
                .parse()
                .unwrap_or(8006),
            user: std::env::var("PVE_USER").unwrap_or_else(|_| "root@pam".to_string()),
            token: std::env::var("PVE_TOKEN").ok(),
            password: std::env::var("PVE_PASSWORD").ok(),
            verify_ssl: std::env::var("PVE_VERIFY_SSL").ok()
                .map(|v| v == "1" || v == "true")
                .unwrap_or(false),
            timeout_secs: std::env::var("PVE_TIMEOUT")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
        }
    }
}