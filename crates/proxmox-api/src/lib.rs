//! proxmox-api core library
//!
//! Shared HTTP client, auth, error types, and API models for PVE tools.

pub mod client;
pub mod api;
pub mod model;
pub mod error;

pub use client::PveClient;
pub use client::ClientConfig;
pub use client::config::ConfigFile;
pub use error::{PveError, PveResult};
