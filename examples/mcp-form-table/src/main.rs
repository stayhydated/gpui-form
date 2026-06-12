use component_shape_gpui::component_shape;
use gpui_form::{GpuiForm, mcp::McpForm as _};
use gpui_table::GpuiTable;
use gpui_table::mcp::McpTable as _;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub struct TicketTitleState;

impl TicketTitleState {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

component_shape! {
    pub struct TicketTitleShape {
        state = TicketTitleState;
        value = String;
        field_suffix = "title";
    }
}

impl gpui_form::runtime::shape::GpuiFormComponentShapePolicy for TicketTitleShape {
    type ValueStoragePolicy = gpui_form::runtime::shape::DirectValueStorage;
}

#[derive(Clone, Debug, Deserialize, GpuiForm, Serialize)]
#[gpui_form(mcp(
    name = "submit_ticket_from_custom_shape",
    title = "Submit ticket",
    description = "Submit a ticket using a custom component shape."
))]
pub struct TicketForm {
    #[gpui_form(component(TicketTitleShape))]
    title: String,

    #[gpui_form(hidden)]
    details: Option<String>,
}

#[derive(Clone, Debug, GpuiTable, Serialize)]
#[gpui_table(
    id = "tickets",
    title = "Tickets",
    filters,
    mcp(
        name = "query_tickets_with_inferred_filter",
        title = "Query tickets",
        description = "Query tickets with an inferred MCP text filter."
    )
)]
struct TicketRow {
    #[gpui_table(filter)]
    title: String,
}

#[gpui_form::mcp_submit]
fn submit_ticket(ticket: TicketForm) -> Result<Value, String> {
    Ok(json!({
        "accepted": true,
        "title": ticket.title,
        "details": ticket.details
    }))
}

#[gpui_table::mcp_query]
fn ticket_rows() -> Result<Vec<TicketRow>, String> {
    Ok(vec![
        TicketRow {
            title: "Add custom component shape".to_string(),
        },
        TicketRow {
            title: "Expose table query over MCP".to_string(),
        },
    ])
}

fn main() -> gpui_form::mcp::ServeStdioResult {
    let mut server = gpui_form::mcp::McpServer::new("mcp-form-table", env!("CARGO_PKG_VERSION"));
    gpui_form::mcp::register(&mut server)?;
    gpui_table::mcp::register(&mut server)?;

    if std::env::args().any(|arg| arg == "--stdio") {
        return server.serve_stdio_blocking();
    }

    let submit = server.call_tool(
        &TicketForm::descriptor().tool_name(),
        Some(json!({
            "title": "DX pass",
            "details": "No manual mcp_input or filter decoder required."
        })),
    );
    let query = server.call_tool(
        &TicketRow::descriptor().tool_name(),
        Some(json!({ "title": "MCP" })),
    );

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "submit": submit.structured_content,
            "query": query.structured_content
        }))
        .expect("example output should serialize")
    );

    Ok(())
}
