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

/// Get pending changes
pub async fn get_pending(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/lxc/{}/pending", node, vmid)).await
}

/// Resize container
pub async fn resize(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/lxc/{}/resize", node, vmid), Some(params)).await
}

/// Move volume
pub async fn move_volume(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/lxc/{}/move_volume", node, vmid), Some(params)).await
}

/// Get network interfaces
pub async fn get_interfaces(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/lxc/{}/interfaces", node, vmid)).await
}

/// Create network interface
pub async fn create_interface(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/lxc/{}/interfaces", node, vmid), Some(params)).await
}

/// Update network interface
pub async fn update_interface(client: &PveClient, node: &str, vmid: u32, iface: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/nodes/{}/lxc/{}/interfaces/{}", node, vmid, urlenc(iface)), Some(params)).await
}

/// Delete network interface
pub async fn delete_interface(client: &PveClient, node: &str, vmid: u32, iface: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/nodes/{}/lxc/{}/interfaces/{}", node, vmid, urlenc(iface))).await
}

/// Migrate container
pub async fn migrate(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/lxc/{}/migrate", node, vmid), Some(params)).await
}

/// Create template
pub async fn to_template(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/lxc/{}/template", node, vmid), None).await
}

/// VNC proxy
pub async fn vnc_proxy(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/lxc/{}/vncproxy", node, vmid), None).await
}

/// VNC WebSocket
pub async fn vnc_websocket(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/lxc/{}/vncwebsocket", node, vmid)).await
}

/// Terminal proxy
pub async fn term_proxy(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/lxc/{}/termproxy", node, vmid), None).await
}

/// Get RRD data
pub async fn get_rrd(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/lxc/{}/rrd", node, vmid)).await
}

/// Get RRD data with timeframe
pub async fn get_rrddata(client: &PveClient, node: &str, vmid: u32, timeframe: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/lxc/{}/rrddata?timeframe={}", node, vmid, timeframe)).await
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