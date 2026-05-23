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
use gpui_form_derive::CustomComponentState;
```

Useful runtime/helper paths:

- `gpui_form_component::custom`
- `gpui_form_component::date_picker`
- `gpui_form_component::file_picker`
- `gpui_form_component::infinite_select`
- `gpui_form::numeric`
- `gpui_form_component::custom_component_shape!`

## Supported Component Syntax

```rust
#[gpui_form(component = gpui_form_collection::input::InputShape::<_>)]
#[gpui_form(component = gpui_form_collection::select::SelectShape::<_>)]
#[gpui_form(component = gpui_form_collection::select::SelectShape::<_>.searchable().partial())]
#[gpui_form(component = gpui_form_collection::checkbox::CheckboxShape)]
#[gpui_form(component = gpui_form_collection::switch::SwitchShape)]
#[gpui_form(component = gpui_form_component::infinite_select::InfiniteSelectState::<_>.searchable().max_depth(3))]
#[gpui_form(component = my::Shape)]
#[gpui_form(component = my::Shape.component(my::ui::Widget))]
#[gpui_form(component = my::Shape.value_binding())]
#[gpui_form(component = my::Shape.field_suffix("input"))]
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

- Use `gpui_form_collection::input::InputShape::<_>` for text-like and
  `FromStr`-parsable values.
- Use `gpui_form_collection::checkbox::CheckboxShape` or
  `gpui_form_collection::switch::SwitchShape` for `bool` fields.
- Use `gpui_form_collection::select::SelectShape::<_>` for a single enum-like
  choice; derive `SelectItem`. Chain `.searchable()` or `.partial()` when the
  select should expose those behaviors.
- Use `gpui_form_component::infinite_select::InfiniteSelectState::<_>` for
  nested/cascading enum trees; derive `InfiniteSelect`. Chain `.searchable()`
  and `.max_depth(...)` when needed.
- Use a custom shape around `gpui_form_component::date_picker` or
  `gpui_form_component::file_picker` when a form field needs those runtimes.
- Use `component = my::Shape` when the app owns the state/widget
  contract. The older `component(custom(...))` form is still accepted.
- Treat the shape expression and chained component behavior methods as derive
  metadata; runtime construction still uses `CustomComponentShape::new`. A
  trailing `.new()` marker remains accepted for compatibility.
- Treat generated value-holder wrapping as internal derive behavior for known
  reusable shapes.

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
    #[gpui_form(
        component = gpui_form_component::infinite_select::InfiniteSelectState::<_>
            .searchable()
            .max_depth(3)
    )]
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
        type = chrono::NaiveDate,
        from = to_form_date,
        into = to_model_timestamp,
        component = crate::DatePickerShape
    )]
    pub birth_date: Option<Timestamp>,
}
```

This is useful for dates, paths, numeric newtypes, and other domain-specific
wrappers.

## Custom Component Patterns

Derive directly on a state type:

```rust
use gpui_form::GpuiForm;
use gpui_form_derive::CustomComponentState;

#[derive(Clone, Debug, CustomComponentState)]
#[gpui_form_custom(new = Self::new, component = TagsInput)]
pub struct TagsInputState;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct PostEditor {
    #[gpui_form(component = TagsInputState)]
    pub tags: Option<Vec<String>>,
}
```

Or declare a reusable shape:

```rust
gpui_form_component::custom_component_shape!(
    pub EmailInputShape,
    state = gpui_component::input::InputState,
    new = gpui_component::input::InputState::new,
    component = gpui_component::input::Input,
);
```
