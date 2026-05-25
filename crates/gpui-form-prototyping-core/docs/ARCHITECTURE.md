# gpui-form-prototyping-core Architecture

`gpui-form-prototyping-core` turns `GpuiFormShape` inventory metadata into
scaffolded GPUI form code.

It is intentionally downstream from the derive system: it consumes the metadata
model produced elsewhere rather than reparsing source structs.

## Purpose

This crate exists to:

1. validate `GpuiFormShape` metadata before generation
1. compute reusable field fragments and imports
1. let callers render complete files through a pluggable layout

## Module Layout

- `src/lib.rs`: public exports
- `src/code_gen.rs`: `FormShapeAdapter`, `FormParts`, and `FormLayout`
- `src/imports.rs`: import tracking and grouped rendering
- `src/error.rs`: structured prototyping errors
- `src/implementations/`: per-component field generators

## Generation Pipeline

1. A caller iterates `inventory::iter::<GpuiFormShape>()`.
1. The caller constructs `FormShapeAdapter::new(shape)`.
1. The adapter validates and resolves the raw shape into typed field analysis.
1. Component-specific generators produce cached render fragments, event
   handlers, imports, subscriptions, and initialization code.
   Infinite-select fields are generated against one runtime
   `InfiniteSelect` subscription instead of raw select-tree glue code,
   render through `InfiniteSelect::form_fields()`, and consume the richer
   `InfiniteSelectEvent<T>` payload directly.
   File-picker fields use one `FilePickerState` entity, render the runtime
   `FilePicker`, and store the first selected path in the form value holder.
   Text input fields parse the form-side value type from `FieldVariant`
   metadata instead of assuming `String`.
1. The adapter returns:
   - `FormParts` for caller-controlled assembly, or
   - a complete `syn::File` through `generate_file(&impl FormLayout)`
1. The caller writes the resulting file and runs a final formatter such as
   `rustfmt`.

## `FormParts` Role

`FormParts` is the boundary between metadata analysis and layout rendering.

It contains:

- stable identifiers
- deduplicated imports
- component creation tokens
- event/subscription/init tokens
- render fragments
- debug helpers
- flags such as `is_empty`, `has_koruma`, and `has_skipped_fields`

This allows callers to define different layout styles without reimplementing
field analysis.

## Import Strategy

Imports are tracked close to where they are needed:

- fragment-level imports live in the core generation layer
- component-specific imports live in each field generator
- `ImportSet` deduplicates and groups them into compact `use` statements

This avoids layouts having to rediscover which imports the generated fragments
need.

## Field Resolution

The generator parses `FieldVariant::value_type` as a full Rust type, not just a
bare identifier. That is important because inventory metadata may carry:

- crate-qualified enum paths
- nested module paths
- type overrides emitted by the derive layer
- source/form conversion metadata and required-value holder behavior

Each field is resolved once into a typed internal representation before shape
generation runs.

## Component Shape Behavior

All component fields are shape-backed:

- the generator still initializes component state into generated `FormFields`
- generated local variable, field, and handler names use
  `FieldVariant::field_name_with_behaviour`, which prefers component-shape prototyping
  suffix metadata before falling back to shape-derived suffixes
- if `FieldVariant::component_path` is present, the generator can emit
  `Component::new(&entity)` and import the component type
- if that metadata is missing, the generator falls back to a placeholder row
- component subscriptions are generated only when `FieldVariant::value_binding`
  is true; the field's shape must provide the generic
  `ComponentValueBinding<T>` hook that maps component events to
  `FormValueChange<T>`
- value-bound component subscriptions use runtime projection aliases and helper
  functions from `gpui_form_component::shape` inline for value-binding state
  seeding and event conversion; owned components expose their own event enum,
  while external wrappers keep their upstream event type

## Coordination Rules

When adding or changing a component:

1. update the shape metadata emitted by the component's `ComponentShape`
   implementation
1. update imports for newly referenced runtime types
1. verify the example generator under `examples/prototyping`

## When To Update This Document

Update this file when:

- the adapter/layout boundary changes
- `FormParts` fields change meaning
- component shape generation behavior changes
- import handling or field-resolution strategy changes
