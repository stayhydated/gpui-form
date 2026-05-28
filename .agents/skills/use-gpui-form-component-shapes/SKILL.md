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
`#[derive(ComponentShape)]`, `component_shape!`, `component_value_binding`, or
manual runtime trait impls should depend on `gpui-form-runtime` directly
because those lower-level surfaces emit direct runtime paths.

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
  implementation.
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
#[gpui_form_shape(state = TagsInputState, field_suffix = "input")]
pub struct TagsInput {
    state: gpui::Entity<TagsInputState>,
}

pub struct TagsInputState;
```

Use the rendered component type as the field shape:

```rust
#[derive(Clone, Debug, Default, gpui_form::GpuiForm)]
pub struct PostEditor {
    #[gpui_form(TagsInput)]
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
- Add `requires_value = false` when the component can synthesize a missing value
  and non-optional fields should use direct `T` value-holder storage by default.
- Add `value_binding` when the derived shape should delegate value binding
  through the backing state's `ComponentStateValueBinding<T>` implementation.
- Add `field_suffix = "..."` when prototyping output should use a stable
  generated field/helper suffix. The suffix must be a non-empty identifier
  suffix.

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
        requires_value = false;
        field_suffix = "input";

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

The macro accepts the same metadata as the derive form, except derive-only
`value_binding`, plus `type State = ...`.
If `new` is omitted, it calls `<State>::new(window, cx)`. Use
`requires_value = false` on reusable wrappers that can seed or synthesize a
missing value; consuming field attributes do not accept `.requires_value(...)`.
Use a path-like `component = ...` value and a non-empty identifier suffix for
`field_suffix = "..."`. Separate metadata entries with semicolons. The block
may also contain `impl` items.

When implementing `gpui_form_runtime::shape::ComponentShape` by hand, include
both policy associated types. Use `NoComponentValueBinding` unless the shape
should inherit reusable `ComponentValueBinding<T>` impls by default; use
`InheritedComponentValueBinding` for that inherited path.

## Value Binding

If generated forms should keep the form value and component state synchronized,
put `#[gpui_form_derive::component_value_binding]` on a
`gpui_form_runtime::shape::ComponentValueBinding<T>` impl for the backing state.
The attribute compiles it as the state-level binding used by component-derived
shapes that opt in with `#[gpui_form_shape(..., value_binding)]`. For wrapper
shapes created by `component_shape!`, prefer putting the `ComponentValueBinding<T>`
impl inside the macro block; nested binding impls are emitted with the shape and
automatically publish shape-level value-binding metadata. The impl target must
be the shape declared by the macro.

Value binding is shape-owned. Use a component-derived shape with explicit
`value_binding` or a wrapper shape with a nested `ComponentValueBinding<T>` impl
when generated forms should inherit synchronization.

## Checks

After code edits, run the narrowest command that proves the affected crate or
example still compiles. For changes to derive syntax, macro examples, or shape
metadata, prefer targeted tests or checks for the crate where the shape is used.
