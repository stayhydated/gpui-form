# gpui-form-runtime

Runtime contracts referenced by `gpui-form` generated code.

Most applications start with [`gpui-form`](../gpui-form/README.md), whose
facade re-exports these contracts as `gpui_form::runtime::shape` for generated
`GpuiForm` code. Depend on `gpui-form-runtime` directly only when writing
lower-level integration code: app-owned `GpuiComponentShape`s, wrapper shapes, or
direct value-binding impls.

```toml
[dependencies]
gpui-form = { version = "*", features = ["derive"] }
gpui-form-runtime = "*"
gpui-form-derive = "*"
```

## Component Shapes

`component-shape-gpui` owns the reusable GPUI shape runtime and macros:
`GpuiComponentShape`, `GpuiComponentRender`, `GpuiComponentShapeFor<T>`,
`GpuiComponentShapeBuilder<Shape>`, `GpuiComponentValueBinding<T>`,
`component_shape!`, and
`#[derive(GpuiComponentShape)]`. This crate re-exports those contracts and adds
the `gpui-form` storage policy contract used by generated forms.

`shape::GpuiComponentShape` is the GPUI construction trait for declared component
shapes. `#[derive(GpuiForm)]` also requires `DeclaredGpuiComponentShape`; GPUI
shapes declared through `component-shape-gpui` satisfy that marker and become
form field shapes when they implement `GpuiFormComponentShapePolicy` for local
form storage metadata. Hand-written form-only `GpuiComponentShape` impls are not
accepted as form field shapes.

The derive macro stores `Entity<<Shape as GpuiComponentShape>::State>` in generated
form fields. Plain component attributes call `GpuiComponentShape::new(window, cx)`;
configured component attributes call `build_component_shape` with a
`GpuiComponentShapeBuilder<Shape>` expression for the same base shape.
`gpui-form`, not `component-shape-gpui`, owns
`ValueStoragePolicy`, which lets generated value holders inherit whether a
non-optional field should use direct `T` storage or missing-aware `Option<T>`
storage. Missing-aware storage is
visible to generated `validate()` and fallible holder-to-model conversion.
`InfallibleValueStorage<T>` marks storage policies whose representation always
contains a value.
Generated forms also assert `GpuiComponentShapeFor<T>` for the form-side value
type. Derive and `component_shape!` shapes emit those impls from explicit
`value = ...` or `values(...)` metadata, or infer them from value-binding
contracts when the declaration opts into value binding.
Component-derived shapes may still provide manual `GpuiComponentShapeFor<Value>`
impls outside the derive when custom bounds or diagnostics are needed.
Shapes also publish a `ComponentShapeMetadata::CAPABILITIES`: use `no value-binding capability` by
default, or `inherited value-binding capability` when fields should inherit
shape-level `GpuiComponentValueBinding<T>` synchronization.

For owned components, derive on the rendered component. The derive infers
`EmailInputState` from `EmailInput` when `state = ...` is omitted; supply
`state = ...` when the backing state does not follow that convention. Generated
forms store the backing state entity. The rendered component must provide
`new(&gpui::Entity<State>) -> impl gpui::IntoElement` so the shape's
`RenderComponent` associated type can render prototyping output:

```rust
use component_shape_gpui::GpuiComponentShape;

#[derive(GpuiComponentShape)]
#[gpui_component_shape(
    value = String,
    field_suffix = "input"
)]
pub struct EmailInput {
    state: gpui::Entity<EmailInputState>,
}

impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for EmailInput {
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
}
```

For wrapper shapes around state from another crate, use the extracted GPUI proc
macro and then attach form storage metadata in the crate that wants to use the
shape with `gpui-form`:

```rust
component_shape_gpui::component_shape! {
    pub struct EmailInputShape {
        state = gpui_component::input::InputState;
        new = gpui_component::input::InputState::new;
        component = gpui_component::input::Input;
        value = String;
        field_suffix = "input";
    }
}
```

`field_suffix = "..."` metadata and direct
`shape::ComponentPrototyping::field_suffix(...)` calls use the same runtime
contract: the suffix must be a non-empty ASCII identifier suffix. The runtime
API validates this in const evaluation so invalid suffixes fail at the shape
implementation. `ComponentPrototyping` and `ComponentSuffix` come from
`component-shape` and are reused by schema inventory metadata.

## Value Binding

Implement `shape::GpuiComponentStateValueBinding<T>` on a component-derived shape's
backing state when generated prototyping code should seed a component from a
form value and map component events back into the form value holder. The derived
shape delegates its `shape::GpuiComponentValueBinding<T>` impl through that
state-level contract when `value_binding` metadata is present.
Wrapper shapes declared with `component_shape_gpui::component_shape!` can place
the `GpuiComponentValueBinding<T>` impl inside the macro block. When no explicit
`value = ...` or `values(...)` metadata is present, that nested impl also
publishes the compatible value type and shape-level value-binding metadata.
