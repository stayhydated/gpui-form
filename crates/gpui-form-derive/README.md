# gpui-form-derive

Procedural macros behind the `gpui-form` ecosystem.

Most users should depend on [`gpui-form`](../gpui-form/README.md) and derive
from the facade crate. Use this crate directly when you want only the
`GpuiForm` proc macro.

## What This Crate Provides

- `#[derive(GpuiForm)]`

GPUI component shape derives and helper macros live in
[`component-shape-gpui`](../../../component-shape/crates/component-shape-gpui).

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
construction delegates to `GpuiComponentShape::new`. The shape type must be
declared with `component_shape_gpui::component_shape!` or
`#[derive(component_shape_gpui::GpuiComponentShape)]`; hand-written
`GpuiComponentShape` impls are not accepted by `#[derive(GpuiForm)]`. Put render
component metadata and prototyping suffix metadata on the GPUI shape
declaration, and put form storage metadata on a
`gpui_form_runtime::shape::GpuiFormComponentShapePolicy` impl.

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
- set `GpuiFormComponentShapePolicy::ValueStoragePolicy` to
  `DirectValueStorage` on reusable shapes that can synthesize default values
- `ComponentShapeMetadata::CAPABILITIES` records whether the component shape
  implements `gpui_form_runtime::shape::GpuiComponentValueBinding<T>` for
  generated prototyping subscriptions; the adapter seeds component state with
  `seed_value_binding_state` and maps component events to `ValueChange<T>`
- generated `GpuiForm` component field code references
  `gpui_form::runtime::shape`; normal application crates do not need
  `gpui-form-runtime` directly just because a field uses a component shape
- `value(type = ..., from_source = ..., into_source = ...)` lets the generated
  holder edit a type that differs from the original model field. The `type = ...`
  option is the form-side base value type; use `type = T`, not
  `type = Option<T>`, even when the source field is optional.
- `gpui_form_collection::input::Input::<_>`,
  `gpui_form_collection::number_input::NumberInput::<_>`, and
  `gpui_form_collection::otp_input::OtpInput::<_>` prototyping code parse
  form-side non-`String` values with `FromStr` instead of assigning raw
  `String`s
- generic component expressions use `::<_>` in the attribute; the derive normalizes
  the path and resolves `_` to the field's form-side type
- generated `FormFields` members and `FormComponents` constructors use the
  source field identifier. Derive-generated inventory still records
  shape-level `field_suffix = "..."` metadata when available, or a resolved
  shape-name fallback, for prototyping-specific DOM IDs, event handlers, and
  helper names. Shape-level suffixes must be non-empty identifier suffixes
- field-level `#[koruma(...)]` attributes are accepted by `GpuiForm` and copied
  onto the generated value holder, which allows validating form-side override
  types without deriving `Koruma` on the original model
- when skipped fields are present, the generated value holder keeps builder
  support and exposes `into_original(...)` instead of an unconditional reverse
  conversion. It also exposes a typed `PresentField` enum and
  `present_fields()` snapshot method so examples and tooling can format current
  editable state outside the derive output.
- `skip` cannot be combined with component or hidden intent on the same field

## Component Shape Macros

Use `component_shape_gpui::component_shape!` or
`#[derive(component_shape_gpui::GpuiComponentShape)]` for GPUI shape
declarations. Pair those declarations with
`gpui_form_runtime::shape::GpuiFormComponentShapePolicy` when the shape is used
as a `GpuiForm` field so the form derive can choose required or direct value
storage.

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
