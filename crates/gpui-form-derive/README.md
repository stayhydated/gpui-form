# gpui-form-derive

Procedural macros behind the `gpui-form` ecosystem.

Most users should depend on [`gpui-form`](../gpui-form/README.md) and derive
from the facade crate. Use this crate directly when you want the proc-macro
layer, including custom component shape derives and helper macros.

## What This Crate Provides

- `#[derive(GpuiForm)]`
- `#[derive(CustomComponent)]`
- `custom_component! { ... }`

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
- `#[gpui_form(component = gpui_form_collection::input::Input::<_>)]`
- `#[gpui_form(component = gpui_form_collection::select::Select::<_>)]`
- `#[gpui_form(component = gpui_form_collection::select::Select::<_>::searchable(true).partial(true))]`
- `#[gpui_form(component = gpui_form_collection::checkbox::Checkbox)]`
- `#[gpui_form(component = gpui_form_collection::switch::Switch)]`
- `#[gpui_form(component = gpui_form_component::infinite_select::InfiniteSelect::<_>::searchable(true).max_depth(3))]`

The expression is parsed as attribute metadata; generated runtime construction
delegates to `CustomComponentShape::new`. Select and infinite-select behavior
uses Koruma-style direct setter chains that expand to bon builders inside the
derive macro.

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
- collection select components expect enum-like values that can populate a
  `gpui_component` select
- `gpui_form_component::infinite_select::InfiniteSelect::<_>` expects the field type
  to derive `gpui_form_component::InfiniteSelect`, which implements
  `gpui_form_component::infinite_select::InfiniteSelectValue`
- `default = ...` seeds the generated value holder
- known reusable components infer required-value behavior internally:
  `gpui_form_collection::input::Input` allows required source fields to be
  absent in the holder until validation or conversion, while select, checkbox,
  switch, and infinite-select components keep required fields as `T`
- `searchable(true)` and `.partial(true)` record select behavior metadata
- `searchable(true)` and `.max_depth(...)` record infinite-select behavior
  metadata
- `.value_binding()` records that the custom shape implements
  `gpui_form_component::custom::CustomComponentValueBinding<T>` for generated
  prototyping subscriptions; the adapter seeds component state with
  `seed_value_binding_state` and maps native events to `FormValueEvent<T>`
- `type`/`from`/`into` let the generated holder edit a type that differs from
  the original model field
- `gpui_form_collection::input::Input::<_>` prototyping code parses
  form-side non-`String` values with `FromStr` instead of assigning raw
  `String`s
- generic component expressions use `::<_>` in the attribute; the derive normalizes
  the path and resolves `_` to the field's form-side type
- custom shape prototyping metadata drives generated `FormFields` and
  `FormComponents` suffixes when present. Collection shapes publish suffixes
  such as `input`, `select`, `checkbox`, and `switch`; reusable app shapes can
  use `field_suffix = "..."`, and otherwise generated identifiers fall back to
  the shape name heuristic
- field-level `#[koruma(...)]` attributes are accepted by `GpuiForm` and copied
  onto the generated value holder, which allows validating form-side override
  types without deriving `Koruma` on the original model
- when skipped fields are present, the generated value holder keeps builder
  support and exposes `into_original(...)` instead of an unconditional reverse
  conversion

## `#[derive(CustomComponent)]`

Implements `gpui_form_component::custom::CustomComponentShape` directly for a state type.

```rs
use gpui_form_derive::CustomComponent;

#[derive(Clone, Debug, CustomComponent)]
#[gpui_form_custom(
    new = crate::state::build,
    component = crate::ui::TagsInput,
    value_binding,
    field_suffix = "input"
)]
pub struct TagsState;
```

By default, the generated implementation calls `Self::new(window, cx)`.
`component = ...` populates `CustomComponentShape::COMPONENT_PATH`.
`value_binding` sets `CustomComponentShape::VALUE_BINDING = true`, so generated
prototyping code can inherit the shape's `CustomComponentValueBinding<T>`
contract without repeating `value_binding` on every field.
`field_suffix = "..."` populates `CustomComponentShape::PROTOTYPING`, giving
prototyping generators a reusable field/helper suffix without relying on shape
name heuristics.

## `custom_component!`

Declares a local shape type for external component/state pairs.

```rs
gpui_form_derive::custom_component! {
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
the local wrapper type that owns the `CustomComponentShape` implementation; the
caller can still add a `CustomComponentValueBinding<T>` impl on that wrapper.

## Feature Flags

- `inventory`: enables `GpuiFormShape` registration for `#[derive(GpuiForm)]`

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for the main facade
- [`gpui-form-collection-derive`](../gpui-form-collection-derive/README.md) for
  `#[derive(SelectItem)]` when using derives directly
- [`gpui-form-component`](../gpui-form-component/README.md) for runtime helpers,
  custom component contracts, and `#[derive(InfiniteSelect)]` with its `derive`
  feature
- [`gpui-form-schema`](../gpui-form-schema/README.md) when you need metadata
  rather than derives
