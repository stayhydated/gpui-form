# gpui-form-derive

[![Codecov: gpui-form-derive][codecov-badge]][codecov]
[![crates.io: gpui-form-derive][crate-badge]][crate]

`gpui-form-derive` provides the procedural macros behind `gpui-form` for
derive-only and lower-level integrations.

The crate exports:

- `#[derive(GpuiForm)]`
- `#[gpui_form_derive::mcp_submit]` with the `mcp` feature

Most applications use the [`gpui-form` facade][gpui-form], which re-exports the
applicable macros and the runtime paths generated code expects. The `inventory`
feature emits concrete form registrations; `mcp` adds MCP form expansion and
enables `inventory`.

[codecov-badge]: https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg?branch=master&component=gpui-form-derive
[codecov]: https://codecov.io/github/stayhydated/gpui-form
[crate-badge]: https://img.shields.io/crates/v/gpui-form-derive.svg?label=gpui-form-derive
[crate]: https://crates.io/crates/gpui-form-derive
[gpui-form]: https://github.com/stayhydated/gpui-form/blob/master/crates/gpui-form/README.md
