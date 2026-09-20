# gpui-form-collection

[![Codecov: gpui-form-collection][codecov-badge]][codecov]
[![crates.io: gpui-form-collection][crate-badge]][crate]

`gpui-form-collection` provides ready-made form shapes for common GPUI Kit
controls, letting application developers assign typed UI behavior directly to
model fields.

## Overview

| Module | Shapes |
| --- | --- |
| `input` | `Input<T>` and configurable `ParsedInput<T, Config>` |
| `select` and `combobox` | Single- and multi-value enum choices |
| `checkbox` and `switch` | Boolean controls |
| `number_input` and `slider` | Numeric controls |
| `color_picker` | `gpui_kit::Hsla` selection |
| `date_picker` | Single-date and date-range selection |
| `otp_input` | One-time-password input |

Use [`gpui-form-collection-derive`][collection-derive] for
`#[derive(SelectItem)]`.

## Example

```rust
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct Account {
    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    pub account_code: String,
}
```

[codecov-badge]: https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg?branch=master&component=gpui-form-collection
[codecov]: https://codecov.io/github/stayhydated/gpui-form
[crate-badge]: https://img.shields.io/crates/v/gpui-form-collection.svg?label=gpui-form-collection
[crate]: https://crates.io/crates/gpui-form-collection
[collection-derive]: https://github.com/stayhydated/gpui-form/blob/master/crates/gpui-form-collection-derive/README.md
