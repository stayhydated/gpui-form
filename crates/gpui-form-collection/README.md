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
    #[gpui_form(component(custom(shape = "gpui_form_collection::input::InputShape<_>")))]
    pub code: AccountCode,
}
```

The `_` in `InputShape<_>` is resolved by `GpuiForm` to the field's form-side
type. Generic shape paths use a string literal because Rust attribute
name-value expressions do not parse `<_>` paths directly.

Currently provided shapes:

- `input::InputShape<T>` for `gpui_component::input::Input`
- `select::SelectShape<T>` for enum-backed `gpui_component::select::Select`
- `checkbox::CheckboxShape` for `gpui_component::checkbox::Checkbox`
- `switch::SwitchShape` for `gpui_component::switch::Switch`

`SelectShape<T>` requires enum values that implement
`gpui_component::select::SelectItem`; derive that trait with
`#[derive(SelectItem)]` (or import it from
`gpui_form::SelectItem` through the facade).

Collection shapes are declared with `gpui_form::custom_component!` and then
add value adapters where the component can synchronize form state generically.
