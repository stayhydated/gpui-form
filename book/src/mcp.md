# MCP form tools

The `mcp` feature exposes concrete generated forms as structured submit tools
and headless edit sessions. The application still owns submission behavior;
`gpui-form` supplies typed decoding, validation, metadata, registration, and
session management.

## Add MCP support

For a crate that also renders GPUI forms:

```toml
[dependencies]
gpui-form = { version = "0.6", features = ["mcp"] }
serde = { version = "1", features = ["derive"] }
```

For a headless form server, disable the default runtime and enable the derive
explicitly:

```toml
gpui-form = { version = "0.6", default-features = false, features = ["derive", "mcp"] }
```

Add `chrono` or `rust_decimal` beside `mcp` when exposed fields or responses
use those values.

## Define a form and handler

MCP forms must be concrete and inventory-backed. Opt in on the model and
register a free submit function with one owned parameter:

```rust,ignore
use gpui_form::GpuiForm;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, GpuiForm, Serialize)]
#[gpui_form(mcp(name = "submit_contact", title = "Submit contact request"))]
struct ContactRequest {
    #[gpui_form(hidden)]
    email: String,
}

#[derive(Debug, gpui_form::mcp::McpJsonSchema, Serialize)]
struct ContactResponse {
    accepted: bool,
}

#[gpui_form::mcp_submit]
async fn submit_contact(
    request: ContactRequest,
) -> Result<ContactResponse, String> {
    Ok(ContactResponse {
        accepted: !request.email.is_empty(),
    })
}

fn main() -> gpui_form::mcp::ServeStdioResult {
    gpui_form::mcp::serve_stdio_blocking()
}
```

Handlers can be synchronous or asynchronous and return `Result<T, E>`, where
the response is serializable with an MCP schema and the error implements
`Display`. Prefer a response struct or newtype with an object-root schema.

Use the source model parameter for normal submission. When a form has skipped
fields, accept its generated `*FormValueHolder` so application code can supply
the skipped context.

## Share application context

For several forms that submit through the same state:

1. Add `context(MyContext)` to each form's `mcp(...)` options.
2. Implement `McpContextSubmit<MyContext>` for the model, or add
   `submit(path::to::async_fn)` so the derive emits the implementation.
3. Add `response(MyResponse)` for a precise output schema.
4. Register all matching forms with
   `register_context_submitters(&mut server, context)`.

Use `map_response(path::to::fn)` when a raw application response needs a
fallible mapping into the published response. Use
`submitter(path::to::Trait)` when an application trait owns the associated
context, response, error, and submit method.

## Choose registration behavior

| Need | API |
|---|---|
| Default inventory server over stdio | `serve_stdio_blocking()` |
| Application-owned server metadata | `server_named(name, version)` |
| Add generated forms to an existing server | `register(&mut server)` |
| Register shared-context submitters | `register_context_submitters(...)` |
| Select submit, editor, or metadata behavior | `register_with_options(...)` |
| Add generated prompt templates | `register_prompt_templates(&mut server)` |

Use per-form registration helpers when one form needs a manual handler or
editor policy.

## Edit a holder through MCP

Generated editor tools use an optimistic revision:

1. Call `*_edit_open` and retain `session_id` and `revision`.
2. Call `*_edit_patch` with `values` and optional `clear`.
3. Pass `expected_revision` when stale writes must fail.
4. Call `*_edit_validate` and inspect form-level and field-level errors.
5. Call `*_edit_submit` when the form has a submit handler.
6. Close abandoned sessions with `*_edit_close`.

Bulk patches are atomic. Sessions have a default count limit and idle timeout;
configure them with `McpFormEditorOptions` and the corresponding
`*_with_editor_options` registration helper.

Editor sessions hold generated values on the MCP server. They do not mutate
live GPUI entities; an application that shows remote edits must provide that
bridge.

## Use generated metadata

Registered forms publish descriptor, schema, and examples resources at:

- `gpui-form://forms/{tool_name}/descriptor`
- `gpui-form://forms/{tool_name}/schema`
- `gpui-form://forms/{tool_name}/examples`

Prompt templates are opt-in and produce `fill_*`, `repair_*`, and `submit_*`
prompts that point clients to those resources and the applicable editor tools.

Field input schemas come from `McpToolValue`. Custom Serde types normally
derive `gpui_form::mcp::McpJsonSchema`. Component-backed fields also attach
value-specific shape MCP metadata when the shape publishes it. Koruma-backed
forms expose validation metadata and structured validation issues.

## Troubleshooting

| Symptom | Action |
|---|---|
| `#[gpui_form(mcp)]` rejects the form | Use a concrete form and keep inventory enabled; remove `no_inventory`. |
| Editor tools exist but the submit tool is absent | Add a `#[gpui_form::mcp_submit]` handler or register a manual/context submitter. |
| Registration reports a duplicate name or URI | Give each exposed form a unique `mcp(name = "...")` value. |
| A field fails schema generation | Derive `McpJsonSchema` or implement `McpToolValue` for the custom value type. |
| An editor patch reports a stale revision | Read the session again and retry against its current revision. |
