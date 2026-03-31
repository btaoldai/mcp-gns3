//! Tool handlers for GNS3 template operations.

use std::fmt::Write;

use gns3_mcp_core::Gns3Api;
use rmcp::model::{CallToolResult, Content, ErrorData};
use serde::Deserialize;

use super::{parse_uuid, to_mcp_error};

/// Handler for `gns3_list_templates`.
pub async fn list_templates(api: &dyn Gns3Api) -> Result<CallToolResult, ErrorData> {
    let templates = api.list_templates().await.map_err(to_mcp_error)?;

    if templates.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No templates found. Install appliances in GNS3 first.",
        )]));
    }

    let mut text = format!("Found {} template(s):\n\n", templates.len());
    text.push_str("| Name | Type | Category | ID |\n|---|---|---|---|\n");
    for t in &templates {
        let _ = writeln!(
            text,
            "| {} | {} | {} | {} |",
            t.name, t.template_type, t.category, t.template_id
        );
    }
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

/// Parameters for `gns3_update_template`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateTemplateParams {
    /// UUID of the template to update.
    #[schemars(description = "Template UUID")]
    pub template_id: String,
    /// Template properties object (RAM, NICs, boot order, etc.).
    #[schemars(description = "Properties object with template-specific fields")]
    pub properties: serde_json::Value,
}

/// Handler for `gns3_update_template`.
pub async fn update_template(
    api: &dyn Gns3Api,
    params: UpdateTemplateParams,
) -> Result<CallToolResult, ErrorData> {
    let template_id = parse_uuid(&params.template_id, "template")?;

    let result = api
        .update_template(template_id, params.properties)
        .await
        .map_err(to_mcp_error)?;

    let text = format!(
        "Template updated successfully.\nTemplate ID: {}\nUpdated properties: {}",
        template_id,
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "N/A".to_string())
    );
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::mock::{text_content, MockGns3Api};

    #[tokio::test]
    async fn list_templates_returns_table() {
        let api = MockGns3Api;
        let result = list_templates(&api).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("1 template(s)"));
        assert!(text.contains("VPCS"));
        assert!(text.contains("vpcs"));
    }

    #[tokio::test]
    async fn update_template_success() {
        use crate::tools::mock::{text_content, TEMPLATE_ID};
        let api = MockGns3Api;
        let props = serde_json::json!({ "ram": 512, "adapters": 4 });
        let params = UpdateTemplateParams {
            template_id: TEMPLATE_ID.to_string(),
            properties: props,
        };
        let result = update_template(&api, params).await.unwrap();
        let text = text_content(&result);
        assert!(text.contains("Template updated successfully"));
        assert!(text.contains("ram"));
    }

    #[tokio::test]
    async fn update_template_error() {
        use crate::tools::mock::{MockGns3ApiError, TEMPLATE_ID};
        let api = MockGns3ApiError;
        let props = serde_json::json!({ "ram": 1024 });
        let params = UpdateTemplateParams {
            template_id: TEMPLATE_ID.to_string(),
            properties: props,
        };
        let result = update_template(&api, params).await;
        assert!(result.is_err());
    }
}
