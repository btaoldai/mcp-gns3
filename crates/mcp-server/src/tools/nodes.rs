//! Tool handlers for GNS3 node operations.

use std::fmt::Write;

use gns3_mcp_core::{CreateNodeRequest, Gns3Api};
use rmcp::model::{CallToolResult, Content, ErrorData};
use serde::Deserialize;

use super::{parse_uuid, to_mcp_error};

/// Parameters for `gns3_create_node`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateNodeParams {
    /// UUID of the project (must be opened first).
    #[schemars(description = "Project UUID (must be opened first via gns3_open_project)")]
    pub project_id: String,
    /// UUID of the template to instantiate.
    #[schemars(description = "Template UUID (use gns3_list_templates to find it)")]
    pub template_id: String,
    /// X coordinate on the canvas.
    #[schemars(description = "X coordinate on the GNS3 canvas")]
    pub x: i32,
    /// Y coordinate on the canvas.
    #[schemars(description = "Y coordinate on the GNS3 canvas")]
    pub y: i32,
    /// Optional node name (uses template default if omitted).
    #[schemars(description = "Optional node name (uses template default if omitted)")]
    pub name: Option<String>,
    /// Compute node to run on (defaults to "local").
    #[schemars(description = "Compute ID (defaults to \"local\")")]
    pub compute_id: Option<String>,
}

/// Parameters for `gns3_start_node`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StartNodeParams {
    /// UUID of the project.
    #[schemars(description = "Project UUID")]
    pub project_id: String,
    /// UUID of the node to start.
    #[schemars(description = "Node UUID (use gns3_list_nodes to find it)")]
    pub node_id: String,
}

/// Handler for `gns3_create_node`.
pub async fn create_node(
    api: &dyn Gns3Api,
    params: CreateNodeParams,
) -> Result<CallToolResult, ErrorData> {
    let project_id = parse_uuid(&params.project_id, "project")?;
    let template_id = parse_uuid(&params.template_id, "template")?;

    let request = CreateNodeRequest {
        x: params.x,
        y: params.y,
        name: params.name,
        compute_id: Some(params.compute_id.unwrap_or_else(|| "local".to_string())),
    };

    let node = api
        .create_node(project_id, template_id, request)
        .await
        .map_err(to_mcp_error)?;

    let text = format!(
        "Node created: \"{}\" (id: {})\nType: {} | Status: {} | Project: {}",
        node.name, node.node_id, node.node_type, node.status, node.project_id
    );
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// Handler for `gns3_start_node`.
pub async fn start_node(
    api: &dyn Gns3Api,
    params: StartNodeParams,
) -> Result<CallToolResult, ErrorData> {
    let project_id = parse_uuid(&params.project_id, "project")?;
    let node_id = parse_uuid(&params.node_id, "node")?;

    let node = api
        .start_node(project_id, node_id)
        .await
        .map_err(to_mcp_error)?;

    let text = format!(
        "Node \"{}\" started successfully.\nStatus: {} | Type: {} | ID: {}",
        node.name, node.status, node.node_type, node.node_id
    );
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// Parameters for `gns3_list_nodes`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListNodesParams {
    /// UUID of the project.
    #[schemars(description = "Project UUID")]
    pub project_id: String,
}

/// Parameters for `gns3_stop_node`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StopNodeParams {
    /// UUID of the project.
    #[schemars(description = "Project UUID")]
    pub project_id: String,
    /// UUID of the node to stop.
    #[schemars(description = "Node UUID")]
    pub node_id: String,
}

/// Parameters for `gns3_delete_node`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteNodeParams {
    /// UUID of the project.
    #[schemars(description = "Project UUID")]
    pub project_id: String,
    /// UUID of the node to delete.
    #[schemars(description = "Node UUID")]
    pub node_id: String,
}

/// Handler for `gns3_list_nodes`.
pub async fn list_nodes(
    api: &dyn Gns3Api,
    params: ListNodesParams,
) -> Result<CallToolResult, ErrorData> {
    let project_id = parse_uuid(&params.project_id, "project")?;
    let nodes = api.list_nodes(project_id).await.map_err(to_mcp_error)?;

    if nodes.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No nodes in this project. Create one with gns3_create_node.",
        )]));
    }

    let mut text = format!("Found {} node(s):\n\n", nodes.len());
    text.push_str("| Name | Type | Status | Console | ID |\n|---|---|---|---|---|\n");
    for n in &nodes {
        let console = n.console.map_or_else(|| "-".to_string(), |p| p.to_string());
        let _ = writeln!(
            text,
            "| {} | {} | {} | {} | {} |",
            n.name, n.node_type, n.status, console, n.node_id
        );
    }
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// Handler for `gns3_stop_node`.
pub async fn stop_node(
    api: &dyn Gns3Api,
    params: StopNodeParams,
) -> Result<CallToolResult, ErrorData> {
    let project_id = parse_uuid(&params.project_id, "project")?;
    let node_id = parse_uuid(&params.node_id, "node")?;

    let node = api
        .stop_node(project_id, node_id)
        .await
        .map_err(to_mcp_error)?;

    let text = format!(
        "Node \"{}\" stopped.\nStatus: {} | Type: {} | ID: {}",
        node.name, node.status, node.node_type, node.node_id
    );
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// Handler for `gns3_delete_node`.
pub async fn delete_node(
    api: &dyn Gns3Api,
    params: DeleteNodeParams,
) -> Result<CallToolResult, ErrorData> {
    let project_id = parse_uuid(&params.project_id, "project")?;
    let node_id = parse_uuid(&params.node_id, "node")?;

    api.delete_node(project_id, node_id)
        .await
        .map_err(to_mcp_error)?;

    let text = format!("Node {node_id} deleted from project {project_id}.");
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// Parameters for `gns3_start_all_nodes`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StartAllNodesParams {
    /// UUID of the project whose nodes should all be started.
    #[schemars(description = "Project UUID (must be opened first via gns3_open_project)")]
    pub project_id: String,
}

/// Parameters for `gns3_stop_all_nodes`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StopAllNodesParams {
    /// UUID of the project whose nodes should all be stopped.
    #[schemars(description = "Project UUID")]
    pub project_id: String,
}

/// Handler for `gns3_start_all_nodes`.
///
/// Issues a bulk start on the project, then returns the resulting node list.
pub async fn start_all_nodes(
    api: &dyn Gns3Api,
    params: StartAllNodesParams,
) -> Result<CallToolResult, ErrorData> {
    let project_id = parse_uuid(&params.project_id, "project")?;
    let nodes = api
        .start_all_nodes(project_id)
        .await
        .map_err(to_mcp_error)?;

    let total = nodes.len();
    let started = nodes
        .iter()
        .filter(|n| n.status == gns3_mcp_core::NodeStatus::Started)
        .count();

    let mut text = format!(
        "Start-all issued for project {project_id}.\n\
         {started}/{total} node(s) running after the operation.\n\n"
    );
    if !nodes.is_empty() {
        text.push_str("| Name | Type | Status | ID |\n|---|---|---|---|\n");
        for n in &nodes {
            let _ = writeln!(
                text,
                "| {} | {} | {} | {} |",
                n.name, n.node_type, n.status, n.node_id
            );
        }
    }
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// Handler for `gns3_stop_all_nodes`.
///
/// Issues a bulk stop on the project, then returns the resulting node list.
pub async fn stop_all_nodes(
    api: &dyn Gns3Api,
    params: StopAllNodesParams,
) -> Result<CallToolResult, ErrorData> {
    let project_id = parse_uuid(&params.project_id, "project")?;
    let nodes = api.stop_all_nodes(project_id).await.map_err(to_mcp_error)?;

    let total = nodes.len();
    let stopped = nodes
        .iter()
        .filter(|n| n.status == gns3_mcp_core::NodeStatus::Stopped)
        .count();

    let mut text = format!(
        "Stop-all issued for project {project_id}.\n\
         {stopped}/{total} node(s) stopped after the operation.\n\n"
    );
    if !nodes.is_empty() {
        text.push_str("| Name | Type | Status | ID |\n|---|---|---|---|\n");
        for n in &nodes {
            let _ = writeln!(
                text,
                "| {} | {} | {} | {} |",
                n.name, n.node_type, n.status, n.node_id
            );
        }
    }
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::mock::{
        text_content, MockGns3Api, MockGns3ApiError, NODE1_ID, NODE2_ID, PROJECT_ID, TEMPLATE_ID,
    };

    #[tokio::test]
    async fn create_node_with_name() {
        let api = MockGns3Api;
        let params = CreateNodeParams {
            project_id: PROJECT_ID.to_string(),
            template_id: TEMPLATE_ID.to_string(),
            x: 100,
            y: 200,
            name: Some("Router1".to_string()),
            compute_id: None,
        };
        let result = create_node(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("Router1"));
        assert!(text.contains("vpcs"));
    }

    #[tokio::test]
    async fn create_node_defaults_compute_id_to_local() {
        let api = MockGns3Api;
        let params = CreateNodeParams {
            project_id: PROJECT_ID.to_string(),
            template_id: TEMPLATE_ID.to_string(),
            x: 0,
            y: 0,
            name: None,
            compute_id: None,
        };
        // Should not error — compute_id defaults to "local"
        let result = create_node(&api, params).await.unwrap();
        assert!(!result.is_error.unwrap_or(false));
    }

    #[tokio::test]
    async fn start_node_returns_started() {
        let api = MockGns3Api;
        let params = StartNodeParams {
            project_id: PROJECT_ID.to_string(),
            node_id: NODE1_ID.to_string(),
        };
        let result = start_node(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("started"));
    }

    #[tokio::test]
    async fn list_nodes_returns_table() {
        let api = MockGns3Api;
        let params = ListNodesParams {
            project_id: PROJECT_ID.to_string(),
        };
        let result = list_nodes(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("2 node(s)"));
        assert!(text.contains("PC1"));
        assert!(text.contains("PC2"));
        assert!(text.contains("5000"));
    }

    #[tokio::test]
    async fn stop_node_returns_stopped() {
        let api = MockGns3Api;
        let params = StopNodeParams {
            project_id: PROJECT_ID.to_string(),
            node_id: NODE1_ID.to_string(),
        };
        let result = stop_node(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("stopped"));
    }

    #[tokio::test]
    async fn delete_node_returns_confirmation() {
        let api = MockGns3Api;
        let params = DeleteNodeParams {
            project_id: PROJECT_ID.to_string(),
            node_id: NODE1_ID.to_string(),
        };
        let result = delete_node(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("deleted"));
        assert!(text.contains(NODE1_ID));
    }

    #[tokio::test]
    async fn create_node_rejects_bad_project_uuid() {
        let api = MockGns3Api;
        let params = CreateNodeParams {
            project_id: "garbage".to_string(),
            template_id: TEMPLATE_ID.to_string(),
            x: 0,
            y: 0,
            name: None,
            compute_id: None,
        };
        let err = create_node(&api, params).await.unwrap_err();
        assert!(err.message.contains("Invalid project UUID"));
    }

    #[tokio::test]
    async fn create_node_propagates_api_error() {
        let api = MockGns3ApiError;
        let params = CreateNodeParams {
            project_id: PROJECT_ID.to_string(),
            template_id: TEMPLATE_ID.to_string(),
            x: 0,
            y: 0,
            name: None,
            compute_id: None,
        };
        let err = create_node(&api, params).await.unwrap_err();
        assert!(err.message.contains("connection refused"));
    }

    #[tokio::test]
    async fn start_node_propagates_api_error() {
        let api = MockGns3ApiError;
        let params = StartNodeParams {
            project_id: PROJECT_ID.to_string(),
            node_id: NODE1_ID.to_string(),
        };
        let err = start_node(&api, params).await.unwrap_err();
        assert!(err.message.contains("connection refused"));
    }

    #[tokio::test]
    async fn list_nodes_propagates_api_error() {
        let api = MockGns3ApiError;
        let params = ListNodesParams {
            project_id: PROJECT_ID.to_string(),
        };
        let err = list_nodes(&api, params).await.unwrap_err();
        assert!(err.message.contains("connection refused"));
    }

    #[tokio::test]
    async fn start_all_nodes_returns_summary() {
        let api = MockGns3Api;
        let params = StartAllNodesParams {
            project_id: PROJECT_ID.to_string(),
        };
        let result = start_all_nodes(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("Start-all issued"));
        assert!(text.contains("2/2 node(s) running"));
        assert!(text.contains("PC1"));
        assert!(text.contains("PC2"));
        assert!(text.contains(NODE1_ID));
        assert!(text.contains(NODE2_ID));
    }

    #[tokio::test]
    async fn stop_all_nodes_returns_summary() {
        let api = MockGns3Api;
        let params = StopAllNodesParams {
            project_id: PROJECT_ID.to_string(),
        };
        let result = stop_all_nodes(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("Stop-all issued"));
        assert!(text.contains("2/2 node(s) stopped"));
        assert!(text.contains("PC1"));
        assert!(text.contains("PC2"));
    }

    #[tokio::test]
    async fn start_all_nodes_rejects_bad_project_uuid() {
        let api = MockGns3Api;
        let params = StartAllNodesParams {
            project_id: "not-a-uuid".to_string(),
        };
        let err = start_all_nodes(&api, params).await.unwrap_err();
        assert!(err.message.contains("Invalid project UUID"));
    }

    #[tokio::test]
    async fn stop_all_nodes_rejects_bad_project_uuid() {
        let api = MockGns3Api;
        let params = StopAllNodesParams {
            project_id: "not-a-uuid".to_string(),
        };
        let err = stop_all_nodes(&api, params).await.unwrap_err();
        assert!(err.message.contains("Invalid project UUID"));
    }

    #[tokio::test]
    async fn start_all_nodes_propagates_api_error() {
        let api = MockGns3ApiError;
        let params = StartAllNodesParams {
            project_id: PROJECT_ID.to_string(),
        };
        let err = start_all_nodes(&api, params).await.unwrap_err();
        assert!(err.message.contains("connection refused"));
    }

    #[tokio::test]
    async fn stop_all_nodes_propagates_api_error() {
        let api = MockGns3ApiError;
        let params = StopAllNodesParams {
            project_id: PROJECT_ID.to_string(),
        };
        let err = stop_all_nodes(&api, params).await.unwrap_err();
        assert!(err.message.contains("connection refused"));
    }
}
