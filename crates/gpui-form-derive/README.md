# gpui-form-derive

Procedural macros behind the `gpui-form` ecosystem.

Most users should depend on [`gpui-form`](../gpui-form/README.md) and derive
from the facade crate. Use this crate directly when you want the proc-macro
layer, including component-shape derives and helper macros.

## What This Crate Provides

- `#[derive(GpuiForm)]`
- `#[derive(ComponentShape)]`
- `component_shape! { ... }`
- `#[gpui_form_derive::component_value_binding]`

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
    #[gpui_form(gpui_form_collection::input::Input::<_>)]
    pub username: Option<String>,

    #[gpui_form(gpui_form_collection::input::Input::<_>)]
    pub age: Option<u32>,
}
```

Supported component forms:

- `#[gpui_form(my::Shape)]`
- `#[gpui_form(my::Shape.component(my::ui::Widget))]`
- `#[gpui_form(my::Shape.value_binding())]`
- `#[gpui_form(my::Shape.field_suffix("input"))]`
- `#[gpui_form(gpui_form_collection::input::Input::<_>)]`
- `#[gpui_form(gpui_form_collection::select::Select::<_>)]`
- `#[gpui_form(gpui_form_collection::combobox::Combobox::<_>)]`
- `#[gpui_form(gpui_form_collection::checkbox::Checkbox)]`
- `#[gpui_form(gpui_form_collection::switch::Switch)]`
- `#[gpui_form(gpui_form_collection::number_input::NumberInput::<_>)]`
- `#[gpui_form(gpui_form_collection::slider::Slider)]`
- `#[gpui_form(gpui_form_collection::color_picker::ColorPicker)]`
- `#[gpui_form(gpui_form_collection::date_picker::DatePicker)]`
- `#[gpui_form(gpui_form_collection::date_picker::DateRangePicker)]`
- `#[gpui_form(gpui_form_collection::otp_input::OtpInput::<_>)]`
- `#[gpui_form(gpui_form_component::date_picker::DatePicker)]`
- `#[gpui_form(gpui_form_component::date_picker::DateRangePicker)]`
- `#[gpui_form(gpui_form_component::file_picker::FilePicker)]`
- `#[gpui_form(gpui_form_component::infinite_select::InfiniteSelect::<_>)]`

The `gpui_form_component` shape examples require that crate's
`component-shape` feature. Infinite-select field types also need the
`InfiniteSelect` derive, available through `gpui-form-component`'s `derive`
feature or by depending on `gpui-form-component-derive` directly.

The explicit key-value form remains available, for example
`#[gpui_form(component = my::Shape)]`. Both forms parse the expression as
attribute metadata; generated runtime construction delegates to
`ComponentShape::new`. `gpui-form` treats every component as a custom shape
contract and does not inspect the shape path for built-in component categories.

Supporting field attributes:

- `#[gpui_form(default = <expr>)]`
- `#[gpui_form(skip)]`
- `#[gpui_form(type = <form_type>)]`
- `#[gpui_form(from = <expr>)]`
- `#[gpui_form(into = <expr>)]`

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
- component shapes own the default required-value policy for non-optional
  fields; shapes that can synthesize a missing value keep generated value-holder
  storage as `T`, while required shapes use `Option<T>` and fail conversion when
  missing
- set `requires_value = false` on the reusable shape definition when the
  component can synthesize missing values
- `.value_binding()` records that the component shape implements
  `gpui_form_runtime::shape::ComponentValueBinding<T>` for generated
  prototyping subscriptions; the adapter seeds component state with
  `seed_value_binding_state` and maps component events to `FormValueChange<T>`
- `type`/`from`/`into` let the generated holder edit a type that differs from
  the original model field
- `gpui_form_collection::input::Input::<_>`,
  `gpui_form_collection::number_input::NumberInput::<_>`, and
  `gpui_form_collection::otp_input::OtpInput::<_>` prototyping code parse
  form-side non-`String` values with `FromStr` instead of assigning raw
  `String`s
- generic component expressions use `::<_>` in the attribute; the derive normalizes
  the path and resolves `_` to the field's form-side type
- generated `FormFields` and `FormComponents` suffixes use field-level
  `.field_suffix(...)` first, then the shape-name heuristic. Shape-level
  `ComponentShape::PROTOTYPING.field_suffix` is inventory metadata for
  prototyping output
- field-level `#[koruma(...)]` attributes are accepted by `GpuiForm` and copied
  onto the generated value holder, which allows validating form-side override
  types without deriving `Koruma` on the original model
- when skipped fields are present, the generated value holder keeps builder
  support and exposes `into_original(...)` instead of an unconditional reverse
  conversion

## `#[derive(ComponentShape)]`

Implements `gpui_form_runtime::shape::ComponentShape` for a rendered component
with separate backing state. Crates that use this derive need an explicit
`gpui-form-runtime` dependency.

```rs
use crate::state::{TagsState, build};
use gpui_form_derive::ComponentShape;

#[derive(IntoElement, ComponentShape)]
#[gpui_form_shape(
    state = TagsState,
    new = build,
    field_suffix = "input"
)]
pub struct TagsInput {
    state: gpui::Entity<TagsState>,
}
```

`state = ...` supplies `ComponentShape::State`. If `component = ...` is omitted,
`ComponentShape::COMPONENT_PATH` defaults to the derived type's module path.
By default, the generated implementation calls `<State>::new(window, cx)`.
`state` and `new` use normal Rust path resolution, so short in-scope names are
fine. `new = ...` accepts a constructor expression. Function paths and closures
are called with `(window, cx)`; full constructor expressions such as
`TagsState::with_label(window, cx, "tags")` are emitted as written.
The `component = ...` option also publishes `COMPONENT_PATH` metadata for
generated/prototyping render code, so prefer a path that remains valid from the
consumer crate, such as `gpui_form_collection::checkbox::CheckboxField`.
Component-derived shapes always publish `VALUE_BINDING = true` and delegate
`ComponentValueBinding<T>` through the backing state's
`ComponentStateValueBinding<T>` implementation.
`field_suffix = "..."` populates `ComponentShape::PROTOTYPING`, giving
prototyping generators a reusable field/helper suffix without relying on shape
name heuristics.

Use `#[gpui_form_derive::component_value_binding]` on the backing state's
binding impl when the component-derived shape should delegate value binding to
that state:

```rs
#[gpui_form_derive::component_value_binding]
impl gpui_form_runtime::shape::ComponentValueBinding<Vec<PathBuf>> for FilePickerState {
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

Use this when the component and state live in another crate. The macro creates
the local wrapper type that owns the `ComponentShape` implementation and the
wrapper's reusable `ComponentValueBinding<T>` impls.
It accepts `new`, `component`, `requires_value`, and `field_suffix` metadata,
with `type State = ...` supplying the wrapped state type. If `new` is omitted,
the generated implementation calls `<State>::new(window, cx)`.
`requires_value = false` publishes that non-optional fields using this shape can
store `T` directly because the component can synthesize a missing value.
Options may be separated with semicolons or commas, and the final separator is
optional.
Nested `ComponentValueBinding<T>` impls are emitted after the generated shape
contract and automatically publish shape-level value-binding metadata.

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
