//! Ceph API endpoints

use crate::client::PveClient;
use crate::error::PveResult;

pub async fn get_ceph_status(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/ceph").await
}

pub async fn get_ceph_fs(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/ceph/fs").await
}

// Ceph Pools
pub async fn list_ceph_pools(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/ceph/pools").await
}

pub async fn create_ceph_pool(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/cluster/ceph/pools", Some(params)).await
}

pub async fn get_ceph_pool(client: &PveClient, name: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/cluster/ceph/pools/{}", urlenc(name))).await
}

pub async fn update_ceph_pool(client: &PveClient, name: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/cluster/ceph/pools/{}", urlenc(name)), Some(params)).await
}

pub async fn delete_ceph_pool(client: &PveClient, name: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/cluster/ceph/pools/{}", urlenc(name))).await
}

// Ceph OSD
pub async fn list_ceph_osd(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/ceph/osd").await
}

pub async fn get_ceph_osd(client: &PveClient, id: u32) -> PveResult<serde_json::Value> {
    client.get(&format!("/cluster/ceph/osd/{}", id)).await
}

pub async fn ceph_osd_in(client: &PveClient, id: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/cluster/ceph/osd/{}/in", id), None).await
}

pub async fn ceph_osd_out(client: &PveClient, id: u32) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/cluster/ceph/osd/{}/out", id), None).await
}

pub async fn delete_ceph_osd(client: &PveClient, id: u32) -> PveResult<serde_json::Value> {
    client.delete(&format!("/cluster/ceph/osd/{}", id)).await
}

// Ceph MON
pub async fn list_ceph_mon(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/ceph/mon").await
}

pub async fn get_ceph_mon(client: &PveClient, name: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/cluster/ceph/mon/{}", urlenc(name))).await
}

// Ceph MGR
pub async fn list_ceph_mgr(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/ceph/mgr").await
}

pub async fn get_ceph_mgr(client: &PveClient, name: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/cluster/ceph/mgr/{}", urlenc(name))).await
}

// Ceph Crush
pub async fn get_ceph_crush(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/ceph/crush").await
}

// Ceph Config
pub async fn get_ceph_config(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/cluster/ceph/config").await
}

pub async fn update_ceph_config(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put("/cluster/ceph/config", Some(params)).await
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