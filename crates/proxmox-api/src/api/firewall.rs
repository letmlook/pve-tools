//! Firewall API endpoints

use crate::client::PveClient;
use crate::error::PveResult;

pub async fn cluster_rules(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/firewall/rules").await
}

pub async fn cluster_add_rule(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/cluster/firewall/rules", Some(params)).await
}

pub async fn cluster_delete_rule(client: &PveClient, pos: u32) -> PveResult<serde_json::Value> {
    client.delete(&format!("/cluster/firewall/rules/{}", pos)).await
}

pub async fn cluster_options(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/firewall/options").await
}

pub async fn cluster_update_options(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put("/cluster/firewall/options", Some(params)).await
}

pub async fn cluster_groups(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/firewall/groups").await
}

pub async fn node_rules(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/firewall/rules", node)).await
}

pub async fn node_add_rule(client: &PveClient, node: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/firewall/rules", node), Some(params)).await
}

pub async fn vm_rules(client: &PveClient, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/qemu/{}/firewall/rules", vmid)).await
}

pub async fn vm_add_rule(client: &PveClient, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/qemu/{}/firewall/rules", vmid), Some(params)).await
}

pub async fn vm_delete_rule(client: &PveClient, vmid: u32, pos: u32) -> PveResult<serde_json::Value> {
    client.delete(&format!("/qemu/{}/firewall/rules/{}", vmid, pos)).await
}

pub async fn vm_options(client: &PveClient, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/qemu/{}/firewall/options", vmid)).await
}

pub async fn vm_update_options(client: &PveClient, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/qemu/{}/firewall/options", vmid), Some(params)).await
}