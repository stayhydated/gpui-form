# gpui-form-prototyping-core

[![Codecov: gpui-form-prototyping-core][codecov-badge]][codecov]
[![crates.io: gpui-form-prototyping-core][crate-badge]][crate]

`gpui-form-prototyping-core` generates GPUI form scaffolds from concrete
`GpuiFormShape` inventory registrations for application-owned layouts.

Requires Rust 1.99 or newer.

Enable `inventory` on the [`gpui-form` facade][gpui-form] and link the crate
that owns the concrete forms into the generator. Implement `FormLayout` for
the target view, then pass each registration to `FormShapeAdapter::generate_file(...)`.

## Overview

`FormShapeAdapter::parts()` exposes validated fragments for custom layouts,
while `generate_file(...)` renders a complete `syn::File`. Generators that write
into another crate can remap source paths before rendering.

## Features

No features are enabled by default.

| Feature | Default | Purpose |
| --- | --- | --- |
| `fluent` | No | Generate `gpui-es-fluent` calls for field labels and validation messages. |

[codecov-badge]: https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg?branch=master&component=gpui-form-prototyping-core
[codecov]: https://codecov.io/github/stayhydated/gpui-form
[crate-badge]: https://img.shields.io/crates/v/gpui-form-prototyping-core.svg?label=gpui-form-prototyping-core
[crate]: https://crates.io/crates/gpui-form-prototyping-core
[gpui-form]: https://github.com/stayhydated/gpui-form/blob/master/crates/gpui-form/README.md
