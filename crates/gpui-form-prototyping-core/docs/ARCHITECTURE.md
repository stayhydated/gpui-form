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
1. The adapter asks `gpui-form-schema` for `ResolvedGpuiFormShape`, which
   validates identifiers and parses Rust type/path/expression metadata once.
1. Component-specific generators produce cached typed fragments, event
   handlers, imports, subscriptions, and initialization code. Optional
   generator outputs are normalized into empty semantic fragments at collection
   time, so later assembly works with structured fragment fields rather than
   optional raw token plumbing.
   Infinite-select fields are generated against one runtime
   `InfiniteSelect` subscription instead of raw select-tree glue code,
   render through `InfiniteSelect::form_fields()`, and consume the richer
   `InfiniteSelectEvent<T>` payload directly.
   File-picker fields use one `FilePickerState` entity, render the runtime
   `FilePicker`, and store the selected path list in the form value holder.
   Text input fields use the resolved form-side value type instead of assuming
   `String`.
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
- component creation fragments as typed `ComponentCreation` records
- field initializer fragments as typed `FieldInitializer` records
- subscription bindings as typed `SubscriptionBinding` records
- event handlers as typed `EventHandler` records
- post-subscription/init tokens
- render fragments
- debug helpers
- flags such as `is_empty`, `has_koruma`, `has_skipped_fields`, and
  `holder_conversion_shape`

`holder_conversion_shape` comes directly from `GpuiFormShape` inventory
metadata as `HolderConversionShape::{Infallible, FallibleRequired,
NeedsSkippedFields}`. The derive layer owns the conversion API decision, and
prototyping layouts consume that source of truth instead of reconstructing it
from individual fields.
For skipped-field forms, layouts format the holder's typed `present_fields()`
snapshot for debug output instead of depending on generated JSON helpers.

This allows callers to define different layout styles without reimplementing
field analysis.
Layouts can inspect component creation, field initializer, subscription, and
event-handler data before rendering tokens, which keeps simple alternate
layouts from relying on token string inspection for those fragments.

## Import Strategy

Imports are tracked close to where they are needed:

- fragment-level imports live in the core generation layer
- component-specific imports live in each field generator
- `ImportSet` deduplicates and groups them into compact `use` statements

This avoids layouts having to rediscover which imports the generated fragments
need.

## Field Resolution

The generator consumes `ResolvedGpuiFormShape` / `ResolvedField` from
`gpui-form-schema`, not raw string metadata. `ResolvedField::value_type` is a
parsed full Rust type, not just a bare identifier. That is important because
inventory metadata may carry:

- crate-qualified enum paths
- nested module paths
- type overrides emitted by the derive layer
- source/form conversion metadata and required-value holder behavior

Each field is resolved once into the schema-owned typed representation before
shape generation runs. Field generators receive `ResolvedField`, including
parsed component shape paths, value presence, conversions, validations, and
component capabilities, so malformed field metadata is reported at resolution
with field context instead of being rediscovered during token rendering.

## Component Shape Behavior

All component fields are shape-backed:

- the generator still initializes component state into generated `FormFields`
- generated local variable, field, and handler names use the resolved component
  field identifier, which is derived from precomputed component-shape
  prototyping suffix metadata
- if the field's render capability is enabled, the generator emits
  `<Shape as ComponentShape>::RenderComponent::new(&entity)` through the
  runtime `ComponentRender` contract
- if render metadata, value-binding metadata, shape path metadata, or required
  default storage support is missing, `FormShapeAdapter::parts()` returns a
  field-specific `PrototypingError` instead of emitting placeholder UI
- component subscriptions require the field's value-binding capability; the
  field's shape must provide the generic `ComponentValueBinding<T>` hook that
  maps component events to `FormValueChange<T>`
- value-bound component subscriptions use runtime projection aliases and helper
  functions from `gpui_form::runtime::shape` inline for value-binding state
  seeding and event conversion; owned components expose their own event enum,
  while external wrappers keep their upstream event type
- clear events reset optional holder storage to `None`; for non-optional
  storage they use the declared source default lowered into the form-side value
  when present, then fall back to the shape-owned default storage policy

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
