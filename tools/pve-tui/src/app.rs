//! pve-tui App state machine

use proxmox_api::{PveClient, ClientConfig};
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum View {
    Dashboard, VMs, Storage, Logs, Help,
}

impl View {
    pub fn next(&self) -> Self {
        match self {
            View::Dashboard => View::VMs,
            View::VMs => View::Storage,
            View::Storage => View::Logs,
            View::Logs => View::Help,
            View::Help => View::Dashboard,
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            View::Dashboard => "Dashboard",
            View::VMs => "VM List",
            View::Storage => "Storage",
            View::Logs => "Logs",
            View::Help => "Help",
        }
    }
    pub fn index(&self) -> usize {
        match self {
            View::Dashboard => 0,
            View::VMs => 1,
            View::Storage => 2,
            View::Logs => 3,
            View::Help => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AppMode {
    Setup,
    Running,
}

#[derive(Clone)]
pub struct VmEntry {
    pub vmid: u32,
    pub name: String,
    pub node: String,
    pub vm_type: String,
    pub status: String,
    pub cpu: f64,
    pub mem: u64,
}

#[derive(Clone)]
pub struct ConnectionConfig {
    pub host: String,
    pub port: String,
    pub user: String,
    pub auth_method: AuthMethod,
    pub token: String,
    pub password: String,
    pub verify_ssl: bool,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        // First try to load from config file
        if let Ok(config_file) = proxmox_api::client::ConfigFile::load() {
            if let Some(profile) = config_file.resolve_profile(None) {
                return Self::from_profile(profile);
            }
        }
        // Fall back to env vars
        Self {
            host: std::env::var("PVE_HOST").unwrap_or_else(|_| "192.168.1.100".to_string()),
            port: std::env::var("PVE_PORT").unwrap_or_else(|_| "8006".to_string()),
            user: std::env::var("PVE_USER").unwrap_or_else(|_| "root@pam".to_string()),
            auth_method: if std::env::var("PVE_TOKEN").ok().map(|s| !s.is_empty()).unwrap_or(false) {
                AuthMethod::Token
            } else {
                AuthMethod::Password
            },
            token: std::env::var("PVE_TOKEN").unwrap_or_default(),
            password: std::env::var("PVE_PASSWORD").unwrap_or_default(),
            verify_ssl: std::env::var("PVE_VERIFY_SSL").ok()
                .map(|v| v == "1" || v == "true")
                .unwrap_or(false),
        }
    }
}

impl ConnectionConfig {
    pub fn from_profile(profile: &ClientConfig) -> Self {
        Self {
            host: profile.host.clone(),
            port: profile.port.to_string(),
            user: profile.user.clone(),
            auth_method: if profile.token.is_some() {
                AuthMethod::Token
            } else {
                AuthMethod::Password
            },
            token: profile.token.clone().unwrap_or_default(),
            password: profile.password.clone().unwrap_or_default(),
            verify_ssl: profile.verify_ssl,
        }
    }

    pub fn save_to_file(&self) -> anyhow::Result<()> {
        let client_config = ClientConfig {
            host: self.host.clone(),
            port: self.port.parse().unwrap_or(8006),
            user: self.user.clone(),
            token: if self.auth_method == AuthMethod::Token && !self.token.is_empty() {
                Some(self.token.clone())
            } else {
                None
            },
            password: if self.auth_method == AuthMethod::Password && !self.password.is_empty() {
                Some(self.password.clone())
            } else {
                None
            },
            verify_ssl: self.verify_ssl,
            timeout_secs: 60,
        };

        let mut config_file = proxmox_api::client::ConfigFile::load().unwrap_or_default();
        config_file.default = Some(client_config);
        config_file.save()?;
        Ok(())
    }

    pub fn test_connection(&self) -> bool {
        let config = self.to_client_config();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            match proxmox_api::PveClient::new(&config).await {
                Ok(_) => true,
                Err(_) => false,
            }
        })
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Validate host
        if self.host.is_empty() {
            errors.push("Host is required".to_string());
        } else if !self.host.contains('.') && self.host != "localhost" {
            // Might be just a hostname without TLD, but not necessarily an error
        }

        // Validate port
        if let Ok(port) = self.host.parse::<u16>() {
            if port == 0 {
                errors.push("Port cannot be 0".to_string());
            }
        }

        // Validate user format
        if self.user.is_empty() {
            errors.push("User is required".to_string());
        } else if !self.user.contains('@') {
            errors.push("User should be in format user@realm (e.g., root@pam)".to_string());
        }

        // Validate auth
        match self.auth_method {
            AuthMethod::Token => {
                if self.token.is_empty() {
                    errors.push("API Token is required when using Token auth".to_string());
                } else if !self.token.contains('=') {
                    errors.push("Token should be in format userid=tokenid=secret".to_string());
                }
            }
            AuthMethod::Password => {
                if self.password.is_empty() {
                    errors.push("Password is required when using Password auth".to_string());
                }
            }
        }

        errors
    }
}

impl ConnectionConfig {
    pub fn to_client_config(&self) -> ClientConfig {
        ClientConfig {
            host: self.host.clone(),
            port: self.port.parse().unwrap_or(8006),
            user: self.user.clone(),
            token: if self.auth_method == AuthMethod::Token && !self.token.is_empty() {
                Some(self.token.clone())
            } else {
                None
            },
            password: if self.auth_method == AuthMethod::Password && !self.password.is_empty() {
                Some(self.password.clone())
            } else {
                None
            },
            verify_ssl: self.verify_ssl,
            timeout_secs: 60,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AuthMethod {
    Token,
    Password,
}

enum SetupSetter {
    Host,
    Port,
    User,
    Token,
    Password,
}

impl SetupSetter {
    fn set(&self, app: &mut AppState, value: String) {
        match self {
            SetupSetter::Host => app.setup_config.host = value,
            SetupSetter::Port => app.setup_config.port = value,
            SetupSetter::User => app.setup_config.user = value,
            SetupSetter::Token => app.setup_config.token = value,
            SetupSetter::Password => app.setup_config.password = value,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SetupField {
    Host,
    Port,
    User,
    AuthMethod,
    Token,
    Password,
    VerifySsl,
    Connect,
}

impl SetupField {
    pub fn all() -> [SetupField; 8] {
        [SetupField::Host, SetupField::Port, SetupField::User, SetupField::AuthMethod, SetupField::Token, SetupField::Password, SetupField::VerifySsl, SetupField::Connect]
    }

    pub fn next(&self) -> Self {
        match self {
            SetupField::Host => SetupField::Port,
            SetupField::Port => SetupField::User,
            SetupField::User => SetupField::AuthMethod,
            SetupField::AuthMethod => SetupField::Token,
            SetupField::Token => SetupField::VerifySsl,
            SetupField::Password => SetupField::VerifySsl,
            SetupField::VerifySsl => SetupField::Connect,
            SetupField::Connect => SetupField::Host,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            SetupField::Host => SetupField::Connect,
            SetupField::Port => SetupField::Host,
            SetupField::User => SetupField::Port,
            SetupField::AuthMethod => SetupField::User,
            SetupField::Token => SetupField::AuthMethod,
            SetupField::Password => SetupField::AuthMethod,
            SetupField::VerifySsl => SetupField::Token,
            SetupField::Connect => SetupField::VerifySsl,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SetupField::Host => "Host",
            SetupField::Port => "Port",
            SetupField::User => "User",
            SetupField::AuthMethod => "Auth",
            SetupField::Token => "API Token",
            SetupField::Password => "Password",
            SetupField::VerifySsl => "Skip SSL",
            SetupField::Connect => "[ Connect ]",
        }
    }
}

pub struct AppState {
    pub mode: AppMode,
    pub view: View,
    pub loading: bool,
    pub version: Option<Value>,
    pub nodes: Option<Value>,
    pub resources: Option<Value>,
    pub storage_list: Option<Value>,
    pub cluster_status: Option<Value>,
    pub vm_list: Vec<VmEntry>,
    pub selected_vm: Option<usize>,
    pub logs: Option<Value>,
    pub pve_host: String,
    pub error_msg: Option<String>,
    // Setup mode
    pub setup_config: ConnectionConfig,
    pub setup_field: SetupField,
    pub setup_cursor: usize,
    pub connecting: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mode: AppMode::Setup,
            view: View::Dashboard,
            loading: false,
            version: None,
            nodes: None,
            resources: None,
            storage_list: None,
            cluster_status: None,
            vm_list: Vec::new(),
            selected_vm: None,
            logs: None,
            pve_host: String::new(),
            error_msg: None,
            setup_config: ConnectionConfig::default(),
            setup_field: SetupField::Host,
            setup_cursor: 0,
            connecting: false,
        }
    }

    pub fn cycle_view(&mut self) {
        self.view = self.view.next();
    }

    pub fn update_vm_list(&mut self) {
        self.vm_list.clear();
        if let Some(res) = &self.resources {
            if let Some(data) = res.pointer("/data").and_then(|d| d.as_array()) {
                for item in data {
                    let vmid = item.get("vmid").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    let name = item.get("name").or_else(|| item.get("id")).and_then(|v| v.as_str()).unwrap_or("-").to_string();
                    let node = item.get("node").and_then(|v| v.as_str()).unwrap_or("-").to_string();
                    let vm_type = item.get("type").or_else(|| item.get("resource")).and_then(|v| v.as_str()).unwrap_or("qemu").to_string();
                    let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                    let cpu = item.get("cpu").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let mem = item.get("mem").and_then(|v| v.as_u64()).unwrap_or(0);
                    if vmid > 0 {
                        self.vm_list.push(VmEntry { vmid, name, node, vm_type, status, cpu, mem });
                    }
                }
            }
        }
    }

    pub async fn load_all(&mut self, client: &PveClient) {
        self.loading = true;
        self.error_msg = None;

        let results = tokio::join!(
            client.get("/version"),
            client.get("/nodes"),
            client.get("/cluster/resources"),
            client.get("/storage"),
            client.get("/cluster/status"),
        );

        self.version = results.0.ok();
        self.nodes = results.1.ok();
        self.resources = results.2.ok();
        self.storage_list = results.3.ok();
        self.cluster_status = results.4.ok();

        self.update_vm_list();
        self.loading = false;
    }

    pub async fn load_logs(&mut self, client: &PveClient) {
        self.loading = true;
        self.error_msg = None;
        self.logs = client.get("/cluster/log?limit=200").await.ok();
        self.loading = false;
    }

    pub fn get_vm_node(&self, vmid: u32) -> String {
        self.vm_list.iter().find(|v| v.vmid == vmid).map(|v| v.node.clone()).unwrap_or_else(|| "localhost".to_string())
    }

    pub fn format_mem(&self, bytes: u64) -> String {
        let gb = bytes as f64 / 1024.0 / 1024.0 / 1024.0;
        if gb >= 1.0 {
            format!("{:.1} GB", gb)
        } else {
            format!("{} MB", bytes / 1024 / 1024)
        }
    }

    pub fn get_value(&self, field: SetupField) -> &str {
        match field {
            SetupField::Host => &self.setup_config.host,
            SetupField::Port => &self.setup_config.port,
            SetupField::User => &self.setup_config.user,
            SetupField::AuthMethod => match self.setup_config.auth_method {
                AuthMethod::Token => "Token",
                AuthMethod::Password => "Password",
            },
            SetupField::Token => &self.setup_config.token,
            SetupField::Password => &self.setup_config.password,
            SetupField::VerifySsl => if self.setup_config.verify_ssl { "true" } else { "false" },
            SetupField::Connect => "",
        }
    }

    pub fn set_value(&mut self, field: SetupField, value: String) {
        match field {
            SetupField::Host => self.setup_config.host = value,
            SetupField::Port => self.setup_config.port = value,
            SetupField::User => self.setup_config.user = value,
            SetupField::AuthMethod => {},
            SetupField::Token => self.setup_config.token = value,
            SetupField::Password => self.setup_config.password = value,
            SetupField::VerifySsl => self.setup_config.verify_ssl = value == "true",
            SetupField::Connect => {}
        }
    }

    pub fn setup_backspace(&mut self) {
        let field = self.setup_field;
        if matches!(field, SetupField::Connect | SetupField::VerifySsl | SetupField::AuthMethod) {
            return;
        }
        let (val, setter) = match field {
            SetupField::Host => (&self.setup_config.host, SetupSetter::Host),
            SetupField::Port => (&self.setup_config.port, SetupSetter::Port),
            SetupField::User => (&self.setup_config.user, SetupSetter::User),
            SetupField::Token => (&self.setup_config.token, SetupSetter::Token),
            SetupField::Password => (&self.setup_config.password, SetupSetter::Password),
            _ => return,
        };
        if self.setup_cursor > 0 && self.setup_cursor <= val.len() {
            let mut new_val = val.clone();
            new_val.remove(self.setup_cursor - 1);
            self.setup_cursor = self.setup_cursor.saturating_sub(1);
            setter.set(self, new_val);
        } else if self.setup_cursor == 0 && !val.is_empty() {
            let mut new_val = val.clone();
            new_val.pop();
            setter.set(self, new_val);
        }
    }

    pub fn setup_type(&mut self, c: char) {
        let field = self.setup_field;
        if matches!(field, SetupField::Connect | SetupField::VerifySsl | SetupField::AuthMethod) {
            return;
        }
        let max_len = if field == SetupField::Port { 5 } else { 100 };
        match field {
            SetupField::Host => {
                if self.setup_config.host.len() >= max_len { return; }
                let pos = self.setup_cursor.min(self.setup_config.host.len());
                self.setup_config.host.insert(pos, c);
                self.setup_cursor += 1;
            }
            SetupField::Port => {
                if self.setup_config.port.len() >= max_len { return; }
                let pos = self.setup_cursor.min(self.setup_config.port.len());
                self.setup_config.port.insert(pos, c);
                self.setup_cursor += 1;
            }
            SetupField::User => {
                if self.setup_config.user.len() >= max_len { return; }
                let pos = self.setup_cursor.min(self.setup_config.user.len());
                self.setup_config.user.insert(pos, c);
                self.setup_cursor += 1;
            }
            SetupField::Token => {
                if self.setup_config.token.len() >= max_len { return; }
                let pos = self.setup_cursor.min(self.setup_config.token.len());
                self.setup_config.token.insert(pos, c);
                self.setup_cursor += 1;
            }
            SetupField::Password => {
                if self.setup_config.password.len() >= max_len { return; }
                let pos = self.setup_cursor.min(self.setup_config.password.len());
                self.setup_config.password.insert(pos, c);
                self.setup_cursor += 1;
            }
            _ => {}
        }
    }

    pub fn to_client_config(&self) -> ClientConfig {
        self.setup_config.to_client_config()
    }
}