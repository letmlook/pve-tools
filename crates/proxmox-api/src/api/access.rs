//! Access/User/ACL API endpoints

use crate::client::PveClient;
use crate::error::PveResult;

pub async fn get_permissions(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/access/permissions").await
}

pub async fn list_users(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/access/users").await
}

pub async fn create_user(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form("/access/users", Some(params)).await
}

pub async fn update_user(client: &PveClient, userid: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put(&format!("/access/users/{}", urlenc(userid)), Some(params)).await
}

pub async fn delete_user(client: &PveClient, userid: &str) -> PveResult<serde_json::Value> {
    client.delete(&format!("/access/users/{}", urlenc(userid))).await
}

pub async fn list_user_tokens(client: &PveClient, userid: &str) -> PveResult<serde_json::Value> {
    client.get(&format!("/access/users/{}/token", urlenc(userid))).await
}

pub async fn create_token(client: &PveClient, userid: &str, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.post_form(&format!("/access/users/{}/token", urlenc(userid)), Some(params)).await
}

pub async fn list_groups(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/access/groups").await
}

pub async fn list_roles(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/access/roles").await
}

pub async fn get_acl(client: &PveClient) -> PveResult<serde_json::Value> {
    client.get("/access/acl").await
}

pub async fn set_acl(client: &PveClient, params: &[(String, String)]) -> PveResult<serde_json::Value> {
    client.put("/access/acl", Some(params)).await
}

pub async fn delete_acl(client: &PveClient, path: &str, userid: Option<&str>, groupid: Option<&str>) -> PveResult<serde_json::Value> {
    let mut params = vec![("path".to_string(), path.to_string())];
    if let Some(u) = userid {
        params.push(("userid".to_string(), u.to_string()));
    }
    if let Some(g) = groupid {
        params.push(("groupid".to_string(), g.to_string()));
    }
    let mut p = params.clone();
    p.push(("delete".to_string(), "1".to_string()));
    client.put("/access/acl", Some(&p)).await
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