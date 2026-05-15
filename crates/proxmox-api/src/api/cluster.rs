//! Cluster API endpoints

use crate::client::PveClient;
use crate::error::PveResult;

/// Cluster status / Quorum
#[derive(Debug, serde::Deserialize)]
pub struct ClusterStatus {
    pub quorate: Option<bool>,
    pub level: Option<String>,
    pub name: Option<String>,
    pub nodes: Option<Vec<serde_json::Value>>,
}

/// Cluster node info
#[derive(Debug, serde::Deserialize)]
pub struct ClusterNode {
    pub nodeid: u32,
    pub name: String,
    pub ip: String,
    pub level: String,
    pub local: bool,
    pub online: bool,
    pub type_: Option<String>,
}

/// Cluster resource entry
#[derive(Debug, serde::Deserialize)]
pub struct ClusterResource {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: String,
    pub node: Option<String>,
    pub status: Option<String>,
    pub uptine: Option<u64>,
    pub maxcpu: Option<f64>,
    pub cpu: Option<f64>,
    pub maxmem: Option<u64>,
    pub mem: Option<u64>,
    pub maxdisk: Option<u64>,
    pub disk: Option<u64>,
}

pub async fn get_status(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/status").await
}

pub async fn get_nodes(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/nodes").await
}

pub async fn get_resources(client: &PveClient, type_filter: Option<&str>) -> PveResult<serde_json::Value> {
    let path = match type_filter {
        Some(t) => format!("/cluster/resources?type={}", t),
        None => "/cluster/resources".to_string(),
    };
    client.get(&path).await
}

pub async fn get_nextid(client: &PveClient) -> PveResult<String> {
    let resp: serde_json::Value = client.get("/cluster/nextid").await?;
    resp.pointer("/data")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or_else(|| crate::error::PveError::NotFound("no nextid in response".to_string()))
}

pub async fn get_log(client: &PveClient, limit: Option<u32>) -> PveResult<serde_json::Value> {
    let path = match limit {
        Some(n) => format!("/cluster/log?limit={}", n),
        None => "/cluster/log".to_string(),
    };
    client.get(&path).await
}

pub async fn get_tasks(client: &PveClient, limit: Option<u32>) -> PveResult<serde_json::Value> {
    let path = match limit {
        Some(n) => format!("/cluster/tasks?limit={}", n),
        None => "/cluster/tasks".to_string(),
    };
    client.get(&path).await
}