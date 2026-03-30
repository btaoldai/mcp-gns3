//! Tool handlers for GNS3 link operations.

use std::fmt::Write;

use gns3_mcp_core::{Gns3Api, LinkEndpoint};
use rmcp::model::{CallToolResult, Content, ErrorData};
use serde::Deserialize;

use super::{parse_uuid, to_mcp_error};

/// A single endpoint in a link creation request.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LinkEndpointParams {
    /// UUID of the node.
    #[schemars(description = "Node UUID")]
    pub node_id: String,
    /// Adapter (interface) number on the node.
    #[schemars(description = "Adapter number (interface index on the node)")]
    pub adapter_number: u32,
    /// Port number on the adapter.
    #[schemars(description = "Port number on the adapter")]
    pub port_number: u32,
}

/// Parameters for `gns3_create_link`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateLinkParams {
    /// UUID of the project.
    #[schemars(description = "Project UUID")]
    pub project_id: String,
    /// The two endpoints to connect (exactly 2 required).
    #[schemars(description = "Exactly 2 endpoints: [{node_id, adapter_number, port_number}, ...]")]
    pub nodes: Vec<LinkEndpointParams>,
}

/// Handler for `gns3_create_link`.
pub async fn create_link(
    api: &dyn Gns3Api,
    params: CreateLinkParams,
) -> Result<CallToolResult, ErrorData> {
    let project_id = parse_uuid(&params.project_id, "project")?;

    if params.nodes.len() != 2 {
        return Err(rmcp::model::ErrorData {
            code: rmcp::model::ErrorCode::INVALID_PARAMS,
            message: std::borrow::Cow::Owned(format!(
                "A link requires exactly 2 endpoints, got {}. Provide [{{}}, {{}}] with node_id, adapter_number, port_number.",
                params.nodes.len()
            )),
            data: None,
        });
    }

    let mut endpoints = Vec::with_capacity(2);
    for (i, ep) in params.nodes.iter().enumerate() {
        let node_id = parse_uuid(&ep.node_id, &format!("node (endpoint {i})"))?;
        endpoints.push(LinkEndpoint {
            node_id,
            adapter_number: ep.adapter_number,
            port_number: ep.port_number,
        });
    }

    let link = api
        .create_link(project_id, endpoints)
        .await
        .map_err(to_mcp_error)?;

    let ep_a = link.nodes.first().map_or_else(
        || "?".to_string(),
        |e| format!("{} port {}/{}", e.node_id, e.adapter_number, e.port_number),
    );
    let ep_b = link.nodes.get(1).map_or_else(
        || "?".to_string(),
        |e| format!("{} port {}/{}", e.node_id, e.adapter_number, e.port_number),
    );

    let text = format!(
        "Link created (id: {})\nConnects: {} <-> {}",
        link.link_id, ep_a, ep_b,
    );
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// Parameters for `gns3_list_links`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListLinksParams {
    /// UUID of the project.
    #[schemars(description = "Project UUID")]
    pub project_id: String,
}

/// Parameters for `gns3_delete_link`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteLinkParams {
    /// UUID of the project.
    #[schemars(description = "Project UUID")]
    pub project_id: String,
    /// UUID of the link to delete.
    #[schemars(description = "Link UUID (use gns3_list_links to find it)")]
    pub link_id: String,
}

/// Handler for `gns3_list_links`.
pub async fn list_links(
    api: &dyn Gns3Api,
    params: ListLinksParams,
) -> Result<CallToolResult, ErrorData> {
    let project_id = parse_uuid(&params.project_id, "project")?;
    let links = api.list_links(project_id).await.map_err(to_mcp_error)?;

    if links.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No links in this project. Create one with gns3_create_link.",
        )]));
    }

    let mut text = format!("Found {} link(s):\n\n", links.len());
    text.push_str("| Link ID | Endpoint A | Endpoint B |\n|---|---|---|\n");
    for l in &links {
        let a = l.nodes.first().map_or_else(
            || "?".to_string(),
            |e| format!("{} port {}/{}", e.node_id, e.adapter_number, e.port_number),
        );
        let b = l.nodes.get(1).map_or_else(
            || "?".to_string(),
            |e| format!("{} port {}/{}", e.node_id, e.adapter_number, e.port_number),
        );
        let _ = writeln!(text, "| {} | {} | {} |", l.link_id, a, b);
    }
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// Handler for `gns3_delete_link`.
pub async fn delete_link(
    api: &dyn Gns3Api,
    params: DeleteLinkParams,
) -> Result<CallToolResult, ErrorData> {
    let project_id = parse_uuid(&params.project_id, "project")?;
    let link_id = parse_uuid(&params.link_id, "link")?;

    api.delete_link(project_id, link_id)
        .await
        .map_err(to_mcp_error)?;

    let text = format!("Link {link_id} deleted from project {project_id}.");
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::mock::{
        text_content, MockGns3Api, MockGns3ApiError, LINK_ID, NODE1_ID, NODE2_ID, PROJECT_ID,
    };

    #[tokio::test]
    async fn create_link_returns_endpoints() {
        let api = MockGns3Api;
        let params = CreateLinkParams {
            project_id: PROJECT_ID.to_string(),
            nodes: vec![
                LinkEndpointParams {
                    node_id: NODE1_ID.to_string(),
                    adapter_number: 0,
                    port_number: 0,
                },
                LinkEndpointParams {
                    node_id: NODE2_ID.to_string(),
                    adapter_number: 0,
                    port_number: 0,
                },
            ],
        };
        let result = create_link(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains(LINK_ID));
        assert!(text.contains("port 0/0"));
    }

    #[tokio::test]
    async fn create_link_rejects_single_endpoint() {
        let api = MockGns3Api;
        let params = CreateLinkParams {
            project_id: PROJECT_ID.to_string(),
            nodes: vec![LinkEndpointParams {
                node_id: NODE1_ID.to_string(),
                adapter_number: 0,
                port_number: 0,
            }],
        };
        let err = create_link(&api, params).await.unwrap_err();
        assert!(err.message.contains("exactly 2 endpoints"));
    }

    #[tokio::test]
    async fn list_links_returns_table() {
        let api = MockGns3Api;
        let params = ListLinksParams {
            project_id: PROJECT_ID.to_string(),
        };
        let result = list_links(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("1 link(s)"));
        assert!(text.contains(LINK_ID));
    }

    #[tokio::test]
    async fn delete_link_returns_confirmation() {
        let api = MockGns3Api;
        let params = DeleteLinkParams {
            project_id: PROJECT_ID.to_string(),
            link_id: LINK_ID.to_string(),
        };
        let result = delete_link(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("deleted"));
        assert!(text.contains(LINK_ID));
    }

    #[tokio::test]
    async fn create_link_propagates_api_error() {
        let api = MockGns3ApiError;
        let params = CreateLinkParams {
            project_id: PROJECT_ID.to_string(),
            nodes: vec![
                LinkEndpointParams {
                    node_id: NODE1_ID.to_string(),
                    adapter_number: 0,
                    port_number: 0,
                },
                LinkEndpointParams {
                    node_id: NODE2_ID.to_string(),
                    adapter_number: 0,
                    port_number: 0,
                },
            ],
        };
        let err = create_link(&api, params).await.unwrap_err();
        assert!(err.message.contains("connection refused"));
    }

    #[tokio::test]
    async fn list_links_propagates_api_error() {
        let api = MockGns3ApiError;
        let params = ListLinksParams {
            project_id: PROJECT_ID.to_string(),
        };
        let err = list_links(&api, params).await.unwrap_err();
        assert!(err.message.contains("connection refused"));
    }
}
