# gpui-form-collection-derive

Collection-oriented procedural macros for `gpui-form`. This crate provides
`#[derive(SelectItem)]` for enum values used by collection-backed selects.

```toml
[dependencies]
gpui-form-collection-derive = "0.6"
strum = { version = "0.28", features = ["derive"] }
```

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

Variant names provide fallback labels. Add `#[select_item(display)]` when
`SelectItem::title()` should use the enum's `Display` implementation.

See the [API documentation](https://docs.rs/gpui-form-collection-derive/) for
derive attributes.
