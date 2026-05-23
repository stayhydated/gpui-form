# gpui-form-schema Architecture

`gpui-form-schema` owns the shared metadata model used by macro expansion,
inventory registration, and downstream prototyping.

## Purpose

This crate is the runtime-safe metadata boundary between:

- derive/codegen internals
- the user-facing facade
- downstream tooling such as `gpui-form-prototyping-core`

It should describe component behavior and discovered form shape information, but
not own proc-macro parsing or token emission.

## Modules

- `src/lib.rs`: exports `components` and `registry`
- `src/components.rs`: component identity and behavior payload types
- `src/registry.rs`: `GpuiFormShape`, `FieldVariant`, and `inventory`
  collection

## Metadata Model

### `ComponentKind`

Static identity for component categories. Today the schema only models custom
component contract fields. It centralizes shared traits such as:

- snake-case component naming
- whether a component is subscribable
- whether a component is focusable
- whether generated holder fields wrap in `Option<T>` by default

### `ComponentsBehaviour`

Per-field runtime behavior metadata. Today this is `Custom`; shape-specific
details live in `FieldVariant::custom_shape`, `custom_component`,
`custom_value_binding`, and `custom_prototyping_field_suffix`.

This is the metadata level that downstream consumers use; derive/codegen
internals should not invent separate parallel runtime models.

### `GpuiFormShape`

Inventory-registered description of one derived form source struct.

Important fields:

- `struct_name`
- `components`
- `source_path`
- `koruma_enabled`
- `has_skipped_fields`

### `FieldVariant`

Per-field metadata for one generated component entry.

Important fields:

- `field_name`
- `value_type`
- `source_value_type`
- `optional`
- `requires_value`
- `behaviour`
- `validations`
- `default_expr`
- `from_expr`
- `into_expr`
- `custom_shape`
- `custom_component`
- `custom_value_binding`
- `custom_prototyping_field_suffix`

`FieldVariant::field_name_with_behaviour()` derives the generated component
field name from `custom_prototyping_field_suffix` or `custom_shape` when
available, so inventory consumers use the same suffixes as the derive output.
The fallback suffix strips shape/state wrappers and removes duplicated
field-name prefixes before falling back to the behavior name. Explicit
prototyping suffix metadata is normalized against the field name the same way.

`requires_value` is emitted metadata and, for legacy `custom(...)` attributes,
a metadata key rather than a component expression-chain setter. Known reusable
shape wrappers infer whether a missing generated holder value is invalid in
`gpui-form-codegen`, while `behaviour` carries shape settings such as select
`searchable` / `partial` and infinite-select `searchable` / `max_depth`.

## Data Flow

1. `gpui-form-codegen` parses field component syntax and turns it into typed
   component definitions.
1. `gpui-form-codegen` emits `ComponentsBehaviour` tokens for each field.
1. `gpui-form-derive` embeds those behavior tokens into generated
   `FieldVariant` metadata.
1. When inventory registration is enabled, `gpui-form-derive` submits a
   `GpuiFormShape`.
1. `gpui-form-prototyping-core` reads that metadata and generates scaffolded
   code.

## Boundary Rules

This crate should own:

- runtime-safe metadata
- shared component identity
- inventory registration types

This crate should not own:

- parsing of `#[gpui_form(...)]` attributes
- proc-macro error diagnostics
- direct GPUI runtime implementations

## Coordination Rules

When adding or changing a component:

1. update `ComponentsBehaviour` and related payload structs here
1. update `gpui-form-codegen` so the derive layer emits the new metadata
1. update `gpui-form-prototyping-core` so the generator consumes the new
   behavior correctly
1. update `gpui-form-component` if runtime support is required

## When To Update This Document

Update this file when:

- `GpuiFormShape` or `FieldVariant` fields change
- component behavior payloads change
- inventory ownership or registration semantics change
