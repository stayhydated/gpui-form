# MCP submit example

`mcp-submit` is a stdio MCP server that demonstrates generated `gpui-form`
submit and editor tools for application developers building headless form
workflows.

## Overview

The example covers:

- an async model submit handler with a typed response
- Koruma validation and component-backed field decoding
- holder submission for a form with skipped application context
- generated editor sessions, resources, and prompt templates

## Example

Run the server from the workspace root, then connect an MCP client over stdio
and list tools to inspect the generated submit and `*_edit_*` tools.

```sh
cargo run -p mcp-submit
```
