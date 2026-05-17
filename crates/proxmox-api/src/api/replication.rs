//! Replication API endpoints

use crate::client::PveClient;
use crate::error::PveResult;

pub async fn get_replication(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/replication").await
}

pub async fn create_replication(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/cluster/replication", Some(params)).await
}

pub async fn get_replication_by_id(client: &PveClient, id: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/cluster/replication/{}", urlenc(id))).await
}

pub async fn update_replication(client: &PveClient, id: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/cluster/replication/{}", urlenc(id)), Some(params)).await
}

pub async fn delete_replication(client: &PveClient, id: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/cluster/replication/{}", urlenc(id))).await
}

pub async fn get_replication_log(client: &PveClient, id: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/cluster/replication/{}/log", urlenc(id))).await
}

pub async fn get_replication_status(client: &PveClient, id: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/cluster/replication/{}/status", urlenc(id))).await
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