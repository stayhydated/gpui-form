# gpui-form-mcp

MCP submit, metadata, and headless editor integration for generated
`gpui-form` holders.

Application crates should normally enable the facade feature:

```toml
[dependencies]
gpui-form = { version = "0.5", default-features = false, features = ["derive", "mcp"] }
serde = { version = "1", features = ["derive"] }
```

Keep the default `gpui-form` features when the same crate also renders GPUI
forms. Enable `chrono` or `rust_decimal` alongside `mcp` when exposed fields or
responses use those value types.

Concrete forms opt in with `#[gpui_form(mcp)]`. Register an application-owned
handler with `#[gpui_form::mcp_submit]`, then serve the generated tools with
`gpui_form::mcp::serve_stdio_blocking()` or compose them into an existing MCP
server.

See [MCP form tools](https://stayhydated.github.io/gpui-form/book/mcp.html) for
submit handlers, context-backed forms, editor sessions, resources, prompts,
schemas, and troubleshooting. Full APIs are on
[docs.rs](https://docs.rs/gpui-form-mcp/).
