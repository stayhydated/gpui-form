# gpui-form-runtime Architecture

`gpui-form-runtime` is the `gpui-form` component-shape contract and
storage-policy layer around reusable GPUI component shapes.

## Purpose

This crate exists so generated `#[derive(GpuiForm)]` output has a stable facade
for shape construction, value storage, and value binding. The reusable GPUI
shape contract itself lives in `component-shape-gpui`; this crate re-exports it
and keeps form-specific storage and holder behavior local to `gpui-form`.

## Modules

- `src/lib.rs`: public module surface
- `src/shape.rs`: component-shape contracts from `component-shape` and
  `component-shape-gpui`, `GpuiComponentShape`, required-value and value-binding
  policy markers, value-holder storage helpers, and generated-code helper
  functions

## Boundaries

`gpui-form-runtime` may depend on `gpui`, `component-shape`, and
`component-shape-gpui` because generated code constructs GPUI component state
and bridges value binding. Component suffix validation and neutral metadata are
owned by `component-shape`.

Generated form field checks require `DeclaredGpuiComponentShape`. Shapes declared
with `component_shape_gpui::component_shape!` satisfy the reusable GPUI marker;
the form runtime bridge accepts them once the consuming crate attaches local
`gpui-form` storage metadata. A hand-written form-only `GpuiComponentShape` impl can
exist for lower-level runtime experiments, but it is not a usable
`#[gpui_form(component(...))]` field shape.

Generated value-binding metadata dispatches through
`ComponentShapeMetadata::CAPABILITIES` rather than const generics. This keeps
generic shape paths such as `Input<T>` type-checkable. Schema and prototyping
metadata read component capabilities and the form-local storage policy directly
so declared shapes cannot make const metadata disagree with their contracts.

Generated field value-type checks dispatch through
`GpuiComponentShapeFor<Value>`. Shape macros emit implementations only for explicit
`value = ...` or `values(...)` metadata.

Generated validation asks `ValueStorage::is_present` whether a
policy-owned holder field currently contains a value. `RequiredValueStorage` reports
`None` as missing; `DirectValueStorage` always reports present because its
storage is direct `T`.

`ComponentPrototyping::field_suffix(...)` stores
`component_shape::ComponentSuffix`, the same newtype re-exported by
`gpui-form-core` and used by schema inventory metadata. This keeps shape
declarations from publishing invalid prototyping metadata while preserving the
runtime crate's no-schema-dependency boundary.

It should not depend on:

- `gpui-form-component`, which owns concrete reusable widgets
- `gpui-form-collection`, which owns curated shape wrappers around
  `gpui-component`
- `gpui-form-schema`, which owns inventory metadata
- `gpui-form-derive`, which owns proc-macro parsing and expansion
