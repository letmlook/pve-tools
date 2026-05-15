//! LXC container API endpoints

use crate::client::PveClient;
use crate::error::PveResult;

pub async fn list(client: &PveClient, node: &str, status: Option<&str>) -> PveResult<serde_json::Value> {
    let path = match status {
        Some(s) => format!("/nodes/{}/lxc?status={}", node, s),
        None => format!("/nodes/{}/lxc", node),
    };
    client.get(&path).await
}

pub async fn get_status(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/lxc/{}/status/current", node, vmid)).await
}

pub async fn get_config(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/lxc/{}/config", node, vmid)).await
}

pub async fn create(client: &PveClient, node: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/lxc", node), Some(params)).await
}

pub async fn update(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/nodes/{}/lxc/{}/config", node, vmid), Some(params)).await
}

pub async fn delete(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.delete(&format!("/nodes/{}/lxc/{}", node, vmid)).await
}

pub async fn start(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/lxc/{}/status/start", node, vmid), None).await
}

pub async fn stop(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/lxc/{}/status/stop", node, vmid), None).await
}

pub async fn shutdown(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/lxc/{}/status/shutdown", node, vmid), None).await
}

pub async fn suspend(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/lxc/{}/status/suspend", node, vmid), None).await
}

pub async fn resume(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/lxc/{}/status/resume", node, vmid), None).await
}

pub async fn clone_vm(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/lxc/{}/clone", node, vmid), Some(params)).await
}

pub async fn list_snapshots(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/lxc/{}/snapshot", node, vmid)).await
}

pub async fn create_snapshot(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/lxc/{}/snapshot", node, vmid), Some(params)).await
}

pub async fn rollback_snapshot(client: &PveClient, node: &str, vmid: u32, name: &str) -> PveResult<serde_json::Value> {
    let enc = urlenc(name);
    client.post_form(&format!("/nodes/{}/lxc/{}/snapshot/{}/rollback", node, vmid, enc), None).await
}

pub async fn delete_snapshot(client: &PveClient, node: &str, vmid: u32, name: &str) -> PveResult<serde_json::Value> {
    let enc = urlenc(name);
    client.delete(&format!("/nodes/{}/lxc/{}/snapshot/{}", node, vmid, enc)).await
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