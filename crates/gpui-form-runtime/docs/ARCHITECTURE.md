# gpui-form-runtime Architecture

`gpui-form-runtime` is the GPUI runtime contract layer for generated forms.

## Purpose

This crate exists so component-shape contracts are not owned by any specific
component collection. It contains the runtime traits and helper types that
generated `#[derive(GpuiForm)]` output references after macro expansion.

## Modules

- `src/lib.rs`: public module surface
- `src/shape.rs`: `ComponentShape`, required-value and value-binding policy
  markers, value-holder storage helpers, value-binding contracts, and
  generated-code helper aliases/functions

## Boundaries

`gpui-form-runtime` may depend on `gpui` because `ComponentShape::new`,
`ComponentValueBinding`, and `ComponentStateValueBinding` use `Window`,
`Context`, and `EventEmitter`.

Generated inherited value-binding checks dispatch through
`ComponentShape::ValueBindingPolicy` and `AssertComponentValueBindingPolicy`
rather than const generics. This keeps generic shape paths such as
`Input<T>` type-checkable. Schema and prototyping metadata read the sealed
policy associated types directly so hand-written shapes cannot make const
metadata disagree with their storage or binding policy.

Generated validation asks `ValueHolderStorage::is_present` whether a
policy-owned holder field currently contains a value. `RequireValue` reports
`None` as missing; `AllowMissingValue` always reports present because its
storage is direct `T`.

`ComponentPrototyping::field_suffix(...)` validates suffixes with the same
const ASCII identifier rules used by derive-generated paths. This keeps manual
runtime implementations from publishing invalid prototyping metadata while
preserving the runtime crate's no-schema-dependency boundary.

It should not depend on:

- `gpui-form-component`, which owns concrete reusable widgets
- `gpui-form-collection`, which owns curated shape wrappers around
  `gpui-component`
- `gpui-form-schema`, which owns inventory metadata
- `gpui-form-derive`, which owns proc-macro parsing and expansion
