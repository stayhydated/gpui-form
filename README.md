# gpui-form

[![CI][ci-badge]][ci]
[![Codecov][codecov-badge]][codecov]
[![Book][book-badge]][book]
[![crates.io: gpui-form][gpui-form-badge]][gpui-form-crate]

`gpui-form` generates typed form state for [GPUI Kit][gpui-kit] applications.
Application developers annotate model fields once, then use generated holders
and components to render, validate, and reconstruct those models.

## Overview

The `GpuiForm` derive generates typed field identities, GPUI entity storage,
component constructors, and a form value holder. Every field uses exactly one
`component(...)`, `hidden`, or `skip` intent.

## Crates

| Crate | Purpose | Source |
| --- | --- | --- |
| `gpui-form` | Public derive, facade, generated runtime paths, schema access, and optional MCP integration | [README][gpui-form-readme] |
| `gpui-form-collection` | Ready-made form shapes for common GPUI Kit controls | [README][collection-readme] |
| `gpui-form-component` | Localized date and file pickers plus cascading infinite-select support | [README][component-readme] |
| `gpui-form-prototyping-core` | GPUI form scaffolds generated from inventory metadata | [README][prototyping-readme] |

Most applications start with `gpui-form` and add only the component crates they
use. The [workspace examples][examples] demonstrate complete form views,
component galleries, scaffold generation, and MCP servers.

## Example

```rust
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct Profile {
    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    pub display_name: String,
}
```

[ci-badge]: https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml/badge.svg?branch=master
[ci]: https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml
[codecov-badge]: https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg?branch=master
[codecov]: https://codecov.io/github/stayhydated/gpui-form
[book-badge]: https://img.shields.io/badge/Book-mdBook-blue
[book]: https://stayhydated.github.io/gpui-form/book/
[gpui-form-badge]: https://img.shields.io/crates/v/gpui-form.svg?label=gpui-form
[gpui-form-crate]: https://crates.io/crates/gpui-form
[gpui-kit]: https://github.com/longbridge/gpui-kit
[gpui-form-readme]: crates/gpui-form/README.md
[collection-readme]: crates/gpui-form-collection/README.md
[component-readme]: crates/gpui-form-component/README.md
[prototyping-readme]: crates/gpui-form-prototyping-core/README.md
[examples]: examples/README.md
