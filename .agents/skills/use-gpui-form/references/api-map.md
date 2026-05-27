# gpui-form User API Map

Use this reference for application code that consumes `gpui-form`.

## Install Shape

Use `gpui-form` as the public entry point. Match `gpui` and `gpui-component`
versions to the compatibility guidance for the `gpui-form` version in use.

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "832c17e8192e2e1d472f0751e7cef2af84ded622" }
gpui-component = { git = "https://github.com/longbridge/gpui-component", branch = "main" }
gpui-form = "*"
gpui-form-component = { version = "*", features = ["derive"] }
gpui-form-collection = "*"
gpui-form-collection-derive = "*"
gpui-form-derive = "*"
```

## Imports

Use the facade for `GpuiForm` and import component derives explicitly:

```rust
use gpui_form::GpuiForm;
use gpui_form_collection_derive::SelectItem;
use gpui_form_component::InfiniteSelect;
use gpui_form_derive::ComponentShape;
```

Useful runtime/helper paths:

- `gpui_form_component::shape`
- `gpui_form_component::date_picker`
- `gpui_form_component::file_picker`
- `gpui_form_component::infinite_select`
- `gpui_form::core::numeric`
- `gpui_form_derive::component_shape!`

## Supported Component Syntax

```rust
#[gpui_form(gpui_form_collection::input::Input::<_>)]
#[gpui_form(gpui_form_collection::select::Select::<_>)]
#[gpui_form(gpui_form_collection::combobox::Combobox::<_>)]
#[gpui_form(gpui_form_collection::number_input::NumberInput::<_>)]
#[gpui_form(gpui_form_collection::slider::Slider)]
#[gpui_form(gpui_form_collection::color_picker::ColorPicker)]
#[gpui_form(gpui_form_collection::date_picker::DatePicker)]
#[gpui_form(gpui_form_collection::date_picker::DateRangePicker)]
#[gpui_form(gpui_form_collection::file_picker::FilePicker)]
#[gpui_form(gpui_form_collection::otp_input::OtpInput::<_>)]
#[gpui_form(gpui_form_collection::checkbox::Checkbox)]
#[gpui_form(gpui_form_collection::switch::Switch)]
#[gpui_form(gpui_form_component::infinite_select::InfiniteSelect::<_>)]
#[gpui_form(my::Shape)]
#[gpui_form(my::Shape.component(my::ui::Widget))]
#[gpui_form(my::Shape.value_binding())]
#[gpui_form(my::Shape.field_suffix("input"))]
#[gpui_form(my::Shape.requires_value(false))]
#[gpui_form(component = my::Shape)]
```

Common field attributes:

```rust
#[gpui_form(default = <expr>)]
#[gpui_form(skip)]
#[gpui_form(type = <form_type>)]
#[gpui_form(from = <expr>)]
#[gpui_form(into = <expr>)]
```

Common struct attributes:

```rust
#[gpui_form(empty)]
```

## Component Selection

- Use `gpui_form_collection::input::Input::<_>` for text-like and
  `FromStr`-parsable values.
- Use `gpui_form_collection::checkbox::Checkbox` or
  `gpui_form_collection::switch::Switch` for `bool` fields.
- Use `gpui_form_collection::select::Select::<_>` for a single enum-like
  choice; derive `SelectItem`. Use a custom `ComponentShape` wrapper when the
  select should expose search or other component-specific behavior.
- Use `gpui_form_collection::combobox::Combobox::<_>` for multi-value enum-like
  choices from `gpui_component::combobox::Combobox`.
- Use `gpui_form_collection::number_input::NumberInput::<_>` for numeric text
  input with step buttons.
- Use `gpui_form_collection::slider::Slider` for continuous numeric values.
- Use `gpui_form_collection::color_picker::ColorPicker` for color selection.
- Use `gpui_form_collection::date_picker::DatePicker` for single-date editing.
- Use `gpui_form_collection::date_picker::DateRangePicker` for date-range editing.
- Use `gpui_form_collection::file_picker::FilePicker` for native file/directory
  selection.
- Use `gpui_form_collection::otp_input::OtpInput::<_>` for OTP inputs.
- Use `gpui_form_component::infinite_select::InfiniteSelect::<_>` for
  nested/cascading enum trees; derive `InfiniteSelect`. Use a custom
  `ComponentShape` wrapper when search or depth limits are needed.
- Use a component shape around `gpui_form_component::date_picker` or
  `gpui_form_component::file_picker` when a form field needs those runtimes.
- Use `#[gpui_form(my::Shape)]` when the app owns the state/widget contract.
- Treat the component expression and chained generic metadata methods as derive
  metadata; runtime construction still uses `ComponentShape::new`.
- Non-optional component fields default to required holder storage; add
  `.requires_value(false)` when a component can safely synthesize a missing
  value.

## Generated Names

For a source struct named `UserProfile`, expect generated types named:

```rust
UserProfileFormFields
UserProfileFormComponents
UserProfileFormValueHolder
```

Use the generated value holder for editable form data, defaults, and conversion
back into the original model.

## Select Pattern

```rust
use gpui_form_collection_derive::SelectItem;
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, PartialEq, SelectItem)]
pub enum Country {
    #[default]
    UnitedStates,
    France,
}
```

Add `#[select_item(fluent)]` when the enum derives `EsFluent` and the app will
handle localized labels outside the `SelectItem::title()` call.

## Infinite Select Pattern

```rust
use gpui_form::GpuiForm;
use gpui_form_component::InfiniteSelect;
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, InfiniteSelect, PartialEq)]
pub enum City {
    #[default]
    Paris,
    Lyon,
}

#[derive(Clone, Debug, EnumIter, InfiniteSelect, PartialEq)]
pub enum Country {
    France(City),
}

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct LocationForm {
    #[gpui_form(gpui_form_component::infinite_select::InfiniteSelect::<_>)]
    pub location: Country,
}
```

Helper state is available from `gpui_form_component::infinite_select`.

## Type Conversion Pattern

Use `type`, `from`, and `into` when the UI edits a different type than the
model stores:

```rust
#[derive(Clone, Debug, gpui_form::GpuiForm)]
pub struct User {
    #[gpui_form(
        crate::DatePickerShape,
        type = chrono::NaiveDate,
        from = to_form_date,
        into = to_model_timestamp
    )]
    pub birth_date: Option<Timestamp>,
}
```

This is useful for dates, paths, numeric newtypes, and other domain-specific
wrappers.

## Component Shape Patterns

Derive directly on a state type:

```rust
use gpui_form::GpuiForm;
use gpui_form_derive::ComponentShape;

#[derive(Clone, Debug, ComponentShape)]
#[gpui_form_shape(component = TagsInput)]
pub struct TagsInputState;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct PostEditor {
    #[gpui_form(TagsInputState)]
    pub tags: Option<Vec<String>>,
}
```

`new` accepts a constructor path or closure and is called with `(window, cx)`.

Or declare a reusable shape:

```rust
gpui_form_derive::component_shape! {
    pub struct EmailInputShape {
        type State = gpui_component::input::InputState;
        component = gpui_component::input::Input;
    }
}
```

`gpui_form_derive::component_shape!` accepts semicolons or commas between
options. Use bare `value_binding` to publish shape-level value-binding metadata;
omit it to leave that metadata disabled.
