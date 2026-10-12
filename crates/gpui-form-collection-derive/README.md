# gpui-form-collection-derive

[![Codecov: gpui-form-collection-derive][codecov-badge]][codecov]
[![crates.io: gpui-form-collection-derive][crate-badge]][crate]

`gpui-form-collection-derive` provides `#[derive(SelectItem)]` for enum values
used by [`gpui-form-collection`][collection] selects. Depend on this macro
crate directly when deriving select metadata for application enums.

Requires Rust 1.99 or newer.

Variant names provide fallback labels. Add `#[select_item(display)]` when
`SelectItem::title()` should use the enum's `Display` implementation.

## Example

This complete example uses GPUI Kit's `SelectItem` trait to inspect the title
emitted by the derive. It needs no GPUI window; the [maintained consumer
fixture][example] exercises the same declaration.

```rust
use gpui_kit::component::select::SelectItem as _;
use gpui_form_collection_derive::SelectItem;

#[derive(Clone, PartialEq, SelectItem)]
enum Country {
    UnitedStates,
    Canada,
}

assert_eq!(Country::UnitedStates.title().as_ref(), "UnitedStates");
assert_eq!(Country::Canada.title().as_ref(), "Canada");
```

[codecov-badge]: https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg?branch=master&component=gpui-form-collection-derive
[codecov]: https://codecov.io/github/stayhydated/gpui-form
[crate-badge]: https://img.shields.io/crates/v/gpui-form-collection-derive.svg?label=gpui-form-collection-derive
[crate]: https://crates.io/crates/gpui-form-collection-derive
[collection]: https://github.com/stayhydated/gpui-form/blob/master/crates/gpui-form-collection/README.md
[example]: https://github.com/stayhydated/gpui-form/blob/master/crates/gpui-form-collection-derive/tests/ui/select_item_no_display.rs
