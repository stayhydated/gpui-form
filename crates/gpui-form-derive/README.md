# gpui-form-derive

Procedural macros behind the `gpui-form` ecosystem.

Most users should depend on [`gpui-form`](../gpui-form/README.md) and derive
from the facade crate. Use this crate directly when you want the proc-macro
layer, including component-shape derives and helper macros.

## What This Crate Provides

- `#[derive(GpuiForm)]`
- `#[derive(ComponentShape)]`
- `component_shape! { ... }`

`#[derive(InfiniteSelect)]` does not live in this crate. It is provided by
[`gpui-form-component-derive`](../gpui-form-component-derive/README.md) and by
`gpui-form-component` when that crate's `derive` feature is enabled.

`#[derive(SelectItem)]` is implemented in
[`gpui-form-collection-derive`](../gpui-form-collection-derive/README.md).

## `#[derive(GpuiForm)]`

Turns a struct into typed form state plus helper types for editing and
submission.

```rs
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct UserProfile {
    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    pub username: Option<String>,

    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    pub age: Option<u32>,
}
```

Supported component forms:

- `#[gpui_form(component(my::Shape))]`
- `#[gpui_form(component(gpui_form_collection::input::Input::<_>))]`
- `#[gpui_form(component(gpui_form_collection::select::Select::<_>))]`
- `#[gpui_form(component(gpui_form_collection::combobox::Combobox::<Country>))]`
- `#[gpui_form(component(gpui_form_collection::checkbox::Checkbox))]`
- `#[gpui_form(component(gpui_form_collection::switch::Switch))]`
- `#[gpui_form(component(gpui_form_collection::number_input::NumberInput::<_>))]`
- `#[gpui_form(component(gpui_form_collection::slider::Slider))]`
- `#[gpui_form(component(gpui_form_collection::color_picker::ColorPicker))]`
- `#[gpui_form(component(gpui_form_collection::date_picker::DatePicker))]`
- `#[gpui_form(component(gpui_form_collection::date_picker::DateRangePicker))]`
- `#[gpui_form(component(gpui_form_collection::otp_input::OtpInput::<_>))]`
- `#[gpui_form(component(gpui_form_component::date_picker::DatePicker))]`
- `#[gpui_form(component(gpui_form_component::date_picker::DateRangePicker))]`
- `#[gpui_form(component(gpui_form_component::file_picker::FilePicker))]`
- `#[gpui_form(component(gpui_form_component::infinite_select::InfiniteSelect::<_>))]`

The `gpui_form_component` shape examples require that crate's
`component-shape` feature. Infinite-select field types also need the
`InfiniteSelect` derive, available through `gpui-form-component`'s `derive`
feature or by depending on `gpui-form-component-derive` directly.

The `component(...)` value is parsed as a Rust type path; generated runtime
construction delegates to `ComponentShape::new`. The shape type must be
declared with `component_shape!` or `#[derive(ComponentShape)]`; hand-written
`ComponentShape` impls are not accepted by `#[derive(GpuiForm)]`. Put render
component metadata and prototyping suffix metadata on the shape declaration,
not on the `#[gpui_form(...)]` field attribute.

Supporting field attributes:

- `#[gpui_form(component(<shape>))]`
- `#[gpui_form(hidden)]`
- `#[gpui_form(component(<shape>, default = <expr>))]`
- `#[gpui_form(hidden(default = <expr>))]`
- `#[gpui_form(skip)]`
- `#[gpui_form(component(<shape>, value(type = <form_type>, from_source = <expr>, into_source = <expr>), default = <expr>))]`
- `#[gpui_form(hidden(value(type = <form_type>, from_source = <expr>, into_source = <expr>), default = <expr>))]`

Every non-empty-form field must choose exactly one intent: component, hidden,
or skipped. Use `hidden` for value-holder-only fields. Defaults and value
conversion options belong inside the component or hidden intent; `from_source`
maps source model values into form values, and `into_source` maps form values
back into source values.

Supporting struct attributes:

- `#[gpui_form(empty)]`
- `#[gpui_form(koruma)]`
- `#[gpui_form(koruma(fluent))]`

Behavior notes:

- reusable gpui-component-backed shapes live in `gpui-form-collection`
- collection components that are enum-driven (`Select`, `Combobox`) expect enum-like
  values that can populate a `gpui_component` list widget
- `gpui_form_component::infinite_select::InfiniteSelect::<_>` expects the field type
  to derive `gpui_form_component::InfiniteSelect`, which implements
  `gpui_form_component::infinite_select::InfiniteSelectValue`
- `default = ...` seeds the generated value holder
- component shapes own the default value-storage policy for non-optional
  fields; shapes that can synthesize a default value keep generated value-holder
  storage as `T`, while required shapes use `Option<T>` and fail conversion when
  missing. Generated `validate()` reports missing required shape values.
  Fallible holder-to-model paths expose `holder.try_into_original()`;
  infallible paths expose `holder.into_original()` and implement `From` when
  the derive can prove infallibility directly, such as optional fields, direct
  fields, or shape-backed fields with a declared field default. Shape-backed
  fields without a field default keep the checked `try_into_original()` path.
- set `value_storage = direct` on the reusable shape definition when the
  component can synthesize default values
- shape-level `ValueBindingPolicy` records that the component shape implements
  `gpui_form_runtime::shape::ComponentValueBinding<T>` for generated
  prototyping subscriptions; the adapter seeds component state with
  `seed_value_binding_state` and maps component events to `FormValueChange<T>`
- generated `GpuiForm` component field code references
  `gpui_form::runtime::shape`; normal application crates do not need
  `gpui-form-runtime` directly just because a field uses a component shape
- `value(type = ..., from_source = ..., into_source = ...)` lets the generated
  holder edit a type that differs from the original model field
- `gpui_form_collection::input::Input::<_>`,
  `gpui_form_collection::number_input::NumberInput::<_>`, and
  `gpui_form_collection::otp_input::OtpInput::<_>` prototyping code parse
  form-side non-`String` values with `FromStr` instead of assigning raw
  `String`s
- generic component expressions use `::<_>` in the attribute; the derive normalizes
  the path and resolves `_` to the field's form-side type
- generated `FormFields` and `FormComponents` suffixes derive from the shape
  type name. Derive-generated inventory inherits shape-level
  `field_suffix = "..."` metadata when available and otherwise records the
  resolved shape-name suffix for prototyping output. Shape-level suffixes must
  be non-empty identifier suffixes
- field-level `#[koruma(...)]` attributes are accepted by `GpuiForm` and copied
  onto the generated value holder, which allows validating form-side override
  types without deriving `Koruma` on the original model
- when skipped fields are present, the generated value holder keeps builder
  support and exposes `into_original(...)` instead of an unconditional reverse
  conversion
- `skip` cannot be combined with component or hidden intent on the same field

## `#[derive(ComponentShape)]`

Implements `gpui_form_runtime::shape::ComponentShape` and
`DeclaredComponentShape` for a rendered component with separate backing state.
Crates that use this derive need an explicit `gpui-form-runtime` dependency;
generated paths resolve renamed runtime dependencies. Add `value = T` or
`values(T, U)` when the derive should emit `ComponentShapeFor<T>`
compatibility impls for supported form-side value types; omit value metadata
when you will implement `ComponentShapeFor<Value>` manually. Each value type
may be listed once.

```rs
use crate::state::{TagsState, build};
use gpui_form_derive::ComponentShape;

#[derive(IntoElement, ComponentShape)]
#[gpui_form_shape(
    state = TagsState,
    new = build,
    value = Vec<String>,
    field_suffix = "input"
)]
pub struct TagsInput {
    state: gpui::Entity<TagsState>,
}
```

`state = ...` supplies `ComponentShape::State`. If `component = ...` is omitted,
the generated `ComponentShape::RenderComponent` contract renders the derived
type.
The rendered component type, whether inferred or supplied with `component = ...`,
must provide `new(&gpui::Entity<State>) -> impl gpui::IntoElement`.
By default, the generated implementation calls `<State>::new(window, cx)`.
`state` and `new` use normal Rust path resolution, so short in-scope names are
fine. `new = ...` accepts a constructor expression. Function paths and closures
are called with `(window, cx)`; full constructor expressions such as
`TagsState::with_label(window, cx, "tags")` are emitted as written.
The `component = ...` option publishes a `RenderComponent` adapter for
generated/prototyping render code, so prefer a type that remains valid from the
consumer crate, such as `gpui_form_collection::checkbox::CheckboxField`.
The value must be a path-like type. `field_suffix = "..."` must be a
non-empty identifier suffix.
`value = ...` accepts one form-side value type. `values(...)` accepts multiple
form-side value types and emits one `ComponentShapeFor<T>` impl per type.
Component-derived shapes publish value-binding metadata only when
`#[gpui_form_shape(..., value_binding)]` is present. In that mode, they delegate
`ComponentValueBinding<T>` through the backing state's
`ComponentStateValueBinding<T>` implementation. The generated
`ValueBindingPolicy` associated type drives compile-time inherited binding
checks and schema/prototyping metadata.
`field_suffix = "..."` populates `ComponentShape::PROTOTYPING`, giving
prototyping generators a reusable field/helper suffix without relying on shape
name heuristics.

Use `value_binding` on the shape metadata and implement
`ComponentStateValueBinding<T>` on the backing state when the component-derived
shape should delegate value binding to that state:

```rs
impl gpui_form_runtime::shape::ComponentStateValueBinding<Vec<PathBuf>> for FilePickerState {
    type Event = FilePickerEvent;
    /* seed_value_binding_state and form_value_change */
}
```

## `component_shape!`

Declares a local shape type for external component/state pairs.

```rs
gpui_form_derive::component_shape! {
    /// Ready-made text input shape.
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

Use this when the component and state live in another crate. The macro creates
the local wrapper type that owns the `ComponentShape` and
`DeclaredComponentShape` implementations plus the wrapper's reusable
`ComponentValueBinding<T>` impls.
It accepts `new`, `component`, `value = ...`, `values(...)`,
`value_storage = require_value|direct`, `value_binding`, and `field_suffix`
metadata, with `type State = ...` supplying the wrapped state type. If `new` is
omitted, the generated implementation calls `<State>::new(window, cx)`.
`value_storage = direct` publishes that non-optional fields using this shape can
store `T` directly because the component can synthesize a default value.
`component = ...` must be path-like, and `field_suffix = "..."` must be a
non-empty ASCII identifier suffix.
Separate metadata entries with semicolons.
Nested `ComponentValueBinding<T>` impls are emitted after the generated shape
contract. Add `value_binding;` to set the generated `ValueBindingPolicy` to
`InheritedComponentValueBinding`; otherwise the macro uses
`NoComponentValueBinding`. The macro emits `ComponentShapeFor<T>` impls only
for `value = ...` / `values(...)` metadata. Omit value metadata when the block
contains a manual `ComponentShapeFor` impl with custom bounds or diagnostics.
Combining value metadata with manual `ComponentShapeFor` impls is rejected so
compatibility behavior stays explicit.

## Feature Flags

- `inventory`: enables `GpuiFormShape` registration for `#[derive(GpuiForm)]`

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for the main facade
- [`gpui-form-collection-derive`](../gpui-form-collection-derive/README.md) for
  `#[derive(SelectItem)]` when using derives directly
- [`gpui-form-component`](../gpui-form-component/README.md) for runtime helpers,
  built-in component shape implementations, and `#[derive(InfiniteSelect)]`
  with its `derive` feature
- [`gpui-form-runtime`](../gpui-form-runtime/README.md) for component shape
  contracts and value-binding helpers
- [`gpui-form-schema`](../gpui-form-schema/README.md) when you need metadata
  rather than derives
