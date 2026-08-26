# Introduction

`gpui-form` turns an application-owned Rust model into typed GPUI form state.
Derive `GpuiForm`, give each field one form intent, and use the generated
holder to validate and reconstruct the model.

This guide is for Rust application developers who already use
`gpui-component` in a GPUI application. The workspace supports Rust 1.98.
Keep the application's `gpui` and `gpui-component` dependencies aligned with
the versions in [Getting started](getting_started.md).

## Mental model

A derived form separates the source model from the state owned by the view:

1. The source struct defines the values the application accepts.
2. Field intents decide which values have components, remain hidden, or stay
   under application control.
3. Component shapes define construction, rendering, value binding, and holder
   storage.
4. The generated holder collects edits, exposes validation, and reconstructs
   the source model.

For a source struct named `UserProfile`, the derive generates:

| Type | Purpose |
|---|---|
| `UserProfileFormField` | Typed identity for component-backed fields |
| `UserProfileFormFields` | GPUI entities owned by the form view |
| `UserProfileFormComponents` | Constructors for those entities |
| `UserProfileFormValueHolder` | Editable values, defaults, validation, and model conversion |

## Choose a path

Start with [Getting started](getting_started.md) to derive and wire a form.
Use [Field intents and generated types](field_intents.md) for hidden, skipped,
or converted values. Continue to [Component shapes](component_shapes.md) to
select widgets and [Validation and conversion](validation.md) before
submission.

[MCP form tools](mcp.md) describes structured submit and headless editing.
[Prototyping](prototyping.md) covers generating complete GPUI wiring from
inventory metadata.
