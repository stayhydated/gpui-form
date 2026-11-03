# Examples

These runnable packages demonstrate the supported `gpui-form` workflows, from
generated GPUI views to scaffold generation and MCP servers.

| Package | Command | Demonstrates |
| --- | --- | --- |
| `mcp-submit` | `cargo run -p mcp-submit` | Generated MCP submit, editor, resource, and prompt surfaces |
| `prototyping` | `cargo run -p prototyping` | Inventory-driven scaffold generation and formatting |
| `some-lib-forms` | `cargo run -p some-lib-forms` | Generated forms in a GPUI Storybook gallery |

`examples/some-lib` contains the shared domain models used by the generated form
gallery. The prototyping command rewrites `examples/some-lib-forms/src/forms`
and mirrors its output under `examples/prototyping/output`.

The unpublished libraries in `examples/components` and `examples/component-derive`
provide file-picker and cascading-selector extensions for the form gallery.
Application repositories own their production widgets.
