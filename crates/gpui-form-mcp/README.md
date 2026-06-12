# gpui-form-mcp

Experimental MCP integration for `gpui-form` submit flows.

Most application users should start with `gpui-form`. Enable its `mcp` feature
only when you want generated form value holders to be callable through an MCP
tool server. MCP submit forms must opt in with `#[gpui_form(mcp)]`, cannot use
`#[gpui_form(no_inventory)]`, and cannot be generic because submit discovery is
inventory-backed.

```toml
[dependencies]
gpui-form = { version = "0.5.1", features = ["mcp"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

```rust
use gpui_form::GpuiForm;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, GpuiForm, Serialize)]
#[gpui_form(mcp(
    name = "submit_contact",
    title = "Submit contact request",
    description = "Submit a contact request from structured MCP arguments."
))]
struct ContactRequest {
    #[gpui_form(hidden)]
    email: String,
}

#[gpui_form::mcp_submit]
async fn submit_contact(request: ContactRequest) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({ "email": request.email }))
}

fn main() -> gpui_form::mcp::ServeStdioResult {
    gpui_form::mcp::serve_stdio_blocking()
}
```

This crate is a research surface. It does not start with GPUI app startup, does
not provide a native CLI adapter, and does not make GPUI widgets the execution
engine for submit calls.

Shared MCP tool result, server, and `rmcp` stdio serving mechanics come from
`component-shape-mcp`; this crate owns form-specific holder decoding,
validation, and submit handler wiring. Use `gpui_form::mcp::server()?` for the
default generated server or `gpui_form::mcp::server_named(name, version)?` when
the MCP server should advertise application-owned metadata. Use
`gpui_form::mcp::builder()` or `builder_named(name, version)` when callers need
the shared builder for deferred setup. Use
`gpui_form::mcp::serve_stdio_blocking()` for the default stdio server.

When composing multiple integrations, such as a binary that also depends on
`gpui-table`, keep using the shared builder:

```rust
let server = gpui_form::mcp::McpServer::builder("my-app", env!("CARGO_PKG_VERSION"))
    .register(gpui_form::mcp::register)
    // If your binary also enables gpui-table MCP integration:
    .register(gpui_table::mcp::register)
    .build()?;
```

Submit handlers can be synchronous or async. Return `Result<T, E>` for handler
errors and normal responses, where `T: serde::Serialize` and `E: fmt::Display`.
`#[gpui_form::mcp_submit]` infers model handlers from source-model parameters
and holder handlers from generated `*FormValueHolder` parameters through
generated `McpSubmitArgument` implementations.
Manual server composition uses
`gpui_form::mcp::form::<Form>(&mut server).model(handler)?` or
`.holder(handler)?` for `Result<T, E>` handlers. The handler must return
`Result`.
Use struct-level
`#[gpui_form(mcp(name = "...", title = "...", description = "..."))]` to
override the generated MCP tool name, title, or description. Registration
reports setup errors such as duplicate tool names.

MCP forms can also register editable holder sessions with
`gpui_form::mcp::form::<Form>(&mut server).editor()?`. The editor adds
`*_edit_open`, `*_edit_read`, `*_edit_patch`, `*_edit_validate`, and
`*_edit_close` tools derived from the form's MCP tool name. These tools keep a
headless generated value holder in server memory: `edit_open` creates a
session from optional initial `values`, `edit_patch` updates one `field` from
structured JSON, `edit_read` returns agent-supplied `values`, missing required
fields, and `submit_arguments`, `edit_validate` reports generated validation
errors, and `edit_close` drops the session. Field patches use the same
generated decoding path as submit calls, including component-shape `MCP_INPUT`
schemas and value-storage policies. The editor does not mutate live GPUI
widgets unless the application adds its own runtime bridge from the holder
session into GPUI entities.

Component-backed fields use value-specific shape metadata from
`<Shape as ComponentShapeFor<Field>>::MCP_INPUT` in generated input schemas
when the shape publishes one. `component_shape_gpui` infers common shape MCP
input from declared primitive values, primitive lists, primitive sets, fixed
arrays, and ranges, then attaches it to the generated
`ComponentShapeFor<Value>` impl. Component-backed fields without value-specific
shape MCP input metadata fall back to the field type's `McpJsonSchema`
implementation. For custom field value types, generated descriptors call
`McpJsonSchema::json_schema()`; derive `gpui_form::mcp::McpJsonSchema` with
`#[mcp(crate = gpui_form::mcp)]` for app-owned object structs, tuple or named
transparent newtypes, or fieldless enums that should expose precise schemas. The derive
follows serde deserialize names, includes enum deserialize aliases, skips
deserialization-skipped fields, rejects flattened fields, and treats
serde-defaulted fields as not required. Use `gpui_form::mcp::McpRange<T>` for
typed `{ "min": ..., "max": ... }` range arguments.
