# gpui-form-collection

Ready-made `gpui-form` shapes for common GPUI Kit controls.

```toml
[dependencies]
gpui-form = "0.6"
gpui-form-collection = "0.6"
```

Use a shape as the field's component intent:

```rust
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct Account {
    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    pub account_code: String,
}
```

Available shapes include:

| Module | Shapes |
|---|---|
| `input` | `Input<T>` and configurable `ParsedInput<T, Config>` |
| `select` and `combobox` | Single- and multi-value enum choices |
| `checkbox` and `switch` | Boolean controls |
| `number_input` and `slider` | Numeric controls |
| `color_picker` | `gpui_kit::Hsla` selection |
| `date_picker` | Single-date and date-range selection |
| `otp_input` | One-time-password input |

Use
[`gpui-form-collection-derive`](../gpui-form-collection-derive/README.md) for
`#[derive(SelectItem)]`. See
[Component shapes](https://stayhydated.github.io/gpui-form/book/component_shapes.html)
for supported value types, configuration, and storage behavior.
