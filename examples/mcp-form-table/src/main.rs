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

fn server() -> Result<gpui_form::mcp::McpServer, gpui_form::mcp::McpToolError> {
    let mut server = gpui_form::mcp::McpServer::new("mcp-form-table", env!("CARGO_PKG_VERSION"));
    gpui_form::mcp::register(&mut server)?;
    gpui_table::mcp::register(&mut server)?;
    Ok(server)
}

fn main() -> gpui_form::mcp::ServeStdioResult {
    let server = server()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composed_mcp_server_registers_form_and_table_tools() {
        let server = server().expect("form and table MCP tools should register together");
        let tool_names: Vec<_> = server
            .list_tools()
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect();

        assert!(tool_names.contains(&TicketForm::descriptor().tool_name()));
        assert!(tool_names.contains(&TicketRow::descriptor().tool_name()));
    }

    #[test]
    fn composed_mcp_server_exposes_usable_form_and_table_schemas() {
        let server = server().expect("form and table MCP tools should register together");

        let form_tool = server
            .list_tools()
            .into_iter()
            .find(|tool| tool.name == TicketForm::descriptor().tool_name())
            .expect("form tool should be registered");
        assert_eq!(
            form_tool.input_schema["properties"]["title"]["type"],
            "string"
        );
        assert_eq!(
            form_tool.input_schema["properties"]["title"]["x-gpuiFormComponentBacked"],
            true
        );
        assert_eq!(form_tool.input_schema["required"], json!(["title"]));

        let table_tool = server
            .list_tools()
            .into_iter()
            .find(|tool| tool.name == TicketRow::descriptor().tool_name())
            .expect("table tool should be registered");
        assert_eq!(
            table_tool.input_schema["properties"]["title"]["type"],
            "string"
        );
        assert_eq!(
            table_tool.input_schema["properties"]["title"]["x-gpuiTableFilterType"],
            "text"
        );
        assert_eq!(table_tool.input_schema["properties"]["limit"]["minimum"], 0);
        let output_schema = table_tool
            .output_schema
            .as_ref()
            .expect("table tool should advertise output schema");
        assert_eq!(
            output_schema["required"],
            json!(["rows", "total", "offset", "limit"])
        );
    }

    #[test]
    fn composed_mcp_server_calls_form_submit_and_table_query_handlers() {
        let server = server().expect("form and table MCP tools should register together");

        let submit = server.call_tool(
            &TicketForm::descriptor().tool_name(),
            Some(json!({
                "title": "DX pass",
                "details": "No manual mcp_input or filter decoder required."
            })),
        );
        assert_eq!(submit.is_error, Some(false));
        assert_eq!(
            submit.structured_content,
            Some(json!({
                "accepted": true,
                "title": "DX pass",
                "details": "No manual mcp_input or filter decoder required."
            }))
        );

        let query = server.call_tool(
            &TicketRow::descriptor().tool_name(),
            Some(json!({ "title": "MCP" })),
        );
        assert_eq!(query.is_error, Some(false));
        let content = query
            .structured_content
            .expect("table query should return content");
        assert_eq!(content["total"], 1);
        assert_eq!(content["rows"][0]["title"], "Expose table query over MCP");
    }
}
