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
    #[gpui_form(component = gpui_form_collection::input::Input::<_>)]
    pub username: Option<String>,

    #[gpui_form(component = gpui_form_collection::input::Input::<_>)]
    pub age: Option<u32>,
}
```

Supported component forms:

- `#[gpui_form(component = my::Shape)]`
- `#[gpui_form(component = my::Shape.component(my::ui::Widget))]`
- `#[gpui_form(component = my::Shape.value_binding())]`
- `#[gpui_form(component = my::Shape.field_suffix("input"))]`
- `#[gpui_form(component = my::Shape.requires_value(false))]`
- `#[gpui_form(component = gpui_form_collection::input::Input::<_>)]`
- `#[gpui_form(component = gpui_form_collection::select::Select::<_>)]`
- `#[gpui_form(component = gpui_form_collection::combobox::Combobox::<_>)]`
- `#[gpui_form(component = gpui_form_collection::checkbox::Checkbox)]`
- `#[gpui_form(component = gpui_form_collection::switch::Switch)]`
- `#[gpui_form(component = gpui_form_collection::number_input::NumberInput::<_>)]`
- `#[gpui_form(component = gpui_form_collection::slider::Slider)]`
- `#[gpui_form(component = gpui_form_collection::color_picker::ColorPicker)]`
- `#[gpui_form(component = gpui_form_collection::date_picker::DatePicker)]`
- `#[gpui_form(component = gpui_form_collection::date_picker::DateRangePicker)]`
- `#[gpui_form(component = gpui_form_collection::file_picker::FilePicker)]`
- `#[gpui_form(component = gpui_form_collection::otp_input::OtpInput::<_>)]`
- `#[gpui_form(component = gpui_form_component::infinite_select::InfiniteSelect::<_>)]`

The expression is parsed as attribute metadata; generated runtime construction
delegates to `ComponentShape::new`. `gpui-form` treats every component as a
custom shape contract and does not inspect the shape path for built-in
component categories.

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
- non-optional component fields default to required holder storage, so the
  generated value holder stores `Option<T>` and conversion back to the source
  model fails when the field is missing
- `.requires_value(false)` keeps a non-optional component field as `T` when the
  component can safely synthesize a value
- `.value_binding()` records that the component shape implements
  `gpui_form_component::shape::ComponentValueBinding<T>` for generated
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

Implements `gpui_form_component::shape::ComponentShape` directly for a state type.

```rs
use gpui_form_derive::ComponentShape;

#[derive(Clone, Debug, ComponentShape)]
#[gpui_form_shape(
    new = crate::state::build,
    component = crate::ui::TagsInput,
    value_binding,
    field_suffix = "input"
)]
pub struct TagsState;
```

By default, the generated implementation calls `Self::new(window, cx)`.
`new = ...` accepts a constructor expression. Function paths and closures are
called with `(window, cx)`; full constructor expressions such as
`Self::with_label(window, cx, "tags")` are emitted as written.
`component = ...` populates `ComponentShape::COMPONENT_PATH`.
`value_binding` or `value_binding = true` sets
`ComponentShape::VALUE_BINDING = true`, so generated prototyping code can
inherit the shape's `ComponentValueBinding<T>` contract without repeating
`value_binding` on every field. `value_binding = false` is equivalent to
omitting it.
`field_suffix = "..."` populates `ComponentShape::PROTOTYPING`, giving
prototyping generators a reusable field/helper suffix without relying on shape
name heuristics.
`shape_crate = ...` overrides the runtime crate path when a crate needs to
target its own local `shape::ComponentShape` module.

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
        value_binding;
        field_suffix = "input";
    }
}
```

Use this when the component and state live in another crate. The macro creates
the local wrapper type that owns the `ComponentShape` implementation; the
caller can still add a `ComponentValueBinding<T>` impl on that wrapper.
It accepts the same metadata keys as `#[derive(ComponentShape)]`, with
`type State = ...` supplying the wrapped state type. If `new` is omitted, the
generated implementation calls `<State>::new(window, cx)`.
Options may be separated with semicolons or commas, and the final separator is
optional.

## Feature Flags

- `inventory`: enables `GpuiFormShape` registration for `#[derive(GpuiForm)]`

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for the main facade
- [`gpui-form-collection-derive`](../gpui-form-collection-derive/README.md) for
  `#[derive(SelectItem)]` when using derives directly
- [`gpui-form-component`](../gpui-form-component/README.md) for runtime helpers,
  component shape contracts, and `#[derive(InfiniteSelect)]` with its `derive`
  feature
- [`gpui-form-schema`](../gpui-form-schema/README.md) when you need metadata
  rather than derives
