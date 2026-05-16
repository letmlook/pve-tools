//! Firewall data model for PVE API responses

use serde::{Deserialize, Serialize};

/// A single firewall rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub pos: u32,
    #[serde(rename = "type")]
    pub rule_type: String,
    pub action: String,
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub dest: String,
    #[serde(default)]
    pub dport: String,
    #[serde(default)]
    pub sport: String,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub enable: u32,
    #[serde(default)]
    pub dest_ip: String,
    #[serde(default)]
    pub source_ip: String,
}

/// Firewall ruleset (list of rules)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRuleset {
    pub rules: Vec<FirewallRule>,
}

/// Firewall group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallGroup {
    pub name: String,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub rules: Vec<FirewallRule>,
}

/// Firewall options/settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallOptions {
    #[serde(default)]
    pub enable: u32,
    #[serde(default)]
    pub policy_in: String,
    #[serde(default)]
    pub policy_out: String,
    #[serde(default)]
    pub log_level_in: String,
    #[serde(default)]
    pub log_level_out: String,
    #[serde(default)]
    pub ebtables: bool,
}

/// Macro for parsing fw options from JSON
impl Default for FirewallOptions {
    fn default() -> Self {
        Self {
            enable: 0,
            policy_in: String::from("ACCEPT"),
            policy_out: String::from("ACCEPT"),
            log_level_in: String::from("info"),
            log_level_out: String::from("info"),
            ebtables: false,
        }
    }
}