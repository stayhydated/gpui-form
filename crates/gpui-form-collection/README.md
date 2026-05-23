# gpui-form-collection

Curated component shapes for `gpui-form`.

Most applications should start with [`gpui-form`](../gpui-form/README.md) for
the derive and contract APIs, then add this crate when they want ready-made
representations for common `gpui-component` widgets.

## Shapes

```rs
use gpui_form::GpuiForm;

#[derive(Clone, Debug, GpuiForm)]
pub struct Account {
    #[gpui_form(component = gpui_form_collection::input::InputShape::<_>)]
    pub code: AccountCode,
}
```

The `_` in `InputShape::<_>` is resolved by `GpuiForm` to the field's
form-side type. Generic shape expressions use Rust turbofish syntax inside the
attribute and are normalized back into type paths by the derive.

Currently provided shapes:

- `input::InputShape<T>` for `gpui_component::input::Input`
- `select::SelectShape<T, D = Vec<T>>` for enum-backed
  `gpui_component::select::Select`
- `checkbox::CheckboxShape` for `gpui_component::checkbox::Checkbox`
- `switch::SwitchShape` for `gpui_component::switch::Switch`

`SelectShape<T, D>` requires enum values that implement
`gpui_component::select::SelectItem`; derive that trait with
`#[derive(SelectItem)]` from `gpui-form-collection-derive`. The derive uses the
default `Vec<T>` delegate unless the field annotation enables searchable select
behavior, which specializes the shape to
`gpui_component::select::SearchableVec<T>`.

Collection shapes implement `gpui_form_component::custom::CustomComponentShape`
and add value adapters where the component can synchronize form state
generically. They also publish prototyping field suffix metadata, so generated
scaffolds use names such as `code_input`, `country_select`,
`enabled_checkbox`, and `notifications_switch` without relying on shape-name
fallbacks.
