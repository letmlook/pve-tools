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

// APT / Package Management
pub async fn get_apt(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/apt", node)).await
}

pub async fn get_apt_changelog(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/apt/changelog", node)).await
}

pub async fn get_apt_repositories(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/apt/repositories", node)).await
}

pub async fn check_apt_update(client: &PveClient, node: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/apt/update", node), Some(params)).await
}

// Subscription
pub async fn get_subscription(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/subscription", node)).await
}

pub async fn set_subscription(client: &PveClient, node: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/nodes/{}/subscription", node), Some(params)).await
}

// Time
pub async fn get_time(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/time", node)).await
}

pub async fn update_time(client: &PveClient, node: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/nodes/{}/time", node), Some(params)).await
}

// DNS
pub async fn get_dns(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/dns", node)).await
}

pub async fn update_dns(client: &PveClient, node: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/nodes/{}/dns", node), Some(params)).await
}

// Certificates
pub async fn get_certificates(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/certificates", node)).await
}

pub async fn add_custom_certificate(client: &PveClient, node: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/certificates/custom", node), Some(params)).await
}

pub async fn delete_custom_certificate(client: &PveClient, node: &str, name: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/nodes/{}/certificates/custom/{}", node, urlenc(name))).await
}

// ACME
pub async fn get_acme(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/acme", node)).await
}

pub async fn request_acme_certificate(client: &PveClient, node: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/acme/certificate", node), Some(params)).await
}

// Journal
pub async fn get_journal(client: &PveClient, node: &str, lines: Option<u32>) -> PveResult<serde_json::Value> {
    let path = match lines {
        Some(n) => format!("/nodes/{}/journal?limit={}", node, n),
        None => format!("/nodes/{}/journal", node),
    };
    client.get(&path).await
}

// Report
pub async fn get_report(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/report", node)).await
}

// Wake-on-LAN
pub async fn wakeonlan(client: &PveClient, node: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/wakeonlan", node), Some(params)).await
}

// Scan
pub async fn scan(client: &PveClient, node: &str, scan_type: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/scan/{}", node, scan_type)).await
}

pub async fn do_scan(client: &PveClient, node: &str, scan_type: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/scan/{}", node, scan_type), Some(params)).await
}

// Hardware
pub async fn get_hardware(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/hardware", node)).await
}

pub async fn get_hardware_by_type(client: &PveClient, node: &str, hw_type: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/hardware/{}", node, hw_type)).await
}

// Disks (extended)
pub async fn get_disk(client: &PveClient, node: &str, disk: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/disks/{}", node, urlenc(disk))).await
}

pub async fn update_disk(client: &PveClient, node: &str, disk: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/nodes/{}/disks/{}", node, urlenc(disk)), Some(params)).await
}

pub async fn get_disk_smart(client: &PveClient, node: &str, disk: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/disks/{}/smart", node, urlenc(disk))).await
}

pub async fn get_disk_lvmthin(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/disks/lvmthin", node)).await
}

pub async fn get_disk_zfs(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/disks/zfs", node)).await
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