//! Cluster-related data models

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNode {
    pub nodeid: u32,
    pub node: String,
    pub status: String,
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub version: String,
    pub repoid: String,
    pub release: String,
}