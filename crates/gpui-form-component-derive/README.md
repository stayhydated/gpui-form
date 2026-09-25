# gpui-form-component-derive

[![Codecov: gpui-form-component-derive][codecov-badge]][codecov]
[![crates.io: gpui-form-component-derive][crate-badge]][crate]

`gpui-form-component-derive` provides `#[derive(InfiniteSelect)]` for nested enum
trees used by `gpui-form-component`.

Most applications enable the `derive` feature on
[`gpui-form-component`][gpui-form-component], which re-exports the macro as
`gpui_form_component::InfiniteSelect`. Derived enum trees must implement
`Clone + Default + PartialEq + 'static`; nested payload types must also
implement `Default`.

[codecov-badge]: https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg?branch=master&component=gpui-form-component-derive
[codecov]: https://codecov.io/github/stayhydated/gpui-form
[crate-badge]: https://img.shields.io/crates/v/gpui-form-component-derive.svg?label=gpui-form-component-derive
[crate]: https://crates.io/crates/gpui-form-component-derive
[gpui-form-component]: https://github.com/stayhydated/gpui-form/blob/master/crates/gpui-form-component/README.md
