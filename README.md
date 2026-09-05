[![Build Status](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml)
[![Codecov](https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg)](https://codecov.io/github/stayhydated/gpui-form)
[![Docs](https://docs.rs/gpui-form/badge.svg)](https://docs.rs/gpui-form/)
[![Crates.io](https://img.shields.io/crates/v/gpui-form.svg)](https://crates.io/crates/gpui-form)

# gpui-form

`gpui-form` generates typed form state for
[GPUI Kit](https://github.com/longbridge/gpui-kit) applications. Add
`#[derive(GpuiForm)]` to an application model, assign each field a form intent,
and use the generated holder and component types to render, validate, and
reconstruct the model.

## Install

Use the published GPUI Kit facade:

```toml
[dependencies]
gpui-kit = "0.6.0"
gpui-form = "0.6"
gpui-form-collection = "0.6"
```

## Define a form

```rust
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, GpuiForm)]
pub struct Profile {
    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    pub display_name: String,
}
```

The derive generates typed field identities, GPUI entity storage, component
constructors, and `ProfileFormValueHolder`. Every field uses exactly one
`component(...)`, `hidden`, or `skip` intent.

## Choose a crate

| Crate | Use it for |
|---|---|
| `gpui-form` | The public derive, facade, generated runtime paths, schema access, and optional MCP integration |
| `gpui-form-collection` | Ready-made form shapes for common GPUI Kit controls |
| `gpui-form-component` | Localized date and file pickers plus cascading infinite-select support |
| `gpui-form-prototyping-core` | Generating GPUI form scaffolds from inventory metadata |

Most applications should start with `gpui-form` and add only the component
crates they use.

## Documentation

- [User guide](https://stayhydated.github.io/gpui-form/book/)
- [API documentation](https://docs.rs/gpui-form/)
- [Workspace examples](examples/README.md)

Licensed under MIT or Apache-2.0.
