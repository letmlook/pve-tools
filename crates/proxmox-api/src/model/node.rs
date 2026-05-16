//! Node data model for PVE API responses

use serde::{Deserialize, Serialize};

/// Basic node info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node: String,
    pub status: String,
    pub uptime: u64,
    #[serde(default)]
    pub cpu: f64,
    #[serde(default)]
    pub maxcpu: u64,
    #[serde(default)]
    pub mem: u64,
    #[serde(default)]
    pub maxmem: u64,
}

/// Detailed node status (from /nodes/{node}/status)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub node: String,
    pub uptime: u64,
    pub cpu: f64,
    pub maxcpu: u64,
    pub mem: u64,
    pub maxmem: u64,
    #[serde(default)]
    pub disk: f64,
    #[serde(default)]
    pub netin: u64,
    #[serde(default)]
    pub netout: u64,
}

/// Service info from /nodes/{node}/services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub state: String,
    #[serde(default)]
    pub description: String,
}

/// Network interface info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub iface: String,
    #[serde(rename = "type")]
    #[serde(default)]
    pub iface_type: String,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub netmask: String,
    #[serde(default)]
    pub gateway: String,
    #[serde(default)]
    pub bridge: String,
    #[serde(default)]
    pub comments: String,
}

/// Task info from /nodes/{node}/tasks or /cluster/tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub id: String,
    pub type_: String,
    pub status: String,
    pub user: String,
    pub node: String,
    pub pid: u32,
    #[serde(default)]
    pub starttime: u64,
    #[serde(default)]
    pub endtime: u64,
    #[serde(default)]
    pub duration: u64,
}

/// Disk info from /nodes/{node}/disks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub device: String,
    pub size: u64,
    pub vendor: String,
    pub model: String,
    pub serial: String,
    pub type_: String,
    pub used: Option<String>,
    #[serde(default)]
    pub gpt: bool,
    #[serde(default)]
    pub mounted: bool,
}