# gpui-form

[![crates.io: gpui-form][crate-badge]][crate]

`gpui-form` is the application-facing facade for generated GPUI forms. It
re-exports `GpuiForm`, schema types, core helpers, and the runtime paths used by
generated code.

Requires Rust 1.99 or newer. Component-backed GPUI forms use GPUI Kit 0.7.
Headless MCP consumers can disable default features to omit this crate's
`runtime` integration.

## Features

The default set enables `derive` and `runtime`. Schema and core helpers remain
available without default features. For a headless form server, select `derive`
and `mcp` with defaults disabled.

| Feature | Default | Purpose |
| --- | --- | --- |
| `chrono` | No | Forward Chrono MCP schema support when `mcp` is also enabled. |
| `default` | Yes | Enable `derive` and `runtime`. |
| `derive` | Yes | Re-export `GpuiForm` and provide generated derive support. |
| `inventory` | No | Emit concrete form registrations when `derive` is enabled. |
| `mcp` | No | Expose MCP runtime APIs, enable `inventory`, and extend enabled derives with MCP support. |
| `runtime` | Yes | Re-export the GPUI storage and value-binding contracts used by generated forms. |
| `rust_decimal` | No | Forward Decimal MCP schema support when `mcp` is also enabled. |

## Example

With the default `derive` and `runtime` features, a hidden field needs no
widget or GPUI window. This complete example round-trips application data
through its generated holder; component-backed models use shapes from
[`gpui-form-collection`][collection] or [`gpui-form-component`][component].

```rust
use gpui_form::GpuiForm;

#[derive(GpuiForm)]
pub struct Profile {
    #[gpui_form(hidden)]
    pub display_name: String,
}

let holder = ProfileFormValueHolder::from(Profile {
    display_name: String::from("Ada"),
});
assert_eq!(holder.into_original().display_name, "Ada");
```

The [maintained form gallery][gallery] supplies component models and complete
GPUI views. The [API reference][api] describes the facade exports.

[crate-badge]: https://img.shields.io/crates/v/gpui-form.svg?label=gpui-form
[crate]: https://crates.io/crates/gpui-form
[collection]: https://github.com/stayhydated/gpui-form/blob/master/crates/gpui-form-collection/README.md
[component]: https://github.com/stayhydated/gpui-form/blob/master/crates/gpui-form-component/README.md
[gallery]: https://github.com/stayhydated/gpui-form/blob/master/examples/README.md
[api]: https://docs.rs/gpui-form/latest/gpui_form/
