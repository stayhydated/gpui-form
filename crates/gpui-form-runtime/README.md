# gpui-form-runtime

Runtime contracts referenced by `gpui-form` generated code.

Most applications start with [`gpui-form`](../gpui-form/README.md), whose
facade re-exports these contracts as `gpui_form::runtime::shape` for generated
`GpuiForm` code. Depend on `gpui-form-runtime` directly only when writing
lower-level integration code: app-owned `ComponentShape`s, wrapper shapes, or
direct value-binding impls.

```toml
[dependencies]
gpui-form = { version = "*", features = ["derive"] }
gpui-form-runtime = "*"
gpui-form-derive = "*"
```

## Component Shapes

`shape::ComponentShape` is the runtime trait implemented by declared component
shapes. `#[derive(GpuiForm)]` also requires `DeclaredComponentShape`, which is
emitted by `gpui_form_derive::component_shape!` and
`#[derive(gpui_form_derive::ComponentShape)]`. Hand-written `ComponentShape`
impls are not accepted as form field shapes.

The derive macro stores `Entity<<Shape as ComponentShape>::State>` in generated
form fields and calls `ComponentShape::new(window, cx)` from the generated
component constructor. Shapes also publish a `ValueStoragePolicy`, which lets
generated value holders inherit whether a non-optional field should use direct
`T` storage or missing-aware `Option<T>` storage. Missing-aware storage is
visible to generated `validate()` and fallible holder-to-model conversion.
`InfallibleValueStorage<T>` marks policies, currently `DirectValueStorage`,
whose storage representation always contains a value.
Generated forms also assert `ComponentShapeFor<T>` for the form-side value
type. Derive and `component_shape!` shapes emit those impls from explicit
`value = ...`, `values(...)`, or `compatibility<Value> where ...;` metadata.
Component-derived shapes may still provide manual `ComponentShapeFor<Value>`
impls outside the derive when custom bounds or diagnostics are needed.
Shapes also publish a `ValueBindingPolicy`: use `NoComponentValueBinding` by
default, or `InheritedComponentValueBinding` when fields should inherit
shape-level `ComponentValueBinding<T>` synchronization.

For owned components, derive on the rendered component and supply `state = ...`;
generated forms store the backing state entity. The rendered component must
provide `new(&gpui::Entity<State>) -> impl gpui::IntoElement` so the shape's
`RenderComponent` associated type can render prototyping output:

```rust
use gpui_form_derive::ComponentShape;

#[derive(ComponentShape)]
#[gpui_form_shape(
    state = EmailInputState,
    value = String,
    value_storage = direct,
    field_suffix = "input"
)]
pub struct EmailInput {
    state: gpui::Entity<EmailInputState>,
}
```

For wrapper shapes around state from another crate, use the proc macro:

```rust
gpui_form_derive::component_shape! {
    pub struct EmailInputShape {
        type State = gpui_component::input::InputState;
        new = gpui_component::input::InputState::new;
        component = gpui_component::input::Input;
        value = String;
        value_storage = direct;
        field_suffix = "input";
    }
}
```

`field_suffix = "..."` metadata and direct
`shape::ComponentPrototyping::field_suffix(...)` calls use the same runtime
contract: the suffix must be a non-empty ASCII identifier suffix. The runtime
API validates this in const evaluation so invalid suffixes fail at the shape
implementation. `ComponentPrototyping` stores the shared
`shape::ComponentSuffix` type, which is also used by schema inventory metadata.

## Value Binding

Implement `shape::ComponentStateValueBinding<T>` on a component-derived shape's
backing state when generated prototyping code should seed a component from a
form value and map component events back into the form value holder. The derived
shape delegates its `shape::ComponentValueBinding<T>` impl through that
state-level contract when `value_binding` metadata is present.
Wrapper shapes declared with `gpui_form_derive::component_shape!` can place the
`ComponentValueBinding<T>` impl inside the macro block; add `value_binding;` to
the macro metadata when the wrapper should publish shape-level value-binding
metadata.
