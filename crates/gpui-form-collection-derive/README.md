# gpui-form-collection-derive

Proc macros for collection-oriented workflows in `gpui-form`.

Most users should depend on [`gpui-form`](../gpui-form/README.md) for the full
derive surface and default re-exports. Use this crate when you only need
collection-specific derive support, currently `#[derive(SelectItem)]`.

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

`SelectItem` is re-exported by the `gpui-form` facade as
`gpui_form::SelectItem`.

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for standard form derives, custom
  component shapes, runtime, and schema wiring.
- [`gpui-form-derive`](../gpui-form-derive/README.md) for `GpuiForm`,
  `CustomComponent`, and `CustomComponentState`.
