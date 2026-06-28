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
    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    pub code: AccountCode,
}
```

The `_` in `Input::<_>` is resolved by `GpuiForm` to the field's
form-side type. Generic component expressions use Rust turbofish syntax inside
the attribute and are normalized back into base shape type paths by the derive.
`Combobox<T>` is different: `T` is the item type for a `Vec<T>` field, so write
`Combobox::<Country>` rather than `Combobox::<_>`.

Currently provided components:

- `input::Input<T>` for `gpui_component::input::Input`
- `input::ParsedInput<T, C>` for `gpui_component::input::Input` with an
  application-owned parser, formatter, placeholder, empty-as-clear policy, and
  optional widget validation
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

Use `ParsedInput<T, C>` when `Input<T>`'s `FromStr + ToString` behavior is too
plain for a value object:

```rs
pub struct AccountCode(String);
pub struct AccountCodeInputConfig;

impl gpui_form_collection::input::ParsedInputConfig<AccountCode> for AccountCodeInputConfig {
    type Error = AccountCodeParseError;

    const PLACEHOLDER: Option<&'static str> = Some("Account code");

    fn parse(value: &str) -> Result<AccountCode, Self::Error> {
        AccountCode::new(value.trim())
    }

    fn format(value: &AccountCode) -> String {
        value.0.clone()
    }
}

#[derive(Clone, Debug, GpuiForm)]
pub struct Account {
    #[gpui_form(component(gpui_form_collection::input::ParsedInput::<_, AccountCodeInputConfig>))]
    pub code: AccountCode,
}
```

`Select<T, D>` requires enum values that implement
`gpui_component::select::SelectItem`; derive that trait with
`#[derive(SelectItem)]` from `gpui-form-collection-derive`. The provided shape
uses the default `Vec<T>` delegate. Use
`#[gpui_form(component(gpui_form_collection::select::Select::<_>::searchable(true)))]`
when a field should construct the select with search enabled. For a completed
configuration value, use
`Select::<_>::from(SelectArgs::builder().searchable(true).build())`.

Collection components are declared with `component-shape-gpui` and add value
adapters where the component can synchronize form state generically. `GpuiForm`
generated code uses these contracts through the facade path
`gpui_form::runtime::shape`, while this crate attaches the `gpui-form` storage
policy locally.
Components that synthesize a default value, such as input, select, combobox,
checkbox, switch, number input, slider, and OTP input, publish direct `T`
value-holder storage as their default value-storage policy. For combobox,
empty selection is explicit: value binding emits `ValueChange::Clear`, so
optional fields clear to `None` and non-optional `Vec<T>` fields reset to
`Vec::default()`. They also publish prototyping field suffix metadata, so
generated scaffolds use names such as `code_input`, `country_select`,
`theme_combobox`, `notifications_switch`, `age_number_input`, `volume_slider`,
`theme_color_picker`, `birth_date_picker`, `holiday_date_range_picker`, and
`otp_code_otp_input` without relying on shape-name fallbacks.
