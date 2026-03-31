//! Domain types mirroring the GNS3 REST API v2 resources.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// GNS3 server version information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    /// Server version string (e.g. "2.2.52").
    pub version: String,
    /// Whether this is the local server.
    pub local: bool,
}

/// Project status in GNS3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    /// Project is opened and ready for operations.
    Opened,
    /// Project is closed.
    Closed,
}

/// A GNS3 project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Unique project identifier.
    pub project_id: Uuid,
    /// Human-readable project name.
    pub name: String,
    /// Current project status.
    #[serde(default = "default_project_status")]
    pub status: ProjectStatus,
}

fn default_project_status() -> ProjectStatus {
    ProjectStatus::Closed
}

/// Node status in GNS3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    /// Node is running.
    Started,
    /// Node is stopped.
    Stopped,
    /// Node is suspended.
    Suspended,
}

/// A GNS3 node within a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique node identifier.
    pub node_id: Uuid,
    /// Human-readable node name.
    pub name: String,
    /// Current node status.
    pub status: NodeStatus,
    /// Node type (e.g. "vpcs", "qemu", "docker").
    pub node_type: String,
    /// Parent project identifier.
    pub project_id: Uuid,
    /// Console port (if available).
    #[serde(default)]
    pub console: Option<u16>,
    /// Console type (e.g. "telnet", "vnc").
    #[serde(default)]
    pub console_type: Option<String>,
}

/// A GNS3 node template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Unique template identifier.
    pub template_id: Uuid,
    /// Template name.
    pub name: String,
    /// Template type (e.g. "vpcs", "qemu", "docker", "cloud").
    pub template_type: String,
    /// Category for UI grouping.
    #[serde(default)]
    pub category: String,
    /// Whether this is a built-in template.
    #[serde(default)]
    pub builtin: bool,
}

/// A port endpoint in a link, identifying a specific interface on a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkEndpoint {
    /// Node identifier.
    pub node_id: Uuid,
    /// Adapter (interface) number on the node.
    pub adapter_number: u32,
    /// Port number on the adapter.
    pub port_number: u32,
}

/// A GNS3 link connecting two node interfaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    /// Unique link identifier.
    pub link_id: Uuid,
    /// Parent project identifier.
    pub project_id: Uuid,
    /// The two endpoints of the link.
    pub nodes: Vec<LinkEndpoint>,
}

/// A GNS3 compute server (execution backend for nodes).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compute {
    /// Compute server identifier (e.g. "local" or a UUID string).
    ///
    /// GNS3 uses the string `"local"` for the built-in compute, not a UUID.
    pub compute_id: String,
    /// Human-readable compute server name.
    pub name: String,
    /// Whether the compute is currently connected to the controller.
    pub connected: bool,
    /// Hostname or IP address of the compute server.
    pub host: String,
    /// TCP port the compute API listens on.
    pub port: u16,
    /// Protocol used to reach the compute API (e.g. "http", "https").
    pub protocol: String,
    /// Current CPU usage in percent (absent when compute is disconnected).
    #[serde(default)]
    pub cpu_usage_percent: Option<f64>,
    /// Current memory usage in percent (absent when compute is disconnected).
    #[serde(default)]
    pub memory_usage_percent: Option<f64>,
}

/// Request payload for creating a node from a template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNodeRequest {
    /// X coordinate on the canvas.
    pub x: i32,
    /// Y coordinate on the canvas.
    pub y: i32,
    /// Optional node name (uses template default if omitted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Compute node to run on (defaults to "local").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_id: Option<String>,
}

/// Request payload for updating an existing node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNodeRequest {
    /// Optional new node name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional compute node ID to move the node to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_id: Option<String>,
    /// Optional properties object for type-specific settings (e.g. QEMU adapters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<serde_json::Value>,
}

/// A drawing (label, shape, or SVG) on the project canvas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drawing {
    /// Unique drawing identifier.
    pub drawing_id: Uuid,
    /// Parent project identifier.
    pub project_id: Uuid,
    /// SVG content or shape definition.
    pub svg: String,
    /// X coordinate on the canvas.
    pub x: i32,
    /// Y coordinate on the canvas.
    pub y: i32,
    /// Z-order (stacking depth).
    pub z: i32,
}

/// Request payload for adding a drawing to a project canvas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddDrawingRequest {
    /// SVG content or shape definition.
    pub svg: String,
    /// X coordinate on the canvas.
    pub x: i32,
    /// Y coordinate on the canvas.
    pub y: i32,
    /// Optional Z-order (defaults to 0 if omitted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z: Option<i32>,
}

/// Port configuration for Ethernet switches.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SwitchPort {
    /// Port name (e.g. "Ethernet0").
    pub name: String,
    /// Port number.
    pub port_number: u32,
    /// Port type (e.g. "access", "trunk").
    #[serde(rename = "type")]
    pub type_: String,
    /// VLAN ID.
    pub vlan: u32,
    /// Optional EtherType for filtering.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethertype: Option<String>,
}

/// Result of exporting a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    /// Size of the exported archive in bytes.
    pub size_bytes: usize,
}

/// A project snapshot for versioning and rollback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Unique snapshot identifier.
    pub snapshot_id: Uuid,
    /// Snapshot name.
    pub name: String,
    /// Creation timestamp as ISO 8601 string.
    pub created_at: String,
}

impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Opened => write!(f, "opened"),
            Self::Closed => write!(f, "closed"),
        }
    }
}

impl std::fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Started => write!(f, "started"),
            Self::Stopped => write!(f, "stopped"),
            Self::Suspended => write!(f, "suspended"),
        }
    }
}
