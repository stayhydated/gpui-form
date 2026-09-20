# gpui-form-mcp

[![Codecov: gpui-form-mcp][codecov-badge]][codecov]
[![crates.io: gpui-form-mcp][crate-badge]][crate]

`gpui-form-mcp` provides MCP submit, metadata, and headless editor integration
for generated `gpui-form` holders.

## Overview

Application crates enable `gpui-form/mcp`. Keep the default features when the
same crate renders GPUI forms; for a headless server, disable default features
and enable `derive` and `mcp`. Enable `chrono` or `rust_decimal` alongside `mcp`
when exposed fields or responses use those value types.

Concrete forms opt in with `#[gpui_form(mcp)]`. Register an application-owned
handler with `#[gpui_form::mcp_submit]`, then serve the generated tools with
`gpui_form::mcp::serve_stdio_blocking()` or compose them into an existing MCP
server.

Use `gpui_form::mcp::tool_registry()` or
`tool_registry_with_options(...)` when the host assembles the same
inventory-discovered definitions and handlers independently. MCP servers retain
editor sessions across calls.

[codecov-badge]: https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg?branch=master&component=gpui-form-mcp
[codecov]: https://codecov.io/github/stayhydated/gpui-form
[crate-badge]: https://img.shields.io/crates/v/gpui-form-mcp.svg?label=gpui-form-mcp
[crate]: https://crates.io/crates/gpui-form-mcp
