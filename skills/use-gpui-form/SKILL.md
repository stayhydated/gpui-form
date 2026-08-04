---
name: use-gpui-form
description: "Use when adding, reviewing, or refactoring application code that consumes gpui-form: #[derive(GpuiForm)], field intents, defaults and value conversions, generated fields/components/value holders, built-in shapes, SelectItem or InfiniteSelect enums, Koruma validation, inventory scaffolding, or MCP submit/editor tools. Route custom shape authoring to use-gpui-form-component-shapes."
---

# Use GPUI Form

## Scope

Apply this skill to consumer-side form work. Keep application code centered on
its models, generated form state, selected component shapes, and submission
logic.

Use `use-gpui-form-component-shapes` when the task defines or adapts an
application-owned or third-party component shape. Use the workspace guidance
for proc-macro internals, generators, releases, or repository maintenance.

## Workflow

1. Inspect the consuming crate's `Cargo.toml`, model, existing form code, and
   GPUI initialization.
2. Depend on `gpui-form` as the facade. Add `gpui-form-collection`,
   `gpui-form-collection-derive`, or `gpui-form-component` only for shapes and
   derives the form uses.
3. Derive `GpuiForm` on the application model. Give every field exactly one
   `component(...)`, `hidden`, or `skip` intent.
4. Choose a shape compatible with the form-side value. Put `default = ...` and
   `value(...)` inside the selected intent.
5. Own the generated holder, component entities, and subscriptions in the GPUI
   view, or use inventory prototyping to generate that wiring.
6. Validate the holder when enabled, then call the generated conversion method
   appropriate to the field contracts.
7. For MCP, opt in only concrete inventory-backed forms and route calls to
   application-owned handlers or context submitters.
8. Run the narrowest consumer check that covers the changed form.

## Core rules

- Use `gpui_form::GpuiForm`. Generated component code resolves runtime
  contracts through `gpui_form::runtime::shape`.
- For `Profile`, expect `ProfileFormField`, `ProfileFormFields`,
  `ProfileFormComponents`, and `ProfileFormValueHolder`.
- Use `hidden` for holder values without widgets. Use `skip` for source values
  the form does not own; pass skipped values to `into_original(...)`.
- In `value(type = T, ...)`, write the base form-side type `T` rather than
  `Option<T>`. Source optionality controls holder optionality.
- Use `into_source` for infallible reverse conversion,
  `try_into_source` for `Result`, and `value(koruma_newtype)` for a Koruma
  newtype's inner value.
- Use `try_into_original()` when a required value or reverse conversion can
  fail. Use `into_original()` for statically infallible forms.
- Generic forms use `#[gpui_form(no_inventory)]` when inventory is enabled.
  MCP forms must be concrete and inventory-backed.

## Minimal form

```rust
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct Profile {
    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    pub name: String,

    #[gpui_form(hidden(default = false))]
    pub imported: bool,
}
```

## Route by task

| Task | Guidance |
|---|---|
| Text, select, combobox, toggle, numeric, slider, color, date, or OTP field | Choose a `gpui-form-collection` shape |
| Localized date/file field or cascading enum tree | Use `gpui-form-component` |
| Custom parsing or formatting in a normal input | Implement `ParsedInputConfig<T>` and use `ParsedInput::<_, Config>` |
| Field-side type differs from the model | Use intent-scoped `value(type = ..., from_source = ..., into_source/try_into_source = ...)` |
| Validation | Add `#[gpui_form(koruma)]` or `koruma(fluent)` and validate the holder |
| Generated GPUI wiring | Enable `inventory` and use `gpui-form-prototyping-core` |
| Structured form tools | Enable `mcp` and use `#[gpui_form(mcp)]` plus an application-owned submit path |
| App-owned widget or external state wrapper | Switch to `use-gpui-form-component-shapes` |

## MCP guardrails

- `#[gpui_form::mcp_submit]` applies to a free synchronous or asynchronous
  function with one owned source-model or generated-holder parameter.
- Return `Result<T, E>` with a serializable MCP-schema response and a
  displayable error.
- Use a holder parameter when skipped fields require application context.
- Use `context(...)` plus `McpContextSubmit` or `submit(...)` for shared
  runtime state.
- Generated editor sessions are headless holder state, not live GPUI entity
  mutation.

## Reference selection

Read [`references/api-map.md`](references/api-map.md) when exact dependency
features, attribute syntax, shape selection, conversion behavior, prototyping,
or MCP registration details matter. Load only the relevant section.
