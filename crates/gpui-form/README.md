[![Build Status](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml)
[![Docs](https://docs.rs/gpui-form/badge.svg)](https://docs.rs/gpui-form/)
[![Crates.io](https://img.shields.io/crates/v/gpui-form.svg)](https://crates.io/crates/gpui-form)

# gpui-form

The public facade for generated GPUI forms. It re-exports `GpuiForm`, schema
types, core helpers, and the runtime paths referenced by generated code.

```toml
[dependencies]
gpui-form = "0.6"
```

Default features enable the derive and GPUI runtime. Additional features are:

| Feature | Purpose |
|---|---|
| `inventory` | Register concrete derived forms for metadata consumers and prototyping |
| `mcp` | Expose opted-in forms through MCP submit and editor tools; also enables inventory |
| `chrono` | Add MCP schema support for Chrono values when `mcp` is enabled |
| `rust_decimal` | Add MCP schema support for `rust_decimal::Decimal` when `mcp` is enabled |

Use `default-features = false` with `features = ["derive", "mcp"]` for a
headless MCP-only form crate.

See the [user guide](https://stayhydated.github.io/gpui-form/book/) for field
intents, generated types, validation, component shapes, MCP, and prototyping.
API details are on [docs.rs](https://docs.rs/gpui-form/).
