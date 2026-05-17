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

// Alias management
pub async fn list_aliases(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/firewall/aliases").await
}

pub async fn create_alias(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/cluster/firewall/aliases", Some(params)).await
}

pub async fn get_alias(client: &PveClient, name: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/cluster/firewall/aliases/{}", urlenc(name))).await
}

pub async fn update_alias(client: &PveClient, name: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/cluster/firewall/aliases/{}", urlenc(name)), Some(params)).await
}

pub async fn delete_alias(client: &PveClient, name: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/cluster/firewall/aliases/{}", urlenc(name))).await
}

// IPSet management
pub async fn list_ipsets(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/firewall/ipset").await
}

pub async fn create_ipset(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/cluster/firewall/ipset", Some(params)).await
}

pub async fn get_ipset(client: &PveClient, name: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/cluster/firewall/ipset/{}", urlenc(name))).await
}

pub async fn update_ipset(client: &PveClient, name: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/cluster/firewall/ipset/{}", urlenc(name)), Some(params)).await
}

pub async fn delete_ipset(client: &PveClient, name: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/cluster/firewall/ipset/{}", urlenc(name))).await
}

// Node firewall options
pub async fn get_node_firewall_options(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/firewall/options", node)).await
}

pub async fn update_node_firewall_options(client: &PveClient, node: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/nodes/{}/firewall/options", node), Some(params)).await
}

// LXC firewall
pub async fn get_lxc_firewall_rules(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/lxc/{}/firewall/rules", node, vmid)).await
}

pub async fn add_lxc_firewall_rule(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/lxc/{}/firewall/rules", node, vmid), Some(params)).await
}

pub async fn delete_lxc_firewall_rule(client: &PveClient, node: &str, vmid: u32, pos: u32) -> PveResult<serde_json::Value> {
    client.delete(&format!("/nodes/{}/lxc/{}/firewall/rules/{}", node, vmid, pos)).await
}

pub async fn get_lxc_firewall_options(client: &PveClient, node: &str, vmid: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/lxc/{}/firewall/options", node, vmid)).await
}

pub async fn update_lxc_firewall_options(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/nodes/{}/lxc/{}/firewall/options", node, vmid), Some(params)).await
}

// VM firewall rule update (move position)
pub async fn update_vm_firewall_rule(client: &PveClient, vmid: u32, pos: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/qemu/{}/firewall/rules/{}", vmid, pos), Some(params)).await
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