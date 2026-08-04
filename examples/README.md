# Examples

Runnable examples for the current `gpui-form` workflows.

| Package | Command | Demonstrates |
|---|---|---|
| `some-lib-forms` | `cargo run -p some-lib-forms` | Generated forms in a GPUI Storybook gallery |
| `gpui-form-component-story` | `cargo run -p gpui-form-component-story` | Reusable date, file, and infinite-select components |
| `prototyping` | `cargo run -p prototyping` | Inventory-driven scaffold generation and formatting |
| `mcp-submit` | `cargo run -p mcp-submit` | Generated MCP submit, editor, resource, and prompt surfaces |

`examples/some-lib` contains the shared domain models used by the generated
form gallery. The prototyping command rewrites
`examples/some-lib-forms/src/forms` and mirrors its output under
`examples/prototyping/output`.

For task-oriented guidance, use the
[`gpui-form` user guide](https://stayhydated.github.io/gpui-form/book/).
