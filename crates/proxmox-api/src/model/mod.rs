//! Data models for PVE API responses

pub mod vm;
pub mod storage;
pub mod cluster;
pub mod node;
pub mod firewall;
pub mod ha;

pub use vm::*;
pub use storage::*;
pub use cluster::*;
pub use node::*;
pub use firewall::*;
pub use ha::*;