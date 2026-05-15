//! HA (High Availability) API endpoints

use crate::client::PveClient;
use crate::error::PveResult;

pub async fn get_status(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/ha/status").await
}

pub async fn list_resources(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/ha/resources").await
}

pub async fn create_resource(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/cluster/ha/resources", Some(params)).await
}

pub async fn update_resource(client: &PveClient, name: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/cluster/ha/resources/{}", urlenc(name)), Some(params)).await
}

pub async fn delete_resource(client: &PveClient, name: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/cluster/ha/resources/{}", urlenc(name))).await
}

pub async fn list_groups(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/ha/groups").await
}

pub async fn create_group(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/cluster/ha/groups", Some(params)).await
}

pub async fn delete_group(client: &PveClient, name: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/cluster/ha/groups/{}", urlenc(name))).await
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