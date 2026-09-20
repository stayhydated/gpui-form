# gpui-form-collection-derive

[![Codecov: gpui-form-collection-derive][codecov-badge]][codecov]
[![crates.io: gpui-form-collection-derive][crate-badge]][crate]

`gpui-form-collection-derive` provides `#[derive(SelectItem)]` for enum values
used by collection-backed GPUI selects.

Variant names provide fallback labels. Add `#[select_item(display)]` when
`SelectItem::title()` should use the enum's `Display` implementation.

## Example

```rust
use gpui_form_collection_derive::SelectItem;
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, PartialEq, SelectItem)]
pub enum Country {
    #[default]
    UnitedStates,
    Canada,
}
```

[codecov-badge]: https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg?branch=master&component=gpui-form-collection-derive
[codecov]: https://codecov.io/github/stayhydated/gpui-form
[crate-badge]: https://img.shields.io/crates/v/gpui-form-collection-derive.svg?label=gpui-form-collection-derive
[crate]: https://crates.io/crates/gpui-form-collection-derive
