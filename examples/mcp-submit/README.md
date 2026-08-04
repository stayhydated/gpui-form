# MCP submit example

A stdio MCP server that exposes generated `gpui-form` tools.

The example covers:

- an async model submit handler with a typed response
- Koruma validation and component-backed field decoding
- holder submission for a form with skipped application context
- generated editor sessions, resources, and prompt templates

Run it from the workspace root:

```sh
cargo run -p mcp-submit
```

Connect an MCP client over stdio and list tools to inspect the generated submit
and `*_edit_*` tools. See
[MCP form tools](https://stayhydated.github.io/gpui-form/book/mcp.html) for the
protocol workflow and registration choices.
