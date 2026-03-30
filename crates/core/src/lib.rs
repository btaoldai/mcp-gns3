//! Core types, traits, and error definitions for gns3-mcp.
//!
//! This crate contains shared domain types used across the workspace.
//! It has zero network dependencies — only `serde`, `thiserror`, and `uuid`.

pub mod error;
pub mod traits;
pub mod types;

pub use error::Gns3Error;
pub use traits::Gns3Api;
pub use types::*;
