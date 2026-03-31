//! Tool handlers for GNS3 drawing operations.

use gns3_mcp_core::Gns3Api;
use rmcp::model::{CallToolResult, Content, ErrorData};
use serde::Deserialize;

use super::{parse_uuid, to_mcp_error};

/// Parameters for `gns3_add_drawing`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddDrawingParams {
    /// UUID of the project.
    #[schemars(description = "Project UUID")]
    pub project_id: String,
    /// SVG content or shape definition.
    #[schemars(description = "SVG content or shape definition")]
    pub svg: String,
    /// X coordinate on the canvas.
    #[schemars(description = "X coordinate on the canvas")]
    pub x: i32,
    /// Y coordinate on the canvas.
    #[schemars(description = "Y coordinate on the canvas")]
    pub y: i32,
    /// Optional Z-order (defaults to 0 if omitted).
    #[schemars(description = "Z-order for stacking (optional, defaults to 0)")]
    pub z: Option<i32>,
}

/// Handler for `gns3_add_drawing`.
pub async fn add_drawing(
    api: &dyn Gns3Api,
    params: AddDrawingParams,
) -> Result<CallToolResult, ErrorData> {
    let project_id = parse_uuid(&params.project_id, "project")?;

    let request = gns3_mcp_core::AddDrawingRequest {
        svg: params.svg,
        x: params.x,
        y: params.y,
        z: params.z,
    };

    let drawing = api
        .add_drawing(project_id, request)
        .await
        .map_err(to_mcp_error)?;

    let text = format!(
        "Drawing created successfully.\nDrawing ID: {}\nPosition: ({}, {}) Z: {}\nProject: {}",
        drawing.drawing_id, drawing.x, drawing.y, drawing.z, drawing.project_id
    );
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::mock::{text_content, MockGns3Api, MockGns3ApiError, PROJECT_ID};

    #[tokio::test]
    async fn add_drawing_success() {
        let api = MockGns3Api;
        let params = AddDrawingParams {
            project_id: PROJECT_ID.to_string(),
            svg: "<circle cx='50' cy='50' r='40'/>".to_string(),
            x: 100,
            y: 200,
            z: Some(5),
        };
        let result = add_drawing(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("Drawing created successfully"));
        assert!(text.contains("(100, 200)"));
        assert!(text.contains("Z: 5"));
    }

    #[tokio::test]
    async fn add_drawing_with_default_z() {
        let api = MockGns3Api;
        let params = AddDrawingParams {
            project_id: PROJECT_ID.to_string(),
            svg: "<rect width='100' height='100'/>".to_string(),
            x: 0,
            y: 0,
            z: None,
        };
        let result = add_drawing(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("Z: 0"));
    }

    #[tokio::test]
    async fn add_drawing_error() {
        let api = MockGns3ApiError;
        let params = AddDrawingParams {
            project_id: PROJECT_ID.to_string(),
            svg: "<circle/>".to_string(),
            x: 50,
            y: 50,
            z: None,
        };
        let result = add_drawing(&api, params).await;
        assert!(result.is_err());
    }
}
