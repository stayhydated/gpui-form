# gpui-form-mcp Architecture

`gpui-form-mcp` is the experimental public integration crate for exposing
generated `gpui-form` submit flows as MCP tools.

It is deliberately not part of the default facade feature set. Applications opt
in through `gpui-form = { features = ["mcp"] }`.

## Purpose

This crate owns:

1. MCP-facing form descriptors and generated submit registrations
1. JSON Schema generation for MCP tool `inputSchema`
1. JSON argument decoding helpers used by generated `McpForm` impls
1. a typed tool handler registry for model and holder submission
1. optional headless edit sessions for generated value holders
1. form-specific wrappers around the shared MCP tool server

It does not own GPUI widget state, GPUI windows, button callbacks, native CLI
parsing, shell completion, command help text, or exit-code policy.
Shared tool result, generic server, and `rmcp` stdio serving live in
`component-shape-mcp`. Low-level protocol errors, version negotiation,
shutdown, and transport framing are handled by `rmcp`.

## Generated Boundary

`gpui-form-derive` emits the `McpForm` implementation. The derive only emits
this boundary for concrete forms that opt in with
`#[gpui_form(mcp)]`; forms without the facade `mcp` feature, generic forms, and
forms marked `no_inventory` are rejected before generation.
The implementation is typed against the generated `*FormValueHolder` and calls
helper functions in this crate:

- `McpToolCall::into_arguments` receives the normalized argument object from
  `component-shape-mcp` as an `McpArguments` decoder.
- `McpArguments::take_required`, `take_present`, and `take_nullable`
  deserialize field values with `serde` while preserving the distinction
  between absent, null, and present values where generated holder defaults need
  it.
- component-backed fields are wrapped into holder storage through the shape's
  `ValueStorage::present` API in generated code.
- `McpArguments::finish` rejects fields that are not part of the generated form
  descriptor.
- The derive also emits `McpEditableForm` when MCP is enabled. That contract
  decodes a single named field into an existing holder for headless
  edit-session tools. Field-level decoding reuses the same generated
  `McpArguments` path as submit calls, so shape-owned storage policies and
  component MCP schemas stay aligned.

The generated descriptor builds non-component fields through
`McpField::typed::<T>` and component-backed fields through
`McpField::component::<T, Shape>`, so every MCP-visible form-side value type
must publish `McpJsonSchema` metadata and component schemas stay coupled to
the selected shape. The shared derive covers named structs, tuple or named
transparent newtypes, and fieldless enums, with built-in schemas for common
containers. The field metadata also stores the Rust value type, value
presence, default flag, component-backed flag, and optional value-specific
component shape MCP input metadata. The runtime maps this metadata to a JSON
Schema object suitable for MCP `tools/list`. Component-backed fields use
`<Shape as ComponentShapeFor<Field>>::MCP_INPUT`; when the shape does not
publish MCP input metadata for that field value type they fall back to the same
typed schema function used by non-component fields. Descriptors are built
through typed constructors rather than stringified Rust type guesses. The
descriptor also stores shared `McpToolMetadata` so
`#[gpui_form(mcp(name = ..., title = ..., description = ...))]` can override
generated tool metadata without changing holder decoding.

## Tool Registry

Application-owned submit functions are usually registered with
`#[gpui_form::mcp_submit]`. The derive implements `McpSubmitArgument` for the
source model when model conversion is available and for the generated
`*FormValueHolder`. The attribute macro requires that trait on the handler
parameter, then emits a `McpSubmitHandlerRegistration` inventory item that
delegates sync or async registration to the generated implementation.
`server()` returns a built shared `component-shape-mcp` server with those
handler registrations already applied. `server_named(name, version)` uses
application-owned server metadata. `builder()` and `builder_named(name,
version)` expose the deferred builder path for callers that want setup errors
reported from `.build()?`, `.serve_stdio().await`, or
`.serve_stdio_blocking()`.
`serve_stdio()` and `serve_stdio_blocking()` are convenience entry points for
serving the default generated registry directly.
`register(&mut server)` returns the same setup errors while adding discovered
form handlers into an application-owned shared server so forms can be served
beside table query tools or other component-shape MCP integrations.

The lower-level manual registration API remains available:

- `form::<Form>(&mut server).model(handler)` decodes arguments, validates the
  holder, converts the holder into `Form` through generated `McpFormModel`, and
  calls a handler returning `Result<T, E>` where `T: serde::Serialize` and
  `E: fmt::Display`.
- `form::<Form>(&mut server).holder(handler)` decodes and validates the holder,
  then passes the holder directly to a `Result<T, E>` handler. This
  is the path for forms with `#[gpui_form(skip)]` fields because the
  application owns the skipped context.
- `model` and `holder` include both sync and async variants and use the same decode
  path for `Result<T, E>` handlers.
- `*_async` variants await async handlers.
- `form::<Form>(&mut server).editor()` registers stateful headless holder
  editing tools when `Form: McpEditableForm`. The tool set is named from the
  form submit tool: `*_edit_open`, `*_edit_read`, `*_edit_patch`,
  `*_edit_validate`, and `*_edit_close`.

Editor sessions live in the MCP server process behind a typed per-form mutex.
`edit_open` starts from a generated holder default and optionally applies a
partial `values` object, `edit_patch` decodes one `field` and `value`,
`edit_read` returns the agent-supplied values plus `submit_arguments`,
`edit_validate` reports missing required fields and generated holder validation
without treating invalid form state as a protocol error, and `edit_close`
drops the session. Required direct storage fields are tracked with a
session-side present-field set because the holder storage itself cannot
distinguish "missing" from a synthesized default. The session stores
agent-supplied JSON values beside the typed holder rather than serializing
holder internals, so submit-only MCP forms do not need serializable field
types. This is a headless holder editing surface. Live GPUI widget mutation
remains an application-owned bridge from session state into GPUI entities.

Manual tool registration returns `Result<(), McpToolError>`. Tool setup
failures are reported during server construction; decode, validation,
conversion, unknown-tool, and handler failures are returned as MCP tool results
with `isError: true`, not as JSON-RPC protocol errors.

Handler responses are serialized by `component-shape-mcp` to
`structuredContent` and mirrored into a text content block.

## Stdio MCP Server

The stdio helper delegates to `component-shape-mcp`, which implements
`rmcp::ServerHandler` for the generic server and serves it over
`rmcp::transport::stdio`. The first proof-of-concept MCP shape remains:

- `initialize`
- `tools/list`
- `tools/call`
- `shutdown`

It uses MCP protocol version `2025-11-25`, advertises the `tools` capability,
and relies on `rmcp` for JSON-RPC parsing and lifecycle handling. Invalid form
values are tool execution errors.

## JSON Schema

The schema mapper uses typed schema functions and component-shape metadata:

- component-backed fields use `<Shape as ComponentShapeFor<Field>>::MCP_INPUT`
  when present, then fall back to the field type schema
- aliases, primitives, strings, paths, chars, known date/time types, lists,
  sets, string-keyed maps, ranges, tuple or named transparent newtypes, named
  structs, and fieldless enums come from `McpJsonSchema`
- unsupported Rust value types keep `x-rustType` metadata and publish an
  impossible schema instead of claiming a primitive type

Non-optional fields without an intent-scoped `default = ...` are marked
required for MCP, even when GPUI holder startup can synthesize a Rust
`Default`. That keeps structured tool calls from silently submitting empty
widget defaults.
