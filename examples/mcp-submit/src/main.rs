use gpui_form::GpuiForm;
use koruma_collection::collection::NonEmptyValidation;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Debug, Deserialize, GpuiForm, Serialize)]
#[gpui_form(
    koruma,
    mcp(
        name = "mcp_submit_support_ticket",
        title = "Submit support ticket",
        description = "Create a support ticket from structured MCP arguments."
    )
)]
pub struct SupportTicket {
    #[gpui_form(component(gpui_form_collection::input::Input::<String>))]
    #[koruma(NonEmptyValidation::<_>)]
    title: String,

    #[gpui_form(hidden)]
    details: Option<String>,

    #[gpui_form(hidden(default = false))]
    urgent: bool,
}

#[derive(Clone, Debug, Deserialize, GpuiForm, Serialize)]
#[gpui_form(mcp(
    name = "mcp_submit_tenant_note",
    title = "Submit tenant note",
    description = "Create a tenant note with application-owned tenant context."
))]
pub struct TenantNote {
    #[gpui_form(hidden)]
    body: String,

    #[gpui_form(skip)]
    tenant_id: u64,
}

#[gpui_form::mcp_submit]
async fn submit_support_ticket(ticket: SupportTicket) -> Result<serde_json::Value, String> {
    Ok::<serde_json::Value, String>(serde_json::json!({
        "accepted": true,
        "kind": "support_ticket",
        "title": ticket.title,
        "details": ticket.details,
        "urgent": ticket.urgent
    }))
}

#[gpui_form::mcp_submit]
fn submit_tenant_note(holder: TenantNoteFormValueHolder) -> Result<serde_json::Value, String> {
    Ok::<serde_json::Value, String>(json!({
        "accepted": true,
        "kind": "tenant_note",
        "body": holder.body,
        "tenant_id": "provided-by-application-context"
    }))
}

fn main() -> gpui_form::mcp::ServeStdioResult {
    let mut server = gpui_form::mcp::McpServer::new("mcp-submit", env!("CARGO_PKG_VERSION"));
    gpui_form::mcp::register(&mut server)?;
    gpui_form::mcp::form::<SupportTicket>(&mut server).editor()?;
    server.serve_stdio_blocking()
}
