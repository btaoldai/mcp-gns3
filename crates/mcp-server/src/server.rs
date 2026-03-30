//! MCP server definition with tool routing.
//!
//! [`Gns3Server`] holds an `Arc<dyn Gns3Api>` and exposes all GNS3
//! operations as MCP tools via the `#[tool_router]` macro.
//! Each `#[tool]` method is a thin wrapper delegating to the
//! corresponding handler in [`crate::tools`].

use std::sync::Arc;

use gns3_mcp_core::Gns3Api;
use rmcp::handler::server::{router::tool::ToolRouter, wrapper::Parameters};
use rmcp::model::{CallToolResult, ErrorData, Implementation, ServerCapabilities, ServerInfo};
use rmcp::{tool, tool_handler, tool_router, ServerHandler};

use crate::tools;

/// MCP server exposing GNS3 operations as tools.
#[derive(Clone)]
pub struct Gns3Server {
    /// Injected GNS3 API implementation.
    api: Arc<dyn Gns3Api>,
    tool_router: ToolRouter<Self>,
}

impl Gns3Server {
    /// Create a new server with the given GNS3 API implementation.
    pub fn new(api: Arc<dyn Gns3Api>) -> Self {
        let tool_router = Self::tool_router();
        Self { api, tool_router }
    }
}

#[tool_router]
impl Gns3Server {
    // ── Project tools ──────────────────────────────────────────

    /// Check GNS3 server connectivity and version.
    #[tool(description = "Check GNS3 server connectivity and retrieve version info")]
    async fn gns3_get_version(&self) -> Result<CallToolResult, ErrorData> {
        tools::projects::get_version(&*self.api).await
    }

    /// List all GNS3 projects.
    #[tool(description = "List all GNS3 projects with their status")]
    async fn gns3_list_projects(&self) -> Result<CallToolResult, ErrorData> {
        tools::projects::list_projects(&*self.api).await
    }

    /// Create a new GNS3 project.
    #[tool(description = "Create a new GNS3 project")]
    async fn gns3_create_project(
        &self,
        Parameters(params): Parameters<tools::projects::CreateProjectParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::projects::create_project(&*self.api, params).await
    }

    /// Open an existing GNS3 project (required before adding nodes).
    #[tool(description = "Open a GNS3 project (required before creating nodes or links)")]
    async fn gns3_open_project(
        &self,
        Parameters(params): Parameters<tools::projects::OpenProjectParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::projects::open_project(&*self.api, params).await
    }

    // ── Template tools ─────────────────────────────────────────

    /// List available GNS3 node templates.
    #[tool(description = "List available GNS3 node templates (routers, switches, etc.)")]
    async fn gns3_list_templates(&self) -> Result<CallToolResult, ErrorData> {
        tools::templates::list_templates(&*self.api).await
    }

    // ── Node tools ─────────────────────────────────────────────

    /// Create a node from a template in a project.
    #[tool(description = "Create a new node from a template in a GNS3 project")]
    async fn gns3_create_node(
        &self,
        Parameters(params): Parameters<tools::nodes::CreateNodeParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::nodes::create_node(&*self.api, params).await
    }

    /// Start a node in a project.
    #[tool(description = "Start a node in a GNS3 project")]
    async fn gns3_start_node(
        &self,
        Parameters(params): Parameters<tools::nodes::StartNodeParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::nodes::start_node(&*self.api, params).await
    }

    // ── Link tools ─────────────────────────────────────────────

    /// Create a link between two node interfaces.
    #[tool(description = "Create a link between two node interfaces in a GNS3 project")]
    async fn gns3_create_link(
        &self,
        Parameters(params): Parameters<tools::links::CreateLinkParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::links::create_link(&*self.api, params).await
    }

    // ── P2: Node tools ────────────────────────────────────────

    /// List all nodes in a project with their status.
    #[tool(description = "List all nodes in a GNS3 project with their status and console ports")]
    async fn gns3_list_nodes(
        &self,
        Parameters(params): Parameters<tools::nodes::ListNodesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::nodes::list_nodes(&*self.api, params).await
    }

    /// Stop a running node.
    #[tool(description = "Stop a running node in a GNS3 project")]
    async fn gns3_stop_node(
        &self,
        Parameters(params): Parameters<tools::nodes::StopNodeParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::nodes::stop_node(&*self.api, params).await
    }

    /// Delete a node from a project.
    #[tool(description = "Delete a node from a GNS3 project (stops it first if running)")]
    async fn gns3_delete_node(
        &self,
        Parameters(params): Parameters<tools::nodes::DeleteNodeParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::nodes::delete_node(&*self.api, params).await
    }

    // ── P2: Link tools ────────────────────────────────────────

    /// List all links in a project.
    #[tool(description = "List all links in a GNS3 project showing connected endpoints")]
    async fn gns3_list_links(
        &self,
        Parameters(params): Parameters<tools::links::ListLinksParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::links::list_links(&*self.api, params).await
    }

    /// Delete a link from a project.
    #[tool(description = "Delete a link between two nodes in a GNS3 project")]
    async fn gns3_delete_link(
        &self,
        Parameters(params): Parameters<tools::links::DeleteLinkParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::links::delete_link(&*self.api, params).await
    }

    // ── P2: Project tools ─────────────────────────────────────

    /// Close a project.
    #[tool(description = "Close a GNS3 project (stops all nodes and releases resources)")]
    async fn gns3_close_project(
        &self,
        Parameters(params): Parameters<tools::projects::CloseProjectParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::projects::close_project(&*self.api, params).await
    }

    /// Delete a project.
    #[tool(description = "Delete a GNS3 project permanently (cannot be undone)")]
    async fn gns3_delete_project(
        &self,
        Parameters(params): Parameters<tools::projects::DeleteProjectParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::projects::delete_project(&*self.api, params).await
    }

    // ── P2: Composite tools ───────────────────────────────────

    /// Get full topology (nodes + links) for a project.
    #[tool(
        description = "Get the full topology of a GNS3 project: all nodes and links in one response"
    )]
    async fn gns3_get_topology(
        &self,
        Parameters(params): Parameters<tools::projects::GetTopologyParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::projects::get_topology(&*self.api, params).await
    }

    // ── Compute tools ─────────────────────────────────────────

    /// List all compute servers registered with the GNS3 controller.
    #[tool(
        description = "List all GNS3 compute servers with their connectivity status and CPU/memory usage"
    )]
    async fn gns3_list_computes(&self) -> Result<CallToolResult, ErrorData> {
        tools::computes::list_computes(&*self.api).await
    }

    // ── Bulk node tools ───────────────────────────────────────

    /// Start all nodes in a project simultaneously.
    #[tool(
        description = "Start all nodes in a GNS3 project at once (equivalent to the Play button in the UI)"
    )]
    async fn gns3_start_all_nodes(
        &self,
        Parameters(params): Parameters<tools::nodes::StartAllNodesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::nodes::start_all_nodes(&*self.api, params).await
    }

    /// Stop all nodes in a project simultaneously.
    #[tool(
        description = "Stop all nodes in a GNS3 project at once (equivalent to the Stop button in the UI)"
    )]
    async fn gns3_stop_all_nodes(
        &self,
        Parameters(params): Parameters<tools::nodes::StopAllNodesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::nodes::stop_all_nodes(&*self.api, params).await
    }
}

#[tool_handler]
impl ServerHandler for Gns3Server {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("gns3-mcp", env!("CARGO_PKG_VERSION")))
            .with_instructions(
                "GNS3 network lab management server. \
                 Create projects, add nodes from templates, connect them with links, \
                 and start/stop nodes. Typical workflow: \
                 1) gns3_get_version (check connectivity) \
                 2) gns3_create_project or gns3_list_projects \
                 3) gns3_open_project \
                 4) gns3_list_templates \
                 5) gns3_create_node (repeat for each device) \
                 6) gns3_create_link (connect interfaces) \
                 7) gns3_start_node (boot devices)",
            )
    }
}
