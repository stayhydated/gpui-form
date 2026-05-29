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

- By default, `SelectItem::title()` uses the enum variant name as fallback text
  and does not require `Display`.
- `#[select_item(display)]` uses `self.to_string()` when the enum intentionally
  provides display text.
- `#[select_item(fluent)]` is accepted for enums that also derive `EsFluent`;
  the generated `SelectItem::title()` output remains plain fallback text
  because this contract has no localizer argument.

`display` and `fluent` are mutually exclusive.

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for standard form derives and schema
  wiring.
- [`gpui-form-derive`](../gpui-form-derive/README.md) for `GpuiForm`,
  `ComponentShape`, and `component_shape!`.
