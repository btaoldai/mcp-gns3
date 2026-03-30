//! Tool handlers for GNS3 compute server operations.

use std::fmt::Write;

use gns3_mcp_core::Gns3Api;
use rmcp::model::{CallToolResult, Content, ErrorData};

use super::to_mcp_error;

/// Handler for `gns3_list_computes`.
///
/// Returns a summary table of all compute servers registered with the
/// GNS3 controller, including connectivity status and resource usage.
pub async fn list_computes(api: &dyn Gns3Api) -> Result<CallToolResult, ErrorData> {
    let computes = api.list_computes().await.map_err(to_mcp_error)?;

    if computes.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No compute servers found. The local compute should always be present — \
             check GNS3 controller connectivity.",
        )]));
    }

    let mut text = format!("Found {} compute server(s):\n\n", computes.len());
    text.push_str("| ID | Name | Connected | Host | Port | Protocol | CPU% | Mem% |\n");
    text.push_str("|---|---|---|---|---|---|---|---|\n");

    for c in &computes {
        let cpu = c
            .cpu_usage_percent
            .map_or_else(|| "-".to_string(), |v| format!("{v:.1}"));
        let mem = c
            .memory_usage_percent
            .map_or_else(|| "-".to_string(), |v| format!("{v:.1}"));
        let connected = if c.connected { "yes" } else { "no" };
        let _ = writeln!(
            text,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            c.compute_id, c.name, connected, c.host, c.port, c.protocol, cpu, mem
        );
    }

    Ok(CallToolResult::success(vec![Content::text(text)]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::mock::{text_content, MockGns3Api, MockGns3ApiError};

    #[tokio::test]
    async fn list_computes_returns_table() {
        let api = MockGns3Api;
        let result = list_computes(&api).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("1 compute server(s)"));
        assert!(text.contains("local"));
        assert!(text.contains("localhost"));
        assert!(text.contains("3080"));
        assert!(text.contains("http"));
        assert!(text.contains("yes"));
    }

    #[tokio::test]
    async fn list_computes_propagates_api_error() {
        let api = MockGns3ApiError;
        let err = list_computes(&api).await.unwrap_err();
        assert!(err.message.contains("connection refused"));
    }
}
