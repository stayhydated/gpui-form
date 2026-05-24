# gpui-form Architecture

`gpui-form` is the facade crate and public feature boundary for form
generation. It intentionally stays thin: applications get the main
`#[derive(GpuiForm)]` entry point, core helpers, schema metadata, and generated
code dependencies from this crate, while component runtimes and component
derive macros are imported from their own crates.

## Purpose

This crate exists to:

1. present the stable `GpuiForm` entry point
1. centralize public feature flags for form generation and inventory metadata
1. re-export UI-neutral lower layers under consistent paths
1. avoid making component collections or runtime helpers implicit facade APIs

## Public Surface

`src/lib.rs` re-exports:

- `gpui_form_derive::GpuiForm` behind the `derive` feature
- `gpui_form_core` as `gpui_form::core`
- `gpui_form_schema` as `gpui_form::schema`
- `bon` as `gpui_form::bon`

`gpui-form` does not re-export `gpui-form-component`,
`gpui-form-collection`, `gpui-form-component-derive`, or
`gpui-form-collection-derive`. Application crates that use component runtime
helpers, collection shapes, `InfiniteSelect`, or `SelectItem` depend on and
import those crates explicitly.

## Feature Flags

- `derive` (default): re-exports `GpuiForm` from `gpui-form-derive`
- `inventory`: forwards inventory-enabled `GpuiForm` behavior so
  `#[derive(GpuiForm)]` emits `GpuiFormShape` registrations

`inventory` is meaningful only when `derive` is also enabled.

## Dependency Role

`gpui-form` depends on these lower layers:

- `gpui-form-core` for UI-neutral helper logic
- `gpui-form-schema` for metadata and inventory types
- `gpui-form-derive` for `GpuiForm`
- `bon` because generated value holders with skipped fields derive
  `::gpui_form::bon::Builder`

Component runtime contracts live in `gpui-form-component`. Curated component
shapes live in `gpui-form-collection`. Their derive crates remain separate so
users opt into those component APIs explicitly.

## Control Flow

### Facade-driven form derive

1. A user depends on `gpui-form` with the `derive` feature.
1. `GpuiForm` resolves to `gpui-form-derive`.
1. `GpuiForm` generated metadata references `gpui_form::schema`.
1. Generated value-holder code references `gpui_form::core` and
   `gpui_form::bon` where needed.
1. Generated component field code references
   `gpui_form_component::custom` when a field uses a component shape
   expression, such as `component = Shape`.
   Users of component-backed fields must depend on `gpui-form-component`
   explicitly.

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
- Component module paths such as `custom`, `date_picker`, `file_picker`, and
  `infinite_select` belong to `gpui-form-component`, not this facade.

## When To Update This Document

Update this file when:

- the facade re-export layout changes
- a feature flag is introduced, removed, or re-routed
- generated code changes which facade or component-runtime paths it targets
