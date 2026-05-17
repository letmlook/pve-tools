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

// SDN Controllers
pub async fn list_controllers(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/sdn/controllers").await
}

pub async fn create_controller(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/cluster/sdn/controllers", Some(params)).await
}

pub async fn get_controller(client: &PveClient, controller: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/cluster/sdn/controllers/{}", urlenc(controller))).await
}

pub async fn update_controller(client: &PveClient, controller: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/cluster/sdn/controllers/{}", urlenc(controller)), Some(params)).await
}

pub async fn delete_controller(client: &PveClient, controller: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/cluster/sdn/controllers/{}", urlenc(controller))).await
}

// IPAM
pub async fn create_ipam(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/cluster/sdn/ipam", Some(params)).await
}

pub async fn get_ipam_by_name(client: &PveClient, name: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/cluster/sdn/ipam/{}", urlenc(name))).await
}

pub async fn delete_ipam(client: &PveClient, name: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/cluster/sdn/ipam/{}", urlenc(name))).await
}

// DNS Zones
pub async fn list_dns_zones(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/sdn/dns").await
}

pub async fn create_dns_zone(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/cluster/sdn/dns", Some(params)).await
}

pub async fn get_dns_zone(client: &PveClient, zone: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/cluster/sdn/dns/{}", urlenc(zone))).await
}

pub async fn update_dns_zone(client: &PveClient, zone: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/cluster/sdn/dns/{}", urlenc(zone)), Some(params)).await
}

pub async fn delete_dns_zone(client: &PveClient, zone: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/cluster/sdn/dns/{}", urlenc(zone))).await
}

// Zone operations
pub async fn get_zone(client: &PveClient, zone: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/cluster/sdn/zones/{}", urlenc(zone))).await
}

pub async fn update_zone(client: &PveClient, zone: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/cluster/sdn/zones/{}", urlenc(zone)), Some(params)).await
}

pub async fn apply_zone(client: &PveClient, zone: &str) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/cluster/sdn/zones/{}/apply", urlenc(zone)), None).await
}

pub async fn reload_zone(client: &PveClient, zone: &str) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/cluster/sdn/zones/{}/reload", urlenc(zone)), None).await
}

// VNet operations
pub async fn get_vnet(client: &PveClient, vnet: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/cluster/sdn/vnets/{}", urlenc(vnet))).await
}

pub async fn update_vnet(client: &PveClient, vnet: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/cluster/sdn/vnets/{}", urlenc(vnet)), Some(params)).await
}

// Subnet operations
pub async fn get_subnet(client: &PveClient, vnet: &str, subnet: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/cluster/sdn/vnets/{}/subnets/{}", urlenc(vnet), urlenc(subnet))).await
}

pub async fn update_subnet(client: &PveClient, vnet: &str, subnet: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/cluster/sdn/vnets/{}/subnets/{}", urlenc(vnet), urlenc(subnet)), Some(params)).await
}

pub async fn delete_subnet(client: &PveClient, vnet: &str, subnet: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/cluster/sdn/vnets/{}/subnets/{}", urlenc(vnet), urlenc(subnet))).await
}

pub async fn apply_subnet(client: &PveClient, vnet: &str, subnet: &str) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/cluster/sdn/vnets/{}/subnets/{}/apply", urlenc(vnet), urlenc(subnet)), None).await
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