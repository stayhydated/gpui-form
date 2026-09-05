# Component shapes

A component shape connects a form-side value to GPUI state, rendering, and
events. Prefer a ready-made collection or component-owned shape. Define an
application-owned shape when the existing contracts cannot represent the
widget or construction behavior you need.

## Choose the package

- Use `gpui-form-collection` for common GPUI Kit controls.
- Use `gpui-form-component` for localized date and file pickers or cascading
  infinite selects.
- Use `component-shape-gpui` and `gpui-form-runtime` when a crate defines its
  own reusable shape.

Applications that only consume existing shapes use the
`gpui_form::runtime::shape` facade and do not need a direct
`gpui-form-runtime` dependency.

## Collection shapes

`gpui-form-collection` provides these form shapes:

| Need | Shape |
|---|---|
| A `FromStr + ToString + 'static` value | `input::Input::<T>` |
| Application-defined text parsing and formatting | `input::ParsedInput::<T, Config>` |
| One enum-like value | `select::Select::<T>` |
| Several enum-like values | `combobox::Combobox::<Item>` |
| Boolean value | `checkbox::Checkbox` or `switch::Switch` |
| Numeric text input | `number_input::NumberInput::<T>` |
| `f32` or `gpui_kit::component::slider::SliderValue` | `slider::Slider` |
| `gpui_kit::Hsla` | `color_picker::ColorPicker` |
| `chrono::NaiveDate` or a date pair | `date_picker::DatePicker` or `DateRangePicker` |
| One-time-password value | `otp_input::OtpInput::<T>` |

`ParsedInput<T, Config>` uses a `ParsedInputConfig<T>` implementation for
parsing, formatting, placeholder text, empty-as-clear behavior, and optional
widget validation.

Select values normally derive `SelectItem` from
`gpui-form-collection-derive` and `EnumIter` from `strum`. The generic
parameter of `Combobox<Item>` is the item type, so a `Vec<Country>` field uses
`Combobox::<Country>`.

## Configure one field

Built-in shapes expose builder expressions inside `component(...)`:

```rust,ignore
#[gpui_form(component(
    gpui_form_collection::select::Select::<_>.searchable(true)
))]
country: Country,
```

Use `Shape.from(options)` when the shape publishes a completed options type.
Configuration changes construction for that field while preserving the base
shape's value compatibility, rendering, storage, and metadata.

## Component-owned shapes

Enable the form-shape implementations and derives you use:

```toml
[dependencies]
gpui-form-component = { version = "0.6", features = ["component-shape", "derive"] }
```

| Need | Shape | Requirement |
|---|---|---|
| Localized date or date range | `gpui_form_component::date_picker::DatePicker` or `DateRangePicker` | Initialize application `gpui-es-fluent` resources |
| Native file or directory selection | `gpui_form_component::file_picker::FilePicker` | Initialize application `gpui-es-fluent` resources |
| Cascading enum choices | `gpui_form_component::infinite_select::InfiniteSelect::<T>` | Derive `InfiniteSelect` and implement `Clone + Default + PartialEq + 'static` throughout the enum tree |

Nested infinite-select payload types must implement `Default`. Use
`InfiniteSelect::<_>.searchable(true)` for search or
`InfiniteSelect::<_>.from(InfiniteSelectOptions::new(true, Some(3)))` for
search plus a maximum depth.

## Define an application-owned shape

Declare an owned rendered component with
`#[derive(component_shape_gpui::GpuiComponentShape)]`, or wrap an external
state/component pair with `component_shape_gpui::component_shape!`. Then attach
the form storage policy:

```rust,ignore
impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for EmailInputShape {
    type ValueStoragePolicy =
        gpui_form_runtime::shape::DirectValueStorage;
}
```

A reusable shape defines:

- backing entity state and construction
- a render component
- supported value types
- event-to-value binding when the form should synchronize automatically
- form holder storage policy
- a stable prototyping field suffix
- MCP input metadata when the wire contract needs shape-specific guidance

A shape declared only through a hand-written `GpuiComponentShape`
implementation lacks the declaration marker required by `GpuiForm`. Use the
derive or macro so the generated contract includes that marker and its
metadata.

### Storage policy

`DirectValueStorage` stores `T` for a non-optional field. Initialization comes
from an intent-scoped default or the form-side type's `Default` implementation.

`RequiredValueStorage` stores `Option<T>` so the form can represent missing
input. Holder conversion reports an absent required value. Generated
validation reports the same condition when Koruma integration is enabled.

## Troubleshooting

| Symptom | Action |
|---|---|
| A component-owned type cannot be used in `component(...)` | Enable `gpui-form-component`'s `component-shape` feature. |
| The derive rejects an application-owned shape as undeclared | Declare it with the `GpuiComponentShape` derive or `component_shape!` macro. |
| A configured shape expression fails its bound | Use a builder produced for the same base shape, or pass its completed options through `Shape.from(...)`. |
| Prototyping reports missing capabilities | Add the missing render, value-binding, shape-path, or storage metadata to the owning shape and regenerate. |
