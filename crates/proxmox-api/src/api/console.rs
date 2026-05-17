//! Console/Proxy API endpoints

use crate::client::PveClient;
use crate::error::PveResult;

/// SPICE proxy ticket for QEMU VM
pub async fn spice_proxy(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/spiceproxy", node, vmid), Some(params)).await
}

/// VNC WebSocket proxy for QEMU VM
pub async fn vnc_websocket_proxy(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/qemu/{}/vvncproxy", node, vmid)).await
}

/// Serial terminal for QEMU VM
pub async fn serial_terminal(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/qemu/{}/serial", node, vmid)).await
}

/// Terminate QEMU VM
pub async fn terminate_vm(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/qemu/{}/terminate", node, vmid), Some(params)).await
}