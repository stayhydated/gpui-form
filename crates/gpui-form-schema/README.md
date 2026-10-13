# gpui-form-schema

[![Codecov: gpui-form-schema][codecov-badge]][codecov]
[![crates.io: gpui-form-schema][crate-badge]][crate]

`gpui-form-schema` provides schema and inventory metadata for generators and
runtime integrations in the `gpui-form` ecosystem. Applications normally use
it through the [`gpui-form` facade][gpui-form] as `gpui_form::schema`.

Requires Rust 1.99 or newer.

## Overview

Key entry points are:

- `registry::GpuiFormShape` and `registry::inventory` for registered form
  metadata
- `registry::FieldVariant` and component/value metadata types
- `resolved::ResolvedGpuiFormShape` and `resolved::ResolvedField` for validated,
  parsed generator input

Use the resolved types to validate and parse metadata before generating Rust
code.

[codecov-badge]: https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg?branch=master&component=gpui-form-schema
[codecov]: https://codecov.io/github/stayhydated/gpui-form
[crate-badge]: https://img.shields.io/crates/v/gpui-form-schema.svg?label=gpui-form-schema
[crate]: https://crates.io/crates/gpui-form-schema
[gpui-form]: https://github.com/stayhydated/gpui-form/blob/master/crates/gpui-form/README.md
