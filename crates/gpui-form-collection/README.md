# gpui-form-collection

Curated form components for `gpui-form`.

Most applications should start with [`gpui-form`](../gpui-form/README.md) for
the derive and contract APIs, then add this crate when they want ready-made
representations for common `gpui-component` widgets.

## Components

```rs
use gpui_form::GpuiForm;

#[derive(Clone, Debug, GpuiForm)]
pub struct Account {
    #[gpui_form(component = gpui_form_collection::input::Input::<_>)]
    pub code: AccountCode,
}
```

The `_` in `Input::<_>` is resolved by `GpuiForm` to the field's
form-side type. Generic component expressions use Rust turbofish syntax inside the
attribute and are normalized back into type paths by the derive.

Currently provided components:

- `input::Input<T>` for `gpui_component::input::Input`
- `select::Select<T, D = Vec<T>>` for enum-backed
  `gpui_component::select::Select`
- `checkbox::Checkbox` for `gpui_component::checkbox::Checkbox`
- `switch::Switch` for `gpui_component::switch::Switch`

`Select<T, D>` requires enum values that implement
`gpui_component::select::SelectItem`; derive that trait with
`#[derive(SelectItem)]` from `gpui-form-collection-derive`. The derive uses the
default `Vec<T>` delegate unless the field annotation enables searchable select
behavior, which specializes the shape to
`gpui_component::select::SearchableVec<T>`.

Collection components implement `gpui_form_component::shape::ComponentShape`
and add value adapters where the component can synchronize form state
generically. They also publish prototyping field suffix metadata, so generated
scaffolds use names such as `code_input`, `country_select`,
`enabled_checkbox`, and `notifications_switch` without relying on shape-name
fallbacks.
