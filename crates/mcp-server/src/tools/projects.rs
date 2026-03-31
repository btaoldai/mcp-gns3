//! Tool handlers for GNS3 project operations.

use std::fmt::Write;

use gns3_mcp_core::Gns3Api;
use rmcp::model::{CallToolResult, Content, ErrorData};
use serde::Deserialize;

use super::{parse_uuid, to_mcp_error};

/// Parameters for `gns3_create_project`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateProjectParams {
    /// Name for the new project.
    #[schemars(description = "Name for the new GNS3 project")]
    pub name: String,
}

/// Parameters for `gns3_open_project`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OpenProjectParams {
    /// UUID of the project to open.
    #[schemars(description = "Project UUID (use gns3_list_projects to find it)")]
    pub project_id: String,
}

/// Handler for `gns3_get_version`.
pub async fn get_version(api: &dyn Gns3Api) -> Result<CallToolResult, ErrorData> {
    let version = api.get_version().await.map_err(to_mcp_error)?;
    let text = format!(
        "GNS3 server is reachable.\nVersion: {}\nLocal: {}",
        version.version, version.local
    );
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// Handler for `gns3_list_projects`.
pub async fn list_projects(api: &dyn Gns3Api) -> Result<CallToolResult, ErrorData> {
    let projects = api.list_projects().await.map_err(to_mcp_error)?;

    if projects.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No projects found. Create one with gns3_create_project.",
        )]));
    }

    let mut text = format!("Found {} project(s):\n\n", projects.len());
    text.push_str("| Name | Status | ID |\n|---|---|---|\n");
    for p in &projects {
        let _ = writeln!(text, "| {} | {} | {} |", p.name, p.status, p.project_id);
    }
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// Handler for `gns3_create_project`.
pub async fn create_project(
    api: &dyn Gns3Api,
    params: CreateProjectParams,
) -> Result<CallToolResult, ErrorData> {
    let project = api
        .create_project(&params.name)
        .await
        .map_err(to_mcp_error)?;
    let text = format!(
        "Project created: \"{}\" (id: {})\nStatus: {}",
        project.name, project.project_id, project.status
    );
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// Handler for `gns3_open_project`.
pub async fn open_project(
    api: &dyn Gns3Api,
    params: OpenProjectParams,
) -> Result<CallToolResult, ErrorData> {
    let project_id = parse_uuid(&params.project_id, "project")?;
    let project = api.open_project(project_id).await.map_err(to_mcp_error)?;
    let text = format!(
        "Project \"{}\" opened successfully.\nStatus: {} | ID: {}",
        project.name, project.status, project.project_id
    );
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// Parameters for `gns3_close_project`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CloseProjectParams {
    /// UUID of the project to close.
    #[schemars(description = "Project UUID")]
    pub project_id: String,
}

/// Parameters for `gns3_delete_project`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteProjectParams {
    /// UUID of the project to delete.
    #[schemars(description = "Project UUID (use gns3_list_projects to find it)")]
    pub project_id: String,
}

/// Parameters for `gns3_get_topology`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetTopologyParams {
    /// UUID of the project.
    #[schemars(description = "Project UUID")]
    pub project_id: String,
}

/// Parameters for `gns3_export_project`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExportProjectParams {
    /// UUID of the project to export.
    #[schemars(description = "Project UUID")]
    pub project_id: String,
    /// Whether to include disk images in the export.
    #[schemars(description = "Include disk images in export (true/false, defaults to false)")]
    pub include_images: Option<bool>,
}

/// Parameters for `gns3_snapshot_project`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SnapshotProjectParams {
    /// UUID of the project.
    #[schemars(description = "Project UUID")]
    pub project_id: String,
    /// Name for the snapshot.
    #[schemars(description = "Snapshot name")]
    pub name: String,
}

/// Handler for `gns3_close_project`.
pub async fn close_project(
    api: &dyn Gns3Api,
    params: CloseProjectParams,
) -> Result<CallToolResult, ErrorData> {
    let project_id = parse_uuid(&params.project_id, "project")?;
    let project = api.close_project(project_id).await.map_err(to_mcp_error)?;
    let text = format!(
        "Project \"{}\" closed.\nStatus: {} | ID: {}",
        project.name, project.status, project.project_id
    );
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// Handler for `gns3_delete_project`.
pub async fn delete_project(
    api: &dyn Gns3Api,
    params: DeleteProjectParams,
) -> Result<CallToolResult, ErrorData> {
    let project_id = parse_uuid(&params.project_id, "project")?;
    api.delete_project(project_id).await.map_err(to_mcp_error)?;
    let text = format!("Project {project_id} deleted.");
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// Handler for `gns3_get_topology` — composite view of nodes + links.
pub async fn get_topology(
    api: &dyn Gns3Api,
    params: GetTopologyParams,
) -> Result<CallToolResult, ErrorData> {
    let project_id = parse_uuid(&params.project_id, "project")?;

    let (nodes, links) = tokio::try_join!(
        async { api.list_nodes(project_id).await.map_err(to_mcp_error) },
        async { api.list_links(project_id).await.map_err(to_mcp_error) },
    )?;

    let mut text = format!(
        "Topology: {} node(s), {} link(s)\n\n",
        nodes.len(),
        links.len()
    );

    // Nodes table
    text.push_str("## Nodes\n\n");
    if nodes.is_empty() {
        text.push_str("No nodes.\n\n");
    } else {
        text.push_str("| Name | Type | Status | Console | ID |\n|---|---|---|---|---|\n");
        for n in &nodes {
            let console = n.console.map_or_else(|| "-".to_string(), |p| p.to_string());
            let _ = writeln!(
                text,
                "| {} | {} | {} | {} | {} |",
                n.name, n.node_type, n.status, console, n.node_id
            );
        }
        text.push('\n');
    }

    // Links table
    text.push_str("## Links\n\n");
    if links.is_empty() {
        text.push_str("No links.\n");
    } else {
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
    }

    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// Handler for `gns3_export_project`.
pub async fn export_project(
    api: &dyn Gns3Api,
    params: ExportProjectParams,
) -> Result<CallToolResult, ErrorData> {
    let project_id = parse_uuid(&params.project_id, "project")?;
    let include_images = params.include_images.unwrap_or(false);

    let result = api
        .export_project(project_id, include_images)
        .await
        .map_err(to_mcp_error)?;

    let size_mb = result.size_bytes / (1024 * 1024);
    let size_kb = (result.size_bytes % (1024 * 1024)) / 1024;
    let text = format!(
        "Project exported successfully.\nProject ID: {}\nSize: {} MB {} KB\nFile: {}.gns3project",
        project_id, size_mb, size_kb, project_id
    );
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// Handler for `gns3_snapshot_project`.
pub async fn snapshot_project(
    api: &dyn Gns3Api,
    params: SnapshotProjectParams,
) -> Result<CallToolResult, ErrorData> {
    let project_id = parse_uuid(&params.project_id, "project")?;

    let snapshot = api
        .snapshot_project(project_id, &params.name)
        .await
        .map_err(to_mcp_error)?;

    let text = format!(
        "Snapshot created successfully.\nSnapshot ID: {}\nName: {}\nCreated: {}\nProject: {}",
        snapshot.snapshot_id, snapshot.name, snapshot.created_at, project_id
    );
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::mock::{text_content, MockGns3Api, MockGns3ApiError, PROJECT_ID};

    #[tokio::test]
    async fn get_version_returns_version_string() {
        let api = MockGns3Api;
        let result = get_version(&api).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("2.2.57"));
        assert!(text.contains("reachable"));
    }

    #[tokio::test]
    async fn list_projects_returns_table() {
        let api = MockGns3Api;
        let result = list_projects(&api).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("TestProject"));
        assert!(text.contains("opened"));
    }

    #[tokio::test]
    async fn create_project_returns_name_and_id() {
        let api = MockGns3Api;
        let params = CreateProjectParams {
            name: "MyLab".to_string(),
        };
        let result = create_project(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("MyLab"));
        assert!(text.contains(PROJECT_ID));
    }

    #[tokio::test]
    async fn open_project_returns_opened_status() {
        let api = MockGns3Api;
        let params = OpenProjectParams {
            project_id: PROJECT_ID.to_string(),
        };
        let result = open_project(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("opened"));
    }

    #[tokio::test]
    async fn open_project_rejects_invalid_uuid() {
        let api = MockGns3Api;
        let params = OpenProjectParams {
            project_id: "not-a-uuid".to_string(),
        };
        let err = open_project(&api, params).await.unwrap_err();
        assert!(err.message.contains("Invalid project UUID"));
    }

    #[tokio::test]
    async fn close_project_returns_closed_status() {
        let api = MockGns3Api;
        let params = CloseProjectParams {
            project_id: PROJECT_ID.to_string(),
        };
        let result = close_project(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("closed"));
    }

    #[tokio::test]
    async fn delete_project_returns_confirmation() {
        let api = MockGns3Api;
        let params = DeleteProjectParams {
            project_id: PROJECT_ID.to_string(),
        };
        let result = delete_project(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("deleted"));
        assert!(text.contains(PROJECT_ID));
    }

    #[tokio::test]
    async fn get_topology_returns_nodes_and_links() {
        let api = MockGns3Api;
        let params = GetTopologyParams {
            project_id: PROJECT_ID.to_string(),
        };
        let result = get_topology(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("2 node(s)"));
        assert!(text.contains("1 link(s)"));
        assert!(text.contains("PC1"));
        assert!(text.contains("PC2"));
        assert!(text.contains("## Nodes"));
        assert!(text.contains("## Links"));
    }

    #[tokio::test]
    async fn get_version_propagates_api_error() {
        let api = MockGns3ApiError;
        let err = get_version(&api).await.unwrap_err();
        assert!(err.message.contains("connection refused"));
    }

    #[tokio::test]
    async fn export_project_returns_size_info() {
        let api = MockGns3Api;
        let params = ExportProjectParams {
            project_id: PROJECT_ID.to_string(),
            include_images: Some(true),
        };
        let result = export_project(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("Project exported successfully"));
        assert!(text.contains(".gns3project"));
    }

    #[tokio::test]
    async fn export_project_defaults_include_images_false() {
        let api = MockGns3Api;
        let params = ExportProjectParams {
            project_id: PROJECT_ID.to_string(),
            include_images: None,
        };
        let result = export_project(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("exported successfully"));
    }

    #[tokio::test]
    async fn export_project_error() {
        let api = MockGns3ApiError;
        let params = ExportProjectParams {
            project_id: PROJECT_ID.to_string(),
            include_images: Some(false),
        };
        let err = export_project(&api, params).await.unwrap_err();
        assert!(err.message.contains("connection refused"));
    }

    #[tokio::test]
    async fn snapshot_project_returns_snapshot_info() {
        let api = MockGns3Api;
        let params = SnapshotProjectParams {
            project_id: PROJECT_ID.to_string(),
            name: "Backup-v1".to_string(),
        };
        let result = snapshot_project(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("Snapshot created successfully"));
        assert!(text.contains("Backup-v1"));
    }

    #[tokio::test]
    async fn snapshot_project_error() {
        let api = MockGns3ApiError;
        let params = SnapshotProjectParams {
            project_id: PROJECT_ID.to_string(),
            name: "ShouldFail".to_string(),
        };
        let err = snapshot_project(&api, params).await.unwrap_err();
        assert!(err.message.contains("connection refused"));
    }

    #[tokio::test]
    async fn create_project_propagates_error() {
        let api = MockGns3ApiError;
        let params = CreateProjectParams {
            name: "ShouldFail".to_string(),
        };
        let err = create_project(&api, params).await.unwrap_err();
        assert!(err.message.contains("connection refused"));
    }

    #[tokio::test]
    async fn get_topology_propagates_error() {
        let api = MockGns3ApiError;
        let params = GetTopologyParams {
            project_id: PROJECT_ID.to_string(),
        };
        let err = get_topology(&api, params).await.unwrap_err();
        assert!(err.message.contains("connection refused"));
    }
}
