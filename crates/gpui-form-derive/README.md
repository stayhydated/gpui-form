# gpui-form-derive

Procedural macros behind the `gpui-form` ecosystem.

Most users should depend on [`gpui-form`](../gpui-form/README.md) and derive
from the facade crate. Use this crate directly when you want the proc-macro
layer without the facade's runtime and metadata re-exports.

## What This Crate Provides

- `#[derive(GpuiForm)]`
- `#[derive(SelectItem)]`
- `#[derive(CustomComponent)]` / `#[derive(CustomComponentState)]`
- `custom_component! { ... }`

`#[derive(InfiniteSelect)]` does not live in this crate. It is provided by
[`gpui-form-component-derive`](../gpui-form-component-derive/README.md) and
re-exported by the facade as `gpui_form::InfiniteSelect`.

## `#[derive(GpuiForm)]`

Turns a struct into typed form state plus helper types for editing and
submission.

```rs
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct UserProfile {
    #[gpui_form(component(custom(shape = "gpui_form_collection::input::InputShape<_>")))]
    pub username: Option<String>,

    #[gpui_form(component(custom(shape = "gpui_form_collection::input::InputShape<_>")))]
    pub age: Option<u32>,
}
```

Supported component forms:

- `#[gpui_form(component(custom(shape = my::Shape)))]`
- `#[gpui_form(component(custom(state = my::State)))]`
- `#[gpui_form(component(custom(shape = my::Shape, component = my::ui::Widget)))]`
- `#[gpui_form(component(custom(shape = my::Shape, wraps_in_option = false)))]`
- `#[gpui_form(component(custom(shape = my::Shape, value_binding)))]`
- `#[gpui_form(component(custom(shape = "gpui_form_collection::input::InputShape<_>")))]`
- `#[gpui_form(component(custom(shape = "gpui_form_collection::select::SelectShape<_>", wraps_in_option = false)))]`
- `#[gpui_form(component(custom(shape = gpui_form_collection::checkbox::CheckboxShape, wraps_in_option = false)))]`
- `#[gpui_form(component(custom(shape = gpui_form_collection::switch::SwitchShape, wraps_in_option = false)))]`
- `#[gpui_form(component(custom(shape = "gpui_form::infinite_select::InfiniteSelectState<_>", wraps_in_option = false)))]`

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
- collection select shapes expect enum-like values that can populate a
  `gpui_component` select
- `gpui_form::infinite_select::InfiniteSelectState<_>` expects the field type
  to implement `gpui_form::InfiniteSelect`
- `default = ...` seeds the generated value holder
- `custom(..., wraps_in_option = false)` keeps the generated value-holder field
  as `T` instead of `Option<T>`
- `custom(..., value_binding)` records that the custom shape implements
  `gpui_form::custom::CustomComponentValueAdapter<T>` for generated
  prototyping subscriptions
- `type`/`from`/`into` let the generated holder edit a type that differs from
  the original model field
- `gpui_form_collection::input::InputShape<_>` prototyping code parses
  form-side non-`String` values with `FromStr` instead of assigning raw
  `String`s
- field-level `#[koruma(...)]` attributes are accepted by `GpuiForm` and copied
  onto the generated value holder, which allows validating form-side override
  types without deriving `Koruma` on the original model
- when skipped fields are present, the generated value holder keeps builder
  support and exposes `into_original(...)` instead of an unconditional reverse
  conversion

## `#[derive(SelectItem)]`

Implements `gpui_component::select::SelectItem` for enums.

```rs
use gpui_form::SelectItem;

#[derive(Clone, Debug, SelectItem)]
pub enum Country {
    USA,
    France,
}
```

Optional attribute:

- `#[select_item(fluent)]` allows enums that derive `EsFluent` to avoid a
  `Display` bound, but `SelectItem::title()` has no localizer argument. Render
  localized select labels in the application layer when localization is needed.

## `#[derive(CustomComponent)]` / `#[derive(CustomComponentState)]`

Implements `gpui_form::custom::CustomComponentShape` directly for a state type.
`CustomComponentState` is kept as a compatibility alias; new reusable custom
component metadata should prefer the shorter `CustomComponent` name.

```rs
use gpui_form::CustomComponent;

#[derive(Clone, Debug, CustomComponent)]
#[gpui_form_custom(
    new = crate::state::build,
    component = crate::ui::TagsInput,
    value_binding
)]
pub struct TagsState;
```

By default, the generated implementation calls `Self::new(window, cx)`.
`component = ...` populates `CustomComponentShape::COMPONENT_PATH`.
`value_binding` sets `CustomComponentShape::VALUE_BINDING = true`, so generated
prototyping code can inherit the shape's `CustomComponentValueAdapter<T>`
contract without repeating `value_binding` on every field.

## `custom_component!`

Declares a local shape type for external component/state pairs.

```rs
gpui_form::custom_component! {
    /// Ready-made text input shape.
    pub struct InputShape<T = String>
    where
        T: std::str::FromStr + ToString + 'static,
    {
        type State = gpui_component::input::InputState;
        new = |window, cx| gpui_component::input::InputState::new(window, cx)
            .validate(|value, _| value.parse::<T>().is_ok());
        component = gpui_component::input::Input;
        value_binding;
    }
}
```

Use this when the component and state live in another crate. The macro creates
the local wrapper type that owns the `CustomComponentShape` implementation; the
caller can still add a `CustomComponentValueAdapter<T>` impl on that wrapper.

## Feature Flags

- `inventory`: enables `GpuiFormShape` registration for `#[derive(GpuiForm)]`

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for the main facade
- [`gpui-form-component-derive`](../gpui-form-component-derive/README.md) for
  `#[derive(InfiniteSelect)]`
- [`gpui-form-schema`](../gpui-form-schema/README.md) when you need metadata
  rather than derives
