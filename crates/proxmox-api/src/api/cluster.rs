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
    pub uptime: Option<u64>,
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

// Cluster options
pub async fn get_cluster_options(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/options").await
}

pub async fn update_cluster_options(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put("/cluster/options", Some(params)).await
}

// Cluster config
pub async fn get_cluster_config(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/config").await
}

pub async fn reconfigure_cluster(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/cluster/config", Some(params)).await
}

// Metrics
pub async fn get_metrics(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/metrics").await
}

pub async fn configure_metrics(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/cluster/metrics", Some(params)).await
}

// Notifications
pub async fn get_notifications(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/notifications").await
}

pub async fn send_notification(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/cluster/notifications", Some(params)).await
}

// Replication (cluster level)
pub async fn get_replication(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/replication").await
}

pub async fn create_replication(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/cluster/replication", Some(params)).await
}

pub async fn get_replication_by_id(client: &PveClient, id: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/cluster/replication/{}", urlenc(id))).await
}

pub async fn update_replication(client: &PveClient, id: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/cluster/replication/{}", urlenc(id)), Some(params)).await
}

pub async fn delete_replication(client: &PveClient, id: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/cluster/replication/{}", urlenc(id))).await
}

// Jobs
pub async fn get_cluster_jobs(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/jobs").await
}

pub async fn create_cluster_job(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/cluster/jobs", Some(params)).await
}

// ACME cluster level
pub async fn get_cluster_acme(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/acme").await
}

pub async fn update_cluster_acme(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put("/cluster/acme", Some(params)).await
}

// Mapping
pub async fn get_cluster_mapping(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/mapping").await
}

pub async fn create_cluster_mapping(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/cluster/mapping", Some(params)).await
}

pub async fn delete_cluster_mapping(client: &PveClient, mapping_type: &str, id: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/cluster/mapping/{}/{}", urlenc(mapping_type), urlenc(id))).await
}

fn urlenc(s: &str) -> String {
    let mut r = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => r.push(b as char),
            _ => r.push_str(&format!("%{:02X}", b)),
        }
    }
    r
}