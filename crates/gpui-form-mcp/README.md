# gpui-form-mcp

Experimental MCP integration for `gpui-form` submit flows.

Most application users should start with `gpui-form`. Enable its `mcp` feature
only when you want generated form value holders to be callable through an MCP
tool server. MCP submit forms must opt in with `#[gpui_form(mcp)]`, cannot use
`#[gpui_form(no_inventory)]`, and cannot be generic because submit discovery is
inventory-backed.

```toml
[dependencies]
gpui-form = { version = "0.5.1", default-features = false, features = ["derive", "mcp"] }
serde = { version = "1", features = ["derive"] }
```

Keep the `gpui-form` defaults, or enable its `runtime` feature, when the same
crate also renders component-backed GPUI forms or references
`gpui_form::runtime`.

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

#[derive(Debug, gpui_form::mcp::McpJsonSchema, Serialize)]
struct ContactSubmitResponse {
    email: String,
}

#[gpui_form::mcp_submit]
async fn submit_contact(request: ContactRequest) -> Result<ContactSubmitResponse, String> {
    Ok(ContactSubmitResponse {
        email: request.email,
    })
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
validation, submit handler wiring, and editable holder-session tools. Use
`gpui_form::mcp::server()?` for the default generated server or
`gpui_form::mcp::server_named(name, version)?` when the MCP server should
advertise application-owned metadata. Generated servers register
`#[gpui_form::mcp_submit]` submit handlers plus editor tools for every
`#[gpui_form(mcp)]` form. Forms that also have a submit handler get
`*_edit_submit`, so agents can submit a valid edit session by `session_id`.
Use `gpui_form::mcp::builder()` or
`builder_named(name, version)` when callers need the shared builder for
deferred setup. Use
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
errors and normal responses, where `T: serde::Serialize + gpui_form::mcp::McpJsonSchema`
and `E: fmt::Display`. Prefer a typed response struct or newtype that derives
`gpui_form::mcp::McpJsonSchema` for precise output schemas, or return
`gpui_form::mcp::McpObject` for dynamic object-shaped JSON.
`#[gpui_form::mcp_submit]` infers model handlers from source-model parameters
and holder handlers from generated `*FormValueHolder` parameters through
generated `McpSubmitArgument` implementations.
For submit flows with shared runtime state, declare the context on the form
itself with `#[gpui_form(mcp(context(MyContext), ...))]`, optionally add
`response(MyResponse)` for a precise output schema, and either implement
`gpui_form::mcp::McpContextSubmit<MyContext>` for the source model or add
`submit(path::to::async_fn)` so the derive emits that impl. Generated submit
impls can also use `map_response(path::to::fn)` to convert a raw handler
response into the published response type, and `error(MyError)` to override
the default `String` error type. Then call
`gpui_form::mcp::register_context_submitters(&mut server, context)?` or
`register_context_submitters_with_editor_options(...)`. The derive inventories
those forms, so one registration call wires every matching context-backed
submit tool plus `*_edit_submit`; the context type must be
`Clone + Send + Sync + 'static`. If `response(...)` is omitted, the generated
registration uses `McpObject`.
Manual server composition uses
`gpui_form::mcp::form::<Form>(&mut server).model(handler)?` or
`.holder(handler)?` for `Result<T, E>` handlers. The handler must return
`Result`.
Use `.model_with_editor(handler)?`, `.model_async_with_editor(handler)?`,
`.holder_with_editor(handler)?`, or `.holder_async_with_editor(handler)?` when
the same form should publish submit, editable holder-session tools, and
`*_edit_submit` for submitting the current edit session through the same
handler.
Use struct-level
`#[gpui_form(mcp(name = "...", title = "...", description = "..."))]` to
override the generated MCP tool name, title, or description. When
`description` is omitted, the derive uses the form type's Rust doc comment.
The same list accepts optional MCP tool annotation hints with
`read_only = ...`, `destructive = ...`, `idempotent = ...`, and
`open_world = ...`, so generated submit tools can advertise whether they
inspect local/server state, mutate it, or call into external systems.
`read_only = true` and `destructive = true` cannot be combined.
Registration reports setup errors such as duplicate tool names.

MCP forms can also register editable holder sessions directly with
`gpui_form::mcp::form::<Form>(&mut server).editor()?`. The editor adds
`*_edit_open`, `*_edit_list`, `*_edit_read`, `*_edit_patch`,
`*_edit_validate`, `*_edit_close`, and `*_edit_close_all` tools derived from
the form's MCP tool
name. These tools keep a headless generated value holder in server memory:
`edit_open` creates a session from optional initial `values`, `edit_patch`
applies a `values` object and/or a `clear` field-name array to the session, and
`edit_patch` can set
`replace = true` to replace the whole pending value set instead of merging and
`expected_revision` to reject stale client updates,
`edit_list` returns every active session for the form plus the effective
`session_limit` and `session_idle_timeout_ms`, `edit_read` returns the session
`revision`, agent-supplied `values`, missing required fields,
validation `errors`, `valid`, `fields` metadata with per-field schemas plus
current `has_value`/`value` state and field-level `errors`, and
`submit_arguments`, `edit_validate` returns the same state snapshot after
running validation, `edit_close` accepts an optional `expected_revision` before
dropping one session, and `edit_close_all` drops every active session for that
form while returning `closed_count` plus the
closed `session_ids`. The published snapshot schema keeps `fields[]` typed with
a per-field `oneOf`. Field patches and clears are
decoded by rebuilding a cloned holder before they are committed, so a failed
bulk patch does not partially mutate the session. Patches use the same generated decoding path as
submit calls, including component-shape
`MCP_INPUT` schemas and value-storage policies. The editor does not mutate live
GPUI widgets unless the application adds its own runtime bridge from the holder
session into GPUI entities.
Editors retain at most `DEFAULT_EDITOR_SESSION_LIMIT` active sessions per form
by default and expire idle sessions after
`DEFAULT_EDITOR_SESSION_IDLE_TIMEOUT`; opening another session evicts the
oldest active session when the cap is exceeded. Use
`.editor_with_options(McpFormEditorOptions::default().with_session_limit(n).with_session_idle_timeout(duration))`,
the matching `*_with_editor_options` submit helpers, or
`McpFormEditorOptions::default().without_session_limit().without_session_idle_timeout()`
when a server needs a different policy.
`edit_open` and `edit_list` report a `cleanup` object with expired and evicted
session IDs so clients can drop local handles that the server removed while
servicing the request.
When the form is registered with one of the `*_with_editor` submit helpers, or
through a generated `#[gpui_form::mcp_submit]` handler, `*_edit_submit` accepts
a `session_id`, optional `expected_revision`, and optional
`close_on_success = false`, validates the current session, and calls the same
submit handler without copying `submit_arguments` into a second tool call.
Successful edit submits close the session by default only when the session has
not advanced while the handler was running; validation, stale revision, handler
failure, or concurrent patch cases leave it open for repair or retry.
Generated editor tool definitions publish MCP annotations: `edit_list`,
`edit_read`, and `edit_validate` are read-only and idempotent, `edit_open` is a
non-destructive session mutation, `edit_patch`, `edit_close`, and
`edit_close_all` are destructive session mutations, and `edit_submit` is a
destructive open-world submit call.

MCP failures keep their text content and also set `structured_content.error`
with a stable `kind` plus relevant fields such as `field`, `value`, `name`, or
`detail`, so agents can inspect decode, validation, session, unknown-tool, and
handler failures without parsing prose. Generated form validation failures also
include a `details` array with individual validation messages.
Generated tool definitions also publish output schemas: submit tools advertise
the handler response type's `McpJsonSchema` or `McpObject` for dynamic JSON,
and editor tools publish strict schemas for session snapshots and close
results. Output schemas must declare root `type: "object"` to match MCP
structured content.

Generated input schemas use the field type's `McpToolValue` schema, so schema
publication and generated decoding stay paired. Component-backed fields also
attach value-specific shape metadata from
`<Shape as ComponentShapeFor<Field>>::MCP_INPUT` when the shape publishes one.
`component_shape_gpui` infers common shape MCP input from declared primitive
values, primitive lists, primitive sets, fixed arrays, and ranges, then
attaches it to the generated `ComponentShapeFor<Value>` impl.
Generic or custom shapes can declare `mcp_input = string`,
`mcp_input = object`, or another `McpInput` expression when the shape knows its
model-facing MCP input better than the value type can be inferred. For custom
field value types, generated descriptors require `gpui_form::mcp::McpToolValue`;
the blanket implementation covers `Deserialize` types that implement or derive
`McpJsonSchema`; use `gpui_form::mcp::McpAny` when a typed field intentionally
accepts unconstrained JSON. Fixed tuples with 1 to 4 elements publish exact
array schemas, and app-owned object structs, tuple or named transparent
newtypes, or fieldless enums that should expose precise schemas can derive
`gpui_form::mcp::McpJsonSchema` directly. In crates that also depend on
another MCP facade such as `gpui-table`, add
`#[mcp(crate = gpui_form::mcp)]` to custom `McpJsonSchema` or `McpToolInput`
derives so generated schema impls target the gpui-form facade. The derive
follows serde deserialize names, records field aliases in `x-mcpAliases`,
includes enum aliases, skips deserialization-skipped fields, rejects flattened
fields, and treats serde-defaulted fields as not required. Use
`gpui_form::mcp::McpRange<T>` for typed `{ "min": ..., "max": ... }` range
arguments. Custom top-level MCP tool argument structs can also derive
`gpui_form::mcp::McpToolInput` through the facade when composing manual typed
tools; that derive also implements `McpJsonSchema`, so object inputs can be
reused as field values.
