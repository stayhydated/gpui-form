# gpui-form-core Architecture

`gpui-form-core` hosts helper logic that should stay usable without `gpui` or
`gpui-component`.

## Purpose

This crate exists so generated form code can share core behavior without
pulling in runtime UI dependencies.

At the moment the crate is intentionally narrow:

- numeric text-entry validation for generated `number_input` fields

## Modules

- `src/lib.rs`: module export surface
- `src/numeric.rs`: signed and unsigned text-entry validation helpers

## Numeric Validation Semantics

`numeric.rs` validates editable text before parsing it into the destination
type.

The helpers intentionally allow intermediate editing states:

- empty strings
- `-` for signed numeric entry

They intentionally reject invalid forms early:

- leading zero patterns such as `01` and `00`
- misplaced or repeated `-`
- non-digit characters for unsigned entry

An optional `require_parse` flag determines whether the helper only validates
the text shape or also verifies that the text can parse into `T`.

## Data Flow

1. `gpui-form-codegen` emits `number_input` handlers that call numeric helpers
   through `gpui_form::core::numeric::*`.
1. The facade re-exports this crate as `gpui_form::core`.
1. Generated number-input code uses the helpers during incremental text edits
   before committing parsed values into the holder.

## Boundary

This crate should remain:

- UI-neutral
- independent from GPUI entity types
- small enough that lower layers can depend on it without dragging in runtime
  dependencies

If logic needs GPUI state contracts, subscriptions, or component types, it
belongs in `gpui-form-runtime` or `gpui-form-component` instead.

## When To Update This Document

Update this file when:

- new helper modules are added
- numeric validation semantics change
- generated `number_input` code starts depending on new core helpers
