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

pub struct AppState {
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
}

impl AppState {
    pub fn new() -> Self {
        Self {
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
            pve_host: std::env::var("PVE_HOST").unwrap_or_else(|_| "localhost".to_string()),
            error_msg: None,
        }
    }

    pub fn make_config(&self) -> ClientConfig {
        let mut c = ClientConfig::from_env();
        c.host = self.pve_host.clone();
        c
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

    pub fn vm_status_color(&self, status: &str) -> &'static str {
        match status {
            "running" => "green",
            "stopped" => "red",
            "paused" => "yellow",
            "suspended" => "yellow",
            _ => "white",
        }
    }
}