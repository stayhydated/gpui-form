---
name: use-gpui-form-component-shapes
description: "Use when Codex needs to add, review, or refactor gpui-form component shape wiring, especially deciding whether an owned rendered component should derive component_shape_gpui::GpuiComponentShape or external component/state pairs should use component_shape_gpui::component_shape!."
---

# Use GPUI Form Component Shapes

## Scope Boundary

Use this skill for user-facing app or integration code that wires custom
components into `#[derive(GpuiForm)]` through `GpuiComponentShape`.
Normal generated `GpuiForm` component fields reach runtime contracts through
`gpui_form::runtime::shape`, but crates that define custom shapes with
`#[derive(component_shape_gpui::GpuiComponentShape)]`,
`component_shape_gpui::component_shape!`, or direct runtime value-binding impls
should depend on `gpui-form-runtime` directly because those lower-level surfaces
emit direct runtime paths.

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
  component and backing state, derive `component_shape_gpui::GpuiComponentShape`
  on the rendered component with `state = ...`.
- External component/state pair: when the state type or UI component lives in
  another crate, declare a local wrapper shape with the function-like
  `component_shape_gpui::component_shape!` proc macro. This avoids orphan-rule
  problems and gives the local crate a type that owns the `GpuiComponentShape`
  and `DeclaredGpuiComponentShape` implementations.
- Existing collection shape: when a suitable `gpui_form_collection::*` shape
  already exists, use it directly instead of wrapping it again.

If a prompt says "functional macro" or "functional! macro", interpret that as
the function-like `component_shape_gpui::component_shape!` proc macro unless the
codebase has a different local macro with that exact name.

## Owned Component Pattern

Use `#[derive(GpuiComponentShape)]` when the rendered component type is local:

```rust
use component_shape_gpui::GpuiComponentShape;

#[derive(GpuiComponentShape)]
#[gpui_component_shape(state = TagsInputState, value = Vec<String>, field_suffix = "input")]
pub struct TagsInput {
    state: gpui::Entity<TagsInputState>,
}

impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for TagsInput {
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
}

pub struct TagsInputState;
```

The rendered component type must provide
`new(&gpui::Entity<TagsInputState>) -> impl gpui::IntoElement`; the derive uses
that constructor through the generated `RenderComponent` associated type.

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
- Add `value = ...` or `values(...)` once for each form-side value type the
  component-derived shape supports, unless you will implement
  `GpuiComponentShapeFor<Value>` manually outside the derive.
- Implement `GpuiFormComponentShapePolicy` for the shape. Use
  `DirectValueStorage` when the component can synthesize a default value and
  non-optional fields should use direct `T` value-holder storage by default;
  use `RequiredValueStorage` when the holder should represent missing required
  values as `Option<T>`.
- Add `value_binding` when the derived shape should delegate value binding
  through the backing state's `GpuiComponentStateValueBinding<T>`
  implementation.
- Add `field_suffix = "..."` when prototyping output should use a stable
  generated prototyping suffix for DOM IDs, event handlers, and helper names.
  Generated `FormFields` members and `FormComponents` constructors use the
  source field name. The suffix must be a non-empty ASCII identifier suffix;
  direct `ComponentPrototyping::field_suffix(...)` calls
  validate the same contract.

## External State Pattern

Use `component_shape_gpui::component_shape!` when wrapping state from another crate:

```rust
component_shape_gpui::component_shape! {
    pub struct EmailInputShape {
        type State = gpui_component::input::InputState;
        component = gpui_component::input::Input;
        value = String;
        field_suffix = "input";
    }
}

impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for EmailInputShape {
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
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
component_shape_gpui::component_shape! {
    pub struct Input<T = String>
    where
        T: std::str::FromStr + ToString + 'static,
    {
        type State = gpui_component::input::InputState;
        new = |window, cx| gpui_component::input::InputState::new(window, cx)
            .validate(|value, _| value.parse::<T>().is_ok());
        component = gpui_component::input::Input;
        value = T;
        field_suffix = "input";
        value_binding;

        impl<T> gpui_form_runtime::shape::GpuiComponentValueBinding<T> for Input<T>
        where
            T: std::str::FromStr + ToString + 'static,
        {
            type Event = gpui_component::input::InputEvent;
            /* seed_value_binding_state and form_value_change */
        }
    }
}

impl<T> gpui_form_runtime::shape::GpuiFormComponentShapePolicy for Input<T>
where
    T: std::str::FromStr + ToString + 'static,
{
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
}
```

The macro accepts `new`, `component`, `value = ...`, `values(...)`,
`value_binding`, and `field_suffix` metadata, plus `type State = ...`. If `new`
is omitted, it calls `<State>::new(window, cx)`. Use
`GpuiFormComponentShapePolicy` on reusable wrappers to select
`DirectValueStorage` or `RequiredValueStorage`; consuming field attributes do
not accept storage-policy metadata.
Use a path-like `component = ...` value and a non-empty ASCII identifier suffix
for `field_suffix = "..."`. Separate metadata entries with semicolons. The
block may also contain `impl` items.
Use `value = ...` / `values(...)` for form-side value-type impls. Do not put
manual `GpuiComponentShapeFor<Value>` impls inside `component_shape!`; the macro
rejects them so form-side value support stays explicit in metadata. Add at
least one value-type declaration.

Do not hand-write `gpui_form_runtime::shape::GpuiComponentShape` for a
`#[gpui_form(component(...))]` field shape. `#[derive(GpuiForm)]` requires the
`DeclaredGpuiComponentShape` marker emitted by
`#[derive(component_shape_gpui::GpuiComponentShape)]` or
`component_shape_gpui::component_shape!`.

## Value Binding

If generated forms should keep the form value and component state synchronized,
implement `gpui_form_runtime::shape::GpuiComponentStateValueBinding<T>`
directly on the backing state for component-derived shapes that opt in with
`#[gpui_component_shape(..., value_binding)]`. For wrapper shapes created by
`component_shape!`, prefer putting the `GpuiComponentValueBinding<T>` impl
inside the macro block and add `value_binding;` to the macro metadata when the
wrapper should publish shape-level value-binding metadata.

Value binding is shape-owned. Use a component-derived shape with explicit
`value_binding` or a wrapper shape with both `value_binding;` and a
`GpuiComponentValueBinding<T>` impl when generated forms should inherit
synchronization.

## Checks

After code edits, run the narrowest command that proves the affected crate or
example compiles. For changes to derive syntax, macro examples, or shape
metadata, prefer targeted tests or checks for the crate where the shape is used.
