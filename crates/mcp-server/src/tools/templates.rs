//! Tool handlers for GNS3 template operations.

use std::fmt::Write;

use gns3_mcp_core::Gns3Api;
use rmcp::model::{CallToolResult, Content, ErrorData};

use super::to_mcp_error;

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
}
