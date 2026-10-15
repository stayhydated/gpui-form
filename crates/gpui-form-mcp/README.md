# gpui-form-mcp

[![Codecov: gpui-form-mcp][codecov-badge]][codecov]
[![crates.io: gpui-form-mcp][crate-badge]][crate]

`gpui-form-mcp` provides MCP submit, metadata, and headless editor integration
for generated `gpui-form` holders. Applications normally enable `mcp` on
the [`gpui-form` facade][gpui-form]; depend directly on this experimental crate
when composing an existing MCP host.

Requires Rust 1.99 or newer.

## Overview

Concrete forms opt in with `#[gpui_form(mcp)]`; application-owned submit
handlers and persistent editor sessions are served or composed through the
[public MCP API][api]. The [complete server example][example] demonstrates
registration and stdio transport.

## Features

No features are enabled by default. Facade consumers select the corresponding
`gpui-form` features alongside `mcp`.

| Feature | Default | Purpose |
| --- | --- | --- |
| `chrono` | No | Enable Chrono schemas through `component-shape-mcp/chrono`. |
| `rust_decimal` | No | Enable Decimal schemas through `component-shape-mcp/rust_decimal`. |

[codecov-badge]: https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg?branch=master&component=gpui-form-mcp
[codecov]: https://codecov.io/github/stayhydated/gpui-form
[crate-badge]: https://img.shields.io/crates/v/gpui-form-mcp.svg?label=gpui-form-mcp
[crate]: https://crates.io/crates/gpui-form-mcp
[gpui-form]: https://github.com/stayhydated/gpui-form/blob/master/crates/gpui-form/README.md
[api]: https://docs.rs/gpui-form-mcp/latest/gpui_form_mcp/
[example]: https://github.com/stayhydated/gpui-form/blob/master/examples/mcp-submit/README.md
