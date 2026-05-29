---
name: use-gpui-form-component-shapes
description: "Use when Codex needs to add, review, or refactor gpui-form component shape wiring, especially deciding whether an owned rendered component should derive gpui_form_derive::ComponentShape or external component/state pairs should use the function-like gpui_form_derive::component_shape! macro."
---

# Use GPUI Form Component Shapes

## Scope Boundary

Use this skill for user-facing app or integration code that wires custom
components into `#[derive(GpuiForm)]` through `ComponentShape`.
Normal generated `GpuiForm` component fields reach runtime contracts through
`gpui_form::runtime::shape`, but crates that define custom shapes with
`#[derive(ComponentShape)]`, `component_shape!`, or `component_value_binding`
should depend on `gpui-form-runtime` directly because those lower-level
surfaces emit direct runtime paths.

Use `use-gpui-form` for ordinary forms built from existing built-in,
collection, or component-owned shapes. Use this skill when the work crosses
into custom shape ownership, wrapper shape declaration, constructor metadata,
or reusable value-binding behavior.

Do not use it as guidance for changing the proc macro implementation itself.
For derive or macro internals, inspect `crates/gpui-form-derive`,
`crates/gpui-form-codegen`, the relevant `docs/ARCHITECTURE.md`, and
repository instructions.

## Decision Rule

First classify the component ownership:

- Owned rendered component: when the app or integration crate owns the rendered
  component and backing state, derive `gpui_form_derive::ComponentShape` on the
  rendered component with `state = ...`.
- External component/state pair: when the state type or UI component lives in
  another crate, declare a local wrapper shape with the function-like
  `gpui_form_derive::component_shape!` proc macro. This avoids orphan-rule
  problems and gives the local crate a type that owns the `ComponentShape`
  and `DeclaredComponentShape` implementations.
- Existing collection shape: when a suitable `gpui_form_collection::*` shape
  already exists, use it directly instead of wrapping it again.

If a prompt says "functional macro" or "functional! macro", interpret that as
the function-like `gpui_form_derive::component_shape!` proc macro unless the
codebase has a different local macro with that exact name.

## Owned Component Pattern

Use `#[derive(ComponentShape)]` when the rendered component type is local:

```rust
use gpui_form_derive::ComponentShape;

#[derive(ComponentShape)]
#[gpui_form_shape(state = TagsInputState, value = Vec<String>, field_suffix = "input")]
pub struct TagsInput {
    state: gpui::Entity<TagsInputState>,
}

pub struct TagsInputState;
```

Use the rendered component type as the field shape:

```rust
#[derive(Clone, Debug, Default, gpui_form::GpuiForm)]
pub struct PostEditor {
    #[gpui_form(component(TagsInput))]
    pub tags: Option<Vec<String>>,
}
```

Metadata rules:

- `state = ...` is required.
- Omit `new` when the state has `State::new(window, cx)`.
- Use `new = some_function` or `new = |window, cx| ...` when the macro should
  pass `(window, cx)` for you.
- Use a full constructor expression such as
  `new = Self::with_mode(window, cx, Mode::Compact)` when the expression should
  be emitted as written.
- Add `component = ...` only when generated metadata should use a path-like
  render component type different from the derived type.
- Add `value = ...` or `values(...)` for each form-side value type the shape
  supports, unless you will implement `ComponentShapeFor<Value>` manually.
- Add `value_storage = direct` when the component can synthesize a default value
  and non-optional fields should use direct `T` value-holder storage by default.
- Add `value_binding` when the derived shape should delegate value binding
  through the backing state's `ComponentStateValueBinding<T>` implementation.
- Add `field_suffix = "..."` when prototyping output should use a stable
  generated prototyping suffix. Generated form field identifiers still derive
  from the shape type name. The suffix must be a non-empty ASCII identifier
  suffix; direct `ComponentPrototyping::field_suffix(...)` calls
  validate the same contract.

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

Use the wrapper shape with `#[gpui_form(component(...))]`:

```rust
#[derive(Clone, Debug, Default, gpui_form::GpuiForm)]
pub struct ContactForm {
    #[gpui_form(component(EmailInputShape))]
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
        value = T;
        value_storage = direct;
        field_suffix = "input";
        value_binding;

        impl<T> gpui_form_runtime::shape::ComponentValueBinding<T> for Input<T>
        where
            T: std::str::FromStr + ToString + 'static,
        {
            type Event = gpui_component::input::InputEvent;
            /* seed_value_binding_state and form_value_change */
        }
    }
}
```

The macro accepts `new`, `component`, `value = ...`, `values(...)`,
`value_storage = require_value|direct`, `value_binding`, and `field_suffix`
metadata, plus `type State = ...`.
If `new` is omitted, it calls `<State>::new(window, cx)`. Use
`value_storage = direct` on reusable wrappers that can seed or synthesize a
missing value; consuming field attributes do not accept `value_storage = ...`.
Use a path-like `component = ...` value and a non-empty ASCII identifier suffix
for `field_suffix = "..."`. Separate metadata entries with semicolons. The
block may also contain `impl` items.
Use `value = ...` / `values(...)` for simple compatibility impls, or omit value
metadata and put manual `ComponentShapeFor<Value>` impls in the block when the
shape needs custom bounds or diagnostics.

Do not hand-write `gpui_form_runtime::shape::ComponentShape` for a
`#[gpui_form(component(...))]` field shape. `#[derive(GpuiForm)]` requires the
`DeclaredComponentShape` marker emitted by `#[derive(ComponentShape)]` or
`component_shape!`.

## Value Binding

If generated forms should keep the form value and component state synchronized,
put `#[gpui_form_derive::component_value_binding]` on a
`gpui_form_runtime::shape::ComponentValueBinding<T>` impl for the backing state.
The attribute compiles it as the state-level binding used by component-derived
shapes that opt in with `#[gpui_form_shape(..., value_binding)]`. For wrapper
shapes created by `component_shape!`, prefer putting the `ComponentValueBinding<T>`
impl inside the macro block and add `value_binding;` to the macro metadata when
the wrapper should publish shape-level value-binding metadata.

Value binding is shape-owned. Use a component-derived shape with explicit
`value_binding` or a wrapper shape with both `value_binding;` and a
`ComponentValueBinding<T>` impl when generated forms should inherit
synchronization.

## Checks

After code edits, run the narrowest command that proves the affected crate or
example still compiles. For changes to derive syntax, macro examples, or shape
metadata, prefer targeted tests or checks for the crate where the shape is used.
