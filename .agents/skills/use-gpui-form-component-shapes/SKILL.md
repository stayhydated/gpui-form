---
name: use-gpui-form-component-shapes
description: "Use when Codex needs to add, review, or refactor gpui-form component shape wiring, especially deciding whether owned component state should derive gpui_form_derive::ComponentShape or external component/state pairs should use the function-like gpui_form_derive::component_shape! macro."
---

# Use GPUI Form Component Shapes

## Scope Boundary

Use this skill for user-facing app or integration code that wires custom
components into `#[derive(GpuiForm)]` through `ComponentShape`.
Ensure the consuming crate depends on `gpui-form-runtime`, because generated
code references `gpui_form_runtime::shape`.

Do not use it as guidance for changing the proc macro implementation itself.
For derive or macro internals, inspect `crates/gpui-form-derive`,
`crates/gpui-form-codegen`, the relevant `docs/ARCHITECTURE.md`, and
repository instructions.

## Decision Rule

First classify the component state type:

- Owned component state: when the app or integration crate owns the state type,
  derive `gpui_form_derive::ComponentShape` directly on that state type.
- External component/state pair: when the state type or UI component lives in
  another crate, declare a local wrapper shape with the function-like
  `gpui_form_derive::component_shape!` proc macro. This avoids orphan-rule
  problems and gives the local crate a type that owns the `ComponentShape`
  implementation.
- Existing collection shape: when a suitable `gpui_form_collection::*` shape
  already exists, use it directly instead of wrapping it again.

If a prompt says "functional macro" or "functional! macro", interpret that as
the function-like `gpui_form_derive::component_shape!` proc macro unless the
codebase has a different local macro with that exact name.

## Owned State Pattern

Use `#[derive(ComponentShape)]` when the state type is local:

```rust
use gpui_form_derive::ComponentShape;

#[derive(Clone, Debug, ComponentShape)]
#[gpui_form_shape(
    component = crate::widgets::TagsInput,
    field_suffix = "input"
)]
pub struct TagsInputState;
```

Use the state type itself as the field shape:

```rust
#[derive(Clone, Debug, Default, gpui_form::GpuiForm)]
pub struct PostEditor {
    #[gpui_form(TagsInputState)]
    pub tags: Option<Vec<String>>,
}
```

Metadata rules:

- Omit `new` when the state has `Self::new(window, cx)`.
- Use `new = some_function` or `new = |window, cx| ...` when the macro should
  pass `(window, cx)` for you.
- Use a full constructor expression such as
  `new = Self::with_mode(window, cx, Mode::Compact)` when the expression should
  be emitted as written.
- Add `component = ...` when generated metadata should know the render
  component path.
- Add bare `value_binding` only when the shape publishes a reusable
  `ComponentValueBinding<T>` contract.
- Add `field_suffix = "..."` when prototyping output should use a stable
  generated field/helper suffix.

## External State Pattern

Use `gpui_form_derive::component_shape!` when wrapping state from another crate:

```rust
gpui_form_derive::component_shape! {
    pub struct EmailInputShape {
        type State = gpui_component::input::InputState;
        component = gpui_component::input::Input;
        field_suffix = "input";
    }
}
```

Use the wrapper shape in `#[gpui_form(...)]`:

```rust
#[derive(Clone, Debug, Default, gpui_form::GpuiForm)]
pub struct ContactForm {
    #[gpui_form(EmailInputShape)]
    pub email: String,
}
```

For generic external wrappers, put the generic parameters and bounds on the
local shape:

```rust
gpui_form_derive::component_shape! {
    pub struct Input<T = String>
    where
        T: std::str::FromStr + ToString + 'static,
    {
        type State = gpui_component::input::InputState;
        new = |window, cx| gpui_component::input::InputState::new(window, cx)
            .validate(|value, _| value.parse::<T>().is_ok());
        component = gpui_component::input::Input;
        value_binding;
        field_suffix = "input";
    }
}
```

The macro accepts the same metadata as the derive form, plus
`type State = ...`. If `new` is omitted, it calls `<State>::new(window, cx)`.
Options may be separated with semicolons or commas.

## Value Binding

If generated forms should keep the form value and component state synchronized,
implement `gpui_form_runtime::shape::ComponentValueBinding<T>` for the shape.
For owned states, the shape is the state type itself. For external states, the
shape is the local wrapper created by `component_shape!`.

Set bare `value_binding` metadata only when that binding should be reusable by
all fields that use the shape. For one-off fields, prefer adding
`.value_binding()` in the field shape expression.

## Checks

After code edits, run the narrowest command that proves the affected crate or
example still compiles. For changes to derive syntax, macro examples, or shape
metadata, prefer targeted tests or checks for the crate where the shape is used.
