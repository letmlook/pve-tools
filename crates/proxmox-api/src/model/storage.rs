//! Storage-related data models

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub storage: String,
    #[serde(rename = "type")]
    pub storage_type: String,
    pub total: u64,
    pub used: u64,
    pub avail: u64,
}