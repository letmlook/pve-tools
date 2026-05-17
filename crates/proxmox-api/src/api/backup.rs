//! Backup API endpoints

use crate::client::PveClient;
use crate::error::PveResult;

pub async fn list_schedules(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/backup").await
}

pub async fn get_schedule(client: &PveClient, id: &str) -> PveResult<serde_json::Value> {
    let path = format!("/cluster/backup/{}", urlenc(id));
    client.get(&path).await
}

pub async fn create_schedule(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/cluster/backup", Some(params)).await
}

pub async fn update_schedule(client: &PveClient, id: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    let path = format!("/cluster/backup/{}", urlenc(id));
    client.put(&path, Some(params)).await
}

pub async fn delete_schedule(client: &PveClient, id: &str) -> PveResult<serde_json::Value> {
    let path = format!("/cluster/backup/{}", urlenc(id));
    client.delete(&path).await
}

pub async fn trigger_backup(client: &PveClient, node: &str, vmid: u32, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    let path = format!("/nodes/{}/qemu/{}/backup", node, vmid);
    client.post_form(&path, Some(params)).await
}

// Backup job operations
pub async fn sync_backup(client: &PveClient, id: &str) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/cluster/backup/{}/sync", urlenc(id)), None).await
}

pub async fn get_backup_joblog(client: &PveClient, id: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/cluster/backup/{}/joblog", urlenc(id))).await
}

pub async fn get_backup_mail(client: &PveClient, id: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/cluster/backup/{}/mail", urlenc(id))).await
}

// Node level vzdump
pub async fn get_node_vzdump(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/vzdump", node)).await
}

pub async fn run_node_vzdump(client: &PveClient, node: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/nodes/{}/vzdump", node), Some(params)).await
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
