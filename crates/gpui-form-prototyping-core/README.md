# gpui-form-prototyping-core

[![Codecov: gpui-form-prototyping-core][codecov-badge]][codecov]
[![crates.io: gpui-form-prototyping-core][crate-badge]][crate]

`gpui-form-prototyping-core` generates GPUI form scaffolds from concrete
`GpuiFormShape` inventory registrations for application-owned layouts.

Enable `gpui-form/inventory` and link the crate that owns the concrete forms
into the generator. Implement `FormLayout` for the target view, then pass each
registration to `FormShapeAdapter::generate_file(...)`.

`FormShapeAdapter::parts()` exposes validated fragments for custom layouts,
while `generate_file(...)` renders a complete `syn::File`. Generators that write
into another crate can remap source paths before rendering.

[codecov-badge]: https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg?branch=master&component=gpui-form-prototyping-core
[codecov]: https://codecov.io/github/stayhydated/gpui-form
[crate-badge]: https://img.shields.io/crates/v/gpui-form-prototyping-core.svg?label=gpui-form-prototyping-core
[crate]: https://crates.io/crates/gpui-form-prototyping-core
