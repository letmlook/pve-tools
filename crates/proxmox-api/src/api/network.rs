//! Network API endpoints (also covers SDN in PVE 8.x+)

use crate::client::PveClient;
use crate::error::PveResult;

pub async fn get_sdn_overview(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/sdn").await
}

pub async fn list_zones(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/sdn/zones").await
}

pub async fn create_zone(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/cluster/sdn/zones", Some(params)).await
}

pub async fn delete_zone(client: &PveClient, zone: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/cluster/sdn/zones/{}", urlenc(zone))).await
}

pub async fn list_vnets(client: &PveClient, zone: Option<&str>) -> PveResult<serde_json::Value> {
    let path = match zone {
        Some(z) => format!("/cluster/sdn/vnets?zone={}", z),
        None => "/cluster/sdn/vnets".to_string(),
    };
    client.get(&path).await
}

pub async fn create_vnet(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/cluster/sdn/vnets", Some(params)).await
}

pub async fn delete_vnet(client: &PveClient, vnet: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/cluster/sdn/vnets/{}", urlenc(vnet))).await
}

pub async fn list_subnets(client: &PveClient, vnet: &str, zone: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/cluster/sdn/vnets/{}/subnets?zone={}", urlenc(vnet), urlenc(zone))).await
}

pub async fn create_subnet(client: &PveClient, vnet: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/cluster/sdn/vnets/{}/subnets", urlenc(vnet)), Some(params)).await
}

pub async fn get_ipam(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/sdn/ipam").await
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