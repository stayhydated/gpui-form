use gpui_form::GpuiForm;
use koruma_collection::collection::NonEmptyValidation;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, gpui_form::mcp::McpJsonSchema, Serialize)]
struct SupportTicketResponse {
    accepted: bool,
    kind: String,
    title: String,
    details: Option<String>,
    urgent: bool,
}

#[derive(Debug, gpui_form::mcp::McpJsonSchema, Serialize)]
struct TenantNoteResponse {
    accepted: bool,
    kind: String,
    body: String,
    tenant_id: String,
}

#[gpui_form::mcp_submit]
async fn submit_support_ticket(ticket: SupportTicket) -> Result<SupportTicketResponse, String> {
    Ok(SupportTicketResponse {
        accepted: true,
        kind: "support_ticket".to_string(),
        title: ticket.title,
        details: ticket.details,
        urgent: ticket.urgent,
    })
}

#[gpui_form::mcp_submit]
fn submit_tenant_note(holder: TenantNoteFormValueHolder) -> Result<TenantNoteResponse, String> {
    Ok(TenantNoteResponse {
        accepted: true,
        kind: "tenant_note".to_string(),
        body: holder.body,
        tenant_id: "provided-by-application-context".to_string(),
    })
}

fn main() -> gpui_form::mcp::ServeStdioResult {
    let mut server = gpui_form::mcp::McpServer::new("mcp-submit", env!("CARGO_PKG_VERSION"));
    gpui_form::mcp::register(&mut server)?;
    gpui_form::mcp::register_prompt_templates(&mut server)?;
    server.serve_stdio_blocking()
}
