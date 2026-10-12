# gpui-form-derive

[![Codecov: gpui-form-derive][codecov-badge]][codecov]
[![crates.io: gpui-form-derive][crate-badge]][crate]

`gpui-form-derive` provides the procedural macros behind `gpui-form` for
derive-only and lower-level integrations.

Requires Rust 1.99 or newer.

Most applications enable `derive` on the [`gpui-form` facade][gpui-form],
which re-exports the macros and generated runtime paths. Depend directly on this
crate only when managing those integration paths yourself.

## Features

No features are enabled by default. `GpuiForm` is always exported; the `mcp`
feature also exports `#[gpui_form_derive::mcp_submit]`.

| Feature | Default | Purpose |
| --- | --- | --- |
| `default` | Yes | Enable no optional macro expansion. |
| `inventory` | No | Emit inventory registrations for concrete forms. |
| `mcp` | No | Enable `inventory`, MCP form expansion, and submit-handler macros. |

[codecov-badge]: https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg?branch=master&component=gpui-form-derive
[codecov]: https://codecov.io/github/stayhydated/gpui-form
[crate-badge]: https://img.shields.io/crates/v/gpui-form-derive.svg?label=gpui-form-derive
[crate]: https://crates.io/crates/gpui-form-derive
[gpui-form]: https://github.com/stayhydated/gpui-form/blob/master/crates/gpui-form/README.md
