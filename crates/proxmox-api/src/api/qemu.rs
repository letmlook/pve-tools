//! QEMU VM API endpoints

use crate::client::PveClient;
use crate::error::PveResult;

/// List VMs on a node
pub async fn list(client: &PveClient, node: &str, filter: Option<&str>, status: Option<&str>) -> PveResult<serde_json::Value> {
    let mut path = format!("/nodes/{}/qemu", node);
    let mut params = Vec::new();

    if let Some(f) = filter {
        params.push(("type".to_string(), f.to_string()));
    }
    if let Some(s) = status {
        params.push(("status".to_string(), s.to_string()));
    }

    if !params.is_empty() {
        let query = params.iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding(v)))
            .collect::<Vec<_>>()
            .join("&");
        path = format!("{}?{}", path, query);
    }

    client.get(&path).await
}

/// Get VM status
pub async fn get_status(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/qemu/{}/status/current", node, vmid)).await
}

/// Get VM config
pub async fn get_config(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/qemu/{}/config", node, vmid)).await
}

/// Create VM
pub async fn create(client: &PveClient, node: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu", node), Some(params)).await
}

/// Update VM config
pub async fn update(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/nodes/{}/qemu/{}/config", node, vmid), Some(params)).await
}

/// Delete VM
pub async fn delete(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.delete(&format!("/nodes/{}/qemu/{}", node, vmid)).await
}

/// Start VM
pub async fn start(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/status/start", node, vmid), None).await
}

/// Stop VM (force)
pub async fn stop(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/status/stop", node, vmid), None).await
}

/// Shutdown VM (graceful)
pub async fn shutdown(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/status/shutdown", node, vmid), None).await
}

/// Reboot VM
pub async fn reboot(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/status/reboot", node, vmid), None).await
}

/// Suspend VM
pub async fn suspend(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/status/suspend", node, vmid), None).await
}

/// Resume VM
pub async fn resume(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/status/resume", node, vmid), None).await
}

/// Reset VM (force restart)
pub async fn reset(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/status/reset", node, vmid), None).await
}

/// Clone VM
pub async fn clone_vm(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/clone", node, vmid), Some(params)).await
}

/// Convert VM to template
pub async fn to_template(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/template", node, vmid), None).await
}

/// List snapshots
pub async fn list_snapshots(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/qemu/{}/snapshot", node, vmid)).await
}

/// Create snapshot
pub async fn create_snapshot(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/snapshot", node, vmid), Some(params)).await
}

/// Rollback snapshot
pub async fn rollback_snapshot(client: &PveClient, node: &str, vmid: u32, name: &str) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/snapshot/{}/rollback", node, vmid, urlencoding(name)), None).await
}

/// Delete snapshot
pub async fn delete_snapshot(client: &PveClient, node: &str, vmid: u32, name: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/nodes/{}/qemu/{}/snapshot/{}", node, vmid, urlencoding(name))).await
}

/// Migrate VM
pub async fn migrate(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/migrate", node, vmid), Some(params)).await
}

/// Resize disk
pub async fn resize_disk(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/resize", node, vmid), Some(params)).await
}

/// Move disk
pub async fn move_disk(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/move_disk", node, vmid), Some(params)).await
}

/// Detach disk
pub async fn unlink_disk(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/unlink", node, vmid), Some(params)).await
}

/// Get pending changes
pub async fn get_pending(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/qemu/{}/pending", node, vmid)).await
}

/// Agent: get info
pub async fn agent_info(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/qemu/{}/agent/info", node, vmid)).await
}

/// Agent: exec command
pub async fn agent_exec(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/agent/exec", node, vmid), Some(params)).await
}

/// Agent: fsinfo
pub async fn agent_fsinfo(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/qemu/{}/agent/fsinfo", node, vmid)).await
}

/// Agent: network info
pub async fn agent_network(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/qemu/{}/agent/networkinfo", node, vmid)).await
}

/// VNC proxy
pub async fn vnc_proxy(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/vncproxy", node, vmid), None).await
}

/// Get VM RRD data
pub async fn get_rrd(client: &PveClient, node: &str, vmid: u32, timeframe: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/qemu/{}/rrd?timeframe={}", node, vmid, timeframe)).await
}

/// Pause VM
pub async fn pause(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/status/pause", node, vmid), None).await
}

/// Unpause VM
pub async fn unpause(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/status/unpause", node, vmid), None).await
}

/// Send key to VM
pub async fn sendkey(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/sendkey", node, vmid), Some(params)).await
}

/// Send monitor command
pub async fn monitor(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/monitor", node, vmid), Some(params)).await
}

/// Generate CloudInit config
pub async fn cloudinit(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/cloudinit", node, vmid), Some(params)).await
}

/// SPICE proxy
pub async fn spiceproxy(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/spiceproxy", node, vmid), Some(params)).await
}

/// VNC WebSocket
pub async fn vnc_websocket(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/qemu/{}/vncwebsocket", node, vmid)).await
}

/// Terminal proxy
pub async fn term_proxy(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/termproxy", node, vmid), None).await
}

/// M-Tunnel (move tunnel)
pub async fn mtunnel(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/qemu/{}/mtunnel", node, vmid)).await
}

/// Get VM RRD data with timeframe
pub async fn get_rrddata(client: &PveClient, node: &str, vmid: u32, timeframe: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/qemu/{}/rrddata?timeframe={}", node, vmid, timeframe)).await
}

/// Get VM features
pub async fn get_feature(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/qemu/{}/feature", node, vmid)).await
}

/// Get VM capabilities
pub async fn get_capabilities(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/qemu/{}/capabilities", node, vmid)).await
}

/// Configure VM from snapshot
pub async fn snapshot_config(client: &PveClient, node: &str, vmid: u32, snapname: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/snapshot/{}/config", node, vmid, urlencoding(snapname)), Some(params)).await
}

/// Update firewall rule (move position)
pub async fn update_firewall_rule(client: &PveClient, node: &str, vmid: u32, pos: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/nodes/{}/qemu/{}/firewall/rules/{}", node, vmid, pos), Some(params)).await
}

fn urlencoding(s: &str) -> String {
    let mut result = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => result.push(byte as char),
            _ => result.push_str(&format!("%{:02X}", byte)),
        }
    }
    result
}