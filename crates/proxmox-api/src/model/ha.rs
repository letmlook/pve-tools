//! HA (High Availability) data model for PVE API responses

use serde::{Deserialize, Serialize};

/// HA managed resource (VM/CT)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaResource {
    pub sid: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub vmid: Option<u32>,
    pub state: String,
    pub group: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub node: String,
    #[serde(default)]
    pub cmt: u64,
}

/// HA resource status detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaResourceStatus {
    pub sid: String,
    pub cmt: u64,
    pub state: String,
    pub node: String,
    #[serde(default)]
    pub leaders: Vec<serde_json::Value>,
    #[serde(default)]
    pub last_change: u64,
}

/// HA group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaGroup {
    pub group: String,
    #[serde(default)]
    pub type_: String,
    pub nodes: String,
    #[serde(default)]
    pub nofailback: u32,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub restricted: u32,
}

/// HA status overall
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaStatus {
    pub quorate: bool,
    pub nodelist: Vec<serde_json::Value>,
    #[serde(default)]
    pub master: String,
}

/// Backup schedule info (from /cluster/backup)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSchedule {
    pub id: String,
    pub enabled: bool,
    pub schedule: String,
    pub storage: String,
    #[serde(default)]
    pub selection: String,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub mail: String,
    #[serde(default)]
    pub comment: String,
}

/// Storage content entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageContent {
    pub volid: String,
    pub format: Option<String>,
    pub size: u64,
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(default)]
    pub format2: String,
    #[serde(default)]
    pub parent: String,
    #[serde(default)]
    pub protected_: bool,
    #[serde(default)]
    pub notes: String,
}