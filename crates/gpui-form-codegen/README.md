# gpui-form-codegen

[![Codecov: gpui-form-codegen][codecov-badge]][codecov]
[![crates.io: gpui-form-codegen][crate-badge]][crate]

`gpui-form-codegen` provides shared parsing and token-generation support for the
`gpui-form` procedural macros. It serves macro authors and workspace
implementation crates rather than application code.

Application developers should use [`gpui-form`][gpui-form]. Proc-macro
consumers that intentionally avoid the facade can use
[`gpui-form-derive`][gpui-form-derive].

[codecov-badge]: https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg?branch=master&component=gpui-form-codegen
[codecov]: https://codecov.io/github/stayhydated/gpui-form
[crate-badge]: https://img.shields.io/crates/v/gpui-form-codegen.svg?label=gpui-form-codegen
[crate]: https://crates.io/crates/gpui-form-codegen
[gpui-form]: https://github.com/stayhydated/gpui-form/blob/master/crates/gpui-form/README.md
[gpui-form-derive]: https://github.com/stayhydated/gpui-form/blob/master/crates/gpui-form-derive/README.md
