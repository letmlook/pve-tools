//! Nodes API endpoints

use crate::client::PveClient;
use crate::error::PveResult;

pub async fn list_nodes(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/nodes").await
}

pub async fn get_status(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/status", node)).await
}

pub async fn get_syslog(client: &PveClient, node: &str, lines: Option<u32>) -> PveResult<serde_json::Value> {
    let path = match lines {
        Some(n) => format!("/nodes/{}/syslog?limit={}", node, n),
        None => format!("/nodes/{}/syslog", node),
    };
    client.get(&path).await
}

pub async fn get_disks(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/disks", node)).await
}

pub async fn get_services(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/services", node)).await
}

pub async fn get_capabilities(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/capabilities", node)).await
}

pub async fn get_network(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/network", node)).await
}

pub async fn create_network(client: &PveClient, node: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/network", node), Some(params)).await
}

pub async fn update_network(client: &PveClient, node: &str, iface: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/nodes/{}/network/{}", node, urlenc(iface)), Some(params)).await
}

pub async fn delete_network(client: &PveClient, node: &str, iface: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/nodes/{}/network/{}", node, urlenc(iface))).await
}

pub async fn get_tasks(client: &PveClient, node: &str, limit: Option<u32>) -> PveResult<serde_json::Value> {
    let path = match limit {
        Some(n) => format!("/nodes/{}/tasks?limit={}", node, n),
        None => format!("/nodes/{}/tasks", node),
    };
    client.get(&path).await
}

pub async fn get_rrd(client: &PveClient, node: &str, timeframe: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/rrd?timeframe={}", node, timeframe)).await
}

pub async fn get_rrddata(client: &PveClient, node: &str, timeframe: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/rrddata?timeframe={}", node, timeframe)).await
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