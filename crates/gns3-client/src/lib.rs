//! HTTP client for the GNS3 REST API v2.
//!
//! This crate provides [`Gns3Client`], the concrete implementation of
//! [`gns3_mcp_core::Gns3Api`] that communicates with a GNS3 server
//! over HTTP using `reqwest` with `rustls`.

pub mod client;
pub mod config;

pub use client::Gns3Client;
pub use config::Gns3ClientConfig;
