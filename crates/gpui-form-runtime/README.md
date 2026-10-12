# gpui-form-runtime

[![Codecov: gpui-form-runtime][codecov-badge]][codecov]
[![crates.io: gpui-form-runtime][crate-badge]][crate]

`gpui-form-runtime` provides GPUI component-shape and value-storage contracts
for reusable shapes and lower-level integrations. Applications normally access
them through the [`gpui-form` facade][gpui-form] with `runtime` enabled,
using `gpui_form::runtime::shape`.

Requires Rust 1.99 or newer and GPUI Kit 0.7.

## Overview

The public surface includes:

- re-exported `component-shape-gpui` construction, rendering, builder, and
  value-binding contracts
- `GpuiFormComponentShapePolicy`
- `DirectValueStorage` and `RequiredValueStorage`
- helpers for seeding component state and converting events to `ValueChange<T>`

[codecov-badge]: https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg?branch=master&component=gpui-form-runtime
[codecov]: https://codecov.io/github/stayhydated/gpui-form
[crate-badge]: https://img.shields.io/crates/v/gpui-form-runtime.svg?label=gpui-form-runtime
[crate]: https://crates.io/crates/gpui-form-runtime
[gpui-form]: https://github.com/stayhydated/gpui-form/blob/master/crates/gpui-form/README.md
