# gpui-form-schema

Schema and inventory metadata for the `gpui-form` ecosystem.

Most applications access this crate through `gpui_form::schema`. Depend on it
directly when building metadata consumers, generators, or runtime integrations
without the full facade.

Key entry points are:

- `registry::GpuiFormShape` and `registry::inventory` for registered form
  metadata
- `registry::FieldVariant` and component/value metadata types
- `resolved::ResolvedGpuiFormShape` and `resolved::ResolvedField` for validated,
  parsed generator input

Prefer the resolved types when generating Rust code. See
[Prototyping](https://stayhydated.github.io/gpui-form/book/prototyping.html) for
the user workflow and [docs.rs](https://docs.rs/gpui-form-schema/) for the full
metadata contract.
