# gpui-form-component-derive

The `#[derive(InfiniteSelect)]` procedural macro for nested enum trees used by
`gpui-form-component`.

Most applications should enable the `derive` feature on
[`gpui-form-component`](../gpui-form-component/README.md), which re-exports the
macro as `gpui_form_component::InfiniteSelect`. Depend on this crate directly
only when the separate proc-macro dependency is useful.

Derived enum trees must implement `Clone + Default + PartialEq + 'static`;
nested payload types must also implement `Default`.

See the [API documentation](https://docs.rs/gpui-form-component-derive/) for
variant keys, skipped variants, and Fluent metadata.
