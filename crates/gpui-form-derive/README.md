# gpui-form-derive

Procedural macros behind `gpui-form`:

- `#[derive(GpuiForm)]`
- `#[gpui_form_derive::mcp_submit]` with the `mcp` feature

Most users should depend on the
[`gpui-form` facade](../gpui-form/README.md), which re-exports the applicable
macros and the runtime paths generated code expects. Use this crate directly
only in derive-only integrations.

The `inventory` feature emits concrete form registrations. The `mcp` feature
adds MCP form expansion and implies `inventory`.

See the [user guide](https://stayhydated.github.io/gpui-form/book/) for the
supported field intents and workflows, and
[docs.rs](https://docs.rs/gpui-form-derive/) for macro API details.
