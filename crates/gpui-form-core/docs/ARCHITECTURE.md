# gpui-form-core Architecture

`gpui-form-core` hosts helper logic that should stay usable without `gpui` or
`gpui-component`.

## Purpose

This crate exists so UI-neutral helper behavior can be shared without pulling
in runtime UI dependencies.

At the moment the crate is intentionally narrow:

- numeric text-entry validation for custom wrappers and future generated code
- `ComponentSuffix` plus ASCII component suffix validation shared by runtime
  and schema metadata

## Modules

- `src/lib.rs`: module export surface
- `src/component_suffix.rs`: shared component suffix newtype and non-empty
  ASCII identifier suffix checks
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

1. The facade re-exports this crate as `gpui_form::core`.
1. Callers that need UI-neutral numeric validation can use
   `gpui_form::core::numeric::*` or depend on `gpui-form-core` directly.
1. Runtime component wrappers remain free to use their own validation strategy
   when they own GPUI state and event behavior.
1. Runtime shape metadata and schema registry metadata both use
   `component_suffix::ComponentSuffix`, so suffix validation and inventory
   storage stay aligned without either layer depending on the other.

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
- generated code or runtime wrappers start depending on new core helpers
