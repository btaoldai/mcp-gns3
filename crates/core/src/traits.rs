//! Async trait defining all GNS3 API operations.
//!
//! Implemented by [`gns3_client::Gns3Client`] in the `gns3-client` crate.
//! Can be mocked for unit testing in `mcp-server`.

use uuid::Uuid;

use crate::error::Gns3Error;
use crate::types::{
    Compute, CreateNodeRequest, Link, LinkEndpoint, Node, Project, Template, Version,
};

/// Async interface for GNS3 REST API v2 operations.
///
/// Each method maps to one GNS3 endpoint. Implementations handle
/// HTTP transport, authentication, and response deserialization.
#[async_trait::async_trait]
pub trait Gns3Api: Send + Sync {
    /// Check server connectivity and retrieve version info.
    ///
    /// Maps to `GET /v2/version`.
    async fn get_version(&self) -> Result<Version, Gns3Error>;

    /// List all projects on the server.
    ///
    /// Maps to `GET /v2/projects`.
    async fn list_projects(&self) -> Result<Vec<Project>, Gns3Error>;

    /// Create a new project.
    ///
    /// Maps to `POST /v2/projects`.
    async fn create_project(&self, name: &str) -> Result<Project, Gns3Error>;

    /// Open an existing project (required before manipulating nodes).
    ///
    /// Maps to `POST /v2/projects/{id}/open`.
    async fn open_project(&self, project_id: Uuid) -> Result<Project, Gns3Error>;

    /// List available node templates.
    ///
    /// Maps to `GET /v2/templates`.
    async fn list_templates(&self) -> Result<Vec<Template>, Gns3Error>;

    /// Create a node from a template within a project.
    ///
    /// Maps to `POST /v2/projects/{project_id}/templates/{template_id}`.
    async fn create_node(
        &self,
        project_id: Uuid,
        template_id: Uuid,
        request: CreateNodeRequest,
    ) -> Result<Node, Gns3Error>;

    /// Start a node.
    ///
    /// Maps to `POST /v2/projects/{project_id}/nodes/{node_id}/start`.
    async fn start_node(&self, project_id: Uuid, node_id: Uuid) -> Result<Node, Gns3Error>;

    /// Create a link between two node interfaces.
    ///
    /// Maps to `POST /v2/projects/{project_id}/links`.
    async fn create_link(
        &self,
        project_id: Uuid,
        endpoints: Vec<LinkEndpoint>,
    ) -> Result<Link, Gns3Error>;

    // ── Priority 2 ────────────────────────────────────────────

    /// List all nodes in a project.
    ///
    /// Maps to `GET /v2/projects/{project_id}/nodes`.
    async fn list_nodes(&self, project_id: Uuid) -> Result<Vec<Node>, Gns3Error>;

    /// Stop a running node.
    ///
    /// Maps to `POST /v2/projects/{project_id}/nodes/{node_id}/stop`.
    async fn stop_node(&self, project_id: Uuid, node_id: Uuid) -> Result<Node, Gns3Error>;

    /// Delete a node from a project.
    ///
    /// Maps to `DELETE /v2/projects/{project_id}/nodes/{node_id}`.
    async fn delete_node(&self, project_id: Uuid, node_id: Uuid) -> Result<(), Gns3Error>;

    /// List all links in a project.
    ///
    /// Maps to `GET /v2/projects/{project_id}/links`.
    async fn list_links(&self, project_id: Uuid) -> Result<Vec<Link>, Gns3Error>;

    /// Delete a link from a project.
    ///
    /// Maps to `DELETE /v2/projects/{project_id}/links/{link_id}`.
    async fn delete_link(&self, project_id: Uuid, link_id: Uuid) -> Result<(), Gns3Error>;

    /// Close a project.
    ///
    /// Maps to `POST /v2/projects/{project_id}/close`.
    async fn close_project(&self, project_id: Uuid) -> Result<Project, Gns3Error>;

    /// Delete a project.
    ///
    /// Maps to `DELETE /v2/projects/{project_id}`.
    async fn delete_project(&self, project_id: Uuid) -> Result<(), Gns3Error>;

    // ── Compute / bulk-node operations ────────────────────────

    /// List all compute servers registered with the GNS3 controller.
    ///
    /// Maps to `GET /v2/computes`.
    async fn list_computes(&self) -> Result<Vec<Compute>, Gns3Error>;

    /// Start all nodes in a project simultaneously.
    ///
    /// Maps to `POST /v2/projects/{project_id}/nodes/start`.
    /// Returns the list of nodes with their updated status.
    async fn start_all_nodes(&self, project_id: Uuid) -> Result<Vec<Node>, Gns3Error>;

    /// Stop all nodes in a project simultaneously.
    ///
    /// Maps to `POST /v2/projects/{project_id}/nodes/stop`.
    /// Returns the list of nodes with their updated status.
    async fn stop_all_nodes(&self, project_id: Uuid) -> Result<Vec<Node>, Gns3Error>;
}
