//! proxmox-api client module

pub mod auth;
pub mod config;
pub mod http;

pub use auth::{AuthContext, TokenAuth, AuthMethod};
pub use config::ClientConfig;
pub use http::PveClient;

pub use crate::error::{PveError, PveResult};