//! Pools API endpoints

use crate::client::PveClient;
use crate::error::PveResult;

pub async fn list(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/pools").await
}

pub async fn get(client: &PveClient, poolid: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/pools/{}", urlenc(poolid))).await
}

pub async fn create(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/pools", Some(params)).await
}

pub async fn update(client: &PveClient, poolid: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/pools/{}", urlenc(poolid)), Some(params)).await
}

pub async fn delete(client: &PveClient, poolid: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/pools/{}", urlenc(poolid))).await
}

/// Get pool members
pub async fn get_members(client: &PveClient, poolid: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/pools/{}/members", urlenc(poolid))).await
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