# gpui-form-schema Architecture

`gpui-form-schema` owns the shared metadata model used by macro expansion,
inventory registration, and downstream prototyping.

## Purpose

This crate is the runtime-safe metadata boundary between:

- derive/codegen internals
- the user-facing facade
- downstream tooling such as `gpui-form-prototyping-core`

It should describe component contract metadata and discovered form shape
information, but not own proc-macro parsing or token emission.

## Modules

- `src/lib.rs`: exports `registry`
- `src/registry.rs`: `GpuiFormShape`, `FieldVariant`, and `inventory`
  collection

## Metadata Model

### `GpuiFormShape`

Inventory-registered description of one derived form source struct.

Important fields:

- `struct_name`
- `components`
- `source_path`
- `koruma_enabled`
- `has_skipped_fields`
- `holder_conversion_can_fail`

### `FieldVariant`

Per-field metadata for one generated component entry.

Important fields:

- `field_name`
- `value_type`
- `source_value_type`
- `optional`
- `requires_value`
- `validations`
- `default_expr`
- `from_expr`
- `into_expr`
- `shape_path`
- `component_type`
- `value_binding`
- `prototyping_field_suffix`

Rust syntax fragments are stored as typed string wrappers:

- `RustType`: form-side/source value types and component UI types
- `RustPath`: component shape paths
- `RustExpr`: default and conversion expressions
- `ComponentSuffix`: prototyping field/helper suffix metadata

The wrappers preserve `'static` inventory data while preventing accidental
mixing between types, paths, expressions, and suffixes at registry construction
sites. Their public `new` constructors validate Rust syntax with `syn`;
unchecked const constructors are reserved for macro-generated inventory, where
the derive layer stringifies syntax it already parsed. Consumers call
`.as_str()` before parsing with `syn`.

`holder_conversion_can_fail` is emitted by `gpui-form-derive` from the same
analysis that chooses the generated value-holder conversion methods. Downstream
generators consume that flag instead of recomputing conversion fallibility from
field-level metadata.

`FieldVariant::field_name_with_component_suffix()` derives the generated component
field name from precomputed `prototyping_field_suffix` metadata. Inventory
consumers use this as their generated-code naming policy. The suffix is
normalized against the field name before falling back to `"shape"`.

`requires_value` is emitted by `gpui-form-codegen` from the shape's
`RequiredValuePolicy` associated type. Non-optional shape-backed fields inherit
the reusable shape's policy, so components that can synthesize missing values
define that once on the shape.

## Data Flow

1. `gpui-form-codegen` parses field component syntax and turns it into typed
   component definitions.
1. `gpui-form-codegen` emits shape-only `FieldVariant` metadata for each
   field.
1. `gpui-form-derive` embeds that metadata into generated inventory
   registration.
1. When inventory registration is enabled, `gpui-form-derive` submits a
   `GpuiFormShape`.
1. `gpui-form-prototyping-core` reads that metadata and generates scaffolded
   code.

## Boundary Rules

This crate should own:

- runtime-safe metadata
- inventory registration types

This crate should not own:

- parsing of `#[gpui_form(...)]` attributes
- proc-macro error diagnostics
- direct GPUI runtime implementations

## Coordination Rules

When adding or changing a reusable component shape:

1. implement or derive `ComponentShape` for the shape
1. publish shape-level metadata such as `COMPONENT_TYPE`,
   `ValueBindingPolicy`, and `PROTOTYPING.field_suffix` when generators need it
1. update `gpui-form-runtime`, `gpui-form-component`, or the owning runtime
   crate if runtime support is required

## When To Update This Document

Update this file when:

- `GpuiFormShape` or `FieldVariant` fields change
- component contract metadata changes
- inventory ownership or registration semantics change
