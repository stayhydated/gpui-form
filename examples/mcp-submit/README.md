# MCP Submit Example

Small stdio MCP server that exposes generated `gpui-form` submit flows as
tools.

It demonstrates:

- `#[gpui_form::mcp_submit]` for an async model handler returning
  `Result<serde_json::Value, String>`
- Koruma validation before submit
- a component-backed field decoded without starting GPUI
- handler-mode inference from the first parameter: source models use model
  submission, and generated `*FormValueHolder` parameters use holder submission
  for skipped-field forms whose missing context is application-owned
- explicit per-tool MCP names, titles, and descriptions with
  `#[gpui_form(mcp(...))]`
- headless edit-session tools for a support ticket value holder:
  `mcp_submit_support_ticket_edit_open`, `_edit_read`, `_edit_patch`,
  `_edit_validate`, and `_edit_close`

Run a tool list request:

```sh
printf '%s\n' \
  '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"manual","version":"0.0.0"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}' \
  '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
  | cargo run -p mcp-submit
```

Call the support ticket tool:

```sh
printf '%s\n' \
  '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"manual","version":"0.0.0"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"mcp_submit_support_ticket","arguments":{"title":"Cannot log in","details":"SSO loops back to the login page","urgent":true}}}' \
  | cargo run -p mcp-submit
```

Call the skipped-field holder tool:

```sh
printf '%s\n' \
  '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"manual","version":"0.0.0"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}' \
  '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"mcp_submit_tenant_note","arguments":{"body":"Roll out beta flag"}}}' \
  | cargo run -p mcp-submit
```
