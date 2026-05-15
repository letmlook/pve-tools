//! Proxmox version API

use crate::client::PveClient;
use crate::error::PveResult;

/// PVE version info
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Version {
    pub version: String,
    pub repoid: String,
    pub release: String,
}

/// Get PVE server version
pub async fn get_version(client: &PveClient) -> PveResult<Version> {
    let resp = client.get("/version").await?;
    let version = resp.get("data")
        .ok_or_else(|| crate::error::PveError::NotFound("no data in version response".to_string()))?;
    Ok(serde_json::from_value(version.clone())?)
}