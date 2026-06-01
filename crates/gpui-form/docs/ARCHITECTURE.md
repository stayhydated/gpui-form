# gpui-form Architecture

`gpui-form` is the facade crate and public feature boundary for form
generation. It intentionally stays thin: applications get the main
`#[derive(GpuiForm)]` entry point, core helpers, schema metadata, and generated
code dependencies from this crate, while concrete component runtimes and
component derive macros are imported from their own crates.

## Purpose

This crate exists to:

1. present the stable `GpuiForm` entry point
1. centralize public feature flags for form generation and inventory metadata
1. re-export lower-layer contracts used by generated code under consistent paths
1. avoid making component collections or concrete runtime helpers implicit
   facade APIs

## Public Surface

`src/lib.rs` re-exports:

- `gpui_form_derive::GpuiForm` behind the `derive` feature
- `gpui_form_core` as `gpui_form::core`
- `gpui_form_runtime` as `gpui_form::runtime`
- `gpui_form_schema` as `gpui_form::schema`
- `bon` as `gpui_form::bon`

`gpui-form` does not re-export `gpui-form-component`,
`gpui-form-collection`, `gpui-form-component-derive`, or
`gpui-form-collection-derive`. Application crates that use concrete component
helpers, collection shapes, `InfiniteSelect`, or `SelectItem` depend on and
import those crates explicitly. Lower-level crates that declare component
shapes with `#[derive(GpuiComponentShape)]`, `component_shape!`, or
direct runtime value-binding impls may also depend on `gpui-form-runtime`
directly.

## Feature Flags

- `derive` (default): re-exports `GpuiForm` from `gpui-form-derive`
- `inventory`: forwards inventory-enabled `GpuiForm` behavior so
  `#[derive(GpuiForm)]` emits `GpuiFormShape` registrations

`inventory` is meaningful only when `derive` is also enabled.

## Dependency Role

`gpui-form` depends on these lower layers:

- `gpui-form-core` for UI-neutral helper logic
- `gpui-form-schema` for metadata and inventory types
- `gpui-form-runtime` for the `gpui_form::runtime` facade path used by
  generated component field code
- `gpui-form-derive` for `GpuiForm`
- `bon` because generated value holders with skipped fields derive
  `::gpui_form::bon::Builder`

Component runtime contracts live in `gpui-form-runtime` and are re-exported
for generated form code. Curated component shapes live in
`gpui-form-collection`. Their derive crates remain separate so users opt into
those component APIs explicitly.

## Control Flow

### Facade-driven form derive

1. A user depends on `gpui-form` with the `derive` feature.
1. `GpuiForm` resolves to `gpui-form-derive`.
1. `GpuiForm` generated metadata references `gpui_form::schema`.
1. Generated value-holder code references `gpui_form::core` and
   `gpui_form::bon` where needed.
1. Generated component field code references
   `gpui_form::runtime::shape` when a field uses a component shape
   expression, such as `#[gpui_form(component(Shape))]`. Users of component-backed
   fields need only the facade plus the crate that owns the concrete shape
   type.

### Prototyping flow

1. The user enables `gpui-form`'s `inventory` feature.
1. `#[derive(GpuiForm)]` emits `GpuiFormShape` registrations through the facade
   path.
1. Downstream tooling iterates `gpui_form::schema::registry::inventory`.
1. `gpui-form-prototyping-core` converts those shapes into scaffolded GPUI
   code.

## Public Path Notes

- `gpui_form::bon` remains public because generated value holders use it.
- Numeric helpers live under `gpui_form::core::numeric`.
- Generated shape contract paths live under `gpui_form::runtime::shape`.
  Direct `gpui-form-runtime` dependencies are reserved for lower-level shape
  integration crates that declare shapes or value bindings.
- Component module paths such as `date_picker`, `file_picker`, and
  `infinite_select` belong to `gpui-form-component`, not this facade.

## When To Update This Document

Update this file when:

- the facade re-export layout changes
- a feature flag is introduced, removed, or re-routed
- generated code changes which facade or component-runtime paths it targets
