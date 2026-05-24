# gpui-form-collection-derive

Proc macros for collection-oriented workflows in `gpui-form`.

Use this crate for collection-specific derive support, currently
`#[derive(SelectItem)]`. `gpui-form` does not re-export this derive.

## Installation

```toml
[dependencies]
gpui-form-collection-derive = "*"
```

## `#[derive(SelectItem)]`

Implements `gpui_component::select::SelectItem` for enums that drive
collection-backed select controls.

```rs
use gpui_form_collection_derive::SelectItem;
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, PartialEq, SelectItem)]
pub enum Country {
    #[default]
    UnitedStates,
    Canada,
}
```

Optional attribute:

- `#[select_item(fluent)]` uses fallback variant names when the enum implements
  `EsFluent` and should avoid a `Display` bound. The generated
  `SelectItem::title()` output remains plain fallback text because this contract
  has no localizer argument.

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for standard form derives and schema
  wiring.
- [`gpui-form-derive`](../gpui-form-derive/README.md) for `GpuiForm`,
  `CustomComponent`, and `custom_component!`.
