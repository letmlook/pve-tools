//! Storage API endpoints

use crate::client::PveClient;
use crate::error::PveResult;

pub async fn list(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/storage").await
}

pub async fn get(client: &PveClient, storage: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/storage/{}", storage)).await
}

pub async fn create(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/storage", Some(params)).await
}

pub async fn update(client: &PveClient, storage: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/storage/{}", storage), Some(params)).await
}

pub async fn delete(client: &PveClient, storage: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/storage/{}", storage)).await
}

pub async fn get_status(client: &PveClient, storage: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/storage/{}/status", storage)).await
}

pub async fn get_content(client: &PveClient, storage: &str, content_type: Option<&str>) -> PveResult<serde_json::Value> {
    let path = match content_type {
        Some(ct) => format!("/storage/{}/content?type={}", storage, urlenc(ct)),
        None => format!("/storage/{}/content", storage),
    };
    client.get(&path).await
}

pub async fn delete_content(client: &PveClient, storage: &str, content_type: &str, volume: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/storage/{}/content/{}/{}", storage, content_type, urlenc(volume))).await
}

pub async fn node_storage_list(client: &PveClient, node: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/nodes/{}/storage", node)).await
}

/// Upload content to storage
pub async fn upload(client: &PveClient, storage: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/storage/{}/upload", storage), Some(params)).await
}

/// Get volume info
pub async fn get_volume(client: &PveClient, storage: &str, content_type: &str, volume: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/storage/{}/content/{}/{}", storage, urlenc(content_type), urlenc(volume))).await
}

/// Update volume properties
pub async fn update_volume(client: &PveClient, storage: &str, content_type: &str, volume: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/storage/{}/content/{}/{}", storage, urlenc(content_type), urlenc(volume)), Some(params)).await
}

/// Prune backup snapshots
pub async fn prune_backups(client: &PveClient, storage: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/storage/{}/prune-backups", storage), Some(params)).await
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
