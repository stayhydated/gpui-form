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
    #[gpui_form(gpui_form_collection::input::Input::<_>)]
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
- `combobox::Combobox<T, D = Vec<T>>` for enum-backed
  `gpui_component::combobox::Combobox`
- `checkbox::Checkbox` for `gpui_component::checkbox::Checkbox`
- `switch::Switch` for `gpui_component::switch::Switch`
- `number_input::NumberInput<T>` for `gpui_component::input::NumberInput`
- `slider::Slider` for `gpui_component::slider::Slider`
- `color_picker::ColorPicker` for `gpui_component::color_picker::ColorPicker`
- `date_picker::DatePicker` for `gpui_component::date_picker::DatePicker`
- `date_picker::DateRangePicker` for range-mode
  `gpui_component::date_picker::DatePicker`
- `otp_input::OtpInput<T>` for `gpui_component::input::OtpInput`

`Select<T, D>` requires enum values that implement
`gpui_component::select::SelectItem`; derive that trait with
`#[derive(SelectItem)]` from `gpui-form-collection-derive`. The provided shape
uses the default `Vec<T>` delegate. If an application needs search or other
select-specific configuration, define a small `ComponentShape` wrapper whose
`new` function configures the underlying `SelectState`.

Collection components implement `gpui_form_runtime::shape::ComponentShape`
and add value adapters where the component can synchronize form state
generically. They also publish prototyping field suffix metadata, so generated
scaffolds use names such as `code_input`, `country_select`,
`theme_combobox`, `notifications_switch`, `age_number_input`, `volume_slider`,
`theme_color_picker`, `birth_date_picker`, `holiday_date_range_picker`, and
`otp_code_otp_input` without relying on shape-name fallbacks.
