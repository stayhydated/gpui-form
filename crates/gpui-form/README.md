# gpui-form

[![crates.io: gpui-form][crate-badge]][crate]

`gpui-form` is the application-facing facade for generated GPUI forms. It
re-exports `GpuiForm`, schema types, core helpers, and the runtime paths used by
generated code.

## Overview

Default features enable the derive and GPUI runtime. Additional features are:

| Feature | Purpose |
| --- | --- |
| `inventory` | Register concrete derived forms for metadata consumers and prototyping |
| `mcp` | Expose opted-in forms through MCP submit and editor tools; also enables inventory |
| `chrono` | Add MCP schema support for Chrono values when `mcp` is enabled |
| `rust_decimal` | Add MCP schema support for `rust_decimal::Decimal` when `mcp` is enabled |

Use `default-features = false` with `derive` and `mcp` for a headless MCP form
crate.

## Example

```rust
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct Profile {
    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    pub display_name: String,
}
```

[crate-badge]: https://img.shields.io/crates/v/gpui-form.svg?label=gpui-form
[crate]: https://crates.io/crates/gpui-form
