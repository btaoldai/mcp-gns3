//! gns3-mcp: MCP server exposing GNS3 REST API v2 to Claude.
//!
//! This binary bootstraps the MCP server over stdio transport.
//! Dependency injection: `Gns3Client` is created here and passed
//! to `Gns3Server` as `Arc<dyn Gns3Api>`.

use std::sync::Arc;

use gns3_client::{Gns3Client, Gns3ClientConfig};
use gns3_mcp_core::Gns3Api;
use rmcp::transport::io::stdio;
use rmcp::ServiceExt;
use tracing_subscriber::EnvFilter;

mod server;
mod tools;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tracing to stderr — stdout is reserved for MCP JSON-RPC protocol
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("gns3-mcp starting");

    let config = Gns3ClientConfig::from_env()?;
    tracing::info!(url = %config.base_url, "connecting to GNS3 server");

    let client = Gns3Client::new(config)?;
    let api: Arc<dyn Gns3Api> = Arc::new(client);

    let server = server::Gns3Server::new(api);
    let service = server.serve(stdio()).await?;

    tracing::info!("gns3-mcp ready — waiting for requests");
    service.waiting().await?;

    tracing::info!("gns3-mcp shutting down");
    Ok(())
}
