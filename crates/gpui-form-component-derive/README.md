# gpui-form-component-derive

Proc macros for the `InfiniteSelect` runtime surface.

Most applications should use [`gpui-form-component`](../gpui-form-component/README.md)
with its `derive` feature, which re-exports `#[derive(InfiniteSelect)]` as
`gpui_form_component::InfiniteSelect`. Use this crate directly when you want
only the infinite-select derive plus the runtime crate.

## Direct Use

When you use this crate directly, depend on the runtime crate normally. The
macro resolves `gpui-form-component` as the runtime crate:

```toml
[dependencies]
gpui-form-component = "*"
gpui-form-component-derive = "*"
```

## `#[derive(InfiniteSelect)]`

Implements the runtime crate's `InfiniteSelect` contract for nested enums used
by cascading selects.

```rs
use gpui_form_component::infinite_select::{InfiniteSelectPath, build_from_path};
use gpui_form_component_derive::InfiniteSelect;

#[derive(Clone, Debug, Default, InfiniteSelect, PartialEq)]
pub enum Country {
    #[default]
    USA(USAState),
    Canada(CanadaProvince),
    UK,
}
```

Variant attributes:

- `#[tuple_enum(skip)]` omits a variant from the select tree
- `#[tuple_enum(key = "...")]` overrides the stable persisted key for a variant
- `#[fluent_kv(keys = ["label", "description"], keys_this)]` emits
  `EsFluentVariants` / `EsFluentLabel` metadata for application-owned
  localizers. Runtime labels use plain fallback names because the runtime trait
  contract is localizer-free.

Behavior notes:

- derived enums must also implement `PartialEq` because the runtime
  `gpui-component` select compares selected values
- derived enums expose stable `variant_key()` values plus `selection_key_path()`
- custom keys are validated for uniqueness within the enum
- fluent metadata is emitted for callers that render through their own
  `es-fluent` localizer; runtime trait methods use fallback names because the
  contract is localizer-free

## Most Users Should Use Instead

- [`gpui-form-component`](../gpui-form-component/README.md) for the runtime
  state helpers targeted by the derive and its `derive` feature re-export
- [`gpui-form-derive`](../gpui-form-derive/README.md) for `GpuiForm`
  and component-shape derives
- [`gpui-form-collection-derive`](../gpui-form-collection-derive/README.md) for
  `SelectItem`
