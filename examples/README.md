# Examples

This directory is the canonical index for runnable `gpui-form` workspace
examples.

## some-lib

Source structs and enums that demonstrate the supported derive surface:

- basic input, number input, checkbox, switch, select, combobox, slider,
  color-picker, date-picker, file-picker, and otp input fields
- select, searchable select, date-picker conversion, and skipped-field workflows
- nested searchable `InfiniteSelect` enums, including index-path and key-path
  round trips plus typed path-error reporting, custom persisted keys, and
  key-path string serialization
- component-shape-backed collection and runtime fields
- Koruma validation wiring
- newtype-backed numeric validation
- `cfg_attr`-gated derive usage
- empty forms and date conversion
- package-local `es-fluent` config, embedded resources, and language enum

## some-lib-forms

Storybook-style GPUI app that renders generated forms around the example types.
The checked-in `location_form` example shows the runtime-owned
`Select` flow with `form_fields()` instead of manual child-select
rebuilding.

Run it with:

```sh
cargo run -p some-lib-forms
```

## gpui-form-component-story

Storybook-style GPUI app for the reusable runtime components themselves:
infinite select, date picker, and file picker.

Run it with:

```sh
cargo run -p gpui-form-component-story
```

## prototyping

Generator example that walks `GpuiFormShape` inventory data and emits scaffolded
form files into `examples/some-lib-forms/src/forms` and mirrors the generated
output under `examples/prototyping/output`, then formats both with `rustfmt`.
Generated Storybook form titles use the example app's active locale. Component
entity fields use source field names such as `email`, while prototyping helper
names keep readable suffixes such as `on_email_input_event`. Holder conversion
debug rows use the derive-emitted inventory flag for `into_original` versus
`try_into_original`.

Run it with:

```sh
cargo run -p prototyping
```

## mcp-submit

Stdio MCP server example that exposes generated form value holders as MCP
tools. It demonstrates generated tool schemas, `tools/call` argument decoding,
holder validation, model conversion, component-backed fields, skipped-field
holder submission, explicit per-tool MCP metadata, field labels/descriptions
and examples in schemas, descriptor/schema/examples resources, opt-in
fill/repair/submit prompt templates, and default headless edit-session tools.
Agents can open a generated value holder, apply atomic bulk
patches through `values` and `clear`, use session `revision` with
`expected_revision` to reject stale edits, inspect `session_limit` and
`session_idle_timeout_ms`, inspect `submit_arguments`, and call
`*_edit_submit` for handler-backed sessions. Editor sessions are bounded and
idle-expiring by default, and evict the oldest session when the per-form limit
is exceeded. `edit_open` and `edit_list` return cleanup metadata for expired
or evicted session IDs.

Run it with:

```sh
printf '%s\n' \
  '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"manual","version":"0.0.0"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}' \
  '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
  '{"jsonrpc":"2.0","id":2,"method":"resources/list","params":{}}' \
  '{"jsonrpc":"2.0","id":3,"method":"prompts/list","params":{}}' \
  | cargo run -p mcp-submit
```

## mcp-form-table

Headless composed MCP server example that declares a custom
`component_shape_gpui` form component shape, exposes a generated form submit
tool, exposes explicit `gpui-table` text filter arguments, and serves both
integrations through one shared MCP server.

Run a local smoke test with:

```sh
cargo run -p mcp-form-table
```

## i18n

`examples/some-lib/i18n.toml` lives next to that package's `Cargo.toml`, and
its `i18n/` directory owns the generated Fluent assets for the example domain
types. The `some-lib` `i18n` module also declares the language enum used by
`some-lib-forms`, so the storybook app consumes one package-owned localization
surface instead of sharing a sibling `examples/i18n` directory.
