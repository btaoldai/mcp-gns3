//! MCP tool implementations organized by GNS3 domain.
//!
//! Each module contains parameter structs and handler functions
//! for a specific resource type. The actual `#[tool]` routing
//! lives in [`crate::server::Gns3Server`].

pub mod computes;
pub mod drawings;
pub mod links;
pub mod nodes;
pub mod projects;
pub mod templates;

use std::borrow::Cow;

use gns3_mcp_core::Gns3Error;
use rmcp::model::{ErrorCode, ErrorData};

/// Convert a [`Gns3Error`] into an MCP [`ErrorData`] with an actionable message.
#[allow(clippy::needless_pass_by_value)] // Required for `.map_err(to_mcp_error)` ergonomics
pub fn to_mcp_error(err: Gns3Error) -> ErrorData {
    let code = match &err {
        Gns3Error::InvalidUuid(_) | Gns3Error::Config(_) => ErrorCode::INVALID_PARAMS,
        Gns3Error::CircuitOpen => ErrorCode::INTERNAL_ERROR,
        _ => ErrorCode::INTERNAL_ERROR,
    };
    ErrorData {
        code,
        message: Cow::Owned(err.to_string()),
        data: None,
    }
}

/// Parse a UUID string, returning an MCP error with an actionable message on failure.
pub fn parse_uuid(value: &str, label: &str) -> Result<uuid::Uuid, ErrorData> {
    value.parse::<uuid::Uuid>().map_err(|e| ErrorData {
        code: ErrorCode::INVALID_PARAMS,
        message: Cow::Owned(format!(
            "Invalid {label} UUID \"{value}\" — {e}. Use the corresponding list tool to get valid IDs."
        )),
        data: None,
    })
}

/// Mock implementation of [`Gns3Api`] for unit tests.
#[cfg(test)]
pub mod mock {
    use gns3_mcp_core::*;
    use uuid::Uuid;

    /// A mock that returns canned responses for each operation.
    pub struct MockGns3Api;

    /// Fixed UUIDs used across all tests.
    pub const PROJECT_ID: &str = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
    pub const NODE1_ID: &str = "11111111-2222-3333-4444-555555555555";
    pub const NODE2_ID: &str = "66666666-7777-8888-9999-aaaaaaaaaaaa";
    pub const TEMPLATE_ID: &str = "cccccccc-dddd-eeee-ffff-000000000000";
    pub const LINK_ID: &str = "bbbbbbbb-cccc-dddd-eeee-ffffffffffff";
    pub const DRAWING_ID: &str = "dddddddd-eeee-ffff-0000-111111111111";
    pub const SNAPSHOT_ID: &str = "eeeeeeee-ffff-0000-1111-222222222222";

    fn pid() -> Uuid {
        PROJECT_ID.parse().unwrap()
    }
    fn nid1() -> Uuid {
        NODE1_ID.parse().unwrap()
    }
    fn nid2() -> Uuid {
        NODE2_ID.parse().unwrap()
    }
    fn tid() -> Uuid {
        TEMPLATE_ID.parse().unwrap()
    }
    fn lid() -> Uuid {
        LINK_ID.parse().unwrap()
    }

    fn did() -> Uuid {
        DRAWING_ID.parse().unwrap()
    }

    fn sid() -> Uuid {
        SNAPSHOT_ID.parse().unwrap()
    }

    fn sample_project() -> Project {
        Project {
            project_id: pid(),
            name: "TestProject".to_string(),
            status: ProjectStatus::Opened,
        }
    }

    fn sample_node(id: Uuid, name: &str, status: NodeStatus) -> Node {
        Node {
            node_id: id,
            name: name.to_string(),
            status,
            node_type: "vpcs".to_string(),
            project_id: pid(),
            console: Some(5000),
            console_type: Some("telnet".to_string()),
        }
    }

    fn sample_link() -> Link {
        Link {
            link_id: lid(),
            project_id: pid(),
            nodes: vec![
                LinkEndpoint {
                    node_id: nid1(),
                    adapter_number: 0,
                    port_number: 0,
                },
                LinkEndpoint {
                    node_id: nid2(),
                    adapter_number: 0,
                    port_number: 0,
                },
            ],
        }
    }

    #[async_trait::async_trait]
    impl Gns3Api for MockGns3Api {
        async fn get_version(&self) -> Result<Version, Gns3Error> {
            Ok(Version {
                version: "2.2.57".to_string(),
                local: true,
            })
        }

        async fn list_projects(&self) -> Result<Vec<Project>, Gns3Error> {
            Ok(vec![sample_project()])
        }

        async fn create_project(&self, name: &str) -> Result<Project, Gns3Error> {
            Ok(Project {
                project_id: pid(),
                name: name.to_string(),
                status: ProjectStatus::Opened,
            })
        }

        async fn open_project(&self, _project_id: Uuid) -> Result<Project, Gns3Error> {
            Ok(sample_project())
        }

        async fn list_templates(&self) -> Result<Vec<Template>, Gns3Error> {
            Ok(vec![Template {
                template_id: tid(),
                name: "VPCS".to_string(),
                template_type: "vpcs".to_string(),
                category: "guest".to_string(),
                builtin: true,
            }])
        }

        async fn create_node(
            &self,
            _project_id: Uuid,
            _template_id: Uuid,
            request: CreateNodeRequest,
        ) -> Result<Node, Gns3Error> {
            Ok(sample_node(
                nid1(),
                request.name.as_deref().unwrap_or("PC1"),
                NodeStatus::Stopped,
            ))
        }

        async fn start_node(&self, _project_id: Uuid, _node_id: Uuid) -> Result<Node, Gns3Error> {
            Ok(sample_node(nid1(), "PC1", NodeStatus::Started))
        }

        async fn create_link(
            &self,
            _project_id: Uuid,
            _endpoints: Vec<LinkEndpoint>,
        ) -> Result<Link, Gns3Error> {
            Ok(sample_link())
        }

        async fn list_nodes(&self, _project_id: Uuid) -> Result<Vec<Node>, Gns3Error> {
            Ok(vec![
                sample_node(nid1(), "PC1", NodeStatus::Stopped),
                sample_node(nid2(), "PC2", NodeStatus::Started),
            ])
        }

        async fn stop_node(&self, _project_id: Uuid, _node_id: Uuid) -> Result<Node, Gns3Error> {
            Ok(sample_node(nid1(), "PC1", NodeStatus::Stopped))
        }

        async fn delete_node(&self, _project_id: Uuid, _node_id: Uuid) -> Result<(), Gns3Error> {
            Ok(())
        }

        async fn list_links(&self, _project_id: Uuid) -> Result<Vec<Link>, Gns3Error> {
            Ok(vec![sample_link()])
        }

        async fn delete_link(&self, _project_id: Uuid, _link_id: Uuid) -> Result<(), Gns3Error> {
            Ok(())
        }

        async fn close_project(&self, _project_id: Uuid) -> Result<Project, Gns3Error> {
            Ok(Project {
                project_id: pid(),
                name: "TestProject".to_string(),
                status: ProjectStatus::Closed,
            })
        }

        async fn delete_project(&self, _project_id: Uuid) -> Result<(), Gns3Error> {
            Ok(())
        }

        async fn list_computes(&self) -> Result<Vec<Compute>, Gns3Error> {
            Ok(vec![Compute {
                compute_id: "local".to_string(),
                name: "Local GNS3 compute".to_string(),
                connected: true,
                host: "localhost".to_string(),
                port: 3080,
                protocol: "http".to_string(),
                cpu_usage_percent: Some(12.5),
                memory_usage_percent: Some(34.0),
            }])
        }

        async fn start_all_nodes(&self, _project_id: Uuid) -> Result<Vec<Node>, Gns3Error> {
            Ok(vec![
                sample_node(nid1(), "PC1", NodeStatus::Started),
                sample_node(nid2(), "PC2", NodeStatus::Started),
            ])
        }

        async fn stop_all_nodes(&self, _project_id: Uuid) -> Result<Vec<Node>, Gns3Error> {
            Ok(vec![
                sample_node(nid1(), "PC1", NodeStatus::Stopped),
                sample_node(nid2(), "PC2", NodeStatus::Stopped),
            ])
        }

        async fn update_node(
            &self,
            _project_id: Uuid,
            _node_id: Uuid,
            update: gns3_mcp_core::UpdateNodeRequest,
        ) -> Result<Node, Gns3Error> {
            Ok(sample_node(
                nid1(),
                update.name.as_deref().unwrap_or("PC1"),
                NodeStatus::Stopped,
            ))
        }

        async fn update_template(
            &self,
            _template_id: Uuid,
            properties: serde_json::Value,
        ) -> Result<serde_json::Value, Gns3Error> {
            Ok(properties)
        }

        async fn add_drawing(
            &self,
            _project_id: Uuid,
            request: gns3_mcp_core::AddDrawingRequest,
        ) -> Result<gns3_mcp_core::Drawing, Gns3Error> {
            Ok(gns3_mcp_core::Drawing {
                drawing_id: did(),
                project_id: pid(),
                svg: request.svg,
                x: request.x,
                y: request.y,
                z: request.z.unwrap_or(0),
            })
        }

        async fn export_project(
            &self,
            _project_id: Uuid,
            _include_images: bool,
        ) -> Result<gns3_mcp_core::ExportResult, Gns3Error> {
            Ok(gns3_mcp_core::ExportResult {
                size_bytes: 1024 * 512, // 512 KB
            })
        }

        async fn configure_switch(
            &self,
            _project_id: Uuid,
            _node_id: Uuid,
            _ports: Vec<gns3_mcp_core::SwitchPort>,
        ) -> Result<Node, Gns3Error> {
            Ok(sample_node(nid1(), "EthernetSwitch1", NodeStatus::Stopped))
        }

        async fn snapshot_project(
            &self,
            _project_id: Uuid,
            name: &str,
        ) -> Result<gns3_mcp_core::Snapshot, Gns3Error> {
            Ok(gns3_mcp_core::Snapshot {
                snapshot_id: sid(),
                name: name.to_string(),
                created_at: "2026-03-31T12:00:00Z".to_string(),
            })
        }
    }

    /// A mock that returns a [`Gns3Error::Network`] for every operation.
    pub struct MockGns3ApiError;

    #[async_trait::async_trait]
    impl Gns3Api for MockGns3ApiError {
        async fn get_version(&self) -> Result<Version, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn list_projects(&self) -> Result<Vec<Project>, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn create_project(&self, _name: &str) -> Result<Project, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn open_project(&self, _project_id: Uuid) -> Result<Project, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn list_templates(&self) -> Result<Vec<Template>, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn create_node(
            &self,
            _project_id: Uuid,
            _template_id: Uuid,
            _request: CreateNodeRequest,
        ) -> Result<Node, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn start_node(&self, _project_id: Uuid, _node_id: Uuid) -> Result<Node, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn create_link(
            &self,
            _project_id: Uuid,
            _endpoints: Vec<LinkEndpoint>,
        ) -> Result<Link, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn list_nodes(&self, _project_id: Uuid) -> Result<Vec<Node>, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn stop_node(&self, _project_id: Uuid, _node_id: Uuid) -> Result<Node, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn delete_node(&self, _project_id: Uuid, _node_id: Uuid) -> Result<(), Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn list_links(&self, _project_id: Uuid) -> Result<Vec<Link>, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn delete_link(&self, _project_id: Uuid, _link_id: Uuid) -> Result<(), Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn close_project(&self, _project_id: Uuid) -> Result<Project, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn delete_project(&self, _project_id: Uuid) -> Result<(), Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn list_computes(&self) -> Result<Vec<Compute>, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn start_all_nodes(&self, _project_id: Uuid) -> Result<Vec<Node>, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn stop_all_nodes(&self, _project_id: Uuid) -> Result<Vec<Node>, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn update_node(
            &self,
            _project_id: Uuid,
            _node_id: Uuid,
            _update: gns3_mcp_core::UpdateNodeRequest,
        ) -> Result<Node, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn update_template(
            &self,
            _template_id: Uuid,
            _properties: serde_json::Value,
        ) -> Result<serde_json::Value, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn add_drawing(
            &self,
            _project_id: Uuid,
            _request: gns3_mcp_core::AddDrawingRequest,
        ) -> Result<gns3_mcp_core::Drawing, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn export_project(
            &self,
            _project_id: Uuid,
            _include_images: bool,
        ) -> Result<gns3_mcp_core::ExportResult, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn configure_switch(
            &self,
            _project_id: Uuid,
            _node_id: Uuid,
            _ports: Vec<gns3_mcp_core::SwitchPort>,
        ) -> Result<Node, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }

        async fn snapshot_project(
            &self,
            _project_id: Uuid,
            _name: &str,
        ) -> Result<gns3_mcp_core::Snapshot, Gns3Error> {
            Err(Gns3Error::Network("connection refused".to_string()))
        }
    }

    /// Extract the text content from the first item of a [`rmcp::model::CallToolResult`].
    ///
    /// Panics if the first content item is not a `Text` variant. Intended for use in
    /// unit tests only.
    pub fn text_content(result: &rmcp::model::CallToolResult) -> &str {
        match &*result.content[0] {
            rmcp::model::RawContent::Text(t) => &t.text,
            _ => panic!("expected text content"),
        }
    }
}

#[cfg(test)]
mod tests {
    use rmcp::model::ErrorCode;

    use super::*;
    use crate::tools::mock::PROJECT_ID;

    #[test]
    fn parse_uuid_valid() {
        let result = parse_uuid(PROJECT_ID, "project");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), PROJECT_ID);
    }

    #[test]
    fn parse_uuid_invalid() {
        let result = parse_uuid("not-a-valid-uuid", "project");
        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::INVALID_PARAMS);
        assert!(err.message.contains("Invalid project UUID"));
        assert!(err.message.contains("not-a-valid-uuid"));
        assert!(err.message.contains("list tool"));
    }

    #[test]
    fn to_mcp_error_maps_codes() {
        // InvalidUuid -> INVALID_PARAMS
        let err_uuid = Gns3Error::InvalidUuid("bad".to_string());
        let mcp_uuid = to_mcp_error(err_uuid);
        assert_eq!(mcp_uuid.code, ErrorCode::INVALID_PARAMS);

        // Config -> INVALID_PARAMS
        let err_cfg = Gns3Error::Config("missing url".to_string());
        let mcp_cfg = to_mcp_error(err_cfg);
        assert_eq!(mcp_cfg.code, ErrorCode::INVALID_PARAMS);

        // Network -> INTERNAL_ERROR
        let err_net = Gns3Error::Network("timeout".to_string());
        let mcp_net = to_mcp_error(err_net);
        assert_eq!(mcp_net.code, ErrorCode::INTERNAL_ERROR);

        // Http -> INTERNAL_ERROR
        let err_http = Gns3Error::Http {
            status: 404,
            message: "not found".to_string(),
        };
        let mcp_http = to_mcp_error(err_http);
        assert_eq!(mcp_http.code, ErrorCode::INTERNAL_ERROR);
    }
}
